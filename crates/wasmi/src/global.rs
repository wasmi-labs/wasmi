use crate::{
    AsContext,
    AsContextMut,
    GlobalType,
    Mutability,
    Val,
    core::CoreGlobal,
    errors::GlobalError,
    store::Stored,
};

define_handle! {
    /// A Wasm global variable reference.
    struct Global(u32, Stored) => CoreGlobal;
}

impl Global {
    /// Creates a new global variable to the store.
    pub fn new(mut ctx: impl AsContextMut, initial_value: Val, mutability: Mutability) -> Self {
        ctx.as_context_mut()
            .store
            .inner
            .alloc_global(CoreGlobal::new(initial_value.into(), mutability))
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
        ctx.as_context()
            .store
            .inner
            .resolve_global(self)
            .get()
            .into()
    }
}
