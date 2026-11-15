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
        let instance = self.instance?;
        get_export_of(instance, name)
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

/// Returns the export of `instance` under `name` if any.
///
/// # Note
///
/// This is deliberately neither generic nor inlinable so that it is always compiled as part of
/// Wasmi, never inside the generic [`Caller::get_export`] that a downstream crate instantiates.
///
/// On `x86_64` the [`Inst`] is an `f64` and is turned back into a pointer here. LLVM's FastISel,
/// which handles code built at `opt-level = 0`, folds that `f64`-to-pointer cast into the load
/// address and ends up using an XMM register as the base register. The emitted load then reads
/// through whatever general purpose register shares the XMM register's number. This crashed
/// embedders built at `opt-level = 0` against an optimized Wasmi.
///
/// See <https://github.com/wasmi-labs/wasmi/issues/2061>.
#[inline(never)]
fn get_export_of(instance: Inst, name: &str) -> Option<Extern> {
    // Safety: an `Inst` held by a `Caller` refers to a live instance of the caller's store.
    let instance = unsafe { instance.as_ptr().as_ref() };
    instance.get_export(name)
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
