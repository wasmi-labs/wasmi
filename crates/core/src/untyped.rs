use crate::{TrapCode, F32, F64};
use core::fmt::{self, Display};

/// An untyped value.
///
/// Provides a dense and simple interface to all functional Wasm operations.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(not(feature = "value128"), repr(transparent))]
#[cfg_attr(feature = "value128", repr(C))]
pub struct UntypedVal {
    /// The low 64-bits of an [`UntypedVal`].
    ///
    /// The low 64-bits are used to encode and decode all types that
    /// are convertible from and to an [`UntypedVal`] that fit into
    /// 64-bits such as `i32`, `i64`, `f32` and `f64`.
    lo64: u64,
    /// The high 64-bits of an [`UntypedVal`].
    ///
    /// This is only used to encode or decode types which do not fit
    /// into the lower 64-bits part such as Wasm's `V128` or `i128`.
    #[cfg(feature = "value128")]
    hi64: u64,
}

/// Implemented by types that can be read (or decoded) as `T`.
///
/// Mainly implemented by [`UntypedVal`].
pub trait ReadAs<T> {
    /// Reads `self` as value of type `T`.
    fn read_as(&self) -> T;
}

macro_rules! impl_read_as_for_int {
    ( $( $int:ty ),* $(,)? ) => {
        $(
            impl ReadAs<$int> for UntypedVal {
                fn read_as(&self) -> $int {
                    self.read_lo64() as $int
                }
            }
        )*
    };
}
impl_read_as_for_int!(i8, i16, i32, i64, u8, u16, u32, u64);

macro_rules! impl_read_as_for_float {
    ( $( $float:ty ),* $(,)? ) => {
        $(
            impl ReadAs<$float> for UntypedVal {
                fn read_as(&self) -> $float {
                    <$float>::from_bits(self.read_lo64() as _)
                }
            }
        )*
    };
}
impl_read_as_for_float!(f32, f64, F32, F64);

impl ReadAs<bool> for UntypedVal {
    fn read_as(&self) -> bool {
        self.read_lo64() != 0
    }
}

/// Implemented by types that can be written to (or encoded) as `T`.
///
/// Mainly implemented by [`UntypedVal`].
pub trait WriteAs<T> {
    /// Writes to `self` as value of type `T`.
    fn write_as(&mut self, value: T);
}

macro_rules! impl_write_as_for_int {
    ( $( $int:ty as $as:ty ),* $(,)? ) => {
        $(
            impl WriteAs<$int> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn write_as(&mut self, value: $int) {
                    self.write_lo64(value as $as as _)
                }
            }
        )*
    };
}
impl_write_as_for_int!(i8 as u8, i16 as u16, i32 as u32, i64 as u64);

macro_rules! impl_write_as_for_uint {
    ( $( $int:ty ),* $(,)? ) => {
        $(
            impl WriteAs<$int> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn write_as(&mut self, value: $int) {
                    self.write_lo64(value as _)
                }
            }
        )*
    };
}
impl_write_as_for_uint!(bool, u8, u16, u32, u64);

macro_rules! impl_write_as_for_float {
    ( $( $float:ty ),* $(,)? ) => {
        $(
            impl WriteAs<$float> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn write_as(&mut self, value: $float) {
                    self.write_lo64(<$float>::to_bits(value) as _)
                }
            }
        )*
    };
}
impl_write_as_for_float!(f32, f64, F32, F64);

impl UntypedVal {
    /// Reads the low 64-bit of the [`UntypedVal`].
    ///
    /// In contract to [`UntypedVal::to_bits64`] this ignores the high-bits entirely.
    fn read_lo64(&self) -> u64 {
        self.lo64
    }

    /// Writes the low 64-bit of the [`UntypedVal`].
    fn write_lo64(&mut self, bits: u64) {
        self.lo64 = bits;
    }

    /// Creates an [`UntypedVal`] from the given lower 64-bit bits.
    ///
    /// This sets the high 64-bits to zero if any.
    pub const fn from_bits64(lo64: u64) -> Self {
        Self {
            lo64,
            #[cfg(feature = "value128")]
            hi64: 0,
        }
    }

