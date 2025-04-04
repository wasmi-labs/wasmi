use crate::{wasm, TrapCode, UntypedVal, ValType, F32, F64, V128};

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

impl From<TypedVal> for UntypedVal {
    fn from(typed_value: TypedVal) -> Self {
        typed_value.value
    }
}

/// An [`UntypedVal`] with its assumed [`ValType`].
///
/// # Note
///
/// We explicitly do not make use of the existing [`Val`]
/// abstraction since [`Val`] is optimized towards being a
/// user facing type whereas [`TypedVal`] is focusing on
/// performance and efficiency in computations.
///
/// [`Val`]: [`crate::core::Value`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TypedVal {
    /// The type of the value.
    ty: ValType,
    /// The underlying raw value.
    value: UntypedVal,
}

impl TypedVal {
    /// Create a new [`TypedVal`].
    pub fn new(ty: ValType, value: UntypedVal) -> Self {
        Self { ty, value }
    }

    /// Returns the [`ValType`] of the [`TypedVal`].
    pub fn ty(&self) -> ValType {
        self.ty
    }

    /// Returns the [`UntypedVal`] of the [`TypedVal`].
    pub fn untyped(&self) -> UntypedVal {
        self.value
    }

    /// Changes the [`ValType`] of `self` to `ty`.
    ///
    /// # Note
    ///
    /// This acts similar to a Wasm reinterpret cast and
    /// the underlying `value` bits are unchanged.
    pub fn reinterpret(self, ty: ValType) -> Self {
        Self { ty, ..self }
    }
}

impl<T> From<T> for TypedVal
where
    T: Typed + Into<UntypedVal>,
{
    fn from(value: T) -> Self {
        Self::new(<T as Typed>::TY, value.into())
    }
}

