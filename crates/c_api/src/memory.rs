use crate::{wasm_extern_t, wasm_memorytype_t, wasm_store_t};
use alloc::boxed::Box;
use core::hint;
use wasmi::{Extern, Memory};

/// A Wasm linear memory.
///
/// Wraps [`Memory`].
#[derive(Clone)]
#[repr(transparent)]
pub struct wasm_memory_t {
    inner: wasm_extern_t,
}

wasmi_c_api_macros::declare_ref!(wasm_memory_t);

/// Type specifying the number of pages of a Wasm linear memory.
pub type wasm_memory_pages_t = u32;

impl wasm_memory_t {
    pub(crate) fn try_from(e: &wasm_extern_t) -> Option<&wasm_memory_t> {
        match &e.which {
            Extern::Memory(_) => Some(unsafe { &*(e as *const _ as *const _) }),
            _ => None,
        }
    }

    pub(crate) fn try_from_mut(e: &mut wasm_extern_t) -> Option<&mut wasm_memory_t> {
        match &mut e.which {
            Extern::Memory(_) => Some(unsafe { &mut *(e as *mut _ as *mut _) }),
            _ => None,
        }
    }

    /// Returns the underlying [`Memory`] of the [`wasm_memory_t`].
    fn memory(&self) -> Memory {
        match self.inner.which {
            Extern::Memory(m) => m,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

/// Creates a new [`wasm_memory_t`] from the given [`wasm_memorytype_t`].
///
/// Wraps [`Memory::new`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_memory_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_memory_new(
    store: &mut wasm_store_t,
    mt: &wasm_memorytype_t,
) -> Option<Box<wasm_memory_t>> {
    let memory = Memory::new(store.inner.context_mut(), mt.ty().ty).ok()?;
    Some(Box::new(wasm_memory_t {
        inner: wasm_extern_t {
            store: store.inner.clone(),
            which: memory.into(),
        },
    }))
}

/// Returns the [`wasm_memory_t`] as mutable reference to [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_memory_as_extern(m: &mut wasm_memory_t) -> &mut wasm_extern_t {
    &mut m.inner
}

/// Returns the [`wasm_memory_t`] as shared reference to [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_memory_as_extern_const(m: &wasm_memory_t) -> &wasm_extern_t {
    &m.inner
}

/// Returns the [`wasm_memorytype_t`] of the [`wasm_memory_t`].
///
/// Wraps [`Memory::ty`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_memory_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_memory_type(m: &wasm_memory_t) -> Box<wasm_memorytype_t> {
    let ty = m.memory().ty(m.inner.store.context());
    Box::new(wasm_memorytype_t::new(ty))
}

/// Returns the underlying data pointer of the [`wasm_memory_t`].
///
/// Wraps [`Memory::data_ptr`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_memory_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_memory_data(m: &wasm_memory_t) -> *mut u8 {
    m.memory().data_ptr(m.inner.store.context())
}

/// Returns the data buffer size of the [`wasm_memory_t`].
///
/// Wraps [`Memory::data_size`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_memory_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_memory_data_size(m: &wasm_memory_t) -> usize {
    m.memory().data_size(m.inner.store.context())
}

/// Returns the current number of Wasm pages of the [`wasm_memory_t`].
///
/// Wraps [`Memory::size`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_memory_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_memory_size(m: &wasm_memory_t) -> wasm_memory_pages_t {
    let size = m.memory().size(m.inner.store.context());
    let Ok(size32) = u32::try_from(size) else {
        panic!("linear memory pages out of bounds: {size}")
    };
    size32
}

/// Grows the [`wasm_memory_t`] by `delta` Wasm pages.
///
/// Returns `true` if the operation was successful.
///
/// Wraps [`Memory::grow`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_memory_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_memory_grow(
    m: &mut wasm_memory_t,
    delta: wasm_memory_pages_t,
) -> bool {
    let memory = m.memory();
    let mut store = m.inner.store.context_mut();
    memory.grow(&mut store, u64::from(delta)).is_ok()
}
