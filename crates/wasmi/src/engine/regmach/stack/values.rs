#![allow(dead_code)] // TODO: remove

use super::err_stack_overflow;
use crate::{
    core::UntypedValue,
    engine::{bytecode2::Register, code_map::CompiledFuncEntity},
};
use core::{fmt, fmt::Debug, iter, marker::PhantomData, mem};
use wasmi_core::TrapCode;

#[cfg(doc)]
use super::calls::CallFrame;
#[cfg(doc)]
use crate::engine::code_map::CompiledFunc;

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

    /// Returns the root [`ValueStackPtr`] pointing to the first value on the [`ValueStack`].
    fn root_stack_ptr(&mut self) -> ValueStackPtr {
        ValueStackPtr::new(self.values.as_mut_ptr())
    }

    /// Returns the [`ValueStackPtr`] at the given `offset`.
    fn stack_ptr_at(&mut self, offset: ValueStackOffset) -> ValueStackPtr {
        self.root_stack_ptr().apply_offset(offset)
    }

    /// Returns the capacity of the [`ValueStack`].
    fn capacity(&self) -> usize {
        self.values.len()
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

    /// Shrink the [`ValueStack`] to the [`ValueStackOffset`].
    #[inline]
    pub fn truncate(&mut self, new_sp: ValueStackOffset) {
        self.sp = new_sp.0;
    }

    /// Allocates a new [`CompiledFunc`] on the [`ValueStack`].
    ///
    /// Returns the [`BaseValueStackOffset`] and [`FrameValueStackOffset`] of the allocated [`CompiledFunc`].
    ///
    /// # Note
    ///
    /// The parameters of the allocated [`CompiledFunc`] are set to zero
    /// and require proper initialization after this call.
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
        let frame_offset: FrameValueStackOffset = self.extend_slice(func.consts()).into();
        let base_offset: BaseValueStackOffset = self.extend_zeros(func.len_cells() as usize).into();
        Ok((base_offset, frame_offset))
    }
}

/// The offset of the [`ValueStackPtr`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueStackOffset(usize);

/// Returned when allocating a new [`CallFrame`] on the [`ValueStack`].
///
/// # Note
///
/// This points to the first cell of the allocated [`CallFrame`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FrameValueStackOffset(ValueStackOffset);

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BaseValueStackOffset(ValueStackOffset);

impl From<ValueStackOffset> for BaseValueStackOffset {
    fn from(offset: ValueStackOffset) -> Self {
        Self(offset)
    }
}

impl ValueStack {
    /// Underlying implementation of [`ValueStack::split`] and [`ValueStack::split_2`].
    fn split_impl(&mut self, offset: ValueStackOffset) -> (&mut Self, ValueStackPtr) {
        let self_ptr = self as *mut _;
        let ptr = self.stack_ptr_at(offset);
        // SAFETY: todo
        (unsafe { &mut *self_ptr }, ptr)
    }

    /// Splits the [`ValueStack`] into [`LockedValueStack`] and [`ValueStackPtr`].
    ///
    /// # Note
    ///
    /// - This is used to efficiently access the registers of a [`CallFrame`].
    /// - Use [`LockedValueStack::merge`] to reverse this operation.
    ///
    /// # Dev. Note
    ///
    /// This API provides a bit more safety when dealing with the [`ValueStackPtr`] abstraction.
    /// It is not perfect nor fail safe but a bit better than having no safety belt.
    pub fn split(&mut self, offset: ValueStackOffset) -> (LockedValueStack<1>, ValueStackPtr) {
        let (this, ptr) = self.split_impl(offset);
        (LockedValueStack(this), ptr)
    }

    /// Splits the [`ValueStack`] into [`LockedValueStack`] and two distinct [`ValueStackPtr`].
    ///
    /// # Note
    ///
    /// - This is used for to efficiently access the registers of two [`CallFrame`] when
    ///   calling a function with its parameters or returning results from a function.
    /// - Use [`LockedValueStack::merge`] to reverse this operation.
    ///
    /// # Dev. Note
    ///
    /// See [`ValueStack::split`].
    pub fn split_2(
        &mut self,
        fst: ValueStackOffset,
        snd: ValueStackOffset,
    ) -> (LockedValueStack<2>, ValueStackPtr, ValueStackPtr) {
        assert_ne!(fst, snd);
        let (this, fst) = self.split_impl(fst);
        let (this, snd) = this.split_impl(snd);
        (LockedValueStack(this), fst, snd)
    }
}

/// Type-enforced wrapper around [`ValueStack`] to prevent manipulations.
pub struct LockedValueStack<'a, const N: usize>(&'a mut ValueStack);

impl<'a> LockedValueStack<'a, 1> {
    /// Merge a single [`ValueStackPtr`] and `self` back into the original [`ValueStack`].
    pub fn merge(self, _ptr: ValueStackPtr<'a>) -> &'a mut ValueStack {
        self.0
    }
}

impl<'a> LockedValueStack<'a, 2> {
    /// Merge a two [`ValueStackPtr`] and `self` back into the original [`ValueStack`].
    pub fn merge(self, _fst: ValueStackPtr<'a>, _snd: ValueStackPtr<'a>) -> &'a mut ValueStack {
        self.0
    }
}

/// Type-wrapper that is excplicitly non-[`Copy`].
#[derive(Debug, Default)]
pub struct NonCopy<T>(T);

/// The [`ValueStack`] pointer.
///
/// # Dev. Note
///
/// [`ValueStackPtr`] is explicitly non-[`Copy`] since it can be seen as a `&mut UntypedValue`.
pub struct ValueStackPtr<'a> {
    /// The underlying raw pointer to a [`CallFrame`] on the [`ValueStack`].
    ptr: *mut UntypedValue,
    /// The lifetime of the associated [`ValueStack`].
    marker: NonCopy<PhantomData<fn() -> &'a ()>>,
}

impl<'a> ValueStackPtr<'a> {
    /// Creates a new [`ValueStackPtr`].
    fn new(ptr: *mut UntypedValue) -> Self {
        Self {
            ptr,
            marker: NonCopy(PhantomData),
        }
    }

    /// Applies the [`ValueStackOffset`] to `self` and returns the result.
    fn apply_offset(self, offset: ValueStackOffset) -> Self {
        Self::new(unsafe { self.ptr.add(offset.0) })
    }

    /// Returns the [`UntypedValue`] at the given [`Register`].
    pub fn get(&self, register: Register) -> UntypedValue {
        let ptr = self.register_ptr(register);
        // SAFETY: todo
        unsafe { *ptr }
    }

    /// Returns an exclusive reference to the [`UntypedValue`] at the given [`Register`].
    pub fn get_mut(&mut self, register: Register) -> &mut UntypedValue {
        let ptr = self.register_ptr(register);
        // SAFETY: todo
        unsafe { &mut *ptr }
    }

    /// Returns the pointer to the [`UntypedValue`] at the [`Register`].
    fn register_ptr(&self, register: Register) -> *mut UntypedValue {
        // SAFETY: todo
        unsafe { self.ptr.offset(register.to_i16() as isize) }
    }
}