macro_rules! impl_from_typed_value_for {
    ( $( $( #[$attr:meta] )* impl From<TypedVal> for $ty:ty );* $(;)? ) => {
        $(
            $( #[$attr] )*
            impl From<TypedVal> for $ty {
                fn from(typed_value: TypedVal) -> Self {
                    // # Note
                    //
                    // We only use a `debug_assert` here instead of a proper `assert`
                    // since the whole translation process assumes that Wasm validation
                    // was already performed and thus type checking does not necessarily
                    // need to happen redundantly outside of debug builds.
                    debug_assert!(matches!(typed_value.ty, <$ty as Typed>::TY));
                    Self::from(typed_value.value)
                }
            }
        )*
    };
}
impl_from_typed_value_for! {
    impl From<TypedVal> for bool;
    impl From<TypedVal> for i32;
    impl From<TypedVal> for u32;
    impl From<TypedVal> for i64;
    impl From<TypedVal> for u64;
    impl From<TypedVal> for f32;
    impl From<TypedVal> for f64;
    #[cfg(feature = "simd")]
    impl From<TypedVal> for V128;
}

macro_rules! impl_from_typed_value_as_for {
    ( $( $( #[$attr:meta] )* impl From<TypedVal> for $ty:ty as $as:ty );* $(;)? ) => {
        $(
            $( #[$attr] )*
            impl From<TypedVal> for $ty {
                fn from(typed_value: TypedVal) -> Self {
                    <$as as From<TypedVal>>::from(typed_value) as $ty
                }
            }
        )*
    };
}
impl_from_typed_value_as_for! {
    impl From<TypedVal> for i8 as i32;
    impl From<TypedVal> for i16 as i32;
    impl From<TypedVal> for u8 as u32;
    impl From<TypedVal> for u16 as u32;
}

macro_rules! impl_forwarding {
    ( $( $(#[$mode:ident])? fn $name:ident $params:tt -> $result_ty:ty );* $(;)? ) => {
        $(
            impl_forwarding!( @impl $(#[$mode])? fn $name $params -> $result_ty );
        )*
    };
    ( @impl #[fallible] fn $name:ident($lhs_ty:ty, $rhs_ty:ty) -> $result_ty:ty ) => {
        #[doc = concat!("Forwards to [`wasm::", stringify!($name), "`] with debug type checks.")]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = concat!("If the forwarded [`wasm::", stringify!($name), "`] returns an error.")]
        #[doc = ""]
        #[doc = "# Panics (Debug)"]
        #[doc = ""]
        #[doc = "If type checks fail."]
        #[doc = ""]
        #[doc = concat!("[`wasm::", stringify!($name), "`]: crate::wasm::", stringify!($name))]
        pub fn $name(self, other: Self) -> Result<Self, TrapCode> {
            wasm::$name(self.into(), other.into()).map(Self::from)
        }
    };
    ( @impl fn $name:ident($lhs_ty:ty, $rhs_ty:ty) -> $result_ty:ty ) => {
        #[doc = concat!("Forwards to [`wasm::", stringify!($name), "`] with debug type checks.")]
        #[doc = ""]
        #[doc = "# Panics (Debug)"]
        #[doc = ""]
        #[doc = "If type checks fail."]
        #[doc = ""]
        #[doc = concat!("[`wasm::", stringify!($name), "`]: crate::wasm::", stringify!($name))]
        pub fn $name(self, other: Self) -> Self {
            wasm::$name(self.into(), other.into()).into()
        }
    };
    ( @impl #[fallible] fn $name:ident($input_ty:ty) -> $result_ty:ty ) => {
        #[doc = concat!("Forwards to [`wasm::", stringify!($name), "`] with debug type checks.")]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = concat!("If the forwarded [`wasm::", stringify!($name), "`] returns an error.")]
        #[doc = ""]
        #[doc = "# Panics (Debug)"]
        #[doc = ""]
        #[doc = "If type checks fail."]
        #[doc = ""]
        #[doc = concat!("[`wasm::", stringify!($name), "`]: crate::wasm::", stringify!($name))]
        pub fn $name(self) -> Result<Self, TrapCode> {
            wasm::$name(self.into()).map(Self::from)
        }
    };
    ( @impl fn $name:ident($input_ty:ty) -> $result_ty:ty ) => {
        #[doc = concat!("Forwards to [`wasm::", stringify!($name), "`] with debug type checks.")]
        #[doc = ""]
        #[doc = "# Panics (Debug)"]
        #[doc = ""]
        #[doc = "If type checks fail."]
        #[doc = ""]
        #[doc = concat!("[`wasm::", stringify!($name), "`]: crate::wasm::", stringify!($name))]
        pub fn $name(self) -> Self {
            wasm::$name(self.into()).into()
        }
    };
}
impl TypedVal {
    impl_forwarding! {
        // Shift & Rotate Instructions

        fn i32_shl(i32, i32) -> i32;
        fn i32_shr_s(i32, i32) -> i32;
        fn i32_shr_u(i32, i32) -> i32;
        fn i32_rotl(i32, i32) -> i32;
        fn i32_rotr(i32, i32) -> i32;

        fn i64_shl(i64, i64) -> i64;
        fn i64_shr_s(i64, i64) -> i64;
        fn i64_shr_u(i64, i64) -> i64;
        fn i64_rotl(i64, i64) -> i64;
        fn i64_rotr(i64, i64) -> i64;

        // Conversions

        #[fallible] fn i32_trunc_f32_s(f32) -> Result<i32, TrapCode>;
        #[fallible] fn i32_trunc_f32_u(f32) -> Result<i32, TrapCode>;
        #[fallible] fn i32_trunc_f64_s(f64) -> Result<i32, TrapCode>;
        #[fallible] fn i32_trunc_f64_u(f64) -> Result<i32, TrapCode>;
        #[fallible] fn i64_trunc_f32_s(f32) -> Result<i64, TrapCode>;
        #[fallible] fn i64_trunc_f32_u(f32) -> Result<i64, TrapCode>;
        #[fallible] fn i64_trunc_f64_s(f64) -> Result<i64, TrapCode>;
        #[fallible] fn i64_trunc_f64_u(f64) -> Result<i64, TrapCode>;
    }
}
