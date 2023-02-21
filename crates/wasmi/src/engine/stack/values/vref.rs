use super::ValueStack;
use crate::{
    core::{TrapCode, UntypedValue},
    engine::DropKeep,
};
use core::fmt;

/// A pointer on the [`ValueStack`].
///
/// Allows for efficient mutable access to the values of the [`ValueStack`].
#[derive(Copy, Clone)]
pub struct ValueStackPtr {
    ptr: *mut UntypedValue,
}

impl From<*mut UntypedValue> for ValueStackPtr {
    #[inline]
    fn from(ptr: *mut UntypedValue) -> Self {
        Self { ptr }
    }
}

impl ValueStackPtr {
    /// Calculates the distance between two [`ValueStackPtr] in units of [`UntypedValue`].
    pub fn offset_from(self, other: Self) -> isize {
        unsafe { self.ptr.offset_from(other.ptr) }
    }

    /// Returns the [`UntypedValue`] at the current stack pointer.
    #[must_use]
    fn get(self) -> UntypedValue {
        unsafe { *self.ptr }
    }

    /// Writes `value` to the cell pointed at by [`ValueStackPtr`].
    fn set(self, value: UntypedValue) {
        unsafe { self.ptr.write(value) }
    }

    /// Returns a [`ValueStackPtr`] offset by `delta` from `self`.
    #[must_use]
    fn offset(self, delta: isize) -> Self {
        Self::from(unsafe { self.ptr.offset(delta) })
    }

    /// Returns a [`ValueStackPtr`] pointing to the n-th [`UntypedValue`] from the back.
    #[must_use]
    fn nth_back(self, delta: usize) -> Self {
        Self::from(unsafe { self.ptr.sub(delta) })
    }

    /// Returns the last [`UntypedValue`] on the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This has the same effect as [`ValueStackPtr::peek`]`(1)`.
    #[inline]
    #[must_use]
    pub fn last(self) -> UntypedValue {
        self.peek(1)
    }

    /// Returns an exclusive reference to the last [`UntypedValue`] on the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This has the same effect as [`ValueStackPtr::peek`]`(1)`.
    #[inline]
    pub fn set_last(self, value: UntypedValue) {
        self.set_nth_back(1, value)
    }

    /// Peeks the entry at the given depth from the last entry.
    ///
    /// # Note
    ///
    /// Given a `depth` of 1 has the same effect as [`ValueStackRef::last`].
    ///
    /// A `depth` of 0 is invalid and undefined.
    #[inline]
    #[must_use]
    pub fn peek(self, depth: usize) -> UntypedValue {
        self.nth_back(depth).get()
    }

    /// Writes `value` to the n-th [`UntypedValue`] from the back.
    ///
    /// # Note
    ///
    /// Given a `depth` of 1 has the same effect as [`ValueStackPtr::set_last`].
    ///
    /// A `depth` of 0 is invalid and undefined.
    #[inline]
    pub fn set_nth_back(self, depth: usize, value: UntypedValue) {
        self.nth_back(depth).set(value)
    }

    /// Bumps the [`ValueStackPtr`] of `self` by one.
    fn inc(&mut self) {
        *self = self.offset(1);
    }

    /// Decreases the [`ValueStackPtr`] of `self` by one.
    fn dec(&mut self) {
        *self = self.offset(-1);
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
    pub fn push(&mut self, value: UntypedValue) {
        self.set(value);
        self.inc();
    }

    /// Pops the last [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    #[inline]
    fn pop(&mut self) -> UntypedValue {
        self.dec();
        self.get()
    }

    /// Pops the last pair of [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    #[inline]
    fn pop2(&mut self) -> (UntypedValue, UntypedValue) {
        let rhs = self.pop();
        let lhs = self.pop();
        (lhs, rhs)
    }

    /// Pops the last triple of [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    #[inline]
    pub fn pop3(&mut self) -> (UntypedValue, UntypedValue, UntypedValue) {
        let trd = self.pop();
        let snd = self.pop();
        let fst = self.pop();
        (fst, snd, trd)
    }

    /// Evaluates the given closure `f` for the 3 top most stack values.
    #[inline]
    pub fn eval_top3<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue, UntypedValue, UntypedValue) -> UntypedValue,
    {
        let (e2, e3) = self.pop2();
        let e1 = self.last();
        self.set_last(f(e1, e2, e3));
    }

    /// Evaluates the given closure `f` for the top most stack value.
    #[inline]
    pub fn eval_top<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue) -> UntypedValue,
    {
        let top = self.last();
        self.set_last(f(top));
    }

    /// Evaluates the given fallible closure `f` for the top most stack value.
    ///
    /// # Errors
    ///
    /// If the closure execution fails.
    #[inline]
    pub fn try_eval_top<F>(&mut self, f: F) -> Result<(), TrapCode>
    where
        F: FnOnce(UntypedValue) -> Result<UntypedValue, TrapCode>,
    {
        let top = self.last();
        self.set_last(f(top)?);
        Ok(())
    }

    /// Evaluates the given closure `f` for the 2 top most stack values.
    #[inline]
    pub fn eval_top2<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue, UntypedValue) -> UntypedValue,
    {
        let rhs = self.pop();
        let lhs = self.last();
        self.set_last(f(lhs, rhs));
    }

    /// Evaluates the given fallible closure `f` for the 2 top most stack values.
    ///
    /// # Errors
    ///
    /// If the closure execution fails.
    #[inline]
    pub fn try_eval_top2<F>(&mut self, f: F) -> Result<(), TrapCode>
    where
        F: FnOnce(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    {
        let rhs = self.pop();
        let lhs = self.last();
        self.set_last(f(lhs, rhs)?);
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
        if keep == 0 {
            // Bail out early when there are no values to keep.
        } else if keep == 1 {
            // Bail out early when there is only one value to copy.
            self.nth_back(drop + 1).set(self.last());
        } else {
            // Copy kept values over to their new place on the stack.
            // Note: We cannot use `memcpy` since the slices may overlap.
            let mut src = self.nth_back(keep);
            let mut dst = self.nth_back(keep + drop);
            for _ in 0..keep {
                dst.set(src.get());
                dst.inc();
                src.inc();
            }
        }
        *self = self.nth_back(drop);
    }
}

