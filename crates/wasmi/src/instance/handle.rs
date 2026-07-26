use crate::{
    Handle,
    instance::{DataSegment, ElementSegment, Func, Global, Memory, Table},
    store::Stored,
};

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
///
/// # Note
///
/// Only [`AnyHandle`] storing the tag is `debug_assertions`-gated; the type itself always
/// exists so that [`AnyHandle::assert_kind`] has the same signature in every profile.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum HandleKind {
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

/// A concrete handle type that an [`AnyHandle`] can be created from.
///
/// # Note
///
/// This exists solely to map a handle type to its [`HandleKind`] tag, which is what makes
/// [`AnyHandle::assert_kind`] usable from generic code.
pub(super) trait HasHandleKind: Handle {
    /// The [`HandleKind`] tag of `Self`.
    const KIND: HandleKind;
}

impl AnyHandle {
    /// Asserts that `self` was created from a `T` handle.
    ///
    /// # Note
    ///
    /// Only debug builds store the [`HandleKind`] tag, so this is a no-op otherwise. Use it to
    /// validate a handle kind once, where it is claimed, instead of on every access.
    #[inline]
    pub(super) fn assert_kind<T: HasHandleKind>(self) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            self.kind,
            <T as HasHandleKind>::KIND,
            "unexpected instance handle kind",
        );
    }
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
                self.assert_kind::<$handle>();
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

macro_rules! impl_from_for_any_handle {
    (
        $( $handle:ident ),* $(,)?
    ) => {
        $(
            impl HasHandleKind for $handle {
                const KIND: HandleKind = HandleKind::$handle;
            }

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