    /// Returns the underlying lower 64-bits of the [`UntypedVal`].
    ///
    /// This ignores the high 64-bits of the [`UntypedVal`] if any.
    pub const fn to_bits64(self) -> u64 {
        self.lo64
    }
}

macro_rules! impl_from_untyped_for_int {
    ( $( $int:ty ),* $(,)? ) => {
        $(
            impl From<UntypedVal> for $int {
                fn from(untyped: UntypedVal) -> Self {
                    untyped.to_bits64() as _
                }
            }
        )*
    };
}
impl_from_untyped_for_int!(i8, i16, i32, i64, u8, u16, u32, u64);

macro_rules! impl_from_untyped_for_float {
    ( $( $float:ty ),* $(,)? ) => {
        $(
            impl From<UntypedVal> for $float {
                fn from(untyped: UntypedVal) -> Self {
                    Self::from_bits(untyped.to_bits64() as _)
                }
            }
        )*
    };
}
impl_from_untyped_for_float!(f32, f64, F32, F64);

impl From<UntypedVal> for bool {
    fn from(untyped: UntypedVal) -> Self {
        untyped.to_bits64() != 0
    }
}

macro_rules! impl_from_unsigned_prim {
    ( $( $prim:ty ),* $(,)? ) => {
        $(
            impl From<$prim> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn from(value: $prim) -> Self {
                    Self::from_bits64(value as _)
                }
            }
        )*
    };
}
#[rustfmt::skip]
impl_from_unsigned_prim!(
    bool, u8, u16, u32, u64,
);

macro_rules! impl_from_signed_prim {
    ( $( $prim:ty as $base:ty ),* $(,)? ) => {
        $(
            impl From<$prim> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn from(value: $prim) -> Self {
                    Self::from_bits64(u64::from(value as $base))
                }
            }
        )*
    };
}
#[rustfmt::skip]
impl_from_signed_prim!(
    i8 as u8,
    i16 as u16,
    i32 as u32,
    i64 as u64,
);

macro_rules! impl_from_float {
    ( $( $float:ty ),* $(,)? ) => {
        $(
            impl From<$float> for UntypedVal {
                fn from(value: $float) -> Self {
                    Self::from_bits64(u64::from(value.to_bits()))
                }
            }
        )*
    };
}
impl_from_float!(f32, f64, F32, F64);

impl UntypedVal {
    /// Execute an infallible generic operation on `T` that returns an `R`.
    fn execute_unary<T, R>(self, op: fn(T) -> R) -> Self
    where
        T: From<Self>,
        R: Into<Self>,
    {
        op(T::from(self)).into()
    }

    /// Execute an infallible generic operation on `T` that returns an `R`.
    fn try_execute_unary<T, R>(self, op: fn(T) -> Result<R, TrapCode>) -> Result<Self, TrapCode>
    where
        T: From<Self>,
        R: Into<Self>,
    {
        op(T::from(self)).map(Into::into)
    }

    /// Execute an infallible generic operation on `T` that returns an `R`.
    fn execute_binary<Lhs, Rhs, Result>(self, rhs: Self, op: fn(Lhs, Rhs) -> Result) -> Self
    where
        Lhs: From<Self>,
        Rhs: From<Self>,
        Result: Into<Self>,
    {
        op(Lhs::from(self), Rhs::from(rhs)).into()
    }

    /// Execute a fallible generic operation on `T` that returns an `R`.
    fn try_execute_binary<T, R>(
        self,
        rhs: Self,
        op: fn(T, T) -> Result<R, TrapCode>,
    ) -> Result<Self, TrapCode>
    where
        T: From<Self>,
        R: Into<Self>,
    {
        op(T::from(self), T::from(rhs)).map(Into::into)
    }
}

