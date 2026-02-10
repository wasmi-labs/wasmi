use crate::{F32, F64, RawVal, V128, ValType};

/// Types that are associated to a static Wasm type.
pub trait Typed {
    /// The static associated Wasm type.
    const TY: ValType;
}
macro_rules! impl_typed_for {
    ( $( $ty:ty => $value_ty:expr );* $(;)? ) => {
        $(
            impl Typed for $ty {
                const TY: ValType = $value_ty;
            }
        )*
    };
}
impl_typed_for! {
    bool => ValType::I32;
    i8 => ValType::I32;
    u8 => ValType::I32;
    i16 => ValType::I32;
    u16 => ValType::I32;
    i32 => ValType::I32;
    u32 => ValType::I32;
    i64 => ValType::I64;
    u64 => ValType::I64;
    f32 => ValType::F32;
    f64 => ValType::F64;
    F32 => ValType::F32;
    F64 => ValType::F64;
    V128 => ValType::V128;
}

impl From<TypedRawVal> for RawVal {
    fn from(typed_value: TypedRawVal) -> Self {
        typed_value.value
    }
}

/// An [`RawVal`] with its assumed [`ValType`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TypedRawVal {
    /// The type of the value.
    ty: ValType,
    /// The underlying raw value.
    value: RawVal,
}

impl TypedRawVal {
    /// Create a new [`TypedRawVal`].
    pub fn new(ty: ValType, value: RawVal) -> Self {
        Self { ty, value }
    }

    /// Returns the [`ValType`] of the [`TypedRawVal`].
    pub fn ty(&self) -> ValType {
        self.ty
    }

    /// Returns the [`RawVal`] of the [`TypedRawVal`].
    pub fn raw(&self) -> RawVal {
        self.value
    }
}

impl<T> From<T> for TypedRawVal
where
    T: Typed + Into<RawVal>,
{
    fn from(value: T) -> Self {
        Self::new(<T as Typed>::TY, value.into())
    }
}

macro_rules! impl_from_typed_value_for {
    ( $( $( #[$attr:meta] )* impl From<TypedRawVal> for $ty:ty );* $(;)? ) => {
        $(
            $( #[$attr] )*
            impl From<TypedRawVal> for $ty {
                fn from(typed_value: TypedRawVal) -> Self {
                    // # Note
                    //
                    // We only use a `debug_assert` here instead of a proper `assert`
                    // since the whole translation process assumes that Wasm validation
                    // was already performed and thus type checking does not necessarily
                    // need to happen redundantly outside of debug builds.
                    debug_assert!(
                        matches!(typed_value.ty, <$ty as Typed>::TY),
                        "type mismatch: expected {:?} but found {:?}",
                        <$ty as Typed>::TY,
                        typed_value.ty,
                    );
                    Self::from(typed_value.value)
                }
            }
        )*
    };
}
impl_from_typed_value_for! {
    impl From<TypedRawVal> for bool;
    impl From<TypedRawVal> for i32;
    impl From<TypedRawVal> for u32;
    impl From<TypedRawVal> for i64;
    impl From<TypedRawVal> for u64;
    impl From<TypedRawVal> for f32;
    impl From<TypedRawVal> for f64;
    #[cfg(feature = "simd")]
    impl From<TypedRawVal> for V128;
}

macro_rules! impl_from_typed_value_as_for {
    ( $( $( #[$attr:meta] )* impl From<TypedRawVal> for $ty:ty as $as:ty );* $(;)? ) => {
        $(
            $( #[$attr] )*
            impl From<TypedRawVal> for $ty {
                fn from(typed_value: TypedRawVal) -> Self {
                    <$as as From<TypedRawVal>>::from(typed_value) as $ty
                }
            }
        )*
    };
}
impl_from_typed_value_as_for! {
    impl From<TypedRawVal> for i8 as i32;
    impl From<TypedRawVal> for i16 as i32;
    impl From<TypedRawVal> for u8 as u32;
    impl From<TypedRawVal> for u16 as u32;
}
