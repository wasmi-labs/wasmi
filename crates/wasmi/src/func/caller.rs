use super::super::{AsContext, AsContextMut, StoreContext, StoreContextMut};
use crate::{Engine, Error, Extern, engine::Inst};

/// Represents the caller’s context when creating a host function via [`Func::wrap`].
///
/// [`Func::wrap`]: struct.Func.html#method.wrap
pub struct Caller<'a, T> {
    ctx: StoreContextMut<'a, T>,
    /// The module instance associated to the call.
    /// This is `Some` if the host function was called from a Wasm function
    /// since all Wasm function are associated to a module instance.
    /// This usually is `None` if the host function was called from the host side.
    instance: Option<Inst>,
}

impl<'a, T> Caller<'a, T> {
    /// Creates a new [`Caller`] from the given store context and [`Instance`] handle.
    ///
    /// [`Instance`]: crate::Instance
    pub(crate) fn new<C>(ctx: &'a mut C, instance: Option<Inst>) -> Self
    where
        C: AsContextMut<Data = T>,
    {
        Self {
            ctx: ctx.as_context_mut(),
            instance,
        }
    }

    /// Queries the caller for an exported definition identifier by `name`.
    ///
    /// Returns `None` if there is no associated [`Instance`] of the caller
    /// or if the caller does not provide an export under the name `name`.
    ///
    /// [`Instance`]: crate::Instance
    pub fn get_export(&self, name: &str) -> Option<Extern> {
        let Some(instance) = &self.instance else {
            return None;
        };
        let instance = unsafe { instance.as_ref() };
        instance.get_export(name)
    }

    /// Returns a shared reference to the user provided host data.
    pub fn data(&self) -> &T {
        self.ctx.store.data()
    }

    /// Returns an exclusive reference to the user provided host data.
    pub fn data_mut(&mut self) -> &mut T {
        self.ctx.store.data_mut()
    }

    /// Returns a shared reference to the used [`Engine`].
    pub fn engine(&self) -> &Engine {
        self.ctx.store.engine()
    }

    /// Returns the remaining fuel of the [`Store`](crate::Store) if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::get_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.ctx.store.get_fuel()
    }

    /// Sets the remaining fuel of the [`Store`](crate::Store) to `value` if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::set_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), Error> {
        self.ctx.store.set_fuel(fuel)
    }
}

impl<T> AsContext for Caller<'_, T> {
    type Data = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        self.ctx.as_context()
    }
}

impl<T> AsContextMut for Caller<'_, T> {
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::Data> {
        self.ctx.as_context_mut()
    }
}

impl<'a, T: AsContextMut> From<&'a mut T> for Caller<'a, T::Data> {
    #[inline]
    fn from(ctx: &'a mut T) -> Self {
        Self {
            ctx: ctx.as_context_mut(),
            instance: None,
        }
    }
}
