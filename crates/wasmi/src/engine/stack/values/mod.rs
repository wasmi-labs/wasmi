//! Data structures to represent the Wasm value stack during execution.

mod sp;

#[cfg(test)]
mod tests;

pub use self::sp::ValueStackPtr;
use super::{err_stack_overflow, DEFAULT_MAX_VALUE_STACK_HEIGHT, DEFAULT_MIN_VALUE_STACK_HEIGHT};
use crate::{core::TrapCode, engine::code_map::FuncHeader};
use alloc::vec::Vec;
use core::{fmt, fmt::Debug, iter, mem::size_of};
use wasmi_core::UntypedValue;

/// The value stack that is used to execute Wasm bytecode.
///
/// # Note
///
/// The [`ValueStack`] implementation heavily relies on the prior
/// validation of the executed Wasm bytecode for correct execution.
#[derive(Clone)]
pub struct ValueStack {
    /// All currently live stack entries.
    entries: Vec<UntypedValue>,
    /// Index of the first free place in the stack.
    stack_ptr: usize,
    /// The maximum value stack height.
    ///
    /// # Note
    ///
    /// Extending the value stack beyond this limit during execution
    /// will cause a stack overflow trap.
    maximum_len: usize,
}

impl Debug for ValueStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValueStack")
            .field("stack_ptr", &self.stack_ptr)
            .field("entries", &&self.entries[..self.stack_ptr])
            .finish()
    }
}

impl PartialEq for ValueStack {
    fn eq(&self, other: &Self) -> bool {
        self.stack_ptr == other.stack_ptr
            && self.entries[..self.stack_ptr] == other.entries[..other.stack_ptr]
    }
}

impl Eq for ValueStack {}

impl Default for ValueStack {
    fn default() -> Self {
        let register_len = size_of::<UntypedValue>();
        Self::new(
            DEFAULT_MIN_VALUE_STACK_HEIGHT / register_len,
            DEFAULT_MAX_VALUE_STACK_HEIGHT / register_len,
        )
    }
}

impl Extend<UntypedValue> for ValueStack {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = UntypedValue>,
    {
        for item in iter {
            self.push(item)
        }
    }
}

