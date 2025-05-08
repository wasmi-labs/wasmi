use super::{CallHooks, FuncInOut, HostFuncEntity, ResourceLimiterRef, StoreInner};
use crate::{core::hint, CallHook, Error, Instance, Store};
use core::{
    any::{type_name, TypeId},
    fmt::{self, Debug},
    mem,
};

#[cfg(test)]
use crate::Engine;

/// A wrapper used to restore a [`PrunedStore`].
///
/// This wrapper exists to provide a `Debug` impl so that `#[derive(Debug)]`
/// works for [`Store`].
#[allow(clippy::type_complexity)]
#[derive(Copy, Clone)]
pub struct RestorePrunedWrapper {
    /// Calls the given [`HostFuncEntity`] with the `params` and `results` on `instance`.
    ///
    /// # Errors
    ///
    /// If the called host function returned an error.
    call_host_func: fn(
        /* store:         */ &mut PrunedStore,
        /* func:          */ &HostFuncEntity,
        /* instance:      */ Option<&Instance>,
        /* params_results:*/ FuncInOut,
        /* call_hooks:    */ CallHooks,
    ) -> Result<(), Error>,
    /// Returns an exclusive reference to [`StoreInner`] and a [`ResourceLimiterRef`].
    store_inner_and_resource_limiter_ref:
        fn(&mut PrunedStore) -> (&mut StoreInner, ResourceLimiterRef<'_>),
}
impl RestorePrunedWrapper {
    pub fn new<T: 'static>() -> Self {
        Self {
            call_host_func: |pruned: &mut PrunedStore,
                             func: &HostFuncEntity,
                             instance: Option<&Instance>,
                             params_results: FuncInOut,
                             call_hooks: CallHooks|
             -> Result<(), Error> {
                let store: &mut Store<T> = pruned.restore_or_panic();
                if matches!(call_hooks, CallHooks::Call) {
                    store.invoke_call_hook(CallHook::CallingHost)?;
                }
                store.call_host_func(func, instance, params_results)?;
                if matches!(call_hooks, CallHooks::Call) {
                    store.invoke_call_hook(CallHook::ReturningFromHost)?;
                }
                Ok(())
            },
            store_inner_and_resource_limiter_ref: |pruned: &mut PrunedStore| {
                pruned
                    .restore_or_panic::<T>()
                    .store_inner_and_resource_limiter_ref()
            },
        }
    }
}
impl RestorePrunedWrapper {
    #[inline]
    fn call_host_func(
        &self,
        pruned: &mut PrunedStore,
        func: &HostFuncEntity,
        instance: Option<&Instance>,
        params_results: FuncInOut,
        call_hooks: CallHooks,
    ) -> Result<(), Error> {
        (self.call_host_func)(pruned, func, instance, params_results, call_hooks)
    }

    #[inline]
    fn store_inner_and_resource_limiter_ref<'a>(
        &self,
        pruned: &'a mut PrunedStore,
    ) -> (&'a mut StoreInner, ResourceLimiterRef<'a>) {
        (self.store_inner_and_resource_limiter_ref)(pruned)
    }
}
impl Debug for RestorePrunedWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RestorePrunedWrapper")
    }
}

/// A [`Store`] with a pruned `T`.
#[derive(Debug)]
#[repr(transparent)]
pub struct PrunedStore {
    /// The underlying [`Store`] with pruned type signature.
    pruned: Store<Pruned>,
}

/// Placeholder type of `T` for a pruned `Store<T>`.
#[derive(Debug)]
pub struct Pruned;

impl<'a, T> From<&'a mut Store<T>> for &'a mut PrunedStore {
    #[inline]
    fn from(store: &'a mut Store<T>) -> Self {
        // Safety: the generic `Store<T>` has its `T` pruned here.
        //
        // - This is safe because we are operating on a `&mut Store<T>` thus it is just
        //   a reference and since `Store<T>` and `Store<Pruned>` have the same size and API.
        // - We make sure in `PrunedStore` to never access the typed parts of the original
        //   `Store<T>` and check in the restoration process the type-ID of the target `T`.
        // - `Store<T>` has the same size and alignment for all `T`.
        unsafe { mem::transmute::<&'a mut Store<T>, &'a mut PrunedStore>(store) }
    }
}

impl<T> Store<T> {
    /// Prune the [`Store`] from `T` returning a [`PrunedStore`].
    #[inline]
    pub(crate) fn prune(&mut self) -> &mut PrunedStore {
        self.into()
    }
}

