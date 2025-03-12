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
    F32,
    F64,
};
use core::{
    fmt::{self, Display},
    ops::{Neg, Shl, Shr},
};

/// An untyped value.
///
/// Provides a dense and simple interface to all functional Wasm operations.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct UntypedVal {
    /// This inner value is required to have enough bits to represent
    /// all fundamental WebAssembly types `i32`, `i64`, `f32` and `f64`.
    bits: u64,
}

impl UntypedVal {
    /// Creates an [`UntypedVal`] from the given `u64` bits.
    pub const fn from_bits(bits: u64) -> Self {
        Self { bits }
    }

    /// Returns the underlying bits of the [`UntypedVal`].
    pub const fn to_bits(self) -> u64 {
        self.bits
    }
}

macro_rules! impl_from_untyped_for_int {
    ( $( $int:ty ),* $(,)? ) => {
        $(
            impl From<UntypedVal> for $int {
                fn from(untyped: UntypedVal) -> Self {
                    untyped.to_bits() as _
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
                    Self::from_bits(untyped.to_bits() as _)
                }
            }
        )*
    };
}
impl_from_untyped_for_float!(f32, f64, F32, F64);

impl From<UntypedVal> for bool {
    fn from(untyped: UntypedVal) -> Self {
        untyped.to_bits() != 0
    }
}

macro_rules! impl_from_unsigned_prim {
    ( $( $prim:ty ),* $(,)? ) => {
        $(
            impl From<$prim> for UntypedVal {
                #[allow(clippy::cast_lossless)]
                fn from(value: $prim) -> Self {
                    Self { bits: value as _ }
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
                    Self { bits: u64::from(value as $base) }
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
                    Self {
                        bits: u64::from(value.to_bits()),
                    }
                }
            }
        )*
    };
}
impl_from_float!(f32, f64, F32, F64);

macro_rules! op {
    ( $operator:tt ) => {{
        |lhs, rhs| lhs $operator rhs
    }};
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

impl UntypedVal {
    /// Executes a generic `T.loadN_[s|u]` Wasm operation.
    ///
    /// # Errors
    ///
    /// - If `ptr + offset` overflows.
    /// - If `ptr + offset` loads out of bounds from `memory`.
    fn load_extend<T, U>(memory: &[u8], ptr: Self, offset: u64) -> Result<Self, TrapCode>
    where
        T: Into<Self>,
        U: LittleEndianConvert + ExtendInto<T>,
    {
        let ptr = u64::from(ptr);
        let address = effective_address(ptr, offset)?;
        Self::load_extend_at::<T, U>(memory, address)
    }

    /// Executes a generic `T.loadN_[s|u]` Wasm operation.
    ///
    /// # Errors
    ///
    /// If `address` loads out of bounds from `memory`.
    fn load_extend_at<T, U>(memory: &[u8], address: usize) -> Result<Self, TrapCode>
    where
        T: Into<Self>,
        U: LittleEndianConvert + ExtendInto<T>,
    {
        let mut buffer = <<U as LittleEndianConvert>::Bytes as Default>::default();
        buffer.load_into(memory, address)?;
        let value: Self = <U as LittleEndianConvert>::from_le_bytes(buffer)
            .extend_into()
            .into();
        Ok(value)
    }
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
        pub fn $load_fn(memory: &[u8], ptr: Self, offset: u64) -> Result<Self, TrapCode> {
            Self::load_extend::<$ty, $wrapped>(memory, ptr, offset)
        }

        #[doc = concat!("Executes a Wasmi `", stringify!($load_at_fn), "` instruction.")]
        ///
        /// # Errors
        ///
        /// If `address` loads out of bounds from `memory`.
        pub fn $load_at_fn(memory: &[u8], address: usize) -> Result<Self, TrapCode> {
            Self::load_extend_at::<$ty, $wrapped>(memory, address)
        }

        gen_load_fn!($($rest)*);
    };
    () => {};
}

impl UntypedVal {
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
}

impl UntypedVal {}