impl ValueStack {
    /// Creates an empty [`ValueStack`] that does not allocate heap memor.
    ///
    /// # Note
    ///
    /// This is required for resumable functions in order to replace their
    /// proper stack with a cheap dummy one.
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            stack_ptr: 0,
            maximum_len: 0,
        }
    }

    /// Returns the current [`ValueStackPtr`] of `self`.
    ///
    /// The returned [`ValueStackPtr`] points to the top most value on the [`ValueStack`].
    #[inline]
    pub fn stack_ptr(&mut self) -> ValueStackPtr {
        self.base_ptr().into_add(self.stack_ptr)
    }

    /// Returns the base [`ValueStackPtr`] of `self`.
    ///
    /// The returned [`ValueStackPtr`] points to the first value on the [`ValueStack`].
    #[inline]
    fn base_ptr(&mut self) -> ValueStackPtr {
        ValueStackPtr::from(self.entries.as_mut_ptr())
    }

    /// Synchronizes [`ValueStack`] with the new [`ValueStackPtr`].
    #[inline]
    pub fn sync_stack_ptr(&mut self, new_sp: ValueStackPtr) {
        let offset = new_sp.offset_from(self.base_ptr());
        self.stack_ptr = offset as usize;
    }

    /// Returns `true` if the [`ValueStack`] is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.capacity() == 0
    }

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
        let entries = vec![UntypedValue::default(); initial_len];
        Self {
            entries,
            stack_ptr: 0,
            maximum_len,
        }
    }

    /// Returns the [`UntypedValue`] at the given `index`.
    ///
    /// # Note
    ///
    /// This is an optimized convenience method that only asserts
    /// that the index is within bounds in `debug` mode.
    ///
    /// # Safety
    ///
    /// This is safe since all wasmi bytecode has been validated
    /// during translation and therefore cannot result in out of
    /// bounds accesses.
    ///
    /// # Panics (Debug)
    ///
    /// If the `index` is out of bounds.
    #[inline]
    fn get_release_unchecked_mut(&mut self, index: usize) -> &mut UntypedValue {
        debug_assert!(index < self.capacity());
        // Safety: This is safe since all wasmi bytecode has been validated
        //         during translation and therefore cannot result in out of
        //         bounds accesses.
        unsafe { self.entries.get_unchecked_mut(index) }
    }

    /// Extends the value stack by the `additional` amount of zeros.
    ///
    /// # Errors
    ///
    /// If the value stack cannot fit `additional` stack values.
    pub fn extend_zeros(&mut self, additional: usize) {
        let cells = self
            .entries
            .get_mut(self.stack_ptr..)
            .and_then(|slice| slice.get_mut(..additional))
            .unwrap_or_else(|| panic!("did not reserve enough value stack space"));
        cells.fill(UntypedValue::default());
        self.stack_ptr += additional;
    }

    /// Prepares the [`ValueStack`] for execution of the given Wasm function.
    pub fn prepare_wasm_call(&mut self, header: &FuncHeader) -> Result<(), TrapCode> {
        let max_stack_height = header.max_stack_height();
        self.reserve(max_stack_height)?;
        let len_locals = header.len_locals();
        self.extend_zeros(len_locals);
        Ok(())
    }

    /// Drops the last value on the [`ValueStack`].
    #[inline]
    pub fn drop(&mut self, depth: usize) {
        self.stack_ptr -= depth;
    }

    /// Pushes the [`UntypedValue`] to the end of the [`ValueStack`].
    ///
    /// # Note
    ///
    /// - This operation heavily relies on the prior validation of
    ///   the executed WebAssembly bytecode for correctness.
    /// - Especially the stack-depth analysis during compilation with
    ///   a manual stack extension before function call prevents this
    ///   procedure from panicking.
    #[inline]
    pub fn push(&mut self, entry: UntypedValue) {
        *self.get_release_unchecked_mut(self.stack_ptr) = entry;
        self.stack_ptr += 1;
    }

    /// Returns the capacity of the [`ValueStack`].
    fn capacity(&self) -> usize {
        self.entries.len()
    }

    /// Returns the current length of the [`ValueStack`].
    fn len(&self) -> usize {
        self.stack_ptr
    }

    /// Reserves enough space for `additional` entries in the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This allows to efficiently operate on the [`ValueStack`] through
    /// [`ValueStackPtr`] which requires external resource management.
    ///
    /// Before executing a function the interpreter calls this function
    /// to guarantee that enough space on the [`ValueStack`] exists for
    /// correct execution to occur.
    /// For this to be working we need a stack-depth analysis during Wasm
    /// compilation so that we are aware of all stack-depths for every
    /// functions.
    pub fn reserve(&mut self, additional: usize) -> Result<(), TrapCode> {
        let new_len = self
            .len()
            .checked_add(additional)
            .filter(|&new_len| new_len <= self.maximum_len)
            .ok_or_else(err_stack_overflow)?;
        if new_len > self.capacity() {
            // Note: By extending with the new length we effectively double
            // the current value stack length and add the additional flat amount
            // on top. This avoids too many frequent reallocations.
            self.entries
                .extend(iter::repeat(UntypedValue::default()).take(new_len));
        }
        Ok(())
    }

    /// Drains the remaining value stack.
    ///
    /// # Note
    ///
    /// This API is mostly used when writing results back to the
    /// caller after function execution has finished.
    #[inline]
    pub fn drain(&mut self) -> &[UntypedValue] {
        let len = self.stack_ptr;
        self.stack_ptr = 0;
        &self.entries[0..len]
    }

    /// Returns an exclusive slice to the last `depth` entries in the value stack.
    #[inline]
    pub fn peek_as_slice_mut(&mut self, depth: usize) -> &mut [UntypedValue] {
        let start = self.stack_ptr - depth;
        let end = self.stack_ptr;
        &mut self.entries[start..end]
    }

    /// Clears the [`ValueStack`] entirely.
    ///
    /// # Note
    ///
    /// This is required since sometimes execution can halt in the middle of
    /// function execution which leaves the [`ValueStack`] in an unspecified
    /// state. Therefore the [`ValueStack`] is required to be reset before
    /// function execution happens.
    pub fn reset(&mut self) {
        self.stack_ptr = 0;
    }
}
