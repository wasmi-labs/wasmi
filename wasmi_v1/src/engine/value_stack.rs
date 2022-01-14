//! Data structures to represent the Wasm value stack during execution.

use super::{DropKeep, DEFAULT_VALUE_STACK_LIMIT};
use crate::{TrapCode, Value, ValueType, F32, F64};
use alloc::vec::Vec;
use core::{fmt, fmt::Debug, iter, mem};

/// A single entry or register in the value stack.
///
/// # Note
///
/// This is a thin-wrapper around [`u64`] to allow us to treat runtime values
/// as efficient tag-free [`u64`] values. Bits that are not required by the runtime
/// value are set to zero.
/// This is safe since all of the supported runtime values fit into [`u64`] and since
/// Wasm modules are validated before execution so that invalid representations do not
/// occur, e.g. interpreting a value of 42 as a [`bool`] value.
///
/// At the boundary between the interpreter and the outside world we convert the
/// stack entry value into the required `Value` type which can then be matched on.
/// It is only possible to convert a [`StackEntry`] into a [`Value`] if and only if
/// the type is statically known which always is the case at these boundaries.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct StackEntry(u64);

impl StackEntry {
    /// Returns the underlying bits of the [`StackEntry`].
    pub fn to_bits(self) -> u64 {
        self.0
    }

    /// Converts the untyped [`StackEntry`] value into a typed [`Value`].
    pub fn with_type(self, value_type: ValueType) -> Value {
        match value_type {
            ValueType::I32 => Value::I32(<_>::from_stack_entry(self)),
            ValueType::I64 => Value::I64(<_>::from_stack_entry(self)),
            ValueType::F32 => Value::F32(<_>::from_stack_entry(self)),
            ValueType::F64 => Value::F64(<_>::from_stack_entry(self)),
        }
    }
}

impl From<Value> for StackEntry {
    fn from(value: Value) -> Self {
        match value {
            Value::I32(value) => value.into(),
            Value::I64(value) => value.into(),
            Value::F32(value) => value.into(),
            Value::F64(value) => value.into(),
        }
    }
}

/// Trait used to convert untyped values of [`StackEntry`] into typed values.
pub trait FromStackEntry
where
    Self: Sized,
{
    /// Converts the untyped [`StackEntry`] into the typed `Self` value.
    ///
    /// # Note
    ///
    /// This heavily relies on the fact that executed Wasm is validated
    /// before execution and therefore might result in conversions that
    /// are only valid in a validated context, e.g. so that a stack entry
    /// with a value of 42 is not interpreted as [`bool`] which does not
    /// have a corresponding representation for 42.
    fn from_stack_entry(entry: StackEntry) -> Self;
}

macro_rules! impl_from_stack_entry_integer {
	($($t:ty),* $(,)?) =>	{
		$(
			impl FromStackEntry for $t {
				fn from_stack_entry(entry: StackEntry) -> Self {
					entry.to_bits() as _
				}
			}

			impl From<$t> for StackEntry {
				fn from(value: $t) -> Self {
					Self(value as _)
				}
			}
		)*
	};
}
impl_from_stack_entry_integer!(i8, u8, i16, u16, i32, u32, i64, u64);

macro_rules! impl_from_stack_entry_float {
	($($t:ty),*) =>	{
		$(
			impl FromStackEntry for $t {
				fn from_stack_entry(entry: StackEntry) -> Self {
					Self::from_bits(entry.to_bits() as _)
				}
			}

			impl From<$t> for StackEntry {
				fn from(value: $t) -> Self {
					Self(value.to_bits() as _)
				}
			}
		)*
	};
}
impl_from_stack_entry_float!(f32, f64, F32, F64);

impl From<bool> for StackEntry {
    fn from(value: bool) -> Self {
        Self(value as _)
    }
}

impl FromStackEntry for bool {
    fn from_stack_entry(entry: StackEntry) -> Self {
        entry.to_bits() != 0
    }
}

/// The value stack that is used to execute Wasm bytecode.
///
/// # Note
///
/// The [`ValueStack`] implementation heavily relies on the prior
/// validation of the executed Wasm bytecode for correct execution.
#[derive(Clone)]
pub struct ValueStack {
    /// All currently live stack entries.
    entries: Vec<StackEntry>,
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
            .field("entries", &&self.entries[..self.stack_ptr])
            .field("stack_ptr", &self.stack_ptr)
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
        Self::new(
            DEFAULT_VALUE_STACK_LIMIT / mem::size_of::<StackEntry>(),
            1024 * DEFAULT_VALUE_STACK_LIMIT / mem::size_of::<StackEntry>(),
        )
    }
}

