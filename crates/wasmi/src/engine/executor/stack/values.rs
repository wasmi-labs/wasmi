use super::err_stack_overflow;
use crate::{
    core::UntypedValue,
    engine::{bytecode::Register, CompiledFuncEntity},
};
use alloc::vec::Vec;
use core::{fmt, fmt::Debug, iter, mem, ptr};
use wasmi_core::TrapCode;

#[cfg(doc)]
use super::calls::CallFrame;
#[cfg(doc)]
use crate::engine::CompiledFunc;

pub struct ValueStack {
    /// The values on the [`ValueStack`].
    values: Vec<UntypedValue>,
    /// Index of the first free value in the `values` buffer.
    sp: usize,
    /// Maximal possible `sp` value.
    max_sp: usize,
}

impl ValueStack {
    /// Default value for initial value stack height in bytes.
    pub const DEFAULT_MIN_HEIGHT: usize = 1024;

    /// Default value for maximum value stack height in bytes.
    pub const DEFAULT_MAX_HEIGHT: usize = 1024 * Self::DEFAULT_MIN_HEIGHT;
}

impl Debug for ValueStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValueStack")
            .field("sp", &self.sp)
            .field("max_sp", &self.max_sp)
            .field("entries", &&self.values[..self.sp])
            .finish()
    }
}

#[cfg(test)]
impl PartialEq for ValueStack {
    fn eq(&self, other: &Self) -> bool {
        self.sp == other.sp && self.values[..self.sp] == other.values[..other.sp]
    }
}

#[cfg(test)]
impl Eq for ValueStack {}

impl Default for ValueStack {
    fn default() -> Self {
        const REGISTER_SIZE: usize = mem::size_of::<UntypedValue>();
        Self::new(
            Self::DEFAULT_MIN_HEIGHT / REGISTER_SIZE,
            Self::DEFAULT_MAX_HEIGHT / REGISTER_SIZE,
        )
    }
}

impl ValueStack {
    /// Creates a new empty [`ValueStack`].
    ///
    /// # Panics
    ///
    /// - If the `initial_len` is zero.
    /// - If the `initial_len` is greater than `maximum_len`.
    pub fn new(initial_len: usize, maximum_len: usize) -> Self {
        assert!(
            initial_len > 0,
            "cannot initialize the value stack with zero length",
        );
        assert!(
            initial_len <= maximum_len,
            "initial value stack length is greater than maximum value stack length",
        );
        Self {
            values: vec![UntypedValue::default(); initial_len],
            sp: 0,
            max_sp: maximum_len,
        }
    }

    /// Creates an empty [`ValueStack`] that does not allocate heap memory.
    ///
    /// # Note
    ///
    /// This is required for resumable functions in order to replace their
    /// proper stack with a cheap dummy one.
    pub fn empty() -> Self {
        Self {
            values: Vec::new(),
            sp: 0,
            max_sp: 0,
        }
    }

    /// Resets the [`ValueStack`] for reuse.
    ///
    /// # Note
    ///
    /// The [`ValueStack`] can sometimes be left in a non-empty state upon
    /// executing a function, for example when a trap is encountered. We
    /// reset the [`ValueStack`] before executing the next function to
    /// provide a clean slate for all executions.
    pub fn reset(&mut self) {
        self.sp = 0;
    }

    /// Returns the root [`FrameRegisters`] pointing to the first value on the [`ValueStack`].
    pub fn root_stack_ptr(&mut self) -> FrameRegisters {
        FrameRegisters::new(self.values.as_mut_ptr())
    }

    /// Returns the [`FrameRegisters`] at the given `offset`.
    pub unsafe fn stack_ptr_at(&mut self, offset: impl Into<ValueStackOffset>) -> FrameRegisters {
        let ptr = self.values.as_mut_ptr().add(offset.into().0);
        FrameRegisters::new(ptr)
    }

