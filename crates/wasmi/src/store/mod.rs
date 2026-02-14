mod context;
mod error;
mod id;
mod inner;
mod pruned;
mod typeid;

pub(crate) use self::id::AsStoreId;
use self::pruned::PrunedStoreVTable;
pub use self::{
    context::{AsContext, AsContextMut, StoreContext, StoreContextMut},
    error::{InternalStoreError, StoreError},
    id::Stored,
    inner::StoreInner,
    pruned::PrunedStore,
};
use crate::{
    Engine,
    Error,
    Handle,
    Memory,
    RawHandle,
    ResourceLimiter,
    collections::arena::{Arena, ArenaError},
    core::{CoreMemory, ResourceLimiterRef},
    engine::{InOutParams, Inst},
    func::{Trampoline, TrampolineEntity},
};
use alloc::boxed::Box;
use core::{
    any::{TypeId, type_name},
    fmt::{self, Debug},
};

/// The store that owns all data associated to Wasm modules.
#[derive(Debug)]
pub struct Store<T> {
    /// All data that is not associated to `T`.
    ///
    /// # Note
    ///
    /// This is re-exported to the rest of the crate since
    /// it is used directly by the engine's executor.
    pub(crate) inner: StoreInner,
    /// The inner parts of the [`Store`] that are generic over a host provided `T`.
    typed: TypedStoreInner<T>,
    /// The [`TypeId`] of the `T` of the `store`.
    ///
    /// This is used in [`PrunedStore::restore`] to check if the
    /// restored `T` matches the original `T` of the `store`.
    id: TypeId,
    /// Used to restore a [`PrunedStore`] to a [`Store<T>`].
    restore_pruned: PrunedStoreVTable,
}

impl<T> Default for Store<T>
where
    T: Default,
{
    fn default() -> Self {
        let engine = Engine::default();
        Self::new(&engine, T::default())
    }
}

impl<T> Store<T> {
    /// Creates a new store.
    pub fn new(engine: &Engine, data: T) -> Self {
        Self {
            inner: StoreInner::new(engine),
            typed: TypedStoreInner::new(data),
            id: typeid::of::<T>(),
            restore_pruned: PrunedStoreVTable::new::<T>(),
        }
    }
}

impl<T> Store<T> {
    /// Returns the [`Engine`] that this store is associated with.
    pub fn engine(&self) -> &Engine {
        self.inner.engine()
    }

    /// Returns a shared reference to the user provided data owned by this [`Store`].
    pub fn data(&self) -> &T {
        &self.typed.data
    }

    /// Returns an exclusive reference to the user provided data owned by this [`Store`].
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.typed.data
    }

    /// Consumes `self` and returns its user provided data.
    pub fn into_data(self) -> T {
        *self.typed.data
    }

    /// Installs a function into the [`Store`] that will be called with the user
    /// data type `T` to retrieve a [`ResourceLimiter`] any time a limited,
    /// growable resource such as a linear memory or table is grown.
    pub fn limiter(
        &mut self,
        limiter: impl (FnMut(&mut T) -> &mut dyn ResourceLimiter) + Send + Sync + 'static,
    ) {
        self.typed.limiter = Some(ResourceLimiterQuery(Box::new(limiter)))
    }

    /// Calls the host function with the `params` and `results` on `instance`.
    ///
    /// # Errors
    ///
    /// If the called host function returned an error.
    fn call_host_func(
        &mut self,
        trampoline: Trampoline,
        instance: Option<Inst>,
        inout: InOutParams,
    ) -> Result<(), StoreError<Error>> {
        let trampoline = self.resolve_trampoline(&trampoline)?.clone();
        trampoline
            .call(self, instance, inout)
            .map_err(StoreError::external)?;
        Ok(())
    }

    /// Returns `true` if it is possible to create `additional` more instances in the [`Store`].
    pub(crate) fn can_create_more_instances(&mut self, additional: usize) -> bool {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.len_instances().saturating_add(additional) > limiter.instances() {
                return false;
            }
        }
        true
    }

    /// Returns `true` if it is possible to create `additional` more linear memories in the [`Store`].
    pub(crate) fn can_create_more_memories(&mut self, additional: usize) -> bool {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.len_memories().saturating_add(additional) > limiter.memories() {
                return false;
            }
        }
        true
    }

    /// Returns `true` if it is possible to create `additional` more tables in the [`Store`].
    pub(crate) fn can_create_more_tables(&mut self, additional: usize) -> bool {
        let (inner, mut limiter) = self.store_inner_and_resource_limiter_ref();
        if let Some(limiter) = limiter.as_resource_limiter() {
            if inner.len_tables().saturating_add(additional) > limiter.tables() {
                return false;
            }
        }
        true
    }

    /// Returns a pair of [`StoreInner`] and [`ResourceLimiterRef`].
    ///
    /// # Note
    ///
    /// This methods mostly exists to satisfy certain use cases that otherwise would conflict with the borrow checker.
    pub(crate) fn store_inner_and_resource_limiter_ref(
        &mut self,
    ) -> (&mut StoreInner, ResourceLimiterRef<'_>) {
        let resource_limiter = match &mut self.typed.limiter {
            Some(query) => {
                let limiter = query.0(&mut self.typed.data);
                ResourceLimiterRef::from(limiter)
            }
            None => ResourceLimiterRef::default(),
        };
        (&mut self.inner, resource_limiter)
    }

    /// Returns the remaining fuel of the [`Store`] if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// Enable fuel metering via [`Config::consume_fuel`](crate::Config::consume_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.inner.get_fuel()
    }

    /// Sets the remaining fuel of the [`Store`] to `value` if fuel metering is enabled.
    ///
    /// # Note
    ///
    /// Enable fuel metering via [`Config::consume_fuel`](crate::Config::consume_fuel).
    ///
    /// # Errors
    ///
    /// If fuel metering is disabled.
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), Error> {
        self.inner.set_fuel(fuel)
    }
}

