use super::super::{AsContext, AsContextMut, StoreContext, StoreContextMut};

pub struct Caller<'a, T> {
    pub(crate) store: StoreContextMut<'a, T>,
    // caller: &'a InstanceHandle,
}

impl<'a, T> AsContext for Caller<'a, T> {
    type UserState = T;

    fn as_context(&self) -> StoreContext<'_, Self::UserState> {
        self.store.as_context()
    }
}

impl<'a, T> AsContextMut for Caller<'a, T> {
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
