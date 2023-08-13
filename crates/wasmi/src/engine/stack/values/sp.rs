use crate::{
    core::{TrapCode, UntypedValue},
    engine::DropKeep,
};

/// A pointer on the [`ValueStack`].
///
/// Allows for efficient mutable access to the values of the [`ValueStack`].
///
/// [`ValueStack`]: super::ValueStack
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
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
    #[inline]
    pub fn offset_from(self, other: Self) -> isize {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and `wasmi` codegen to never run out
        //         of valid bounds using this method.
        unsafe { self.ptr.offset_from(other.ptr) }
    }

    /// Returns the [`UntypedValue`] at the current stack pointer.
    #[must_use]
    #[inline]
    fn get(self) -> UntypedValue {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and `wasmi` codegen to never run out
        //         of valid bounds using this method.
        unsafe { *self.ptr }
    }

    /// Writes `value` to the cell pointed at by [`ValueStackPtr`].
    #[inline]
    fn set(self, value: UntypedValue) {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and `wasmi` codegen to never run out
        //         of valid bounds using this method.
        *unsafe { &mut *self.ptr } = value;
    }

    /// Returns a [`ValueStackPtr`] with a pointer value increased by `delta`.
    ///
    /// # Note
    ///
    /// The amount of `delta` is in number of bytes per [`UntypedValue`].
    #[must_use]
    #[inline]
    pub fn into_add(mut self, delta: usize) -> Self {
        self.inc_by(delta);
        self
    }

    /// Returns a [`ValueStackPtr`] with a pointer value decreased by `delta`.
    ///
    /// # Note
    ///
    /// The amount of `delta` is in number of bytes per [`UntypedValue`].
    #[must_use]
    #[inline]
    pub fn into_sub(mut self, delta: usize) -> Self {
        self.dec_by(delta);
        self
    }

    /// Returns the last [`UntypedValue`] on the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This has the same effect as [`ValueStackPtr::nth_back`]`(1)`.
    ///
    /// [`ValueStack`]: super::ValueStack
    #[inline]
    #[must_use]
    pub fn last(self) -> UntypedValue {
        self.nth_back(1)
    }

    /// Peeks the entry at the given depth from the last entry.
    ///
    /// # Note
    ///
    /// Given a `depth` of 1 has the same effect as [`ValueStackPtr::last`].
    ///
    /// A `depth` of 0 is invalid and undefined.
    #[inline]
    #[must_use]
    pub fn nth_back(self, depth: usize) -> UntypedValue {
        self.into_sub(depth).get()
    }

    /// Writes `value` to the n-th [`UntypedValue`] from the back.
    ///
    /// # Note
    ///
    /// Given a `depth` of 1 has the same effect as mutating [`ValueStackPtr::last`].
    ///
    /// A `depth` of 0 is invalid and undefined.
    #[inline]
    pub fn set_nth_back(self, depth: usize, value: UntypedValue) {
        self.into_sub(depth).set(value)
    }

    /// Bumps the [`ValueStackPtr`] of `self` by one.
    #[inline]
    fn inc_by(&mut self, delta: usize) {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and `wasmi` codegen to never run out
        //         of valid bounds using this method.
        self.ptr = unsafe { self.ptr.add(delta) };
    }

    /// Decreases the [`ValueStackPtr`] of `self` by one.
    #[inline]
    fn dec_by(&mut self, delta: usize) {
        // SAFETY: Within Wasm bytecode execution we are guaranteed by
        //         Wasm validation and `wasmi` codegen to never run out
        //         of valid bounds using this method.
        self.ptr = unsafe { self.ptr.sub(delta) };
    }

    /// Pushes the `T` to the end of the [`ValueStack`].
    ///
    /// # Note
    ///
    /// - This operation heavily relies on the prior validation of
    ///   the executed WebAssembly bytecode for correctness.
    /// - Especially the stack-depth analysis during compilation with
    ///   a manual stack extension before function call prevents this
    ///   procedure from panicking.
    ///
    /// [`ValueStack`]: super::ValueStack
    #[inline]
    pub fn push_as<T>(&mut self, value: T)
    where
        T: Into<UntypedValue>,
    {
        self.push(value.into())
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
    ///
    /// [`ValueStack`]: super::ValueStack
    #[inline]
    pub fn push(&mut self, value: UntypedValue) {
        self.set(value);
        self.inc_by(1);
    }

    /// Drops the last [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    ///
    /// [`ValueStack`]: super::ValueStack
    #[inline]
    pub fn drop(&mut self) {
        self.dec_by(1);
    }

    /// Pops the last [`UntypedValue`] from the [`ValueStack`] as `T`.
    ///
    /// [`ValueStack`]: super::ValueStack
    #[inline]
    pub fn pop_as<T>(&mut self) -> T
    where
        T: From<UntypedValue>,
    {
        T::from(self.pop())
    }

    /// Pops the last [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    ///
    /// [`ValueStack`]: super::ValueStack
    #[inline]
    pub fn pop(&mut self) -> UntypedValue {
        self.dec_by(1);
        self.get()
    }

    /// Pops the last pair of [`UntypedValue`] from the [`ValueStack`].
    ///
    /// # Note
    ///
    /// This operation heavily relies on the prior validation of
    /// the executed WebAssembly bytecode for correctness.
    ///
    /// [`ValueStack`]: super::ValueStack
    #[inline]
    pub fn pop2(&mut self) -> (UntypedValue, UntypedValue) {
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
    ///
    /// [`ValueStack`]: super::ValueStack
    #[inline]
    pub fn pop3(&mut self) -> (UntypedValue, UntypedValue, UntypedValue) {
        let (snd, trd) = self.pop2();
        let fst = self.pop();
        (fst, snd, trd)
    }

    /// Evaluates the given closure `f` for the top most stack value.
    #[inline]
    pub fn eval_top<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue) -> UntypedValue,
    {
        let last = self.into_sub(1);
        last.set(f(last.get()))
    }

    /// Evaluates the given closure `f` for the 2 top most stack values.
    #[inline]
    pub fn eval_top2<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue, UntypedValue) -> UntypedValue,
    {
        let rhs = self.pop();
        let last = self.into_sub(1);
        let lhs = last.get();
        last.set(f(lhs, rhs));
    }

    /// Evaluates the given closure `f` for the 3 top most stack values.
    #[inline]
    pub fn eval_top3<F>(&mut self, f: F)
    where
        F: FnOnce(UntypedValue, UntypedValue, UntypedValue) -> UntypedValue,
    {
        let (e2, e3) = self.pop2();
        let last = self.into_sub(1);
        let e1 = last.get();
        last.set(f(e1, e2, e3));
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
        let last = self.into_sub(1);
        last.set(f(last.get())?);
        Ok(())
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
        let last = self.into_sub(1);
        let lhs = last.get();
        last.set(f(lhs, rhs)?);
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
        fn drop_keep_impl(this: ValueStackPtr, drop_keep: DropKeep) {
            let keep = drop_keep.keep();
            if keep == 0 {
                // Case: no values need to be kept.
                return;
            }
            let keep = keep as usize;
            let src = this.into_sub(keep);
            let dst = this.into_sub(keep + drop_keep.drop() as usize);
            if keep == 1 {
                // Case: only one value needs to be kept.
                dst.set(src.get());
                return;
            }
            // Case: many values need to be kept and moved on the stack.
            for i in 0..keep {
                dst.into_add(i).set(src.into_add(i).get());
            }
        }

        let drop = drop_keep.drop();
        if drop == 0 {
            // Nothing to do in this case.
            return;
        }
        drop_keep_impl(*self, drop_keep);
        self.dec_by(drop as usize);
    }
}