macro_rules! impl_untyped_val {
    (
        $(#[$attr:meta])*
        fn $name:ident(value: $ty:ty) -> Result<$ret_ty:ty>; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        ///
        /// # Errors
        ///
        $( #[$attr] )*
        pub fn $name(self) -> Result<Self, TrapCode> {
            self.try_execute_unary::<$ty, $ret_ty>($crate::wasm::$name)
        }

        impl_untyped_val!( $($tt)* );
    };
    (
        fn $name:ident(value: $ty:ty) -> $ret_ty:ty; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        pub fn $name(self) -> Self {
            self.execute_unary::<$ty, $ret_ty>($crate::wasm::$name)
        }

        impl_untyped_val!( $($tt)* );
    };
    (
        $(#[$attr:meta])*
        fn $name:ident(lhs: $lhs_ty:ty, rhs: $rhs_ty:ty) -> Result<$ret_ty:ty>; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the fallible `", stringify!($name), "` Wasm instruction.")]
        ///
        /// # Errors
        ///
        $( #[$attr] )*
        pub fn $name(self, rhs: Self) -> Result<Self, TrapCode> {
            self.try_execute_binary::<$lhs_ty, $ret_ty>(rhs, $crate::wasm::$name)
        }

        impl_untyped_val!( $($tt)* );
    };
    (
        fn $name:ident(lhs: $lhs_ty:ty, rhs: $rhs_ty:ty) -> $ret_ty:ty; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        pub fn $name(self, rhs: Self) -> Self {
            self.execute_binary::<$lhs_ty, $rhs_ty, $ret_ty>(rhs, $crate::wasm::$name)
        }

        impl_untyped_val!( $($tt)* );
    };
    () => {};
}

impl UntypedVal {
    impl_untyped_val! {
        // Wasm Integer Instructions

        fn i32_add(lhs: i32, rhs: i32) -> i32;
        fn i64_add(lhs: i64, rhs: i64) -> i64;
        fn i32_sub(lhs: i32, rhs: i32) -> i32;
        fn i64_sub(lhs: i64, rhs: i64) -> i64;
        fn i32_mul(lhs: i32, rhs: i32) -> i32;
        fn i64_mul(lhs: i64, rhs: i64) -> i64;

        fn i32_and(lhs: i32, rhs: i32) -> i32;
        fn i64_and(lhs: i64, rhs: i64) -> i64;
        fn i32_or(lhs: i32, rhs: i32) -> i32;
        fn i64_or(lhs: i64, rhs: i64) -> i64;
        fn i32_xor(lhs: i32, rhs: i32) -> i32;
        fn i64_xor(lhs: i64, rhs: i64) -> i64;

        fn i32_shl(lhs: i32, rhs: i32) -> i32;
        fn i64_shl(lhs: i64, rhs: i64) -> i64;
        fn i32_shr_s(lhs: i32, rhs: i32) -> i32;
        fn i64_shr_s(lhs: i64, rhs: i64) -> i64;
        fn i32_shr_u(lhs: i32, rhs: i32) -> i32;
        fn i64_shr_u(lhs: i64, rhs: i64) -> i64;
        fn i32_rotl(lhs: i32, rhs: i32) -> i32;
        fn i64_rotl(lhs: i64, rhs: i64) -> i64;
        fn i32_rotr(lhs: i32, rhs: i32) -> i32;
        fn i64_rotr(lhs: i64, rhs: i64) -> i64;
    }

    impl_untyped_val! {
        // Wasm Integer Division and Remainder Instructions

        /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
        /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
        fn i32_div_s(lhs: i32, rhs: i32) -> Result<i32>;
        /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
        /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
        fn i64_div_s(lhs: i64, rhs: i64) -> Result<i64>;
        /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
        /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
        fn i32_div_u(lhs: u32, rhs: u32) -> Result<u32>;
        /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
        /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
        fn i64_div_u(lhs: u64, rhs: u64) -> Result<u64>;
        /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
        /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
        fn i32_rem_s(lhs: i32, rhs: i32) -> Result<i32>;
        /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
        /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
        fn i64_rem_s(lhs: i64, rhs: i64) -> Result<i64>;
        /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
        /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
        fn i32_rem_u(lhs: u32, rhs: u32) -> Result<u32>;
        /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
        /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
        fn i64_rem_u(lhs: u64, rhs: u64) -> Result<u64>;
    }

    impl_untyped_val! {
        // Wasm Unary Instructions

        fn i32_clz(value: i32) -> i32;
        fn i64_clz(value: i64) -> i64;
        fn i32_ctz(value: i32) -> i32;
        fn i64_ctz(value: i64) -> i64;
        fn i32_popcnt(value: i32) -> i32;
        fn i64_popcnt(value: i64) -> i64;

        fn i32_eqz(value: i32) -> bool;
        fn i64_eqz(value: i64) -> bool;
    }

    impl_untyped_val! {
        // Wasm Comparison Instructions

        fn i32_eq(lhs: i32, rhs: i32) -> bool;
        fn i64_eq(lhs: i64, rhs: i64) -> bool;
        fn f32_eq(lhs: f32, rhs: f32) -> bool;
        fn f64_eq(lhs: f64, rhs: f64) -> bool;
        fn i32_ne(lhs: i32, rhs: i32) -> bool;
        fn i64_ne(lhs: i64, rhs: i64) -> bool;
        fn f32_ne(lhs: f32, rhs: f32) -> bool;
        fn f64_ne(lhs: f64, rhs: f64) -> bool;

        fn i32_lt_s(lhs: i32, rhs: i32) -> bool;
        fn i64_lt_s(lhs: i64, rhs: i64) -> bool;
        fn i32_lt_u(lhs: u32, rhs: u32) -> bool;
        fn i64_lt_u(lhs: u64, rhs: u64) -> bool;
        fn f32_lt(lhs: f32, rhs: f32) -> bool;
        fn f64_lt(lhs: f64, rhs: f64) -> bool;

        fn i32_le_s(lhs: i32, rhs: i32) -> bool;
        fn i64_le_s(lhs: i64, rhs: i64) -> bool;
        fn i32_le_u(lhs: u32, rhs: u32) -> bool;
        fn i64_le_u(lhs: u64, rhs: u64) -> bool;
        fn f32_le(lhs: f32, rhs: f32) -> bool;
        fn f64_le(lhs: f64, rhs: f64) -> bool;

        fn i32_gt_s(lhs: i32, rhs: i32) -> bool;
        fn i64_gt_s(lhs: i64, rhs: i64) -> bool;
        fn i32_gt_u(lhs: u32, rhs: u32) -> bool;
        fn i64_gt_u(lhs: u64, rhs: u64) -> bool;
        fn f32_gt(lhs: f32, rhs: f32) -> bool;
        fn f64_gt(lhs: f64, rhs: f64) -> bool;

        fn i32_ge_s(lhs: i32, rhs: i32) -> bool;
        fn i64_ge_s(lhs: i64, rhs: i64) -> bool;
        fn i32_ge_u(lhs: u32, rhs: u32) -> bool;
        fn i64_ge_u(lhs: u64, rhs: u64) -> bool;
        fn f32_ge(lhs: f32, rhs: f32) -> bool;
        fn f64_ge(lhs: f64, rhs: f64) -> bool;
    }

    impl_untyped_val! {
        // Wasm Float Instructions

        fn f32_abs(value: f32) -> f32;
        fn f64_abs(value: f64) -> f64;
        fn f32_neg(value: f32) -> f32;
        fn f64_neg(value: f64) -> f64;
        fn f32_ceil(value: f32) -> f32;
        fn f64_ceil(value: f64) -> f64;
        fn f32_floor(value: f32) -> f32;
        fn f64_floor(value: f64) -> f64;
        fn f32_trunc(value: f32) -> f32;
        fn f64_trunc(value: f64) -> f64;
        fn f32_nearest(value: f32) -> f32;
        fn f64_nearest(value: f64) -> f64;
        fn f32_sqrt(value: f32) -> f32;
        fn f64_sqrt(value: f64) -> f64;

        fn f32_add(lhs: f32, rhs: f32) -> f32;
        fn f64_add(lhs: f64, rhs: f64) -> f64;
        fn f32_sub(lhs: f32, rhs: f32) -> f32;
        fn f64_sub(lhs: f64, rhs: f64) -> f64;
        fn f32_mul(lhs: f32, rhs: f32) -> f32;
        fn f64_mul(lhs: f64, rhs: f64) -> f64;
        fn f32_div(lhs: f32, rhs: f32) -> f32;
        fn f64_div(lhs: f64, rhs: f64) -> f64;
        fn f32_min(lhs: f32, rhs: f32) -> f32;
        fn f64_min(lhs: f64, rhs: f64) -> f64;
        fn f32_max(lhs: f32, rhs: f32) -> f32;
        fn f64_max(lhs: f64, rhs: f64) -> f64;
        fn f32_copysign(lhs: f32, rhs: f32) -> f32;
        fn f64_copysign(lhs: f64, rhs: f64) -> f64;
    }

    impl_untyped_val! {
        // Wasm Conversion Routines

        fn i32_wrap_i64(value: i64) -> i32;
        fn i64_extend_i32_s(value: i32) -> i64;
        fn f32_demote_f64(value: f64) -> f32;
        fn f64_promote_f32(value: f32) -> f64;

        /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
        /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `i32` value
        fn i32_trunc_f32_s(value: f32) -> Result<i32>;
        /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
        /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `i64` value
        fn i64_trunc_f32_s(value: f32) -> Result<i64>;
        /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
        /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `u32` value
        fn i32_trunc_f32_u(value: f32) -> Result<u32>;
        /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
        /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `u64` value
        fn i64_trunc_f32_u(value: f32) -> Result<u64>;
        /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
        /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `i32` value
        fn i32_trunc_f64_s(value: f64) -> Result<i32>;
        /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
        /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `i64` value
        fn i64_trunc_f64_s(value: f64) -> Result<i64>;
        /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
        /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `u32` value
        fn i32_trunc_f64_u(value: f64) -> Result<u32>;
        /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
        /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `u64` value
        fn i64_trunc_f64_u(value: f64) -> Result<u64>;

        fn f32_convert_i32_s(value: i32) -> f32;
        fn f32_convert_i32_u(value: u32) -> f32;
        fn f32_convert_i64_s(value: i64) -> f32;
        fn f32_convert_i64_u(value: u64) -> f32;
        fn f64_convert_i32_s(value: i32) -> f64;
        fn f64_convert_i32_u(value: u32) -> f64;
        fn f64_convert_i64_s(value: i64) -> f64;
        fn f64_convert_i64_u(value: u64) -> f64;
    }

    impl_untyped_val! {
        // Wasm `sign-extension` proposal

        fn i32_extend8_s(value: i32) -> i32;
        fn i32_extend16_s(value: i32) -> i32;
        fn i64_extend8_s(value: i64) -> i64;
        fn i64_extend16_s(value: i64) -> i64;
        fn i64_extend32_s(value: i64) -> i64;
    }

    impl_untyped_val! {
        // Wasm `saturating-float-to-int` proposal

        fn i32_trunc_sat_f32_s(value: f32) -> i32;
        fn i32_trunc_sat_f32_u(value: f32) -> u32;
        fn i32_trunc_sat_f64_s(value: f64) -> i32;
        fn i32_trunc_sat_f64_u(value: f64) -> u32;
        fn i64_trunc_sat_f32_s(value: f32) -> i64;
        fn i64_trunc_sat_f32_u(value: f32) -> u64;
        fn i64_trunc_sat_f64_s(value: f64) -> i64;
        fn i64_trunc_sat_f64_u(value: f64) -> u64;
    }
}

/// Macro to help implement generic trait implementations for tuple types.
macro_rules! for_each_tuple {
    ($mac:ident) => {
        $mac!( 0 );
        $mac!( 1 T1);
        $mac!( 2 T1 T2);
        $mac!( 3 T1 T2 T3);
        $mac!( 4 T1 T2 T3 T4);
        $mac!( 5 T1 T2 T3 T4 T5);
        $mac!( 6 T1 T2 T3 T4 T5 T6);
        $mac!( 7 T1 T2 T3 T4 T5 T6 T7);
        $mac!( 8 T1 T2 T3 T4 T5 T6 T7 T8);
        $mac!( 9 T1 T2 T3 T4 T5 T6 T7 T8 T9);
        $mac!(10 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10);
        $mac!(11 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11);
        $mac!(12 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12);
        $mac!(13 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13);
        $mac!(14 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14);
        $mac!(15 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15);
        $mac!(16 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16);
    }
}

/// An error that may occur upon encoding or decoding slices of [`UntypedVal`].
#[derive(Debug, Copy, Clone)]
pub enum UntypedError {
    /// The [`UntypedVal`] slice length did not match `Self`.
    InvalidLen,
}

impl UntypedError {
    /// Creates a new `InvalidLen` [`UntypedError`].
    #[cold]
    pub fn invalid_len() -> Self {
        Self::InvalidLen
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UntypedError {}

impl Display for UntypedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UntypedError::InvalidLen => {
                write!(f, "mismatched length of the untyped slice",)
            }
        }
    }
}

impl UntypedVal {
    /// Decodes the slice of [`UntypedVal`] as a value of type `T`.
    ///
    /// # Note
    ///
    /// `T` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `T` and the length of `slice` does not match.
    pub fn decode_slice<T>(slice: &[Self]) -> Result<T, UntypedError>
    where
        T: DecodeUntypedSlice,
    {
        <T as DecodeUntypedSlice>::decode_untyped_slice(slice)
    }

    /// Encodes the slice of [`UntypedVal`] from the given value of type `T`.
    ///
    /// # Note
    ///
    /// `T` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `T` and the length of `slice` does not match.
    pub fn encode_slice<T>(slice: &mut [Self], input: T) -> Result<(), UntypedError>
    where
        T: EncodeUntypedSlice,
    {
        <T as EncodeUntypedSlice>::encode_untyped_slice(input, slice)
    }
}

/// Tuple types that allow to decode a slice of [`UntypedVal`].
pub trait DecodeUntypedSlice: Sized {
    /// Decodes the slice of [`UntypedVal`] as a value of type `Self`.
    ///
    /// # Note
    ///
    /// `Self` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `Self` and the length of `slice` does not match.
    fn decode_untyped_slice(params: &[UntypedVal]) -> Result<Self, UntypedError>;
}

impl<T1> DecodeUntypedSlice for T1
where
    T1: From<UntypedVal>,
{
    #[inline]
    fn decode_untyped_slice(results: &[UntypedVal]) -> Result<Self, UntypedError> {
        <(T1,) as DecodeUntypedSlice>::decode_untyped_slice(results).map(|t| t.0)
    }
}

macro_rules! impl_decode_untyped_slice {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> DecodeUntypedSlice for ($($tuple,)*)
        where
            $(
                $tuple: From<UntypedVal>
            ),*
        {
            #[allow(non_snake_case)]
            #[inline]
            fn decode_untyped_slice(results: &[UntypedVal]) -> Result<Self, UntypedError> {
                match results {
                    &[ $($tuple),* ] => Ok((
                        $(
                            <$tuple as From<UntypedVal>>::from($tuple),
                        )*
                    )),
                    _ => Err(UntypedError::invalid_len()),
                }
            }
        }
    };
}
for_each_tuple!(impl_decode_untyped_slice);

/// Tuple types that allow to encode a slice of [`UntypedVal`].
pub trait EncodeUntypedSlice {
    /// Encodes the slice of [`UntypedVal`] from the given value of type `Self`.
    ///
    /// # Note
    ///
    /// `Self` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `Self` and the length of `slice` does not match.
    fn encode_untyped_slice(self, results: &mut [UntypedVal]) -> Result<(), UntypedError>;
}

impl<T1> EncodeUntypedSlice for T1
where
    T1: Into<UntypedVal>,
{
    #[inline]
    fn encode_untyped_slice(self, results: &mut [UntypedVal]) -> Result<(), UntypedError> {
        <(T1,) as EncodeUntypedSlice>::encode_untyped_slice((self,), results)
    }
}

macro_rules! impl_encode_untyped_slice {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> EncodeUntypedSlice for ($($tuple,)*)
        where
            $(
                $tuple: Into<UntypedVal>
            ),*
        {
            #[allow(non_snake_case)]
            #[inline]
            fn encode_untyped_slice<'a>(self, results: &'a mut [UntypedVal]) -> Result<(), UntypedError> {
                let Ok(_results) = <&'a mut [UntypedVal; $n]>::try_from(results) else {
                    return Err(UntypedError::invalid_len())
                };
                let ( $( $tuple ,)* ) = self;
                let mut _i = 0;
                $(
                    _results[_i] = <$tuple as Into<UntypedVal>>::into($tuple);
                    _i += 1;
                )*
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_encode_untyped_slice);
