use super::super::{AsContext, AsContextMut, StoreContext, StoreContextMut};

/// Represents the caller’s context when creating a host function via [`Func::wrap`].
pub struct Caller<'a, T> {
    pub(crate) store: StoreContextMut<'a, T>,
    // TODO: add instance handle
}

impl<T> AsContext for Caller<'_, T> {
    type UserState = T;

    fn as_context(&self) -> StoreContext<'_, Self::UserState> {
        self.store.as_context()
    }
}

impl<T> AsContextMut for Caller<'_, T> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::UserState> {
        self.store.as_context_mut()
    }
}

impl<'a, T: AsContextMut> From<&'a mut T> for Caller<'a, T::UserState> {
    fn from(ctx: &'a mut T) -> Self {
        Self {
            store: ctx.as_context_mut(),
        }
    }
}
