use super::{err_stack_overflow, StackOffsets};
use crate::{
    core::{ReadAs, UntypedVal, WriteAs},
    engine::code_map::CompiledFuncRef,
    ir::Slot,
    TrapCode,
};
use alloc::vec::Vec;
use core::{
    fmt::{self, Debug},
    mem::{self, MaybeUninit},
    ops::Range,
    ptr,
    slice,
};

#[cfg(doc)]
use super::calls::CallFrame;
#[cfg(doc)]
use crate::engine::EngineFunc;

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

    /// Returns the root [`FrameSlots`] pointing to the first value on the [`ValueStack`].
    pub fn root_stack_ptr(&mut self) -> FrameSlots {
        FrameSlots::new(self.values.as_mut_ptr())
    }

    /// Returns the [`FrameSlots`] at the given `offset`.
    pub unsafe fn stack_ptr_at(&mut self, offset: impl Into<ValueStackOffset>) -> FrameSlots {
        let ptr = self.values.as_mut_ptr().add(offset.into().0);
        FrameSlots::new(ptr)
    }

    /// Returns the capacity of the [`ValueStack`].
    pub fn capacity(&self) -> usize {
        debug_assert!(self.values.len() <= self.values.capacity());
        self.values.capacity()
    }

    /// Reserves enough space for `additional` cells on the [`ValueStack`].
    ///
    /// This may heap allocate in case the [`ValueStack`] ran out of preallocated memory.
    ///
    /// # Errors
    ///
    /// When trying to grow the [`ValueStack`] over its maximum size limit.
    #[inline(always)]
    pub fn extend_by(
        &mut self,
        additional: usize,
        on_resize: impl FnOnce(&mut Self),
    ) -> Result<&mut [MaybeUninit<UntypedVal>], TrapCode> {
        if additional >= self.max_len() - self.len() {
            return Err(err_stack_overflow());
        }
        let prev_capacity = self.capacity();
        self.values.reserve(additional);
        if prev_capacity != self.capacity() {
            on_resize(self);
        }
        let spare = self.values.spare_capacity_mut().as_mut_ptr();
        unsafe { self.values.set_len(self.values.len() + additional) };
        Ok(unsafe { slice::from_raw_parts_mut(spare, additional) })
    }

    /// Returns the current length of the [`ValueStack`].
    #[inline(always)]
    fn len(&self) -> usize {
        debug_assert!(self.values.len() <= self.max_len);
        self.values.len()
    }

    /// Returns the maximum length of the [`ValueStack`].
    #[inline(always)]
    fn max_len(&self) -> usize {
        debug_assert!(self.values.len() <= self.max_len);
        self.max_len
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

    /// Drop the last `amount` cells of the [`ValueStack`] and returns a slice to them.
    ///
    /// # Panics (Debug)
    ///
    /// If `amount` is greater than the [`ValueStack`] height.
    #[inline(always)]
    pub fn drop_return(&mut self, amount: usize) -> &[UntypedVal] {
        let len = self.len();
        let dropped = unsafe { self.values.get_unchecked(len - amount..) }.as_ptr();
        self.drop(amount);
        unsafe { slice::from_raw_parts(dropped, amount) }
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

    /// Allocates a new [`EngineFunc`] on the [`ValueStack`].
    ///
    /// Returns the [`BaseValueStackOffset`] and [`FrameValueStackOffset`] of the allocated [`EngineFunc`].
    ///
    /// # Note
    ///
    /// - All live [`FrameSlots`] might be invalidated and need to be reinstantiated.
    /// - The parameters of the allocated [`EngineFunc`] are set to zero
    ///   and require proper initialization after this call.
    ///
    /// # Errors
    ///
    /// When trying to grow the [`ValueStack`] over its maximum size limit.
    pub fn alloc_call_frame(
        &mut self,
        func: CompiledFuncRef,
        on_resize: impl FnMut(&mut Self),
    ) -> Result<(FrameParams, StackOffsets), TrapCode> {
        let len_stack_slots = func.len_stack_slots();
        let len_consts = func.consts().len();
        let len = self.len();
        let mut spare = self
            .extend_by(len_stack_slots as usize, on_resize)?
            .iter_mut();
        (&mut spare)
            .zip(func.consts())
            .for_each(|(uninit, const_value)| {
                uninit.write(*const_value);
            });
        let params = FrameParams::new(spare.into_slice());
        let frame = ValueStackOffset(len);
        let base = ValueStackOffset(len + len_consts);
        Ok((
            params,
            StackOffsets {
                base: BaseValueStackOffset(base),
                frame: FrameValueStackOffset(frame),
            },
        ))
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
    /// - This invalidates all [`FrameSlots`] within the range `from..` and the caller has to
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

/// The offset of the [`FrameSlots`].
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
/// The first mutable cell of a [`CallFrame`] is accessed by [`Slot(0)`].
///
/// [`Slot(0)`]: [`Slot`]
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

/// Uninitialized parameters of a [`CallFrame`].
pub struct FrameParams {
    range: Range<*mut MaybeUninit<UntypedVal>>,
}

impl FrameParams {
    /// Creates a new [`FrameSlots`].
    pub fn new(ptr: &mut [MaybeUninit<UntypedVal>]) -> Self {
        Self {
            range: ptr.as_mut_ptr_range(),
        }
    }

    /// Sets the value of the `register` to `value`.`
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Slot`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn init_next(&mut self, value: UntypedVal) {
        self.range.start.write(MaybeUninit::new(value));
        self.range.start = self.range.start.add(1);
    }

    /// Zero-initialize the remaining locals and parameters.
    pub fn init_zeroes(mut self) {
        debug_assert!(
            self.range.start <= self.range.end,
            "failed to zero-initialize `FrameParams`: start = {:?}, end = {:?}",
            self.range.start,
            self.range.end,
        );
        while !core::ptr::eq(self.range.start, self.range.end) {
            // Safety: We do not write out-of-buffer due to the above condition.
            unsafe { self.init_next(UntypedVal::from(0_u64)) }
        }
    }
}

/// Accessor to the [`Slot`] values of a [`CallFrame`] on the [`CallStack`].
///
/// [`CallStack`]: [`super::CallStack`]
pub struct FrameSlots {
    /// The underlying raw pointer to a [`CallFrame`] on the [`ValueStack`].
    ptr: *mut UntypedVal,
}

impl Debug for FrameSlots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.ptr)
    }
}

impl FrameSlots {
    /// Creates a new [`FrameSlots`].
    fn new(ptr: *mut UntypedVal) -> Self {
        Self { ptr }
    }

    /// Returns the [`UntypedVal`] at the given [`Slot`].
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Slot`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn get(&self, slot: Slot) -> UntypedVal {
        ptr::read(self.register_offset(slot))
    }

    /// Returns the [`UntypedVal`] at the given [`Slot`].
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Slot`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn read_as<T>(&self, slot: Slot) -> T
    where
        UntypedVal: ReadAs<T>,
    {
        UntypedVal::read_as(&*self.register_offset(slot))
    }

    /// Sets the value of the `register` to `value`.`
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Slot`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn set(&mut self, slot: Slot, value: UntypedVal) {
        ptr::write(self.register_offset(slot), value)
    }

    /// Sets the value of the `register` to `value`.`
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to provide a [`Slot`] that
    /// does not access the underlying [`ValueStack`] out of bounds.
    pub unsafe fn write_as<T>(&mut self, slot: Slot, value: T)
    where
        UntypedVal: WriteAs<T>,
    {
        let val: &mut UntypedVal = &mut *self.register_offset(slot);
        val.write_as(value);
    }

    /// Returns the underlying pointer offset by the [`Slot`] index.
    unsafe fn register_offset(&self, slot: Slot) -> *mut UntypedVal {
        unsafe { self.ptr.offset(isize::from(i16::from(slot))) }
    }
}
