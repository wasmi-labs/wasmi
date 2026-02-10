use crate::{
    AsContext,
    AsContextMut,
    GlobalType,
    Mutability,
    Val,
    core::{CoreGlobal, UntypedVal},
    errors::GlobalError,
    store::Stored,
};

define_handle! {
    /// A Wasm global variable reference.
    struct Global(u32, Stored) => CoreGlobal;
}

impl Global {
    /// Creates a new global variable to the store.
    pub fn new(mut ctx: impl AsContextMut, value: Val, mutability: Mutability) -> Self {
        let ty = GlobalType::new(value.ty(), mutability);
        let value = UntypedVal::from(value);
        ctx.as_context_mut()
            .store
            .inner
            .alloc_global(CoreGlobal::new(value, ty))
    }

    /// Returns the [`GlobalType`] of the global variable.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Global`].
    pub fn ty(&self, ctx: impl AsContext) -> GlobalType {
        ctx.as_context().store.inner.resolve_global(self).ty()
    }

    /// Sets a new value to the global variable.
    ///
    /// # Errors
    ///
    /// - If the global variable is immutable.
    /// - If there is a type mismatch between the global variable and the new value.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Global`].
    pub fn set(&self, mut ctx: impl AsContextMut, new_value: Val) -> Result<(), GlobalError> {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_global_mut(self)
            .set(new_value.into())
    }

    /// Returns the current value of the global variable.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Global`].
    pub fn get(&self, ctx: impl AsContext) -> Val {
        let store = &ctx.as_context().store.inner;
        let value = store.resolve_global(self).get();
        Val::from_raw_parts(value.untyped(), value.ty(), store)
    }
}
