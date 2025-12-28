use crate::{wasm_extern_t, wasm_ref_t, wasm_store_t, wasm_tabletype_t};
use alloc::boxed::Box;
use core::hint;
use wasmi::{Extern, ExternRef, Func, Nullable, Ref, Table, TableType};

/// A Wasm table.
///
/// Wraps [`Table`].
#[derive(Clone)]
#[repr(transparent)]
pub struct wasm_table_t {
    inner: wasm_extern_t,
}

wasmi_c_api_macros::declare_ref!(wasm_table_t);

/// Type specifying the number of cells of a Wasm table.
pub type wasm_table_size_t = u32;

impl wasm_table_t {
    pub(crate) fn try_from(e: &wasm_extern_t) -> Option<&wasm_table_t> {
        match &e.which {
            Extern::Table(_) => Some(unsafe { &*(e as *const _ as *const _) }),
            _ => None,
        }
    }

    pub(crate) fn try_from_mut(e: &mut wasm_extern_t) -> Option<&mut wasm_table_t> {
        match &mut e.which {
            Extern::Table(_) => Some(unsafe { &mut *(e as *mut _ as *mut _) }),
            _ => None,
        }
    }

    /// Returns the underlying [`Table`] of the [`wasm_table_t`].
    fn table(&self) -> Table {
        match self.inner.which {
            Extern::Table(t) => t,
            _ => unsafe { hint::unreachable_unchecked() },
        }
    }
}

/// Returns the [`Ref`] respective to the optional [`wasm_ref_t`].
///
/// Returns a `null` [`Ref`] if [`wasm_ref_t`] is `None`.
fn option_wasm_ref_t_to_ref(r: Option<&wasm_ref_t>, table_ty: &TableType) -> Ref {
    r.map(|r| r.inner)
        .unwrap_or_else(|| match table_ty.element() {
            wasmi::RefType::Func => Ref::Func(<Nullable<Func>>::Null),
            wasmi::RefType::Extern => Ref::Extern(<Nullable<ExternRef>>::Null),
        })
}

/// Creates a new [`wasm_table_t`] from the given [`wasm_tabletype_t`].
///
/// Wraps [`Table::new`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_table_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_table_new(
    store: &mut wasm_store_t,
    tt: &wasm_tabletype_t,
    init: Option<&wasm_ref_t>,
) -> Option<Box<wasm_table_t>> {
    let tt = tt.ty().ty;
    let init = option_wasm_ref_t_to_ref(init, &tt);
    let table = Table::new(store.inner.context_mut(), tt, init).ok()?;
    Some(Box::new(wasm_table_t {
        inner: wasm_extern_t {
            store: store.inner.clone(),
            which: table.into(),
        },
    }))
}

/// Returns the [`wasm_tabletype_t`] of the [`wasm_table_t`].
///
/// Wraps [`Table::ty`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_table_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_table_type(t: &wasm_table_t) -> Box<wasm_tabletype_t> {
    let table = t.table();
    let store = t.inner.store.context();
    Box::new(wasm_tabletype_t::new(table.ty(store)))
}

/// Returns the element at `index` of [`wasm_table_t`] `t`.
///
/// Wraps [`Table::get`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_table_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_table_get(
    t: &mut wasm_table_t,
    index: wasm_table_size_t,
) -> Option<Box<wasm_ref_t>> {
    let table = t.table();
    let value = table.get(t.inner.store.context_mut(), u64::from(index))?;
    wasm_ref_t::new(value)
}

/// Sets the value of the element at `index` of [`wasm_table_t`] to `new_value`.
///
/// Wraps [`Table::set`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_table_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_table_set(
    t: &mut wasm_table_t,
    index: wasm_table_size_t,
    new_value: Option<&wasm_ref_t>,
) -> bool {
    let table = t.table();
    let new_value = option_wasm_ref_t_to_ref(new_value, &table.ty(t.inner.store.context()));
    table
        .set(t.inner.store.context_mut(), u64::from(index), new_value)
        .is_ok()
}

/// Returns the number of cells of the [`wasm_table_t`].
///
/// Wraps [`Table::size`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_table_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_table_size(t: &wasm_table_t) -> wasm_table_size_t {
    let table = t.table();
    let store = t.inner.store.context();
    let size = table.size(store);
    u32::try_from(size).unwrap()
}
/// Grows the number of cells of the [`wasm_table_t`] by `delta`.
///
/// Returns `true` if the operation was successful.
///
/// Wraps [`Table::grow`].
///
/// # Safety
///
/// It is the caller's responsibility not to alias the [`wasm_table_t`]
/// with its underlying, internal [`WasmStoreRef`](crate::WasmStoreRef).
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub unsafe extern "C" fn wasm_table_grow(
    t: &mut wasm_table_t,
    delta: wasm_table_size_t,
    init: Option<&wasm_ref_t>,
) -> bool {
    let table = t.table();
    let init = option_wasm_ref_t_to_ref(init, &table.ty(t.inner.store.context()));
    table
        .grow(t.inner.store.context_mut(), u64::from(delta), init)
        .is_ok()
}

/// Returns the [`wasm_table_t`] as mutable reference to [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_table_as_extern(t: &mut wasm_table_t) -> &mut wasm_extern_t {
    &mut t.inner
}

/// Returns the [`wasm_table_t`] as shared reference to [`wasm_extern_t`].
#[cfg_attr(not(feature = "prefix-symbols"), no_mangle)]
#[cfg_attr(feature = "prefix-symbols", wasmi_c_api_macros::prefix_symbol)]
pub extern "C" fn wasm_table_as_extern_const(t: &wasm_table_t) -> &wasm_extern_t {
    &t.inner
}