/// Generically handle an [`ArenaError`] given a contextual message.
#[cold]
fn handle_arena_err(err: ArenaError, context: &str) -> ! {
    panic!("{context}: {err}")
}

impl<T> Store<T> {
    /// Allocates a new [`TrampolineEntity`] and returns a [`Trampoline`] reference to it.
    pub(super) fn alloc_trampoline(&mut self, value: TrampolineEntity<T>) -> Trampoline {
        let key = match self.typed.trampolines.alloc(value) {
            Ok(key) => key,
            Err(err) => handle_arena_err(err, "alloc host func trampoline"),
        };
        Trampoline::from(self.inner.id().wrap(key))
    }

    /// Returns an exclusive reference to the [`CoreMemory`] associated to the given [`Memory`]
    /// and an exclusive reference to the user provided host state.
    ///
    /// # Note
    ///
    /// This method exists to properly handle use cases where
    /// otherwise the Rust borrow-checker would not accept.
    ///
    /// # Panics
    ///
    /// - If the [`Memory`] does not originate from this [`Store`].
    /// - If the [`Memory`] cannot be resolved to its entity.
    pub(super) fn resolve_memory_and_state_mut(
        &mut self,
        memory: &Memory,
    ) -> (&mut CoreMemory, &mut T) {
        (self.inner.resolve_memory_mut(memory), &mut self.typed.data)
    }

    /// Returns a shared reference to the associated entity of the host function trampoline.
    ///
    /// # Panics
    ///
    /// - If the [`Trampoline`] does not originate from this [`Store`].
    /// - If the [`Trampoline`] cannot be resolved to its entity.
    fn resolve_trampoline(
        &self,
        key: &Trampoline,
    ) -> Result<&TrampolineEntity<T>, InternalStoreError> {
        let raw_key = self.inner.unwrap_stored(key.raw())?;
        let Ok(trampoline) = self.typed.trampolines.get(*raw_key) else {
            return Err(InternalStoreError::not_found());
        };
        Ok(trampoline)
    }

    /// Sets a callback function that is executed whenever a WebAssembly
    /// function is called from the host or a host function is called from
    /// WebAssembly, or these functions return.
    ///
    /// The function is passed a `&mut T` to the underlying store, and a
    /// [`CallHook`]. [`CallHook`] can be used to find out what kind of function
    /// is being called or returned from.
    ///
    /// The callback can either return `Ok(())` or an `Err` with an
    /// [`Error`]. If an error is returned, it is returned to the host
    /// caller. If there are nested calls, only the most recent host caller
    /// receives the error and it is not propagated further automatically. The
    /// hook may be invoked again as new functions are called and returned from.
    pub fn call_hook(
        &mut self,
        hook: impl FnMut(&mut T, CallHook) -> Result<(), Error> + Send + Sync + 'static,
    ) {
        self.typed.call_hook = Some(CallHookWrapper(Box::new(hook)));
    }