    /// Returns the [`FrameRegisters`] at the given `offset` from the back.
    ///
    /// # Panics (Debug)
    ///
    /// If `n` is greater than the height of the [`ValueStack`].
    pub unsafe fn stack_ptr_last_n(&mut self, n: usize) -> FrameRegisters {
        let len_values = self.len();
        debug_assert!(n <= len_values);
        let offset = len_values - n;
        self.stack_ptr_at(ValueStackOffset(offset))
    }

    /// Returns the capacity of the [`ValueStack`].
    fn capacity(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the [`ValueStack`] is empty.
    pub fn is_empty(&self) -> bool {
        self.values.capacity() == 0
    }

    /// Returns the current length of the [`ValueStack`].
    fn len(&self) -> usize {
        self.sp
    }

    /// Reserves enough space for `additional` cells on the [`ValueStack`].
    ///
    /// This may heap allocate in case the [`ValueStack`] ran out of preallocated memory.
    ///
    /// # Errors
    ///
    /// When trying to grow the [`ValueStack`] over its maximum size limit.
    pub fn reserve(&mut self, additional: usize) -> Result<(), TrapCode> {
        let new_len = self
            .len()
            .checked_add(additional)
            .filter(|&new_len| new_len <= self.max_sp)
            .ok_or_else(err_stack_overflow)?;
        if new_len > self.capacity() {
            // Note: By extending with the new length we effectively double
            // the current value stack length and add the additional flat amount
            // on top. This avoids too many frequent reallocations.
            self.values
                .extend(iter::repeat(UntypedValue::default()).take(new_len));
        }
        Ok(())
    }

    /// Extends the [`ValueStack`] by the `amount` of zeros.
    ///
    /// Returns the [`ValueStackOffset`] before this operation.
    /// Use [`ValueStack::truncate`] to undo the [`ValueStack`] state change.
    ///
    /// # Panics
    ///
    /// If the value stack cannot fit `additional` stack values.
    pub fn extend_zeros(&mut self, amount: usize) -> ValueStackOffset {
        if amount == 0 {
            return ValueStackOffset(self.sp);
        }
        let old_sp = self.sp;
        let cells = self
            .values
            .get_mut(self.sp..)
            .and_then(|slice| slice.get_mut(..amount))
            .unwrap_or_else(|| panic!("did not reserve enough value stack space"));
        cells.fill(UntypedValue::default());
        self.sp += amount;
        ValueStackOffset(old_sp)
    }

    /// Extends the [`ValueStack`] by the `values` slice.
    ///
    /// Returns the [`ValueStackOffset`] before this operation.
    /// Use [`ValueStack::truncate`] to undo the [`ValueStack`] state change.
    ///
    /// # Panics
    ///
    /// If the value stack cannot fit `additional` stack values.
    pub fn extend_slice(&mut self, values: &[UntypedValue]) -> ValueStackOffset {
        if values.is_empty() {
            return ValueStackOffset(self.sp);
        }
        let old_sp = self.sp;
        let len_values = values.len();
        let cells = self
            .values
            .get_mut(self.sp..)
            .and_then(|slice| slice.get_mut(..len_values))
            .unwrap_or_else(|| panic!("did not reserve enough value stack space"));
        cells.copy_from_slice(values);
        self.sp += len_values;
        ValueStackOffset(old_sp)
    }

    /// Drop the last `amount` cells of the [`ValueStack`].
    ///
    /// # Panics (Debug)
    ///
    /// If `amount` is greater than the [`ValueStack`] height.
    #[inline]
    pub fn drop(&mut self, amount: usize) {
        debug_assert!(self.sp >= amount);
        self.sp -= amount;
    }

    /// Shrink the [`ValueStack`] to the [`ValueStackOffset`].
    ///
    /// # Panics (Debug)
    ///
    /// If `new_sp` is greater than the current [`ValueStack`] pointer.
    #[inline]
    pub fn truncate(&mut self, new_sp: impl Into<ValueStackOffset>) {
        let new_sp = new_sp.into().0;
        debug_assert!(new_sp <= self.sp);
        self.sp = new_sp;
    }

    /// Allocates a new [`CompiledFunc`] on the [`ValueStack`].
    ///
    /// Returns the [`BaseValueStackOffset`] and [`FrameValueStackOffset`] of the allocated [`CompiledFunc`].
    ///
    /// # Note
    ///
    /// - All live [`FrameRegisters`] might be invalidated and need to be reinstantiated.
    /// - The parameters of the allocated [`CompiledFunc`] are set to zero
    ///   and require proper initialization after this call.
    ///
    /// # Errors
    ///
    /// When trying to grow the [`ValueStack`] over its maximum size limit.
    pub fn alloc_call_frame(
        &mut self,
        func: &CompiledFuncEntity,
    ) -> Result<(BaseValueStackOffset, FrameValueStackOffset), TrapCode> {
        let len_registers = func.len_registers();
        self.reserve(len_registers as usize)?;
        let frame_offset = FrameValueStackOffset(self.extend_slice(func.consts()));
        let base_offset = BaseValueStackOffset(self.extend_zeros(func.len_cells() as usize));
        Ok((base_offset, frame_offset))
    }

    /// Fills the [`ValueStack`] cells at `offset` with `values`.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that `offset` is valid for the range of
    /// `values` required to be stored on the [`ValueStack`].
    pub unsafe fn fill_at<I>(&mut self, offset: impl Into<ValueStackOffset>, values: I)
    where
        I: IntoIterator<Item = UntypedValue>,
    {
        let offset = offset.into().0;
        let mut values = values.into_iter();
        if offset >= self.sp {
            // In this case we can assert that `values` must be empty since
            // otherwise there is a buffer overflow on the value stack.
            debug_assert!(values.next().is_none());
        }
        let cells = &mut self.values[offset..];
        for (cell, value) in cells.iter_mut().zip(values) {
            *cell = value;
        }
    }

    /// Returns a shared slice over the values of the [`ValueStack`].
    #[inline]
    pub fn as_slice(&self) -> &[UntypedValue] {
        &self.values[0..self.sp]
    }

    /// Returns an exclusive slice over the values of the [`ValueStack`].
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [UntypedValue] {
        &mut self.values[0..self.sp]
    }

    /// Removes the slice `from..to` of [`UntypedValue`] cells from the [`ValueStack`].
    ///
    /// Returns the number of drained [`ValueStack`] cells.
    ///
    /// # Safety
    ///
    /// - This invalidates all [`FrameRegisters`] within the range `from..` and the caller has to
    /// make sure to properly reinstantiate all those pointers after this operation.
    /// - This also invalidates all [`FrameValueStackOffset`] and [`BaseValueStackOffset`] indices
    /// within the range `from..`.
    #[inline]
    pub fn drain(&mut self, from: FrameValueStackOffset, to: FrameValueStackOffset) -> usize {
        debug_assert!(from <= to);
        let from = from.0 .0;
        let to = to.0 .0;
        debug_assert!(from <= self.sp);
        debug_assert!(to <= self.sp);
        let len_drained = to - from;
        self.sp -= len_drained;
        self.values.drain(from..to);
        len_drained
    }
}

/// The offset of the [`FrameRegisters`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueStackOffset(usize);

impl From<FrameValueStackOffset> for ValueStackOffset {
    fn from(offset: FrameValueStackOffset) -> Self {
        offset.0
    }
}

impl From<BaseValueStackOffset> for ValueStackOffset {
    fn from(offset: BaseValueStackOffset) -> Self {
        offset.0
    }
}

/// Returned when allocating a new [`CallFrame`] on the [`ValueStack`].
///
/// # Note
///
/// This points to the first cell of the allocated [`CallFrame`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameValueStackOffset(ValueStackOffset);

impl FrameValueStackOffset {
    /// Creates a new [`FrameValueStackOffset`] at the `index`.
    pub(super) fn new(index: usize) -> Self {
        Self(ValueStackOffset(index))
    }
}

impl From<FrameValueStackOffset> for usize {
    fn from(offset: FrameValueStackOffset) -> usize {
        offset.0 .0
    }
}

impl From<ValueStackOffset> for FrameValueStackOffset {
    fn from(offset: ValueStackOffset) -> Self {
        Self(offset)
    }
}

/// Returned when allocating a new [`CallFrame`] on the [`ValueStack`].
///
/// # Note
///
/// This points to the first mutable cell of the allocated [`CallFrame`].
/// The first mutable cell of a [`CallFrame`] is accessed by [`Register(0)`].
///
/// [`Register(0)`]: [`Register`]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BaseValueStackOffset(ValueStackOffset);

impl BaseValueStackOffset {
    /// Creates a new [`BaseValueStackOffset`] at the `index`.
    pub(super) fn new(index: usize) -> Self {
        Self(ValueStackOffset(index))
    }
}

impl From<BaseValueStackOffset> for usize {
    fn from(offset: BaseValueStackOffset) -> usize {
        offset.0 .0
    }
}

/// Accessor to the [`Register`] values of a [`CallFrame`] on the [`CallStack`].
///
/// [`CallStack`]: [`super::CallStack`]
pub struct FrameRegisters {
    /// The underlying raw pointer to a [`CallFrame`] on the [`ValueStack`].
    ptr: *mut UntypedValue,
}

impl Debug for FrameRegisters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.ptr)
    }
}