impl UntypedVal {
    /// Executes a generic `T.store[N]` Wasm operation.
    ///
    /// # Errors
    ///
    /// - If `ptr + offset` overflows.
    /// - If `ptr + offset` stores out of bounds from `memory`.
    fn store_wrap<T, U>(
        memory: &mut [u8],
        ptr: Self,
        offset: u64,
        value: Self,
    ) -> Result<(), TrapCode>
    where
        T: From<Self> + WrapInto<U>,
        U: LittleEndianConvert,
    {
        let ptr = u64::from(ptr);
        let address = effective_address(ptr, offset)?;
        Self::store_wrap_at::<T, U>(memory, address, value)
    }

    /// Executes a generic `T.store[N]` Wasm operation.
    ///
    /// # Errors
    ///
    /// - If `address` stores out of bounds from `memory`.
    fn store_wrap_at<T, U>(memory: &mut [u8], address: usize, value: Self) -> Result<(), TrapCode>
    where
        T: From<Self> + WrapInto<U>,
        U: LittleEndianConvert,
    {
        let wrapped = T::from(value).wrap_into();
        let buffer = <U as LittleEndianConvert>::into_le_bytes(wrapped);
        buffer.store_from(memory, address)?;
        Ok(())
    }
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
        pub fn $store_fn(memory: &mut [u8], ptr: Self, offset: u64, value: Self) -> Result<(), TrapCode> {
            Self::store_wrap::<$ty, $wrapped>(memory, ptr, offset, value)
        }

        #[doc = concat!("Executes a Wasmi `", stringify!($store_at_fn), "` instruction.")]
        ///
        /// # Errors
        ///
        /// If `address` stores out of bounds from `memory`.
        pub fn $store_at_fn(memory: &mut [u8], address: usize, value: Self) -> Result<(), TrapCode> {
            Self::store_wrap_at::<$ty, $wrapped>(memory, address, value)
        }

        gen_store_fn!($($rest)*);
    };
    () => {};
}

