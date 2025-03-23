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
    ( $( $( #[$attr:meta] )* impl From<TypedValue> for $ty:ty );* $(;)? ) => {
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
    impl From<TypedValue> for bool;
    impl From<TypedValue> for i32;
    impl From<TypedValue> for u32;
    impl From<TypedValue> for i64;
    impl From<TypedValue> for u64;
    impl From<TypedValue> for f32;
    impl From<TypedValue> for f64;
    #[cfg(feature = "simd")]
    impl From<TypedValue> for V128;
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
        // Comparison Instructions

        fn i32_eq(i32, i32) -> i32;
        fn i64_eq(i64, i64) -> i32;
        fn f32_eq(f32, f32) -> i32;
        fn f64_eq(f64, f64) -> i32;

        fn i32_ne(i32, i32) -> i32;
        fn i64_ne(i64, i64) -> i32;
        fn f32_ne(f32, f32) -> i32;
        fn f64_ne(f64, f64) -> i32;

        fn i32_lt_s(i32, i32) -> i32;
        fn i32_lt_u(i32, i32) -> i32;
        fn i32_gt_s(i32, i32) -> i32;
        fn i32_gt_u(i32, i32) -> i32;
        fn i32_le_s(i32, i32) -> i32;
        fn i32_le_u(i32, i32) -> i32;
        fn i32_ge_s(i32, i32) -> i32;
        fn i32_ge_u(i32, i32) -> i32;

        fn i64_lt_s(i64, i64) -> i32;
        fn i64_lt_u(i64, i64) -> i32;
        fn i64_gt_s(i64, i64) -> i32;
        fn i64_gt_u(i64, i64) -> i32;
        fn i64_le_s(i64, i64) -> i32;
        fn i64_le_u(i64, i64) -> i32;
        fn i64_ge_s(i64, i64) -> i32;
        fn i64_ge_u(i64, i64) -> i32;

        fn f32_lt(f32, f32) -> i32;
        fn f32_gt(f32, f32) -> i32;
        fn f32_le(f32, f32) -> i32;
        fn f32_ge(f32, f32) -> i32;

        fn f64_lt(f64, f64) -> i32;
        fn f64_gt(f64, f64) -> i32;
        fn f64_le(f64, f64) -> i32;
        fn f64_ge(f64, f64) -> i32;

        // Integer Arithmetic Instructions

        fn i32_clz(i32) -> i32;
        fn i32_ctz(i32) -> i32;
        fn i32_popcnt(i32) -> i32;

        fn i64_clz(i64) -> i64;
        fn i64_ctz(i64) -> i64;
        fn i64_popcnt(i64) -> i64;

        fn i32_add(i32, i32) -> i32;
        fn i32_sub(i32, i32) -> i32;
        fn i32_mul(i32, i32) -> i32;

        fn i64_add(i64, i64) -> i64;
        fn i64_sub(i64, i64) -> i64;
        fn i64_mul(i64, i64) -> i64;

        #[fallible] fn i32_div_s(i32, i32) -> Result<i32, TrapCode>;
        #[fallible] fn i32_div_u(i32, i32) -> Result<i32, TrapCode>;
        #[fallible] fn i32_rem_s(i32, i32) -> Result<i32, TrapCode>;
        #[fallible] fn i32_rem_u(i32, i32) -> Result<i32, TrapCode>;

        #[fallible] fn i64_div_s(i64, i64) -> Result<i64, TrapCode>;
        #[fallible] fn i64_div_u(i64, i64) -> Result<i64, TrapCode>;
        #[fallible] fn i64_rem_s(i64, i64) -> Result<i64, TrapCode>;
        #[fallible] fn i64_rem_u(i64, i64) -> Result<i64, TrapCode>;

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

        // Bitwise Instructions

        fn i32_and(i32, i32) -> i32;
        fn i32_or(i32, i32) -> i32;
        fn i32_xor(i32, i32) -> i32;

        fn i64_and(i64, i64) -> i64;
        fn i64_or(i64, i64) -> i64;
        fn i64_xor(i64, i64) -> i64;

        // Float Arithmetic Instructions

        fn f32_abs(f32) -> f32;
        fn f32_neg(f32) -> f32;
        fn f32_ceil(f32) -> f32;
        fn f32_floor(f32) -> f32;
        fn f32_trunc(f32) -> f32;
        fn f32_nearest(f32) -> f32;
        fn f32_sqrt(f32) -> f32;

        fn f64_abs(f64) -> f64;
        fn f64_neg(f64) -> f64;
        fn f64_ceil(f64) -> f64;
        fn f64_floor(f64) -> f64;
        fn f64_trunc(f64) -> f64;
        fn f64_nearest(f64) -> f64;
        fn f64_sqrt(f64) -> f64;

        fn f32_add(f32, f32) -> f32;
        fn f32_sub(f32, f32) -> f32;
        fn f32_mul(f32, f32) -> f32;
        fn f32_div(f32, f32) -> f32;
        fn f32_min(f32, f32) -> f32;
        fn f32_max(f32, f32) -> f32;
        fn f32_copysign(f32, f32) -> f32;

        fn f64_add(f64, f64) -> f64;
        fn f64_sub(f64, f64) -> f64;
        fn f64_mul(f64, f64) -> f64;
        fn f64_div(f64, f64) -> f64;
        fn f64_min(f64, f64) -> f64;
        fn f64_max(f64, f64) -> f64;
        fn f64_copysign(f64, f64) -> f64;

        // Conversions

        fn i32_wrap_i64(i64) -> i32;
        fn i64_extend_i32_s(i32) -> i64;

        fn f32_demote_f64(f64) -> f32;
        fn f64_promote_f32(f32) -> f64;

        #[fallible] fn i32_trunc_f32_s(f32) -> Result<i32, TrapCode>;
        #[fallible] fn i32_trunc_f32_u(f32) -> Result<i32, TrapCode>;
        #[fallible] fn i32_trunc_f64_s(f64) -> Result<i32, TrapCode>;
        #[fallible] fn i32_trunc_f64_u(f64) -> Result<i32, TrapCode>;
        #[fallible] fn i64_trunc_f32_s(f32) -> Result<i64, TrapCode>;
        #[fallible] fn i64_trunc_f32_u(f32) -> Result<i64, TrapCode>;
        #[fallible] fn i64_trunc_f64_s(f64) -> Result<i64, TrapCode>;
        #[fallible] fn i64_trunc_f64_u(f64) -> Result<i64, TrapCode>;

        fn i32_trunc_sat_f32_s(f32) -> i32;
        fn i32_trunc_sat_f32_u(f32) -> i32;
        fn i32_trunc_sat_f64_s(f64) -> i32;
        fn i32_trunc_sat_f64_u(f64) -> i32;
        fn i64_trunc_sat_f32_s(f32) -> i64;
        fn i64_trunc_sat_f32_u(f32) -> i64;
        fn i64_trunc_sat_f64_s(f64) -> i64;
        fn i64_trunc_sat_f64_u(f64) -> i64;

        fn f32_convert_i32_s(i32) -> f32;
        fn f32_convert_i32_u(i32) -> f32;
        fn f32_convert_i64_s(i64) -> f32;
        fn f32_convert_i64_u(i64) -> f32;
        fn f64_convert_i32_s(i32) -> f64;
        fn f64_convert_i32_u(i32) -> f64;
        fn f64_convert_i64_s(i64) -> f64;
        fn f64_convert_i64_u(i64) -> f64;

        fn i32_extend8_s(i32) -> i32;
        fn i32_extend16_s(i32) -> i32;
        fn i64_extend8_s(i64) -> i64;
        fn i64_extend16_s(i64) -> i64;
        fn i64_extend32_s(i64) -> i64;
    }
}