    /// Executes the callback set by [`Store::call_hook`] if any has been set.
    ///
    /// # Note
    ///
    /// - Returns the value returned by the call hook.
    /// - Returns `Ok(())` if no call hook exists.
    #[inline]
    pub(crate) fn invoke_call_hook(&mut self, call_type: CallHook) -> Result<(), Error> {
        match self.typed.call_hook.as_mut() {
            None => Ok(()),
            Some(call_hook) => {
                Self::invoke_call_hook_impl(&mut self.typed.data, call_type, call_hook)
            }
        }
    }

    /// Utility function to invoke the [`Store::call_hook`] that is asserted to
    /// be available in this case.
    ///
    /// This is kept as a separate `#[cold]` function to help the compiler speed
    /// up the code path without any call hooks.
    #[cold]
    fn invoke_call_hook_impl(
        data: &mut T,
        call_type: CallHook,
        call_hook: &mut CallHookWrapper<T>,
    ) -> Result<(), Error> {
        call_hook.0(data, call_type)
    }
}

/// The inner parts of the [`Store`] which are generic over a host provided `T`.
#[derive(Debug)]
pub struct TypedStoreInner<T> {
    /// Stored host function trampolines.
    trampolines: Arena<RawHandle<Trampoline>, TrampolineEntity<T>>,
    /// User provided hook to retrieve a [`ResourceLimiter`].
    limiter: Option<ResourceLimiterQuery<T>>,
    /// User provided callback called when a host calls a WebAssembly function
    /// or a WebAssembly function calls a host function, or these functions
    /// return.
    call_hook: Option<CallHookWrapper<T>>,
    /// User provided host data owned by the [`Store`].
    data: Box<T>,
}

impl<T> TypedStoreInner<T> {
    /// Creates a new [`TypedStoreInner`] from the given data of type `T`.
    fn new(data: T) -> Self {
        Self {
            trampolines: Arena::new(),
            data: Box::new(data),
            limiter: None,
            call_hook: None,
        }
    }
}

/// A wrapper around a boxed `dyn FnMut(&mut T)` returning a `&mut dyn`
/// [`ResourceLimiter`]; in other words a function that one can call to retrieve
/// a [`ResourceLimiter`] from the [`Store`] object's user data type `T`.
///
/// This wrapper exists both to make types a little easier to read and to
/// provide a `Debug` impl so that `#[derive(Debug)]` works on structs that
/// contain it.
struct ResourceLimiterQuery<T>(Box<dyn (FnMut(&mut T) -> &mut dyn ResourceLimiter) + Send + Sync>);
impl<T> Debug for ResourceLimiterQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceLimiterQuery<{}>(...)", type_name::<T>())
    }
}

/// A wrapper used to store hooks added with [`Store::call_hook`], containing a
/// boxed `FnMut(&mut T, CallHook) -> Result<(), Error>`.
///
/// This wrapper exists to provide a `Debug` impl so that `#[derive(Debug)]`
/// works for [`Store`].
#[allow(clippy::type_complexity)]
struct CallHookWrapper<T>(Box<dyn FnMut(&mut T, CallHook) -> Result<(), Error> + Send + Sync>);
impl<T> Debug for CallHookWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CallHook<{}>", type_name::<T>())
    }
}

/// Argument to the callback set by [`Store::call_hook`] to indicate why the
/// callback was invoked.
#[derive(Debug)]
pub enum CallHook {
    /// Indicates that a WebAssembly function is being called from the host.
    CallingWasm,
    /// Indicates that a WebAssembly function called from the host is returning.
    ReturningFromWasm,
    /// Indicates that a host function is being called from a WebAssembly function.
    CallingHost,
    /// Indicates that a host function called from a WebAssembly function is returning.
    ReturningFromHost,
}

/// The call hook behavior when calling a host function.
#[derive(Debug, Copy, Clone)]
pub enum CallHooks {
    /// Invoke the host call hooks.
    Call,
    /// Ignore the host call hooks.
    Ignore,
}

#[test]
fn test_store_is_send_sync() {
    const _: () = {
        #[allow(clippy::extra_unused_type_parameters)]
        fn assert_send<T: Send>() {}
        #[allow(clippy::extra_unused_type_parameters)]
        fn assert_sync<T: Sync>() {}
        let _ = assert_send::<Store<()>>;
        let _ = assert_sync::<Store<()>>;
    };
}
