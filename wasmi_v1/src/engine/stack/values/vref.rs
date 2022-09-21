use super::ValueStack;
use crate::{
    core::{TrapCode, UntypedValue},
    engine::DropKeep,
};

/// A mutable view over the [`ValueStack`].
///
/// This allows for a more efficient access to the [`ValueStack`] during execution.
#[derive(Debug)]
pub struct ValueStackRef<'a> {
    pub(super) stack_ptr: usize,
    pub(super) values: &'a mut [UntypedValue],
}

impl<'a> ValueStackRef<'a> {
    /// Creates a new [`ValueStackRef`] from the given [`ValueStack`].
    ///
    /// This also returns an exclusive reference to the stack pointer of
    /// the underlying [`ValueStack`]. This is important in order to synchronize
    /// the [`ValueStack`] with the changes done to the [`ValueStackRef`]
    /// when necessary.
    pub fn new(stack: &'a mut ValueStack) -> (&'a mut usize, Self) {
        let sp = &mut stack.stack_ptr;
        let stack_ptr = *sp;
        (
            sp,
            Self {
                stack_ptr,
                values: &mut stack.entries[..],
            },
        )
    }

    /// Returns the current value of the stack pointer.
    pub fn stack_ptr(&self) -> usize {
        self.stack_ptr
    }

    /// Returns the current capacity of the underlying [`ValueStack`].
    fn capacity(&self) -> usize {
        self.values.len()
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
    fn get_release_unchecked(&self, index: usize) -> UntypedValue {
        debug_assert!(index < self.capacity());
        // Safety: This is safe since all wasmi bytecode has been validated
        //         during translation and therefore cannot result in out of
        //         bounds accesses.
        unsafe { *self.values.get_unchecked(index) }
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
        unsafe { self.values.get_unchecked_mut(index) }
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
            *self.get_release_unchecked_mut(dst + i) = self.get_release_unchecked(src + i);
        }
        self.stack_ptr -= drop;
    }

    /// Returns the last stack entry of the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This has the same effect as [`ValueStack::peek`]`(0)`.
    #[inline]
    pub fn last(&self) -> UntypedValue {
        self.get_release_unchecked(self.stack_ptr - 1)
    }

    /// Returns the last stack entry of the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This has the same effect as [`ValueStack::peek`]`(0)`.
    #[inline]
    pub fn last_mut(&mut self) -> &mut UntypedValue {
        self.get_release_unchecked_mut(self.stack_ptr - 1)
    }

    /// Peeks the entry at the given depth from the last entry.
    ///
    /// # Note
    ///
    /// Given a `depth` of 1 has the same effect as [`ValueStack::last`].
    ///
    /// A `depth` of 0 is invalid and undefined.
    pub fn peek(&self, depth: usize) -> UntypedValue {
        self.get_release_unchecked(self.stack_ptr - depth)
    }

    /// Peeks the `&mut` entry at the given depth from the last entry.
    ///
    /// # Note
    ///
    /// Given a `depth` of 1 has the same effect as [`ValueStack::last_mut`].
    ///
    /// A `depth` of 0 is invalid and undefined.
    pub fn peek_mut(&mut self, depth: usize) -> &mut UntypedValue {
        self.get_release_unchecked_mut(self.stack_ptr - depth)
    }

    /// Pops the last [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    pub fn pop(&mut self) -> UntypedValue {
        self.stack_ptr -= 1;
        self.get_release_unchecked(self.stack_ptr)
    }

    /// Pops the last [`UntypedValue`] from the [`ValueStack`] as `T`.
    pub fn pop_as<T>(&mut self) -> T
    where
        T: From<UntypedValue>,
    {
        T::from(self.pop())
    }

    /// Pops the last pair of [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// - This operation is slightly more efficient than using
    ///   [`ValueStack::pop`] twice.
    /// - This operation heavily relies on the prior validation of
    ///   the executed WebAssembly bytecode for correctness.
    pub fn pop2(&mut self) -> (UntypedValue, UntypedValue) {
        self.stack_ptr -= 2;
        (
            self.get_release_unchecked(self.stack_ptr),
            self.get_release_unchecked(self.stack_ptr + 1),
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
        F: FnOnce(&mut UntypedValue, UntypedValue, UntypedValue),
    {
        let (e2, e3) = self.pop2();
        let e1 = self.last_mut();
        f(e1, e2, e3)
    }

    /// Evaluates the given closure `f` for the top most stack value.
    pub fn eval_top<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue) -> UntypedValue,
    {
        let top = self.last();
        *self.last_mut() = f(top);
    }

    /// Evaluates the given fallible closure `f` for the top most stack value.
    ///
    /// # Errors
    ///
    /// If the closure execution fails.
    pub fn try_eval_top<F>(&mut self, f: F) -> Result<(), TrapCode>
    where
        F: FnOnce(UntypedValue) -> Result<UntypedValue, TrapCode>,
    {
        let top = self.last();
        *self.last_mut() = f(top)?;
        Ok(())
    }

    /// Evaluates the given closure `f` for the 2 top most stack values.
    pub fn eval_top2<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue, UntypedValue) -> UntypedValue,
    {
        let rhs = self.pop();
        let lhs = self.last();
        *self.last_mut() = f(lhs, rhs);
    }

    /// Evaluates the given fallible closure `f` for the 2 top most stack values.
    ///
    /// # Errors
    ///
    /// If the closure execution fails.
    pub fn try_eval_top2<F>(&mut self, f: F) -> Result<(), TrapCode>
    where
        F: FnOnce(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    {
        let rhs = self.pop();
        let lhs = self.last();
        *self.last_mut() = f(lhs, rhs)?;
        Ok(())
    }

    pub fn push<T>(&mut self, entry: T)
    where
        T: Into<UntypedValue>,
    {
        *self.get_release_unchecked_mut(self.stack_ptr) = entry.into();
        self.stack_ptr += 1;
    }
}
