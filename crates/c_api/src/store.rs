use crate::wasm_engine_t;
use alloc::{boxed::Box, sync::Arc};
use core::cell::UnsafeCell;
use wasmi::{AsContext, AsContextMut, Store, StoreContext, StoreContextMut};

/// This representation of a `Store` is used to implement the `wasm.h` API (and
/// *not* the `wasmtime.h` API!)
///
/// This is stored alongside `Func` and such for `wasm.h` so each object is
/// independently owned. The usage of `Arc` here is mostly to just get it to be
/// safe to drop across multiple threads, but otherwise acquiring the `context`
/// values from this struct is considered unsafe due to it being unknown how the
/// aliasing is working on the C side of things.
///
/// The aliasing requirements are documented in the C API `wasm.h` itself (at
/// least Wasmi's implementation).
#[derive(Clone)]
pub struct WasmStoreRef {
    inner: Arc<UnsafeCell<Store<()>>>,
}

impl WasmStoreRef {
    /// Returns shared access to the store context of the [`WasmStoreRef`].
    ///
    /// Wraps [`wasmi::AsContext`].
    pub unsafe fn context(&self) -> StoreContext<'_, ()> {
        (*self.inner.get()).as_context()
    }

    /// Returns mutable access to the store context of the [`WasmStoreRef`].
    ///
    /// Wraps [`wasmi::AsContextMut`].
    pub unsafe fn context_mut(&mut self) -> StoreContextMut<'_, ()> {
        (*self.inner.get()).as_context_mut()
    }
}

/// The Wasm store.
///
/// Wraps [`wasmi::Store<()>`](wasmi::Store).
#[repr(C)]
#[derive(Clone)]
pub struct wasm_store_t {
    pub(crate) inner: WasmStoreRef,
}

wasmtime_c_api_macros::declare_own!(wasm_store_t);

/// Creates a new [`Store<()>`](wasmi::Store) for the given `engine`.
///
/// The returned [`wasm_store_t`] must be freed using [`wasm_store_delete`].
///
/// Wraps [`<wasmi::Store<()>>::new`](wasmi::Store::new).
#[no_mangle]
pub extern "C" fn wasm_store_new(engine: &wasm_engine_t) -> Box<wasm_store_t> {
    let engine = &engine.inner;
    let store = Store::new(engine, ());
    Box::new(wasm_store_t {
        inner: WasmStoreRef {
            inner: Arc::new(UnsafeCell::new(store)),
        },
    })
}
