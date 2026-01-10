use crate::{from_valtype, into_valtype, utils, wasm_ref_t, wasm_valkind_t};
use alloc::boxed::Box;
use core::{mem::MaybeUninit, ptr};
use wasmi::{Func, Nullable, Ref, Val, ValType, F32, F64};

/// A Wasm value.
///
/// Mirrors [`Val`].
#[repr(C)]
pub struct wasm_val_t {
    /// The kind of the Wasm value.
    pub kind: wasm_valkind_t,
    /// The underlying data of the Wasm value classified by `kind`.
    pub of: wasm_val_union,
}

/// The underlying data of a [`wasm_val_t`].
#[repr(C)]
#[derive(Copy, Clone)]
pub union wasm_val_union {
    /// A Wasm 32-bit signed integer.
    pub i32: i32,
    /// A Wasm 64-bit signed integer.
    pub i64: i64,
    /// A Wasm 32-bit unsigned integer.
    pub u32: u32,
    /// A Wasm 64-bit unsigned integer.
    pub u64: u64,
    /// A Wasm 32-bit float.
    pub f32: f32,
    /// A Wasm 64-bit float.
    pub f64: f64,
    /// A Wasm referenced object.
    pub ref_: *mut wasm_ref_t,
}

impl Drop for wasm_val_t {
    fn drop(&mut self) {
        if into_valtype(self.kind).is_ref() && !unsafe { self.of.ref_ }.is_null() {
            drop(unsafe { Box::from_raw(self.of.ref_) });
        }
    }
}

impl Clone for wasm_val_t {
    fn clone(&self) -> Self {
        let mut ret = wasm_val_t {
            kind: self.kind,
            of: self.of,
        };
        unsafe {
            if into_valtype(self.kind).is_ref() && !self.of.ref_.is_null() {
                ret.of.ref_ = Box::into_raw(Box::new((*self.of.ref_).clone()));
            }
        }
        ret
    }
}

impl Default for wasm_val_t {
    fn default() -> Self {
        wasm_val_t {
            kind: wasm_valkind_t::WASM_I32,
            of: wasm_val_union { i32: 0 },
        }
    }
}

impl From<Val> for wasm_val_t {
    fn from(val: Val) -> Self {
        match val {
            Val::I32(value) => Self {
                kind: from_valtype(&ValType::I32),
                of: wasm_val_union { i32: value },
            },
            Val::I64(value) => Self {
                kind: from_valtype(&ValType::I64),
                of: wasm_val_union { i64: value },
            },
            Val::F32(value) => Self {
                kind: from_valtype(&ValType::F32),
                of: wasm_val_union {
                    u32: value.to_bits(),
                },
            },
            Val::F64(value) => Self {
                kind: from_valtype(&ValType::F64),
                of: wasm_val_union {
                    u64: value.to_bits(),
                },
            },
            Val::V128(value) => core::panic!(
                "`wasm_val_t`: creating a `wasm_val_t` from an `v128` value is not supported but found: {value:?}"
            ),
            Val::FuncRef(funcref) => Self {
                kind: from_valtype(&ValType::FuncRef),
                of: wasm_val_union {
                    ref_: {
                        match funcref.is_null() {
                            true => ptr::null_mut(),
                            false => Box::into_raw(Box::new(wasm_ref_t {
                                inner: Ref::Func(funcref),
                            })),
                        }
                    },
                },
            },
            Val::ExternRef(_) => {
                core::panic!("`wasm_val_t`: creating a `wasm_val_t` from an `externref`")
            }
        }
    }
}

impl wasm_val_t {
    /// Creates a new [`Val`] from the [`wasm_val_t`].
    ///
    /// # Note
    ///
    /// This effectively clones the [`wasm_val_t`] if necessary.
    pub fn to_val(&self) -> Val {
        match self.kind {
            wasm_valkind_t::WASM_I32 => Val::from(unsafe { self.of.i32 }),
            wasm_valkind_t::WASM_I64 => Val::from(unsafe { self.of.i64 }),
            wasm_valkind_t::WASM_F32 => Val::from(F32::from(unsafe { self.of.f32 })),
            wasm_valkind_t::WASM_F64 => Val::from(F64::from(unsafe { self.of.f64 })),
            wasm_valkind_t::WASM_FUNCREF => match unsafe { self.of.ref_ }.is_null() {
                true => Val::FuncRef(<Nullable<Func>>::Null),
                false => Val::from((unsafe { &*self.of.ref_ }).inner),
            },
            wasm_valkind_t::WASM_EXTERNREF => {
                core::unreachable!("`wasm_val_t`: cannot contain non-function reference values")
            }
        }
    }
}

/// Copies the [`wasm_val_t`] and stores the result in `out`.
///
/// # Safety
///
/// The caller is responsible to provide a valid [`wasm_val_t`] that can safely be copied.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_val_copy(out: &mut MaybeUninit<wasm_val_t>, source: &wasm_val_t) {
    utils::initialize(out, source.clone());
}

/// Deletes the [`wasm_val_t`].
///
/// # Safety
///
/// The caller is responsible to provide a valid [`wasm_val_t`] that can safely be deleted.
/// The same [`wasm_val_t`] must not be deleted more than once.
#[cfg_attr(not(feature = "prefix-symbols"), unsafe(no_mangle))]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_val_delete(val: *mut wasm_val_t) {
    ptr::drop_in_place(val);
}