impl Extend<StackEntry> for ValueStack {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = StackEntry>,
    {
        for item in iter {
            self.push(item)
        }
    }
}

impl FromIterator<StackEntry> for ValueStack {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = StackEntry>,
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
    /// If the `initial_len` is zero.
    pub fn new(initial_len: usize, maximum_len: usize) -> Self {
        assert!(
            initial_len > 0,
            "cannot initialize the value stack with zero length"
        );
        let entries = vec![StackEntry(0x00); initial_len];
        Self {
            entries,
            stack_ptr: 0,
            maximum_len,
        }
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
        cells.fill(Default::default());
        self.stack_ptr += additional;
        Ok(())
    }

    /// Drops some amount of entries and keeps some amount of them at the new top.
    ///
    /// # Note
    ///
    /// For an amount of entries to keep `k` and an amount of entries to drop `d`
    /// this has the following effect on stack `s` and stack pointer `sp`.
    ///
    /// 1) Copy `k` elements from indices starting at `sp - k` to `sp - k - d`.
    /// 2) Adjust stack pointer: `sp -= d`
    ///
    /// After this operation the value stack will have `d` fewer entries and the
    /// top `k` entries are the top `k` entries before this operation.
    ///
    /// Note that `k + d` cannot be greater than the stack length.
    pub fn drop_keep(&mut self, drop_keep: DropKeep) {
        let drop = drop_keep.drop();
        if drop == 0 {
            // Nothing to do in this case.
            return;
        }
        let keep = drop_keep.keep();
        // Copy kept values over to their new place on the stack.
        // Note: We cannot use `memcpy` since the slices may overlap.
        let src = self.stack_ptr - keep;
        let dst = self.stack_ptr - keep - drop;
        for i in 0..keep {
            self.entries[dst + i] = self.entries[src + i];
        }
        self.stack_ptr -= drop;
    }

    /// Returns the last stack entry of the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This has the same effect as [`ValueStack::peek`]`(0)`.
    pub fn last(&self) -> StackEntry {
        self.entries[self.stack_ptr - 1]
    }

    /// Returns the last stack entry of the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This has the same effect as [`ValueStack::peek`]`(0)`.
    pub fn last_mut(&mut self) -> &mut StackEntry {
        &mut self.entries[self.stack_ptr - 1]
    }

    /// Peeks the entry at the given depth from the last entry.
    ///
    /// # Note
    ///
    /// Given a `depth` of 0 has the same effect as [`ValueStack::last`].
    pub fn peek(&self, depth: usize) -> StackEntry {
        self.entries[self.stack_ptr - depth - 1]
    }

    /// Peeks the `&mut` entry at the given depth from the last entry.
    ///
    /// # Note
    ///
    /// Given a `depth` of 0 has the same effect as [`ValueStack::last_mut`].
    pub fn peek_mut(&mut self, depth: usize) -> &mut StackEntry {
        &mut self.entries[self.stack_ptr - depth - 1]
    }

