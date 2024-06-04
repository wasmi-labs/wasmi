use super::err_stack_overflow;
use crate::{
    core::{TrapCode, UntypedVal},
    engine::{bytecode::Register, CompiledFuncEntity},
};
use core::{
    fmt::{self, Debug},
    mem::{self, MaybeUninit},
    ptr,
    slice,
};
use std::vec::Vec;

#[cfg(doc)]
use super::calls::CallFrame;
#[cfg(doc)]
use crate::engine::CompiledFunc;

pub struct ValueStack {
    /// The values on the [`ValueStack`].
    values: Vec<UntypedVal>,
    /// Maximal possible `sp` value.
    max_len: usize,
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
            .field("max_len", &self.max_len)
            .field("entries", &&self.values[..])
            .finish()
    }
}

#[cfg(test)]
impl PartialEq for ValueStack {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

#[cfg(test)]
impl Eq for ValueStack {}

impl Default for ValueStack {
    fn default() -> Self {
        const REGISTER_SIZE: usize = mem::size_of::<UntypedVal>();
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
            values: Vec::with_capacity(initial_len),
            max_len: maximum_len,
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
            max_len: 0,
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
        self.values.clear();
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
    pub fn capacity(&self) -> usize {
        debug_assert!(self.values.len() <= self.values.capacity());
        self.values.capacity()
    }

    /// Returns `true` if the [`ValueStack`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reserves enough space for `additional` cells on the [`ValueStack`].
    ///
    /// This may heap allocate in case the [`ValueStack`] ran out of preallocated memory.
    ///
    /// # Errors
    ///
    /// When trying to grow the [`ValueStack`] over its maximum size limit.
    pub fn extend_by(
        &mut self,
        additional: usize,
    ) -> Result<&mut [MaybeUninit<UntypedVal>], TrapCode> {
        if additional >= self.max_len() - self.len() {
            return Err(err_stack_overflow());
        }
        self.values.reserve(additional);
        let spare = self.values.spare_capacity_mut().as_mut_ptr();
        unsafe { self.values.set_len(self.values.len() + additional) };
        Ok(unsafe { slice::from_raw_parts_mut(spare, additional) })
    }

    /// Returns the current length of the [`ValueStack`].
    fn len(&self) -> usize {
        debug_assert!(self.values.len() <= self.max_len);
        self.values.len()
    }

    /// Returns the maximum length of the [`ValueStack`].
    fn max_len(&self) -> usize {
        debug_assert!(self.values.len() <= self.max_len);
        self.max_len
    }

    /// Reserves enough space for `additional` cells on the [`ValueStack`].
    ///
    /// This may heap allocate in case the [`ValueStack`] ran out of preallocated memory.
    ///
    /// # Errors
    ///
    /// When trying to grow the [`ValueStack`] over its maximum size limit.
    #[deprecated(note = "use extend_by instead")]
    pub fn reserve(&mut self, additional: usize) -> Result<(), TrapCode> {
        if additional >= self.max_len() - self.len() {
            return Err(err_stack_overflow());
        }
        self.values.reserve(additional);
        Ok(())
    }

    /// Extends the [`ValueStack`] by the `amount` of zeros.
    ///
    /// Returns the [`ValueStackOffset`] before this operation.
    /// Use [`ValueStack::truncate`] to undo the [`ValueStack`] state change.
    ///
    /// # Safety
    ///
    /// The caller is responsible to make sure enough space is reserved for `amount` new values.
    #[deprecated(note = "use extend_by instead")]
    pub unsafe fn extend_zeros(&mut self, amount: usize) -> ValueStackOffset {
        if amount == 0 {
            return ValueStackOffset(self.len());
        }
        let remaining = self.values.spare_capacity_mut();
        let uninit = unsafe { remaining.get_unchecked_mut(..amount) };
        uninit.fill(MaybeUninit::new(UntypedVal::default()));
        let old_len = self.len();
        unsafe { self.values.set_len(old_len + amount) };
        ValueStackOffset(old_len)
    }

    /// Drop the last `amount` cells of the [`ValueStack`].
    ///
    /// # Panics (Debug)
    ///
    /// If `amount` is greater than the [`ValueStack`] height.
    #[inline(always)]
    pub fn drop(&mut self, amount: usize) {
        assert!(self.len() >= amount);
        // Safety: we just asserted that the current length is large enough to not underflow.
        unsafe { self.values.set_len(self.len() - amount) };
    }

    /// Shrink the [`ValueStack`] to the [`ValueStackOffset`].
    ///
    /// # Panics (Debug)
    ///
    /// If `new_sp` is greater than the current [`ValueStack`] pointer.
    #[inline(always)]
    pub fn truncate(&mut self, new_len: impl Into<ValueStackOffset>) {
        let new_len = new_len.into().0;
        assert!(new_len <= self.len());
        // Safety: we just asserted that the new length is valid.
        unsafe { self.values.set_len(new_len) };
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
        let len = self.len();
        let mut spare = self.extend_by(len_registers as usize)?.into_iter();
        (&mut spare)
            .zip(func.consts())
            .for_each(|(uninit, const_value)| {
                uninit.write(*const_value);
            });
        spare.for_each(|uninit| {
            uninit.write(UntypedVal::from(0));
        });
        let frame = ValueStackOffset(len);
        let base = ValueStackOffset(len + func.consts().len());
        Ok((BaseValueStackOffset(base), FrameValueStackOffset(frame)))
    }

    /// Fills the [`ValueStack`] cells at `offset` with `values`.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that `offset` is valid for the range of
    /// `values` required to be stored on the [`ValueStack`].
    pub unsafe fn fill_at<IntoIter, Iter>(
        &mut self,
        offset: impl Into<ValueStackOffset>,
        values: IntoIter,
    ) where
        IntoIter: IntoIterator<IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = UntypedVal>,
    {
        let offset = offset.into().0;
        let values = values.into_iter();
        let len_values = values.len();
        let cells = &mut self.values[offset..offset + len_values];
        for (cell, value) in cells.iter_mut().zip(values) {
            *cell = value;
        }
    }

    /// Returns a shared slice over the values of the [`ValueStack`].
    #[inline(always)]
    pub fn as_slice(&self) -> &[UntypedVal] {
        self.values.as_slice()
    }

    /// Returns an exclusive slice over the values of the [`ValueStack`].
    #[inline(always)]
    pub fn as_slice_mut(&mut self) -> &mut [UntypedVal] {
        self.values.as_mut_slice()
    }

    /// Removes the slice `from..to` of [`UntypedVal`] cells from the [`ValueStack`].
    ///
    /// Returns the number of drained [`ValueStack`] cells.
    ///
    /// # Safety
    ///
    /// - This invalidates all [`FrameRegisters`] within the range `from..` and the caller has to
    ///   make sure to properly reinstantiate all those pointers after this operation.
    /// - This also invalidates all [`FrameValueStackOffset`] and [`BaseValueStackOffset`] indices
    ///   within the range `from..`.
    #[inline(always)]
    pub fn drain(&mut self, from: FrameValueStackOffset, to: FrameValueStackOffset) -> usize {
        debug_assert!(from <= to);
        let from = from.0 .0;
        let to = to.0 .0;
        debug_assert!(from <= self.len());
        debug_assert!(to <= self.len());
        let len_drained = to - from;
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
    ptr: *mut UntypedVal,
}

impl Debug for FrameRegisters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.ptr)
    }
}

impl FrameRegisters {
    /// Creates a new [`FrameRegisters`].
    fn new(ptr: *mut UntypedVal) -> Self {
        Self { ptr }
    }

    /// Returns the [`UntypedVal`] at the given [`Register`].
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Register`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn get(&self, register: Register) -> UntypedVal {
        ptr::read(self.register_offset(register))
    }

    /// Sets the value of the `register` to `value`.`
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Register`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn set(&mut self, register: Register, value: UntypedVal) {
        ptr::write(self.register_offset(register), value)
    }

    /// Returns the underlying pointer offset by the [`Register`] index.
    unsafe fn register_offset(&self, register: Register) -> *mut UntypedVal {
        unsafe { self.ptr.offset(register.to_i16() as isize) }
    }
}