impl FrameRegisters {
    /// Creates a new [`FrameRegisters`].
    fn new(ptr: *mut UntypedValue) -> Self {
        Self { ptr }
    }

    /// Returns the [`UntypedValue`] at the given [`Register`].
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Register`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn get(&self, register: Register) -> UntypedValue {
        ptr::read(self.register_offset(register))
    }

    /// Sets the value of the `register` to `value`.`
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Register`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn set(&mut self, register: Register, value: UntypedValue) {
        ptr::write(self.register_offset(register), value)
    }

    /// Returns the underlying pointer offset by the [`Register`] index.
    unsafe fn register_offset(&self, register: Register) -> *mut UntypedValue {
        unsafe { self.ptr.offset(register.to_i16() as isize) }
    }
}

/// An iterator over the [`UntypedValue`] of a [`FrameRegisters`].
///
/// This is a convenience abstraction for efficiently iterating through the values of a [`FrameRegisters`].
pub struct FrameRegistersIter {
    ptr: *mut UntypedValue,
}

impl From<FrameRegisters> for FrameRegistersIter {
    fn from(registers: FrameRegisters) -> Self {
        Self { ptr: registers.ptr }
    }
}

impl FrameRegistersIter {
    /// Creates a new [`FrameRegistersIter`] from the given [`FrameRegisters`] and a [`Register`] offset.
    pub fn new(ptr: FrameRegisters, register: Register) -> Self {
        let ptr = unsafe { ptr.ptr.offset(isize::from(register.to_i16())) };
        Self { ptr }
    }

    /// Sets the next [`UntypedValue`] of the [`FrameRegistersIter`].
    pub unsafe fn set_next(&mut self, value: UntypedValue) {
        unsafe {
            ptr::write(self.ptr, value);
            self.ptr = self.ptr.add(1);
        }
    }

    /// Returns the next [`UntypedValue`] of the [`FrameRegistersIter`].
    #[allow(dead_code)] // TODO: remove
    pub unsafe fn get_next(&mut self) -> UntypedValue {
        unsafe {
            let value = ptr::read(self.ptr);
            self.ptr = self.ptr.add(1);
            value
        }
    }
}

impl Debug for FrameRegistersIter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.ptr)
    }
}