impl PrunedStore {
    /// Returns a shared reference to the underlying [`StoreInner`].
    #[inline]
    pub fn inner(&self) -> &StoreInner {
        &self.pruned.inner
    }

    /// Returns an exclusive reference to the underlying [`StoreInner`].
    #[inline]
    pub fn inner_mut(&mut self) -> &mut StoreInner {
        &mut self.pruned.inner
    }

    /// Calls a host `func` at `instance` with `params_results` buffer.
    ///
    /// # Errors
    ///
    /// If the host function returns an error.
    #[inline]
    pub fn call_host_func(
        &mut self,
        func: &HostFuncEntity,
        instance: Option<&Instance>,
        params_results: FuncInOut,
        call_hooks: CallHooks,
    ) -> Result<(), Error> {
        self.pruned.restore_pruned.clone().call_host_func(
            self,
            func,
            instance,
            params_results,
            call_hooks,
        )
    }

    /// Returns an exclusive reference to [`StoreInner`] and a [`ResourceLimiterRef`].
    #[inline]
    pub fn store_inner_and_resource_limiter_ref(
        &mut self,
    ) -> (&mut StoreInner, ResourceLimiterRef) {
        self.pruned
            .restore_pruned
            .clone()
            .store_inner_and_resource_limiter_ref(self)
    }

    /// Restores `self` to a proper [`Store<T>`] if possible.
    ///
    /// # Panics
    ///
    /// If the `T` of the resulting [`Store<T>`] does not match the given `T`.
    fn restore_or_panic<T: 'static>(&mut self) -> &mut Store<T> {
        let Ok(store) = self.restore() else {
            panic!(
                "failed to convert `PrunedStore` back into `Store<{}>`",
                type_name::<T>(),
            );
        };
        store
    }

    /// Restores `self` to a proper [`Store<T>`] if possible.
    ///
    /// # Errors
    ///
    /// If the `T` of the resulting [`Store<T>`] does not match the given `T`.
    #[inline]
    fn restore<T: 'static>(&mut self) -> Result<&mut Store<T>, PrunedStoreError> {
        if hint::unlikely(TypeId::of::<T>() != self.pruned.id) {
            return Err(PrunedStoreError);
        }
        let store = {
            // Safety: we restore the original `Store<T>` from the pruned `Store<Pruned>`.
            //
            // This is safe because we have already checked above that the `TypedId` of `T`
            // matches the `id` of the original `Store<T>` and thus the `T`'s are identical.
            //
            // Furthermore, we are only operating on `&mut` pointers and not values.
            // Finally, `Store<T>` has the same size and alignment for all `T`.
            unsafe { mem::transmute::<&mut PrunedStore, &mut Store<T>>(self) }
        };
        Ok(store)
    }
}

/// Returned when [`PrunedStore::restore`] failed.
#[derive(Debug)]
pub struct PrunedStoreError;

#[test]
fn pruning_works() {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let pruned = store.prune();
    assert!(pruned.restore::<()>().is_ok());
}

#[test]
fn pruning_errors() {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let pruned = store.prune();
    assert!(pruned.restore::<i32>().is_err());
}

#[test]
fn pruned_store_deref() {
    let mut config = crate::Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    let mut store = Store::new(&engine, ());
    let fuel_amount = 100;
    store.set_fuel(fuel_amount).unwrap();
    let pruned = store.prune();
    assert_eq!(
        PrunedStore::inner(pruned).fuel.get_fuel().unwrap(),
        fuel_amount
    );
    PrunedStore::inner_mut(pruned)
        .fuel
        .set_fuel(fuel_amount * 2)
        .unwrap();
    assert_eq!(
        PrunedStore::inner(pruned).fuel.get_fuel().unwrap(),
        fuel_amount * 2
    );
}

#[test]
fn equal_size() {
    use super::TypedStoreInner;
    type SmallType = ();
    type BigType = [i64; 16];
    // Note: `TypedStore<T>` must be of the same size for all `T` so that
    //       `PrunedStore` works and is a safe abstraction.
    use core::mem::size_of;
    assert_eq!(size_of::<Store<SmallType>>(), size_of::<Store<BigType>>(),);
    assert_eq!(
        size_of::<TypedStoreInner<SmallType>>(),
        size_of::<TypedStoreInner<BigType>>(),
    );
}
