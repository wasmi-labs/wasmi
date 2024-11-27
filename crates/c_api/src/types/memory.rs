use crate::{wasm_externtype_t, wasm_limits_t, CExternType};
use alloc::boxed::Box;
use wasmi::MemoryType;

/// A Wasm linear memory type.
///
/// Wraps [`MemoryType`].
#[repr(transparent)]
#[derive(Clone)]
pub struct wasm_memorytype_t {
    ext: wasm_externtype_t,
}

wasmi_c_api_macros::declare_ty!(wasm_memorytype_t);

#[derive(Clone)]
pub(crate) struct CMemoryType {
    pub(crate) ty: MemoryType,
    limits: wasm_limits_t,
}

impl wasm_memorytype_t {
    pub(crate) fn new(ty: MemoryType) -> wasm_memorytype_t {
        wasm_memorytype_t {
            ext: wasm_externtype_t::from_extern_type(ty.into()),
        }
    }

    pub(crate) fn try_from(e: &wasm_externtype_t) -> Option<&wasm_memorytype_t> {
        match &e.which {
            CExternType::Memory(_) => Some(unsafe { &*(e as *const _ as *const _) }),
            _ => None,
        }
    }

    pub(crate) fn try_from_mut(e: &mut wasm_externtype_t) -> Option<&mut wasm_memorytype_t> {
        match &mut e.which {
            CExternType::Memory(_) => Some(unsafe { &mut *(e as *mut _ as *mut _) }),
            _ => None,
        }
    }

    pub(crate) fn ty(&self) -> &CMemoryType {
        match &self.ext.which {
            CExternType::Memory(f) => f,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl CMemoryType {
    pub(crate) fn new(ty: MemoryType) -> CMemoryType {
        let min: u32 = ty.initial_pages().into();
        let max: u32 = ty.maximum_pages().map(Into::into).unwrap_or(u32::MAX);
        CMemoryType {
            ty,
            limits: wasm_limits_t { min, max },
        }
    }
}

/// Creates a new [`wasm_memorytype_t`] with the given `limits`.
///
/// Wraps [`MemoryType::new`].
#[no_mangle]
pub extern "C" fn wasm_memorytype_new(limits: &wasm_limits_t) -> Box<wasm_memorytype_t> {
    let memory_type = MemoryType::new(limits.min, limits.max()).unwrap();
    Box::new(wasm_memorytype_t::new(memory_type))
}

/// Returns a shared reference to the table limits of the [`wasm_memorytype_t`].
#[no_mangle]
pub extern "C" fn wasm_memorytype_limits(mt: &wasm_memorytype_t) -> &wasm_limits_t {
    &mt.ty().limits
}

/// Returns a mutable reference to the element type of [`wasm_memorytype_t`] as [`wasm_externtype_t`].
#[no_mangle]
pub extern "C" fn wasm_memorytype_as_externtype(
    ty: &mut wasm_memorytype_t,
) -> &mut wasm_externtype_t {
    &mut ty.ext
}

/// Returns a shared reference to the element type of [`wasm_memorytype_t`] as [`wasm_externtype_t`].
#[no_mangle]
pub extern "C" fn wasm_memorytype_as_externtype_const(
    ty: &wasm_memorytype_t,
) -> &wasm_externtype_t {
    &ty.ext
}