impl UntypedVal {
    gen_store_fn! {
        (fn store32, fn store32_at, u32);
        (fn store64, fn store64_at, u64);
        (fn i32_store8, fn i32_store8_at, i32 => i8);
        (fn i32_store16, fn i32_store16_at, i32 => i16);
        (fn i64_store8, fn i64_store8_at, i64 => i8);
        (fn i64_store16, fn i64_store16_at, i64 => i16);
        (fn i64_store32, fn i64_store32_at, i64 => i32);
    }
}

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
    fn execute_binary<T, R>(self, rhs: Self, op: fn(T, T) -> R) -> Self
    where
        T: From<Self>,
        R: Into<Self>,
    {
        op(T::from(self), T::from(rhs)).into()
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
        fn $name:ident(value: $ty:ty) -> Result<$ret_ty:ty> = $f:expr; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        ///
        /// # Errors
        ///
        $( #[$attr] )*
        pub fn $name(self) -> Result<Self, TrapCode> {
            self.try_execute_unary::<$ty, $ret_ty>($f)
        }

        impl_untyped_val!( $($tt)* );
    };
    (
        fn $name:ident(value: $ty:ty) -> $ret_ty:ty = $f:expr; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        pub fn $name(self) -> Self {
            self.execute_unary::<$ty, $ret_ty>($f)
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
        pub fn $name(self, rhs: Self) -> Result<Self, TrapCode> {
            self.try_execute_binary::<$lhs_ty, $ret_ty>(rhs, $f)
        }

        impl_untyped_val!( $($tt)* );
    };
    (
        fn $name:ident(lhs: $lhs_ty:ty, rhs: $rhs_ty:ty) -> $ret_ty:ty = $f:expr; $($tt:tt)*
    ) => {
        #[doc = concat!("Execute the `", stringify!($name), "` Wasm instruction.")]
        pub fn $name(self, rhs: Self) -> Self {
            self.execute_binary::<$lhs_ty, $ret_ty>(rhs, $f)
        }

        impl_untyped_val!( $($tt)* );
    };
    () => {};
}

impl UntypedVal {
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

        fn i32_shl(lhs: i32, rhs: i32) -> i32 = |l, r| l.shl(r & 0x1F);
        fn i64_shl(lhs: i64, rhs: i64) -> i64 = |l, r| l.shl(r & 0x3F);
        fn i32_shr_s(lhs: i32, rhs: i32) -> i32 = |l, r| l.shr(r & 0x1F);
        fn i64_shr_s(lhs: i64, rhs: i64) -> i64 = |l, r| l.shr(r & 0x3F);
        fn i32_shr_u(lhs: u32, rhs: u32) -> u32 = |l, r| l.shr(r & 0x1F);
        fn i64_shr_u(lhs: u64, rhs: u64) -> u64 = |l, r| l.shr(r & 0x3F);
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
        fn i64_extend_i32_s(value: i32) -> i64 = |v| v as i64;
        fn f32_demote_f64(value: f64) -> f32 = |v| v as f32;
        fn f64_promote_f32(value: f32) -> f64 = |v| v as f64;

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
        fn f64_convert_i32_s(value: i32) -> f64 = |v| v as f64;
        fn f64_convert_i32_u(value: u32) -> f64 = |v| v as f64;
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
}

impl UntypedVal {
    /// Combines the two 64-bit `lo` and `hi` into a single `i128` value.
    fn combine128(lo: Self, hi: Self) -> i128 {
        let lo = i128::from(u64::from(lo));
        let hi = i128::from(u64::from(hi));
        (hi << 64) | lo
    }

    /// Splits the single `i128` value into a 64-bit `lo` and `hi` part.
    fn split128(value: i128) -> (Self, Self) {
        let hi = (value >> 64) as u64;
        let lo = value as u64;
        (Self::from(lo), Self::from(hi))
    }

    /// Execute an `i64.add128` Wasm instruction.
    ///
    /// Returns a pair of `(lo, hi)` 64-bit values representing the 128-bit result.
    ///
    /// # Note
    ///
    /// This instruction is part of the Wasm `wide-arithmetic` proposal.
    pub fn i64_add128(lhs_lo: Self, lhs_hi: Self, rhs_lo: Self, rhs_hi: Self) -> (Self, Self) {
        let lhs = Self::combine128(lhs_lo, lhs_hi);
        let rhs = Self::combine128(rhs_lo, rhs_hi);
        let result = lhs.wrapping_add(rhs);
        Self::split128(result)
    }

    /// Execute an `i64.sub128` Wasm instruction.
    ///
    /// Returns a pair of `(lo, hi)` 64-bit values representing the 128-bit result.
    ///
    /// # Note
    ///
    /// This instruction is part of the Wasm `wide-arithmetic` proposal.
    pub fn i64_sub128(lhs_lo: Self, lhs_hi: Self, rhs_lo: Self, rhs_hi: Self) -> (Self, Self) {
        let lhs = Self::combine128(lhs_lo, lhs_hi);
        let rhs = Self::combine128(rhs_lo, rhs_hi);
        let result = lhs.wrapping_sub(rhs);
        Self::split128(result)
    }

    /// Execute an `i64.mul_wide_s` Wasm instruction.
    ///
    /// Returns a pair of `(lo, hi)` 64-bit values representing the 128-bit result.
    ///
    /// # Note
    ///
    /// This instruction is part of the Wasm `wide-arithmetic` proposal.
    pub fn i64_mul_wide_s(self, rhs: Self) -> (Self, Self) {
        let lhs = i128::from(i64::from(self));
        let rhs = i128::from(i64::from(rhs));
        let result = lhs.wrapping_mul(rhs);
        Self::split128(result)
    }

    /// Execute an `i64.mul_wide_s` Wasm instruction.
    ///
    /// Returns a pair of `(lo, hi)` 64-bit values representing the 128-bit result.
    ///
    /// # Note
    ///
    /// This instruction is part of the Wasm `wide-arithmetic` proposal.
    pub fn i64_mul_wide_u(self, rhs: Self) -> (Self, Self) {
        let lhs = u128::from(u64::from(self));
        let rhs = u128::from(u64::from(rhs));
        let result = lhs.wrapping_mul(rhs);
        Self::split128(result as i128)
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
