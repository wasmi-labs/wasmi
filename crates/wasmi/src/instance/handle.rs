use crate::{
    Handle,
    instance::{DataSegment, ElementSegment, Func, Global, Memory, Table},
    store::{StoreInner, Stored},
};
use core::ptr::NonNull;

/// We just define it because we need it for the `define_handle` macro.
pub struct AnyHandleEntity;

define_handle! {
    /// The type-erased raw representation of an [`AnyHandle`].
    ///
    /// This is the value that all concrete handles are `transmute`d to and from.
    struct RawAnyHandle(u32, Stored) => AnyHandleEntity;
}

/// The concrete handle type stored in an [`AnyHandle`].
///
/// Used in debug builds to guard the type-erased [`AnyHandle::cast_global`] etc. casts
/// against being applied to a handle of the wrong type.
#[cfg(debug_assertions)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum HandleKind {
    Global,
    Memory,
    Table,
    Func,
    ElementSegment,
    DataSegment,
}

/// A generic Wasm handle of any type.
///
/// # Note
///
/// This is type-erased and only stores the concrete handle type as a [`HandleKind`] tag in
/// debug builds. In release builds it has the exact same size and layout as [`RawAnyHandle`]
/// (and thus any concrete handle), so the merged instance handle buffer stays compact.
#[derive(Debug, Copy, Clone)]
pub struct AnyHandle {
    /// The type-erased raw handle.
    raw: RawAnyHandle,
    /// The concrete handle type, used to guard casts in debug builds.
    #[cfg(debug_assertions)]
    kind: HandleKind,
}

macro_rules! impl_cast_for_any_handle {
    ( $(
        pub unsafe fn $cast_ident:ident(self) -> $handle:ident;
    )* $(;)? ) => {
        $(
            #[doc = concat!("Casts `self` into a [`", stringify!($handle), "`] handle.")]
            #[doc = ""]
            #[doc = "# Safety"]
            #[doc = ""]
            #[doc = "The caller must guarantee that `self` was created from a"]
            #[doc = concat!("[`", stringify!($handle), "`] handle.")]
            #[doc = "Casting to any other handle type is undefined behavior."]
            #[inline]
            pub unsafe fn $cast_ident(self) -> $handle {
                #[cfg(debug_assertions)]
                debug_assert_eq!(
                    self.kind,
                    HandleKind::$handle,
                    concat!("tried to cast an `AnyHandle` to a `", stringify!($handle), "`"),
                );
                unsafe { ::core::mem::transmute::<RawAnyHandle, $handle>(self.raw) }
            }
        )*
    };
}
impl AnyHandle {
    impl_cast_for_any_handle! {
        pub unsafe fn cast_global(self) -> Global;
        pub unsafe fn cast_func(self) -> Func;
        pub unsafe fn cast_memory(self) -> Memory;
        pub unsafe fn cast_table(self) -> Table;
        pub unsafe fn cast_data(self) -> DataSegment;
        pub unsafe fn cast_elem(self) -> ElementSegment;
    }
}

/// A [`Handle`] that an [`InstanceEntity`] stores in its `handles` buffer.
///
/// This is what makes [`HandleAndEntity<T>`] generic: it provides the two per-kind operations
/// that a type-erased [`AnyHandle`] cannot perform on its own.
///
/// [`InstanceEntity`]: crate::InstanceEntity
/// [`HandleAndEntity<T>`]: crate::instance::HandleAndEntity
pub trait InstanceHandle: Handle<Entity: Sized> {
    /// Casts the type-erased `handle` into `Self`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `handle` was created from a `Self` handle.
    unsafe fn cast(handle: AnyHandle) -> Self;

    /// Resolves the entity pointer of `handle` in `store`.
    ///
    /// # Panics
    ///
    /// If `handle` is unknown to `store`.
    fn resolve_ptr(store: &mut StoreInner, handle: &Self) -> NonNull<<Self as Handle>::Entity>;
}

macro_rules! impl_instance_handle {
    (
        $(
            impl InstanceHandle for $handle:ident {
                cast: $cast:expr,
                resolve: $resolve:expr,
            }
        )*
    ) => {
        $(
            impl InstanceHandle for $handle {
                #[inline]
                unsafe fn cast(handle: AnyHandle) -> Self {
                    // Safety: guaranteed by the caller.
                    unsafe { $cast(handle) }
                }

                #[inline]
                fn resolve_ptr(
                    store: &mut StoreInner,
                    handle: &Self,
                ) -> NonNull<<Self as Handle>::Entity> {
                    $resolve(store, handle)
                }
            }
        )*
    };
}
impl_instance_handle! {
    impl InstanceHandle for Memory {
        cast: AnyHandle::cast_memory,
        resolve: StoreInner::resolve_memory_ptr,
    }

    impl InstanceHandle for Global {
        cast: AnyHandle::cast_global,
        resolve: StoreInner::resolve_global_ptr,
    }

    impl InstanceHandle for Table {
        cast: AnyHandle::cast_table,
        resolve: StoreInner::resolve_table_ptr,
    }

    impl InstanceHandle for Func {
        cast: AnyHandle::cast_func,
        resolve: StoreInner::resolve_func_ptr,
    }

    impl InstanceHandle for ElementSegment {
        cast: AnyHandle::cast_elem,
        resolve: StoreInner::resolve_element_ptr,
    }

    impl InstanceHandle for DataSegment {
        cast: AnyHandle::cast_data,
        resolve: StoreInner::resolve_data_ptr,
    }
}

macro_rules! impl_from_for_any_handle {
    (
        $( $handle:ident ),* $(,)?
    ) => {
        $(
            impl From<$handle> for AnyHandle {
                #[inline]
                fn from(handle: $handle) -> Self {
                    // Safety: `RawAnyHandle` has the same size as any concrete handle and its
                    //         raw `u32` can represent any concrete handle's raw value (in
                    //         particular a `NonZero<u32>` is always a valid `u32`).
                    let raw = unsafe { ::core::mem::transmute::<$handle, RawAnyHandle>(handle) };
                    Self {
                        raw,
                        #[cfg(debug_assertions)]
                        kind: HandleKind::$handle,
                    }
                }
            }
        )*
    };
}
impl_from_for_any_handle!(Global, Func, Memory, Table, DataSegment, ElementSegment,);
