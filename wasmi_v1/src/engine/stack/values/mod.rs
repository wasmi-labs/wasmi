//! Data structures to represent the Wasm value stack during execution.

mod vref;

#[cfg(test)]
mod tests;

pub use self::vref::ValueStackRef;
use super::{err_stack_overflow, DEFAULT_MAX_VALUE_STACK_HEIGHT, DEFAULT_MIN_VALUE_STACK_HEIGHT};
use crate::core::TrapCode;
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

impl FromIterator<UntypedValue> for ValueStack {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = UntypedValue>,
    {
        let mut stack = ValueStack::default();
        stack.extend(iter);
        stack
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
    pub fn extend_zeros(&mut self, additional: usize) -> Result<(), TrapCode> {
        let cells = self
            .entries
            .get_mut(self.stack_ptr..self.stack_ptr + additional)
            .ok_or(TrapCode::StackOverflow)?;
        cells.fill(UntypedValue::default());
        self.stack_ptr += additional;
        Ok(())
    }

    /// Drops the last value on the [`ValueStack`].
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
    pub fn push<T>(&mut self, entry: T)
    where
        T: Into<UntypedValue>,
    {
        *self.get_release_unchecked_mut(self.stack_ptr) = entry.into();
        self.stack_ptr += 1;
    }

    /// Returns the capacity of the [`ValueStack`].
    fn capacity(&self) -> usize {
        self.entries.len()
    }

    /// Returns the current length of the [`ValueStack`].
    pub fn len(&self) -> usize {
        self.stack_ptr
    }

    /// Reserves enough space for `additional` entries in the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This allows efficient implementation of [`ValueStack::push`] and
    /// [`ValueStackRef::pop`] operations.
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
    pub fn drain(&mut self) -> &[UntypedValue] {
        let len = self.stack_ptr;
        self.stack_ptr = 0;
        &self.entries[0..len]
    }

    /// Returns an exclusive slice to the last `depth` entries in the value stack.
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
    pub fn clear(&mut self) {
        self.stack_ptr = 0;
    }
}
