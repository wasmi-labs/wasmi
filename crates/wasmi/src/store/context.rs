use crate::{Engine, Error, Store};

/// A trait used to get shared access to a [`Store`] in Wasmi.
pub trait AsContext {
    /// The user state associated with the [`Store`], aka the `T` in `Store<T>`.
    type Data;

    /// Returns the store context that this type provides access to.
    fn as_context(&self) -> StoreContext<Self::Data>;
}

/// A trait used to get exclusive access to a [`Store`] in Wasmi.
pub trait AsContextMut: AsContext {
    /// Returns the store context that this type provides access to.
    fn as_context_mut(&mut self) -> StoreContextMut<Self::Data>;
}

/// A temporary handle to a [`&Store<T>`][`Store`].
///
/// This type is suitable for [`AsContext`] trait bounds on methods if desired.
/// For more information, see [`Store`].
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct StoreContext<'a, T> {
    pub(crate) store: &'a Store<T>,
}

impl<T> StoreContext<'_, T> {
    /// Returns the underlying [`Engine`] this store is connected to.
    pub fn engine(&self) -> &Engine {
        self.store.engine()
    }

    /// Access the underlying data owned by this store.
    ///
    /// Same as [`Store::data`].
    pub fn data(&self) -> &T {
        self.store.data()
    }

    /// Returns the remaining fuel of the [`Store`] if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::get_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.store.get_fuel()
    }
}

impl<'a, T: AsContext> From<&'a T> for StoreContext<'a, T::Data> {
    #[inline]
    fn from(ctx: &'a T) -> Self {
        ctx.as_context()
    }
}

impl<'a, T: AsContext> From<&'a mut T> for StoreContext<'a, T::Data> {
    #[inline]
    fn from(ctx: &'a mut T) -> Self {
        T::as_context(ctx)
    }
}

impl<'a, T: AsContextMut> From<&'a mut T> for StoreContextMut<'a, T::Data> {
    #[inline]
    fn from(ctx: &'a mut T) -> Self {
        ctx.as_context_mut()
    }
}

/// A temporary handle to a [`&mut Store<T>`][`Store`].
///
/// This type is suitable for [`AsContextMut`] or [`AsContext`] trait bounds on methods if desired.
/// For more information, see [`Store`].
#[derive(Debug)]
#[repr(transparent)]
pub struct StoreContextMut<'a, T> {
    pub(crate) store: &'a mut Store<T>,
}

impl<T> StoreContextMut<'_, T> {
    /// Returns the underlying [`Engine`] this store is connected to.
    pub fn engine(&self) -> &Engine {
        self.store.engine()
    }

    /// Access the underlying data owned by this store.
    ///
    /// Same as [`Store::data`].
    pub fn data(&self) -> &T {
        self.store.data()
    }

    /// Access the underlying data owned by this store.
    ///
    /// Same as [`Store::data_mut`].
    pub fn data_mut(&mut self) -> &mut T {
        self.store.data_mut()
    }

    /// Returns the remaining fuel of the [`Store`] if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::get_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.store.get_fuel()
    }

    /// Sets the remaining fuel of the [`Store`] to `value` if fuel metering is enabled.
    ///
    /// For more information see [`Store::get_fuel`](crate::Store::set_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), Error> {
        self.store.set_fuel(fuel)
    }
}

impl<T> AsContext for &'_ T
where
    T: AsContext,
{
    type Data = T::Data;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, T::Data> {
        T::as_context(*self)
    }
}

impl<T> AsContext for &'_ mut T
where
    T: AsContext,
{
    type Data = T::Data;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, T::Data> {
        T::as_context(*self)
    }
}

impl<T> AsContextMut for &'_ mut T
where
    T: AsContextMut,
{
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, T::Data> {
        T::as_context_mut(*self)
    }
}

impl<T> AsContext for StoreContext<'_, T> {
    type Data = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        StoreContext { store: self.store }
    }
}

impl<T> AsContext for StoreContextMut<'_, T> {
    type Data = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        StoreContext { store: self.store }
    }
}

impl<T> AsContextMut for StoreContextMut<'_, T> {
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::Data> {
        StoreContextMut {
            store: &mut *self.store,
        }
    }
}

impl<T> AsContext for Store<T> {
    type Data = T;

    #[inline]
    fn as_context(&self) -> StoreContext<'_, Self::Data> {
        StoreContext { store: self }
    }
}

impl<T> AsContextMut for Store<T> {
    #[inline]
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::Data> {
        StoreContextMut { store: self }
    }
}