/// A mutable view over the [`ValueStack`].
///
/// This allows for a more efficient access to the [`ValueStack`] during execution.
pub struct ValueStackRef<'a> {
    pub(super) sp: ValueStackPtr,
    pub(super) stack: &'a mut ValueStack,
}

impl<'a> fmt::Debug for ValueStackRef<'a> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "{:?}", &self.values[0..self.stack_ptr])
        todo!()
    }
}

impl<'a> ValueStackRef<'a> {
    /// Creates a new [`ValueStackRef`] from the given [`ValueStack`].
    ///
    /// This also returns an exclusive reference to the stack pointer of
    /// the underlying [`ValueStack`]. This is important in order to synchronize
    /// the [`ValueStack`] with the changes done to the [`ValueStackRef`]
    /// when necessary.
    pub fn new(stack: &'a mut ValueStack) -> Self {
        let sp = stack.stack_ptr();
        Self { sp, stack }
    }

    /// Synchronizes the original value stack pointer.
    pub fn sync(&mut self) {
        self.stack.sync_stack_ptr(self.sp);
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
        self.sp.drop_keep(drop_keep)
    }

    /// Returns the last stack entry of the [`ValueStackRef`].
    ///
    /// # Note
    ///
    /// This has the same effect as [`ValueStackRef::peek`]`(1)`.
    #[inline]
    pub fn last(&self) -> UntypedValue {
        self.sp.last()
    }

    /// Peeks the entry at the given depth from the last entry.
    ///
    /// # Note
    ///
    /// Given a `depth` of 1 has the same effect as [`ValueStackRef::last`].
    ///
    /// A `depth` of 0 is invalid and undefined.
    #[inline]
    pub fn peek(&self, depth: usize) -> UntypedValue {
        self.sp.peek(depth)
    }

    /// Peeks the `&mut` entry at the given depth from the last entry.
    ///
    /// # Note
    ///
    /// Given a `depth` of 1 has the same effect as [`ValueStackRef::last_mut`].
    ///
    /// A `depth` of 0 is invalid and undefined.
    #[inline]
    pub fn set_nth_back(&mut self, depth: usize, value: UntypedValue) {
        self.sp.set_nth_back(depth, value)
    }

    /// Pops the last [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    #[inline]
    pub fn pop(&mut self) -> UntypedValue {
        self.sp.pop()
    }

    /// Pops the last [`UntypedValue`] from the [`ValueStack`] as `T`.
    #[inline]
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
    ///   [`ValueStackRef::pop`] twice.
    /// - This operation heavily relies on the prior validation of
    ///   the executed WebAssembly bytecode for correctness.
    #[inline]
    pub fn pop2(&mut self) -> (UntypedValue, UntypedValue) {
        self.sp.pop2()
    }

    /// Pops the last triple of [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// - This operation is slightly more efficient than using
    ///   [`ValueStackRef::pop`] trice.
    /// - This operation heavily relies on the prior validation of
    ///   the executed WebAssembly bytecode for correctness.
    #[inline]
    pub fn pop3(&mut self) -> (UntypedValue, UntypedValue, UntypedValue) {
        self.sp.pop3()
    }

    /// Evaluates the given closure `f` for the 3 top most stack values.
    #[inline]
    pub fn eval_top3<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue, UntypedValue, UntypedValue) -> UntypedValue,
    {
        self.sp.eval_top3(f)
    }

    /// Evaluates the given closure `f` for the top most stack value.
    #[inline]
    pub fn eval_top<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue) -> UntypedValue,
    {
        self.sp.eval_top(f)
    }

    /// Evaluates the given fallible closure `f` for the top most stack value.
    ///
    /// # Errors
    ///
    /// If the closure execution fails.
    #[inline]
    pub fn try_eval_top<F>(&mut self, f: F) -> Result<(), TrapCode>
    where
        F: FnOnce(UntypedValue) -> Result<UntypedValue, TrapCode>,
    {
        self.sp.try_eval_top(f)
    }

    /// Evaluates the given closure `f` for the 2 top most stack values.
    #[inline]
    pub fn eval_top2<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue, UntypedValue) -> UntypedValue,
    {
        self.sp.eval_top2(f)
    }

    /// Evaluates the given fallible closure `f` for the 2 top most stack values.
    ///
    /// # Errors
    ///
    /// If the closure execution fails.
    #[inline]
    pub fn try_eval_top2<F>(&mut self, f: F) -> Result<(), TrapCode>
    where
        F: FnOnce(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    {
        self.sp.try_eval_top2(f)
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
    pub fn push<T>(&mut self, entry: T)
    where
        T: Into<UntypedValue>,
    {
        self.sp.push(entry.into())
    }
}