    /// Pops the last [`StackEntry`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    pub fn pop(&mut self) -> StackEntry {
        self.stack_ptr -= 1;
        self.entries[self.stack_ptr]
    }

    pub fn drop(&mut self, depth: usize) {
        self.stack_ptr -= depth;
    }

    /// Pops the last [`StackEntry`] from the [`ValueStack`] as `T`.
    pub fn pop_as<T>(&mut self) -> T
    where
        T: FromStackEntry,
    {
        T::from_stack_entry(self.pop())
    }

    /// Pops the last pair of [`StackEntry`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// - This operation is slightly more efficient than using
    ///   [`ValueStack::pop`] twice.
    /// - This operation heavily relies on the prior validation of
    ///   the executed WebAssembly bytecode for correctness.
    pub fn pop2(&mut self) -> (StackEntry, StackEntry) {
        self.stack_ptr -= 2;
        (
            self.entries[self.stack_ptr],
            self.entries[self.stack_ptr + 1],
        )
    }

    /// Evaluates `f` on the top three stack entries.
    ///
    /// In summary this procedure does the following:
    ///
    /// - Pop entry `e3`.
    /// - Pop entry `e2`.
    /// - Peek entry `&mut e1_ptr`.
    /// - Evaluate `f(e1_ptr, e2, e3)`.
    pub fn pop2_eval<F>(&mut self, f: F)
    where
        F: FnOnce(&mut StackEntry, StackEntry, StackEntry),
    {
        let (e2, e3) = self.pop2();
        let e1 = self.last_mut();
        f(e1, e2, e3)
    }

    /// Pushes the [`StackEntry`] to the end of the [`ValueStack`].
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
        T: Into<StackEntry>,
    {
        self.entries[self.stack_ptr] = entry.into();
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

    /// Returns `true` if the [`ValueStack`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reserves enough space for `additional` entries in the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This allows efficient implementation of [`ValueStack::push`] and
    /// [`ValueStack::pop`] operations.
    ///
    /// Before executing a function the interpreter calls this function
    /// to guarantee that enough space on the [`ValueStack`] exists for
    /// correct execution to occur.
    /// For this to be working we need a stack-depth analysis during Wasm
    /// compilation so that we are aware of all stack-depths for every
    /// functions.
    pub fn reserve(&mut self, additional: usize) -> Result<(), TrapCode> {
        if self.len() + additional > self.maximum_len {
            return Err(TrapCode::StackOverflow);
        }
        let required_len = self.len() + additional;
        if required_len > self.capacity() {
            // By extending with the required new length we effectively double
            // the current value stack length and add the additional flat amount
            // on top. This avoids too many frequent reallocations.
            self.entries
                .extend(iter::repeat(StackEntry(0x00)).take(required_len));
        }
        Ok(())
    }

    /// Drains the remaining value stack.
    ///
    /// # Note
    ///
    /// This API is mostly used when writing results back to the
    /// caller after function execution has finished.
    pub fn drain(&mut self) -> &[StackEntry] {
        let len = self.stack_ptr;
        self.stack_ptr = 0;
        &self.entries[0..len]
    }

    /// Returns an exclusive slice to the last `depth` entries in the value stack.
    pub fn peek_as_slice_mut(&mut self, depth: usize) -> &mut [StackEntry] {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_keep_works() {
        fn assert_drop_keep<E>(stack: &ValueStack, drop_keep: DropKeep, expected: E)
        where
            E: IntoIterator,
            E::Item: Into<StackEntry>,
        {
            let mut s = stack.clone();
            s.drop_keep(drop_keep);
            assert_eq!(
                s,
                expected.into_iter().map(Into::into).collect::<ValueStack>()
            );
        }

        let test_inputs = [1, 2, 3, 4, 5, 6];
        let stack = test_inputs
            .into_iter()
            .map(StackEntry::from)
            .collect::<ValueStack>();

        // Drop is always 0 but keep varies:
        for keep in 0..stack.len() {
            // Assert that nothing was changed since nothing was dropped.
            assert_drop_keep(&stack, DropKeep::new(0, keep), test_inputs);
        }

        // Drop is always 1 but keep varies:
        assert_drop_keep(&stack, DropKeep::new(1, 0), [1, 2, 3, 4, 5]);
        assert_drop_keep(&stack, DropKeep::new(1, 1), [1, 2, 3, 4, 6]);
        assert_drop_keep(&stack, DropKeep::new(1, 2), [1, 2, 3, 5, 6]);
        assert_drop_keep(&stack, DropKeep::new(1, 3), [1, 2, 4, 5, 6]);
        assert_drop_keep(&stack, DropKeep::new(1, 4), [1, 3, 4, 5, 6]);
        assert_drop_keep(&stack, DropKeep::new(1, 5), [2, 3, 4, 5, 6]);

        // Drop is always 2 but keep varies:
        assert_drop_keep(&stack, DropKeep::new(2, 0), [1, 2, 3, 4]);
        assert_drop_keep(&stack, DropKeep::new(2, 1), [1, 2, 3, 6]);
        assert_drop_keep(&stack, DropKeep::new(2, 2), [1, 2, 5, 6]);
        assert_drop_keep(&stack, DropKeep::new(2, 3), [1, 4, 5, 6]);
        assert_drop_keep(&stack, DropKeep::new(2, 4), [3, 4, 5, 6]);

        // Drop is always 3 but keep varies:
        assert_drop_keep(&stack, DropKeep::new(3, 0), [1, 2, 3]);
        assert_drop_keep(&stack, DropKeep::new(3, 1), [1, 2, 6]);
        assert_drop_keep(&stack, DropKeep::new(3, 2), [1, 5, 6]);
        assert_drop_keep(&stack, DropKeep::new(3, 3), [4, 5, 6]);

        // Drop is always 4 but keep varies:
        assert_drop_keep(&stack, DropKeep::new(4, 0), [1, 2]);
        assert_drop_keep(&stack, DropKeep::new(4, 1), [1, 6]);
        assert_drop_keep(&stack, DropKeep::new(4, 2), [5, 6]);

        // Drop is always 5 but keep varies:
        assert_drop_keep(&stack, DropKeep::new(5, 0), [1]);
        assert_drop_keep(&stack, DropKeep::new(5, 1), [6]);

        // Drop is always 6.
        assert_drop_keep(&stack, DropKeep::new(6, 0), iter::repeat(0).take(0));
    }
}
