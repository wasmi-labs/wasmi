use super::{AsContext, AsContextMut, Stored};
use crate::{
    GlobalType,
    Mutability,
    Val,
    collections::arena::ArenaIndex,
    core::CoreGlobal,
    errors::GlobalError,
};

/// A raw index to a global variable entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalIdx(u32);

impl ArenaIndex for GlobalIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as global index: {error}")
        });
        Self(value)
    }
}

/// A Wasm global variable reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Global(Stored<GlobalIdx>);

impl Global {
    /// Creates a new stored global variable reference.
    ///
    /// # Note
    ///
    /// This API is primarily used by the [`Store`] itself.
    ///
    /// [`Store`]: [`crate::Store`]
    pub(super) fn from_inner(stored: Stored<GlobalIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn as_inner(&self) -> &Stored<GlobalIdx> {
        &self.0
    }

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
