use crate::{
    instance::{DataSegment, ElementSegment, Func, Global, Memory, Table},
    store::Stored,
};

/// We just define it because we need it for the `define_handle` macro.
pub struct AnyHandleEntity;

define_handle! {
    /// A generic Wasm handle of any type.
    struct AnyHandle(u32, Stored) => AnyHandleEntity;
}
macro_rules! impl_cast_for_any_handle {
    ( $(
        pub unsafe fn $cast_ident:ident(self) -> $handle:ty;
    )* $(;)? ) => {
        $(
            #[doc = concat!("Cast `self` into a [`", stringify!($handle), "`] handle.")]
            #[inline]
            pub unsafe fn $cast_ident(self) -> $handle {
                unsafe { ::core::mem::transmute::<Self, $handle>(self) }
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
        $( $handle:ty ),* $(,)?
    ) => {
        $(
            impl From<$handle> for AnyHandle {
                #[inline]
                fn from(handle: $handle) -> Self {
                    unsafe { ::core::mem::transmute::<$handle, Self>(handle) }
                }
            }
        )*
    };
}
impl_from_for_any_handle!(Global, Func, Memory, Table, DataSegment, ElementSegment);
