//! Execution helpers for Wasm or Wasmi instructions.

use crate::{
    value::{LoadInto, StoreFrom},
    ArithmeticOps,
    ExtendInto,
    Float,
    Integer,
    LittleEndianConvert,
    SignExtendFrom,
    TrapCode,
    TruncateSaturateInto,
    TryTruncateInto,
    WrapInto,
};
use core::ops::{Neg, Shl, Shr};

macro_rules! op {
    ( $operator:tt ) => {{
        |lhs, rhs| lhs $operator rhs
    }};
}

macro_rules! impl_untyped_val {
    (
        $(#[$attr:meta])*
        fn $name:ident(value: $ty:ty) -> Result<$ret_ty:ty> = $f:expr; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        ///
        /// # Errors
        ///
        $( #[$attr] )*
        pub fn $name(value: $ty) -> Result<$ret_ty, TrapCode> {
            ($f)(value)
        }

        impl_untyped_val!( $($tt)* );
    };
    (
        fn $name:ident(value: $ty:ty) -> $ret_ty:ty = $f:expr; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        pub fn $name(value: $ty) -> $ret_ty {
            ($f)(value)
        }

        impl_untyped_val!( $($tt)* );
    };
    (
        $(#[$attr:meta])*
        fn $name:ident(lhs: $lhs_ty:ty, rhs: $rhs_ty:ty) -> Result<$ret_ty:ty> = $f:expr; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the fallible `", stringify!($name), "` Wasm instruction.")]
        ///
        /// # Errors
        ///
        $( #[$attr] )*
        pub fn $name(lhs: $lhs_ty, rhs: $rhs_ty) -> Result<$ret_ty, TrapCode> {
            ($f)(lhs, rhs)
        }

        impl_untyped_val!( $($tt)* );
    };
    (
        fn $name:ident(lhs: $lhs_ty:ty, rhs: $rhs_ty:ty) -> $ret_ty:ty = $f:expr; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        pub fn $name(lhs: $lhs_ty, rhs: $rhs_ty) -> $ret_ty {
            ($f)(lhs, rhs)
        }

        impl_untyped_val!( $($tt)* );
    };
    () => {};
}

impl_untyped_val! {
    // Wasm Integer Instructions

    fn i32_add(lhs: i32, rhs: i32) -> i32 = ArithmeticOps::add;
    fn i64_add(lhs: i64, rhs: i64) -> i64 = ArithmeticOps::add;
    fn i32_sub(lhs: i32, rhs: i32) -> i32 = ArithmeticOps::sub;
    fn i64_sub(lhs: i64, rhs: i64) -> i64 = ArithmeticOps::sub;
    fn i32_mul(lhs: i32, rhs: i32) -> i32 = ArithmeticOps::mul;
    fn i64_mul(lhs: i64, rhs: i64) -> i64 = ArithmeticOps::mul;

    fn i32_and(lhs: i32, rhs: i32) -> i32 = op!(&);
    fn i64_and(lhs: i64, rhs: i64) -> i64 = op!(&);
    fn i32_or(lhs: i32, rhs: i32) -> i32 = op!(|);
    fn i64_or(lhs: i64, rhs: i64) -> i64 = op!(|);
    fn i32_xor(lhs: i32, rhs: i32) -> i32 = op!(^);
    fn i64_xor(lhs: i64, rhs: i64) -> i64 = op!(^);

    fn i32_shl(lhs: i32, rhs: i32) -> i32 = |l: i32, r: i32| -> i32 { l.shl(r & 0x1F) };
    fn i64_shl(lhs: i64, rhs: i64) -> i64 = |l: i64, r: i64| -> i64 { l.shl(r & 0x3F) };
    fn i32_shr_s(lhs: i32, rhs: i32) -> i32 = |l: i32, r: i32| -> i32 { l.shr(r & 0x1F) };
    fn i64_shr_s(lhs: i64, rhs: i64) -> i64 = |l: i64, r: i64| -> i64 { l.shr(r & 0x3F) };
    fn i32_shr_u(lhs: i32, rhs: i32) -> i32 = |l: i32, r: i32| -> i32 { (l as u32).shr(r & 0x1F) as _ };
    fn i64_shr_u(lhs: i64, rhs: i64) -> i64 = |l: i64, r: i64| -> i64 { (l as u64).shr(r & 0x3F) as _ };
    fn i32_rotl(lhs: i32, rhs: i32) -> i32 = Integer::rotl;
    fn i64_rotl(lhs: i64, rhs: i64) -> i64 = Integer::rotl;
    fn i32_rotr(lhs: i32, rhs: i32) -> i32 = Integer::rotr;
    fn i64_rotr(lhs: i64, rhs: i64) -> i64 = Integer::rotr;
}

impl_untyped_val! {
    // Wasm Integer Division and Remainder Instructions

    /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
    /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
    fn i32_div_s(lhs: i32, rhs: i32) -> Result<i32> = Integer::div;
    /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
    /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
    fn i64_div_s(lhs: i64, rhs: i64) -> Result<i64> = Integer::div;
    /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
    /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
    fn i32_div_u(lhs: u32, rhs: u32) -> Result<u32> = Integer::div;
    /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
    /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
    fn i64_div_u(lhs: u64, rhs: u64) -> Result<u64> = Integer::div;
    /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
    /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
    fn i32_rem_s(lhs: i32, rhs: i32) -> Result<i32> = Integer::rem;
    /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
    /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
    fn i64_rem_s(lhs: i64, rhs: i64) -> Result<i64> = Integer::rem;
    /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
    /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
    fn i32_rem_u(lhs: u32, rhs: u32) -> Result<u32> = Integer::rem;
    /// - [`TrapCode::IntegerDivisionByZero`]: if `rhs` is zero.
    /// - [`TrapCode::IntegerOverflow`]: if `lhs` is [`i32::MIN`] and `rhs` is `-1`.
    fn i64_rem_u(lhs: u64, rhs: u64) -> Result<u64> = Integer::rem;
}

impl_untyped_val! {
    // Wasm Unary Instructions

    fn i32_clz(value: i32) -> i32 = Integer::leading_zeros;
    fn i64_clz(value: i64) -> i64 = Integer::leading_zeros;
    fn i32_ctz(value: i32) -> i32 = Integer::trailing_zeros;
    fn i64_ctz(value: i64) -> i64 = Integer::trailing_zeros;
    fn i32_popcnt(value: i32) -> i32 = Integer::count_ones;
    fn i64_popcnt(value: i64) -> i64 = Integer::count_ones;

    fn i32_eqz(value: i32) -> bool = |v| v == 0;
    fn i64_eqz(value: i64) -> bool = |v| v == 0;
}

impl_untyped_val! {
    // Wasm Comparison Instructions

    fn i32_eq(lhs: i32, rhs: i32) -> bool = op!(==);
    fn i64_eq(lhs: i64, rhs: i64) -> bool = op!(==);
    fn f32_eq(lhs: f32, rhs: f32) -> bool = op!(==);
    fn f64_eq(lhs: f64, rhs: f64) -> bool = op!(==);
    fn i32_ne(lhs: i32, rhs: i32) -> bool = op!(!=);
    fn i64_ne(lhs: i64, rhs: i64) -> bool = op!(!=);
    fn f32_ne(lhs: f32, rhs: f32) -> bool = op!(!=);
    fn f64_ne(lhs: f64, rhs: f64) -> bool = op!(!=);

    fn i32_lt_s(lhs: i32, rhs: i32) -> bool = op!(<);
    fn i64_lt_s(lhs: i64, rhs: i64) -> bool = op!(<);
    fn i32_lt_u(lhs: u32, rhs: u32) -> bool = op!(<);
    fn i64_lt_u(lhs: u64, rhs: u64) -> bool = op!(<);
    fn f32_lt(lhs: f32, rhs: f32) -> bool = op!(<);
    fn f64_lt(lhs: f64, rhs: f64) -> bool = op!(<);

    fn i32_le_s(lhs: i32, rhs: i32) -> bool = op!(<=);
    fn i64_le_s(lhs: i64, rhs: i64) -> bool = op!(<=);
    fn i32_le_u(lhs: u32, rhs: u32) -> bool = op!(<=);
    fn i64_le_u(lhs: u64, rhs: u64) -> bool = op!(<=);
    fn f32_le(lhs: f32, rhs: f32) -> bool = op!(<=);
    fn f64_le(lhs: f64, rhs: f64) -> bool = op!(<=);

    fn i32_gt_s(lhs: i32, rhs: i32) -> bool = op!(>);
    fn i64_gt_s(lhs: i64, rhs: i64) -> bool = op!(>);
    fn i32_gt_u(lhs: u32, rhs: u32) -> bool = op!(>);
    fn i64_gt_u(lhs: u64, rhs: u64) -> bool = op!(>);
    fn f32_gt(lhs: f32, rhs: f32) -> bool = op!(>);
    fn f64_gt(lhs: f64, rhs: f64) -> bool = op!(>);

    fn i32_ge_s(lhs: i32, rhs: i32) -> bool = op!(>=);
    fn i64_ge_s(lhs: i64, rhs: i64) -> bool = op!(>=);
    fn i32_ge_u(lhs: u32, rhs: u32) -> bool = op!(>=);
    fn i64_ge_u(lhs: u64, rhs: u64) -> bool = op!(>=);
    fn f32_ge(lhs: f32, rhs: f32) -> bool = op!(>=);
    fn f64_ge(lhs: f64, rhs: f64) -> bool = op!(>=);
}

impl_untyped_val! {
    // Wasm Float Instructions

    fn f32_abs(value: f32) -> f32 = Float::abs;
    fn f64_abs(value: f64) -> f64 = Float::abs;
    fn f32_neg(value: f32) -> f32 = Neg::neg;
    fn f64_neg(value: f64) -> f64 = Neg::neg;
    fn f32_ceil(value: f32) -> f32 = Float::ceil;
    fn f64_ceil(value: f64) -> f64 = Float::ceil;
    fn f32_floor(value: f32) -> f32 = Float::floor;
    fn f64_floor(value: f64) -> f64 = Float::floor;
    fn f32_trunc(value: f32) -> f32 = Float::trunc;
    fn f64_trunc(value: f64) -> f64 = Float::trunc;
    fn f32_nearest(value: f32) -> f32 = Float::nearest;
    fn f64_nearest(value: f64) -> f64 = Float::nearest;
    fn f32_sqrt(value: f32) -> f32 = Float::sqrt;
    fn f64_sqrt(value: f64) -> f64 = Float::sqrt;

    fn f32_add(lhs: f32, rhs: f32) -> f32 = ArithmeticOps::add;
    fn f64_add(lhs: f64, rhs: f64) -> f64 = ArithmeticOps::add;
    fn f32_sub(lhs: f32, rhs: f32) -> f32 = ArithmeticOps::sub;
    fn f64_sub(lhs: f64, rhs: f64) -> f64 = ArithmeticOps::sub;
    fn f32_mul(lhs: f32, rhs: f32) -> f32 = ArithmeticOps::mul;
    fn f64_mul(lhs: f64, rhs: f64) -> f64 = ArithmeticOps::mul;
    fn f32_div(lhs: f32, rhs: f32) -> f32 = Float::div;
    fn f64_div(lhs: f64, rhs: f64) -> f64 = Float::div;
    fn f32_min(lhs: f32, rhs: f32) -> f32 = Float::min;
    fn f64_min(lhs: f64, rhs: f64) -> f64 = Float::min;
    fn f32_max(lhs: f32, rhs: f32) -> f32 = Float::max;
    fn f64_max(lhs: f64, rhs: f64) -> f64 = Float::max;
    fn f32_copysign(lhs: f32, rhs: f32) -> f32 = Float::copysign;
    fn f64_copysign(lhs: f64, rhs: f64) -> f64 = Float::copysign;
}

impl_untyped_val! {
    // Wasm Conversion Routines

    fn i32_wrap_i64(value: i64) -> i32 = |v| v as i32;
    fn i64_extend_i32_s(value: i32) -> i64 = i64::from;
    fn f32_demote_f64(value: f64) -> f32 = |v| v as f32;
    fn f64_promote_f32(value: f32) -> f64 = f64::from;

    /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
    /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `i32` value
    fn i32_trunc_f32_s(value: f32) -> Result<i32> = TryTruncateInto::try_truncate_into;
    /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
    /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `i64` value
    fn i64_trunc_f32_s(value: f32) -> Result<i64> = TryTruncateInto::try_truncate_into;
    /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
    /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `u32` value
    fn i32_trunc_f32_u(value: f32) -> Result<u32> = TryTruncateInto::try_truncate_into;
    /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
    /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `u64` value
    fn i64_trunc_f32_u(value: f32) -> Result<u64> = TryTruncateInto::try_truncate_into;
    /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
    /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `i32` value
    fn i32_trunc_f64_s(value: f64) -> Result<i32> = TryTruncateInto::try_truncate_into;
    /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
    /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `i64` value
    fn i64_trunc_f64_s(value: f64) -> Result<i64> = TryTruncateInto::try_truncate_into;
    /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
    /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `u32` value
    fn i32_trunc_f64_u(value: f64) -> Result<u32> = TryTruncateInto::try_truncate_into;
    /// - [`TrapCode::BadConversionToInteger`]: if `value` is NaN
    /// - [`TrapCode::IntegerOverflow`]: if `value` exceeds the bounds of an `u64` value
    fn i64_trunc_f64_u(value: f64) -> Result<u64> = TryTruncateInto::try_truncate_into;

    fn f32_convert_i32_s(value: i32) -> f32 = |v| v as f32;
    fn f32_convert_i32_u(value: u32) -> f32 = |v| v as f32;
    fn f32_convert_i64_s(value: i64) -> f32 = |v| v as f32;
    fn f32_convert_i64_u(value: u64) -> f32 = |v| v as f32;
    fn f64_convert_i32_s(value: i32) -> f64 = f64::from;
    fn f64_convert_i32_u(value: u32) -> f64 = f64::from;
    fn f64_convert_i64_s(value: i64) -> f64 = |v| v as f64;
    fn f64_convert_i64_u(value: u64) -> f64 = |v| v as f64;
}

impl_untyped_val! {
    // Wasm `sign-extension` proposal

    fn i32_extend8_s(value: i32) -> i32 = <_ as SignExtendFrom<i8>>::sign_extend_from;
    fn i32_extend16_s(value: i32) -> i32 = <_ as SignExtendFrom<i16>>::sign_extend_from;
    fn i64_extend8_s(value: i64) -> i64 = <_ as SignExtendFrom<i8>>::sign_extend_from;
    fn i64_extend16_s(value: i64) -> i64 = <_ as SignExtendFrom<i16>>::sign_extend_from;
    fn i64_extend32_s(value: i64) -> i64 = <_ as SignExtendFrom<i32>>::sign_extend_from;
}

impl_untyped_val! {
    // Wasm `saturating-float-to-int` proposal

    fn i32_trunc_sat_f32_s(value: f32) -> i32 = TruncateSaturateInto::truncate_saturate_into;
    fn i32_trunc_sat_f32_u(value: f32) -> u32 = TruncateSaturateInto::truncate_saturate_into;
    fn i32_trunc_sat_f64_s(value: f64) -> i32 = TruncateSaturateInto::truncate_saturate_into;
    fn i32_trunc_sat_f64_u(value: f64) -> u32 = TruncateSaturateInto::truncate_saturate_into;
    fn i64_trunc_sat_f32_s(value: f32) -> i64 = TruncateSaturateInto::truncate_saturate_into;
    fn i64_trunc_sat_f32_u(value: f32) -> u64 = TruncateSaturateInto::truncate_saturate_into;
    fn i64_trunc_sat_f64_s(value: f64) -> i64 = TruncateSaturateInto::truncate_saturate_into;
    fn i64_trunc_sat_f64_u(value: f64) -> u64 = TruncateSaturateInto::truncate_saturate_into;
}

/// Calculates the effective address of a linear memory access.
///
/// # Errors
///
/// If the resulting effective address overflows.
fn effective_address(ptr: u64, offset: u64) -> Result<usize, TrapCode> {
    let Some(address) = ptr.checked_add(offset) else {
        return Err(TrapCode::MemoryOutOfBounds);
    };
    usize::try_from(address).map_err(|_| TrapCode::MemoryOutOfBounds)
}

/// Executes a generic `T.loadN_[s|u]` Wasm operation.
///
/// # Errors
///
/// - If `ptr + offset` overflows.
/// - If `ptr + offset` loads out of bounds from `memory`.
fn load_extend<T, U>(memory: &[u8], ptr: u64, offset: u64) -> Result<T, TrapCode>
where
    U: LittleEndianConvert + ExtendInto<T>,
{
    let address = effective_address(ptr, offset)?;
    load_extend_at::<T, U>(memory, address)
}

/// Executes a generic `T.loadN_[s|u]` Wasm operation.
///
/// # Errors
///
/// If `address` loads out of bounds from `memory`.
fn load_extend_at<T, U>(memory: &[u8], address: usize) -> Result<T, TrapCode>
where
    U: LittleEndianConvert + ExtendInto<T>,
{
    let mut buffer = <<U as LittleEndianConvert>::Bytes as Default>::default();
    buffer.load_into(memory, address)?;
    let value: T = <U as LittleEndianConvert>::from_le_bytes(buffer).extend_into();
    Ok(value)
}

macro_rules! gen_load_fn {
    (
        (fn $load_fn:ident, fn $load_at_fn:ident, $ty:ty); $($rest:tt)*
    ) => {
        gen_load_fn!(
            (fn $load_fn, fn $load_at_fn, $ty => $ty);
        );
        gen_load_fn!($($rest)*);
    };
    (
        (fn $load_fn:ident, fn $load_at_fn:ident, $wrapped:ty => $ty:ty); $($rest:tt)*
    ) => {
        #[doc = concat!("Executes a Wasmi `", stringify!($load_fn), "` instruction.")]
        ///
        /// # Errors
        ///
        /// - If `ptr + offset` overflows.
        /// - If `ptr + offset` loads out of bounds from `memory`.
        pub fn $load_fn(memory: &[u8], ptr: u64, offset: u64) -> Result<$ty, TrapCode> {
            load_extend::<$ty, $wrapped>(memory, ptr, offset)
        }

        #[doc = concat!("Executes a Wasmi `", stringify!($load_at_fn), "` instruction.")]
        ///
        /// # Errors
        ///
        /// If `address` loads out of bounds from `memory`.
        pub fn $load_at_fn(memory: &[u8], address: usize) -> Result<$ty, TrapCode> {
            load_extend_at::<$ty, $wrapped>(memory, address)
        }

        gen_load_fn!($($rest)*);
    };
    () => {};
}

gen_load_fn! {
    (fn load32, fn load32_at, u32);
    (fn load64, fn load64_at, u64);
    (fn i32_load8_s, fn i32_load8_s_at, i8 => i32);
    (fn i32_load8_u, fn i32_load8_u_at, u8 => i32);
    (fn i32_load16_s, fn i32_load16_s_at, i16 => i32);
    (fn i32_load16_u, fn i32_load16_u_at, u16 => i32);
    (fn i64_load8_s, fn i64_load8_s_at, i8 => i64);
    (fn i64_load8_u, fn i64_load8_u_at, u8 => i64);
    (fn i64_load16_s, fn i64_load16_s_at, i16 => i64);
    (fn i64_load16_u, fn i64_load16_u_at, u16 => i64);
    (fn i64_load32_s, fn i64_load32_s_at, i32 => i64);
    (fn i64_load32_u, fn i64_load32_u_at, u32 => i64);
}

/// Executes a generic `T.store[N]` Wasm operation.
///
/// # Errors
///
/// - If `ptr + offset` overflows.
/// - If `ptr + offset` stores out of bounds from `memory`.
fn store_wrap<T, U>(memory: &mut [u8], ptr: u64, offset: u64, value: T) -> Result<(), TrapCode>
where
    T: WrapInto<U>,
    U: LittleEndianConvert,
{
    let address = effective_address(ptr, offset)?;
    store_wrap_at::<T, U>(memory, address, value)
}

/// Executes a generic `T.store[N]` Wasm operation.
///
/// # Errors
///
/// - If `address` stores out of bounds from `memory`.
fn store_wrap_at<T, U>(memory: &mut [u8], address: usize, value: T) -> Result<(), TrapCode>
where
    T: WrapInto<U>,
    U: LittleEndianConvert,
{
    let wrapped = value.wrap_into();
    let buffer = <U as LittleEndianConvert>::into_le_bytes(wrapped);
    buffer.store_from(memory, address)?;
    Ok(())
}

macro_rules! gen_store_fn {
    (
        (fn $store_fn:ident, fn $store_at_fn:ident, $ty:ty); $($rest:tt)*
    ) => {
        gen_store_fn!(
            (fn $store_fn, fn $store_at_fn, $ty => $ty);
        );
        gen_store_fn!($($rest)*);
    };
    (
        (fn $store_fn:ident, fn $store_at_fn:ident, $ty:ty => $wrapped:ty); $($rest:tt)*
    ) => {
        #[doc = concat!("Executes a Wasmi `", stringify!($store_fn), "` instruction.")]
        ///
        /// # Errors
        ///
        /// - If `ptr + offset` overflows.
        /// - If `ptr + offset` stores out of bounds from `memory`.
        pub fn $store_fn(memory: &mut [u8], ptr: u64, offset: u64, value: $ty) -> Result<(), TrapCode> {
            store_wrap::<$ty, $wrapped>(memory, ptr, offset, value)
        }

        #[doc = concat!("Executes a Wasmi `", stringify!($store_at_fn), "` instruction.")]
        ///
        /// # Errors
        ///
        /// If `address` stores out of bounds from `memory`.
        pub fn $store_at_fn(memory: &mut [u8], address: usize, value: $ty) -> Result<(), TrapCode> {
            store_wrap_at::<$ty, $wrapped>(memory, address, value)
        }

        gen_store_fn!($($rest)*);
    };
    () => {};
}

gen_store_fn! {
    (fn store32, fn store32_at, u32);
    (fn store64, fn store64_at, u64);
    (fn i32_store8, fn i32_store8_at, i32 => i8);
    (fn i32_store16, fn i32_store16_at, i32 => i16);
    (fn i64_store8, fn i64_store8_at, i64 => i8);
    (fn i64_store16, fn i64_store16_at, i64 => i16);
    (fn i64_store32, fn i64_store32_at, i64 => i32);
}
