use crate::{
    ArithmeticOps,
    ExtendInto,
    Float,
    Integer,
    SignExtendFrom,
    TrapCode,
    TruncateSaturateInto,
    TryTruncateInto,
    Value,
    ValueType,
    WrapInto,
    F32,
    F64,
};
use core::{
    fmt::{self, Display},
    ops::{Neg, Shl, Shr},
};

/// An untyped [`Value`].
///
/// Provides a dense and simple interface to all functional Wasm operations.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct UntypedValue {
    /// This inner value is required to have enough bits to represent
    /// all fundamental WebAssembly types `i32`, `i64`, `f32` and `f64`.
    bits: u64,
}

impl UntypedValue {
    /// Returns the underlying bits of the [`UntypedValue`].
    pub fn to_bits(self) -> u64 {
        self.bits
    }

    /// Converts the [`UntypedValue`] into a [`Value`].
    pub fn with_type(self, value_type: ValueType) -> Value {
        match value_type {
            ValueType::I32 => Value::I32(<_>::from(self)),
            ValueType::I64 => Value::I64(<_>::from(self)),
            ValueType::F32 => Value::F32(<_>::from(self)),
            ValueType::F64 => Value::F64(<_>::from(self)),
        }
    }
}

macro_rules! impl_from_untyped_for_int {
    ( $( $int:ty ),* $(,)? ) => {
        $(
            impl From<UntypedValue> for $int {
                fn from(untyped: UntypedValue) -> Self {
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
            impl From<UntypedValue> for $float {
                fn from(untyped: UntypedValue) -> Self {
                    Self::from_bits(untyped.to_bits() as _)
                }
            }
        )*
    };
}
impl_from_untyped_for_float!(f32, f64, F32, F64);

impl From<UntypedValue> for bool {
    fn from(untyped: UntypedValue) -> Self {
        untyped.to_bits() != 0
    }
}

impl From<Value> for UntypedValue {
    fn from(value: Value) -> Self {
        match value {
            Value::I32(value) => value.into(),
            Value::I64(value) => value.into(),
            Value::F32(value) => value.into(),
            Value::F64(value) => value.into(),
        }
    }
}

macro_rules! impl_from_unsigned_prim {
    ( $( $prim:ty ),* $(,)? ) => {
        $(
            impl From<$prim> for UntypedValue {
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
            impl From<$prim> for UntypedValue {
                fn from(value: $prim) -> Self {
                    Self { bits: value as $base as _ }
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
            impl From<$float> for UntypedValue {
                fn from(value: $float) -> Self {
                    Self {
                        bits: value.to_bits() as _,
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

impl UntypedValue {
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

    /// Execute `i32.add` Wasm operation.
    pub fn i32_add(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i32 as ArithmeticOps<i32>>::add)
    }

    /// Execute `i64.add` Wasm operation.
    pub fn i64_add(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i64 as ArithmeticOps<i64>>::add)
    }

    /// Execute `i32.sub` Wasm operation.
    pub fn i32_sub(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i32 as ArithmeticOps<i32>>::sub)
    }

    /// Execute `i64.sub` Wasm operation.
    pub fn i64_sub(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i64 as ArithmeticOps<i64>>::sub)
    }

    /// Execute `i32.mul` Wasm operation.
    pub fn i32_mul(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i32 as ArithmeticOps<i32>>::mul)
    }

    /// Execute `i64.mul` Wasm operation.
    pub fn i64_mul(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i64 as ArithmeticOps<i64>>::mul)
    }

    /// Execute `i32.div_s` Wasm operation.
    pub fn i32_div_s(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <i32 as ArithmeticOps<i32>>::div)
    }

    /// Execute `i64.div_s` Wasm operation.
    pub fn i64_div_s(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <i64 as ArithmeticOps<i64>>::div)
    }

    /// Execute `i32.div_u` Wasm operation.
    pub fn i32_div_u(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <u32 as ArithmeticOps<u32>>::div)
    }

    /// Execute `i64.div_u` Wasm operation.
    pub fn i64_div_u(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <u64 as ArithmeticOps<u64>>::div)
    }

    /// Execute `i32.rem_s` Wasm operation.
    pub fn i32_rem_s(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <i32 as Integer<i32>>::rem)
    }

    /// Execute `i64.rem_s` Wasm operation.
    pub fn i64_rem_s(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <i64 as Integer<i64>>::rem)
    }

    /// Execute `i32.rem_u` Wasm operation.
    pub fn i32_rem_u(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <u32 as Integer<u32>>::rem)
    }

    /// Execute `i64.rem_u` Wasm operation.
    pub fn i64_rem_u(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <u64 as Integer<u64>>::rem)
    }

    /// Execute `i32.and` Wasm operation.
    pub fn i32_and(self, rhs: Self) -> Self {
        self.execute_binary::<i32, _>(rhs, op!(&))
    }

    /// Execute `i64.and` Wasm operation.
    pub fn i64_and(self, rhs: Self) -> Self {
        self.execute_binary::<i64, _>(rhs, op!(&))
    }

    /// Execute `i32.or` Wasm operation.
    pub fn i32_or(self, rhs: Self) -> Self {
        self.execute_binary::<i32, _>(rhs, op!(|))
    }

    /// Execute `i64.or` Wasm operation.
    pub fn i64_or(self, rhs: Self) -> Self {
        self.execute_binary::<i64, _>(rhs, op!(|))
    }

    /// Execute `i32.xor` Wasm operation.
    pub fn i32_xor(self, rhs: Self) -> Self {
        self.execute_binary::<i32, _>(rhs, op!(^))
    }

    /// Execute `i64.xor` Wasm operation.
    pub fn i64_xor(self, rhs: Self) -> Self {
        self.execute_binary::<i64, _>(rhs, op!(^))
    }

    /// Execute `i32.shl` Wasm operation.
    pub fn i32_shl(self, rhs: Self) -> Self {
        self.execute_binary::<i32, _>(rhs, |lhs, rhs| lhs.shl(rhs & 0x1F))
    }

    /// Execute `i64.shl` Wasm operation.
    pub fn i64_shl(self, rhs: Self) -> Self {
        self.execute_binary::<i64, _>(rhs, |lhs, rhs| lhs.shl(rhs & 0x3F))
    }

    /// Execute `i32.shr_s` Wasm operation.
    pub fn i32_shr_s(self, rhs: Self) -> Self {
        self.execute_binary::<i32, _>(rhs, |lhs, rhs| lhs.shr(rhs & 0x1F))
    }

    /// Execute `i64.shr_s` Wasm operation.
    pub fn i64_shr_s(self, rhs: Self) -> Self {
        self.execute_binary::<i64, _>(rhs, |lhs, rhs| lhs.shr(rhs & 0x3F))
    }

    /// Execute `i32.shr_u` Wasm operation.
    pub fn i32_shr_u(self, rhs: Self) -> Self {
        self.execute_binary::<u32, _>(rhs, |lhs, rhs| lhs.shr(rhs & 0x1F))
    }

    /// Execute `i64.shr_u` Wasm operation.
    pub fn i64_shr_u(self, rhs: Self) -> Self {
        self.execute_binary::<u64, _>(rhs, |lhs, rhs| lhs.shr(rhs & 0x3F))
    }

    /// Execute `i32.clz` Wasm operation.
    pub fn i32_clz(self) -> Self {
        self.execute_unary(<i32 as Integer<i32>>::leading_zeros)
    }

    /// Execute `i64.clz` Wasm operation.
    pub fn i64_clz(self) -> Self {
        self.execute_unary(<i64 as Integer<i64>>::leading_zeros)
    }

    /// Execute `i32.ctz` Wasm operation.
    pub fn i32_ctz(self) -> Self {
        self.execute_unary(<i32 as Integer<i32>>::trailing_zeros)
    }

    /// Execute `i64.ctz` Wasm operation.
    pub fn i64_ctz(self) -> Self {
        self.execute_unary(<i64 as Integer<i64>>::trailing_zeros)
    }

    /// Execute `i32.popcnt` Wasm operation.
    pub fn i32_popcnt(self) -> Self {
        self.execute_unary(<i32 as Integer<i32>>::count_ones)
    }

    /// Execute `i64.popcnt` Wasm operation.
    pub fn i64_popcnt(self) -> Self {
        self.execute_unary(<i64 as Integer<i64>>::count_ones)
    }

    /// Execute `i32.rotl` Wasm operation.
    pub fn i32_rotl(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i32 as Integer<i32>>::rotl)
    }

    /// Execute `i64.rotl` Wasm operation.
    pub fn i64_rotl(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i64 as Integer<i64>>::rotl)
    }

    /// Execute `i32.rotr` Wasm operation.
    pub fn i32_rotr(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i32 as Integer<i32>>::rotr)
    }

    /// Execute `i64.rotr` Wasm operation.
    pub fn i64_rotr(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <i64 as Integer<i64>>::rotr)
    }

    /// Execute `i32.eqz` Wasm operation.
    pub fn i32_eqz(self) -> Self {
        self.execute_unary::<i32, bool>(|value| value == 0)
    }

    /// Execute `i64.eqz` Wasm operation.
    pub fn i64_eqz(self) -> Self {
        self.execute_unary::<i64, bool>(|value| value == 0)
    }

    /// Execute `i32.eq` Wasm operation.
    pub fn i32_eq(self, rhs: Self) -> Self {
        self.execute_binary::<i32, bool>(rhs, op!(==))
    }

    /// Execute `i64.eq` Wasm operation.
    pub fn i64_eq(self, rhs: Self) -> Self {
        self.execute_binary::<i64, bool>(rhs, op!(==))
    }

    /// Execute `f32.eq` Wasm operation.
    pub fn f32_eq(self, rhs: Self) -> Self {
        self.execute_binary::<F32, bool>(rhs, op!(==))
    }

    /// Execute `f64.eq` Wasm operation.
    pub fn f64_eq(self, rhs: Self) -> Self {
        self.execute_binary::<F64, bool>(rhs, op!(==))
    }

    /// Execute `i32.ne` Wasm operation.
    pub fn i32_ne(self, rhs: Self) -> Self {
        self.execute_binary::<i32, bool>(rhs, op!(!=))
    }

    /// Execute `i64.ne` Wasm operation.
    pub fn i64_ne(self, rhs: Self) -> Self {
        self.execute_binary::<i64, bool>(rhs, op!(!=))
    }

    /// Execute `f32.ne` Wasm operation.
    pub fn f32_ne(self, rhs: Self) -> Self {
        self.execute_binary::<F32, bool>(rhs, op!(!=))
    }

    /// Execute `f64.ne` Wasm operation.
    pub fn f64_ne(self, rhs: Self) -> Self {
        self.execute_binary::<F64, bool>(rhs, op!(!=))
    }

    /// Execute `i32.lt_s` Wasm operation.
    pub fn i32_lt_s(self, rhs: Self) -> Self {
        self.execute_binary::<i32, bool>(rhs, op!(<))
    }

    /// Execute `i64.lt_s` Wasm operation.
    pub fn i64_lt_s(self, rhs: Self) -> Self {
        self.execute_binary::<i64, bool>(rhs, op!(<))
    }

    /// Execute `i32.lt_u` Wasm operation.
    pub fn i32_lt_u(self, rhs: Self) -> Self {
        self.execute_binary::<u32, bool>(rhs, op!(<))
    }

    /// Execute `i64.lt_u` Wasm operation.
    pub fn i64_lt_u(self, rhs: Self) -> Self {
        self.execute_binary::<u64, bool>(rhs, op!(<))
    }

    /// Execute `f32.lt` Wasm operation.
    pub fn f32_lt(self, rhs: Self) -> Self {
        self.execute_binary::<F32, bool>(rhs, op!(<))
    }

    /// Execute `f64.lt` Wasm operation.
    pub fn f64_lt(self, rhs: Self) -> Self {
        self.execute_binary::<F64, bool>(rhs, op!(<))
    }

    /// Execute `i32.le_s` Wasm operation.
    pub fn i32_le_s(self, rhs: Self) -> Self {
        self.execute_binary::<i32, bool>(rhs, op!(<=))
    }

    /// Execute `i64.le_s` Wasm operation.
    pub fn i64_le_s(self, rhs: Self) -> Self {
        self.execute_binary::<i64, bool>(rhs, op!(<=))
    }

    /// Execute `i32.le_u` Wasm operation.
    pub fn i32_le_u(self, rhs: Self) -> Self {
        self.execute_binary::<u32, bool>(rhs, op!(<=))
    }

    /// Execute `i64.le_u` Wasm operation.
    pub fn i64_le_u(self, rhs: Self) -> Self {
        self.execute_binary::<u64, bool>(rhs, op!(<=))
    }

    /// Execute `f32.le` Wasm operation.
    pub fn f32_le(self, rhs: Self) -> Self {
        self.execute_binary::<F32, bool>(rhs, op!(<=))
    }

    /// Execute `f64.le` Wasm operation.
    pub fn f64_le(self, rhs: Self) -> Self {
        self.execute_binary::<F64, bool>(rhs, op!(<=))
    }

    /// Execute `i32.gt_s` Wasm operation.
    pub fn i32_gt_s(self, rhs: Self) -> Self {
        self.execute_binary::<i32, bool>(rhs, op!(>))
    }

    /// Execute `i64.gt_s` Wasm operation.
    pub fn i64_gt_s(self, rhs: Self) -> Self {
        self.execute_binary::<i64, bool>(rhs, op!(>))
    }

    /// Execute `i32.gt_u` Wasm operation.
    pub fn i32_gt_u(self, rhs: Self) -> Self {
        self.execute_binary::<u32, bool>(rhs, op!(>))
    }

    /// Execute `i64.gt_u` Wasm operation.
    pub fn i64_gt_u(self, rhs: Self) -> Self {
        self.execute_binary::<u64, bool>(rhs, op!(>))
    }

    /// Execute `f32.gt` Wasm operation.
    pub fn f32_gt(self, rhs: Self) -> Self {
        self.execute_binary::<F32, bool>(rhs, op!(>))
    }

    /// Execute `f64.gt` Wasm operation.
    pub fn f64_gt(self, rhs: Self) -> Self {
        self.execute_binary::<F64, bool>(rhs, op!(>))
    }

    /// Execute `i32.ge_s` Wasm operation.
    pub fn i32_ge_s(self, rhs: Self) -> Self {
        self.execute_binary::<i32, bool>(rhs, op!(>=))
    }

    /// Execute `i64.ge_s` Wasm operation.
    pub fn i64_ge_s(self, rhs: Self) -> Self {
        self.execute_binary::<i64, bool>(rhs, op!(>=))
    }

    /// Execute `i32.ge_u` Wasm operation.
    pub fn i32_ge_u(self, rhs: Self) -> Self {
        self.execute_binary::<u32, bool>(rhs, op!(>=))
    }

    /// Execute `i64.ge_u` Wasm operation.
    pub fn i64_ge_u(self, rhs: Self) -> Self {
        self.execute_binary::<u64, bool>(rhs, op!(>=))
    }

    /// Execute `f32.ge` Wasm operation.
    pub fn f32_ge(self, rhs: Self) -> Self {
        self.execute_binary::<F32, bool>(rhs, op!(>=))
    }

    /// Execute `f64.ge` Wasm operation.
    pub fn f64_ge(self, rhs: Self) -> Self {
        self.execute_binary::<F64, bool>(rhs, op!(>=))
    }

    /// Execute `f32.abs` Wasm operation.
    pub fn f32_abs(self) -> Self {
        self.execute_unary(<F32 as Float<F32>>::abs)
    }

    /// Execute `f32.neg` Wasm operation.
    pub fn f32_neg(self) -> Self {
        self.execute_unary(<F32 as Neg>::neg)
    }

    /// Execute `f32.ceil` Wasm operation.
    pub fn f32_ceil(self) -> Self {
        self.execute_unary(<F32 as Float<F32>>::ceil)
    }

    /// Execute `f32.floor` Wasm operation.
    pub fn f32_floor(self) -> Self {
        self.execute_unary(<F32 as Float<F32>>::floor)
    }

    /// Execute `f32.trunc` Wasm operation.
    pub fn f32_trunc(self) -> Self {
        self.execute_unary(<F32 as Float<F32>>::trunc)
    }

    /// Execute `f32.nearest` Wasm operation.
    pub fn f32_nearest(self) -> Self {
        self.execute_unary(<F32 as Float<F32>>::nearest)
    }

    /// Execute `f32.sqrt` Wasm operation.
    pub fn f32_sqrt(self) -> Self {
        self.execute_unary(<F32 as Float<F32>>::sqrt)
    }

    /// Execute `f32.min` Wasm operation.
    pub fn f32_min(self, other: Self) -> Self {
        self.execute_binary(other, <F32 as Float<F32>>::min)
    }

    /// Execute `f32.max` Wasm operation.
    pub fn f32_max(self, other: Self) -> Self {
        self.execute_binary(other, <F32 as Float<F32>>::max)
    }

    /// Execute `f32.copysign` Wasm operation.
    pub fn f32_copysign(self, other: Self) -> Self {
        self.execute_binary(other, <F32 as Float<F32>>::copysign)
    }

    /// Execute `f64.abs` Wasm operation.
    pub fn f64_abs(self) -> Self {
        self.execute_unary(<F64 as Float<F64>>::abs)
    }

    /// Execute `f64.neg` Wasm operation.
    pub fn f64_neg(self) -> Self {
        self.execute_unary(<F64 as Neg>::neg)
    }

    /// Execute `f64.ceil` Wasm operation.
    pub fn f64_ceil(self) -> Self {
        self.execute_unary(<F64 as Float<F64>>::ceil)
    }

    /// Execute `f64.floor` Wasm operation.
    pub fn f64_floor(self) -> Self {
        self.execute_unary(<F64 as Float<F64>>::floor)
    }

    /// Execute `f64.trunc` Wasm operation.
    pub fn f64_trunc(self) -> Self {
        self.execute_unary(<F64 as Float<F64>>::trunc)
    }

    /// Execute `f64.nearest` Wasm operation.
    pub fn f64_nearest(self) -> Self {
        self.execute_unary(<F64 as Float<F64>>::nearest)
    }

    /// Execute `f64.sqrt` Wasm operation.
    pub fn f64_sqrt(self) -> Self {
        self.execute_unary(<F64 as Float<F64>>::sqrt)
    }

    /// Execute `f32.add` Wasm operation.
    pub fn f32_add(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <F32 as ArithmeticOps<F32>>::add)
    }

    /// Execute `f64.add` Wasm operation.
    pub fn f64_add(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <F64 as ArithmeticOps<F64>>::add)
    }

    /// Execute `f32.sub` Wasm operation.
    pub fn f32_sub(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <F32 as ArithmeticOps<F32>>::sub)
    }

    /// Execute `f64.sub` Wasm operation.
    pub fn f64_sub(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <F64 as ArithmeticOps<F64>>::sub)
    }

    /// Execute `f32.mul` Wasm operation.
    pub fn f32_mul(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <F32 as ArithmeticOps<F32>>::mul)
    }

    /// Execute `f64.mul` Wasm operation.
    pub fn f64_mul(self, rhs: Self) -> Self {
        self.execute_binary(rhs, <F64 as ArithmeticOps<F64>>::mul)
    }

    /// Execute `f32.div` Wasm operation.
    pub fn f32_div(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <F32 as ArithmeticOps<F32>>::div)
    }

    /// Execute `f64.div` Wasm operation.
    pub fn f64_div(self, rhs: Self) -> Result<Self, TrapCode> {
        self.try_execute_binary(rhs, <F64 as ArithmeticOps<F64>>::div)
    }

    /// Execute `f64.min` Wasm operation.
    pub fn f64_min(self, other: Self) -> Self {
        self.execute_binary(other, <F64 as Float<F64>>::min)
    }

    /// Execute `f64.max` Wasm operation.
    pub fn f64_max(self, other: Self) -> Self {
        self.execute_binary(other, <F64 as Float<F64>>::max)
    }

    /// Execute `f64.copysign` Wasm operation.
    pub fn f64_copysign(self, other: Self) -> Self {
        self.execute_binary(other, <F64 as Float<F64>>::copysign)
    }

    /// Execute `i32.wrap_i64` Wasm operation.
    pub fn i32_wrap_i64(self) -> Self {
        self.execute_unary(<i64 as WrapInto<i32>>::wrap_into)
    }

    /// Execute `i32.trunc_f32_s` Wasm operation.
    pub fn i32_trunc_f32_s(self) -> Result<Self, TrapCode> {
        self.try_execute_unary(<F32 as TryTruncateInto<i32, TrapCode>>::try_truncate_into)
    }

    /// Execute `i32.trunc_f32_u` Wasm operation.
    pub fn i32_trunc_f32_u(self) -> Result<Self, TrapCode> {
        self.try_execute_unary(<F32 as TryTruncateInto<u32, TrapCode>>::try_truncate_into)
    }

    /// Execute `i32.trunc_f64_s` Wasm operation.
    pub fn i32_trunc_f64_s(self) -> Result<Self, TrapCode> {
        self.try_execute_unary(<F64 as TryTruncateInto<i32, TrapCode>>::try_truncate_into)
    }

    /// Execute `i32.trunc_f64_u` Wasm operation.
    pub fn i32_trunc_f64_u(self) -> Result<Self, TrapCode> {
        self.try_execute_unary(<F64 as TryTruncateInto<u32, TrapCode>>::try_truncate_into)
    }

    /// Execute `i64.extend_i32_s` Wasm operation.
    pub fn i64_extend_i32_s(self) -> Self {
        self.execute_unary(<i32 as ExtendInto<i64>>::extend_into)
    }

    /// Execute `i64.extend_i32_u` Wasm operation.
    pub fn i64_extend_i32_u(self) -> Self {
        self.execute_unary(<u32 as ExtendInto<i64>>::extend_into)
    }

    /// Execute `i64.trunc_f32_s` Wasm operation.
    pub fn i64_trunc_f32_s(self) -> Result<Self, TrapCode> {
        self.try_execute_unary(<F32 as TryTruncateInto<i64, TrapCode>>::try_truncate_into)
    }

    /// Execute `i64.trunc_f32_u` Wasm operation.
    pub fn i64_trunc_f32_u(self) -> Result<Self, TrapCode> {
        self.try_execute_unary(<F32 as TryTruncateInto<u64, TrapCode>>::try_truncate_into)
    }

    /// Execute `i64.trunc_f64_s` Wasm operation.
    pub fn i64_trunc_f64_s(self) -> Result<Self, TrapCode> {
        self.try_execute_unary(<F64 as TryTruncateInto<i64, TrapCode>>::try_truncate_into)
    }

    /// Execute `i64.trunc_f64_u` Wasm operation.
    pub fn i64_trunc_f64_u(self) -> Result<Self, TrapCode> {
        self.try_execute_unary(<F64 as TryTruncateInto<u64, TrapCode>>::try_truncate_into)
    }

    /// Execute `f32.convert_i32_s` Wasm operation.
    pub fn f32_convert_i32_s(self) -> Self {
        self.execute_unary(<i32 as ExtendInto<F32>>::extend_into)
    }

    /// Execute `f32.convert_i32_u` Wasm operation.
    pub fn f32_convert_i32_u(self) -> Self {
        self.execute_unary(<u32 as ExtendInto<F32>>::extend_into)
    }

    /// Execute `f32.convert_i64_s` Wasm operation.
    pub fn f32_convert_i64_s(self) -> Self {
        self.execute_unary(<i64 as WrapInto<F32>>::wrap_into)
    }

    /// Execute `f32.convert_i64_u` Wasm operation.
    pub fn f32_convert_i64_u(self) -> Self {
        self.execute_unary(<u64 as WrapInto<F32>>::wrap_into)
    }

    /// Execute `f32.demote_f64` Wasm operation.
    pub fn f32_demote_f64(self) -> Self {
        self.execute_unary(<F64 as WrapInto<F32>>::wrap_into)
    }

    /// Execute `f64.convert_i32_s` Wasm operation.
    pub fn f64_convert_i32_s(self) -> Self {
        self.execute_unary(<i32 as ExtendInto<F64>>::extend_into)
    }

    /// Execute `f64.convert_i32_u` Wasm operation.
    pub fn f64_convert_i32_u(self) -> Self {
        self.execute_unary(<u32 as ExtendInto<F64>>::extend_into)
    }

    /// Execute `f64.convert_i64_s` Wasm operation.
    pub fn f64_convert_i64_s(self) -> Self {
        self.execute_unary(<i64 as ExtendInto<F64>>::extend_into)
    }

    /// Execute `f64.convert_i64_u` Wasm operation.
    pub fn f64_convert_i64_u(self) -> Self {
        self.execute_unary(<u64 as ExtendInto<F64>>::extend_into)
    }

    /// Execute `f64.promote_f32` Wasm operation.
    pub fn f64_promote_f32(self) -> Self {
        self.execute_unary(<F32 as ExtendInto<F64>>::extend_into)
    }

    /// Execute `i32.extend8_s` Wasm operation.
    pub fn i32_extend8_s(self) -> Self {
        self.execute_unary(<i32 as SignExtendFrom<i8>>::sign_extend_from)
    }

    /// Execute `i32.extend16_s` Wasm operation.
    pub fn i32_extend16_s(self) -> Self {
        self.execute_unary(<i32 as SignExtendFrom<i16>>::sign_extend_from)
    }

    /// Execute `i64.extend8_s` Wasm operation.
    pub fn i64_extend8_s(self) -> Self {
        self.execute_unary(<i64 as SignExtendFrom<i8>>::sign_extend_from)
    }

    /// Execute `i64.extend16_s` Wasm operation.
    pub fn i64_extend16_s(self) -> Self {
        self.execute_unary(<i64 as SignExtendFrom<i16>>::sign_extend_from)
    }

    /// Execute `i64.extend32_s` Wasm operation.
    pub fn i64_extend32_s(self) -> Self {
        self.execute_unary(<i64 as SignExtendFrom<i32>>::sign_extend_from)
    }

    /// Execute `i32.trunc_sat_f32_s` Wasm operation.
    pub fn i32_trunc_sat_f32_s(self) -> Self {
        self.execute_unary(<F32 as TruncateSaturateInto<i32>>::truncate_saturate_into)
    }

    /// Execute `i32.trunc_sat_f32_u` Wasm operation.
    pub fn i32_trunc_sat_f32_u(self) -> Self {
        self.execute_unary(<F32 as TruncateSaturateInto<u32>>::truncate_saturate_into)
    }

    /// Execute `i32.trunc_sat_f64_s` Wasm operation.
    pub fn i32_trunc_sat_f64_s(self) -> Self {
        self.execute_unary(<F64 as TruncateSaturateInto<i32>>::truncate_saturate_into)
    }

    /// Execute `i32.trunc_sat_f64_u` Wasm operation.
    pub fn i32_trunc_sat_f64_u(self) -> Self {
        self.execute_unary(<F64 as TruncateSaturateInto<u32>>::truncate_saturate_into)
    }

    /// Execute `i64.trunc_sat_f32_s` Wasm operation.
    pub fn i64_trunc_sat_f32_s(self) -> Self {
        self.execute_unary(<F32 as TruncateSaturateInto<i64>>::truncate_saturate_into)
    }

    /// Execute `i64.trunc_sat_f32_u` Wasm operation.
    pub fn i64_trunc_sat_f32_u(self) -> Self {
        self.execute_unary(<F32 as TruncateSaturateInto<u64>>::truncate_saturate_into)
    }

    /// Execute `i64.trunc_sat_f64_s` Wasm operation.
    pub fn i64_trunc_sat_f64_s(self) -> Self {
        self.execute_unary(<F64 as TruncateSaturateInto<i64>>::truncate_saturate_into)
    }

    /// Execute `i64.trunc_sat_f64_u` Wasm operation.
    pub fn i64_trunc_sat_f64_u(self) -> Self {
        self.execute_unary(<F64 as TruncateSaturateInto<u64>>::truncate_saturate_into)
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

/// An error that may occur upon encoding or decoding slices of [`UntypedValue`].
#[derive(Debug, Copy, Clone)]
pub enum UntypedError {
    /// The [`UntypedValue`] slice length did not match.
    InvalidLen {
        /// Expected number of [`UntypedValue`] elements.
        expected: usize,
        /// Found number of [`UntypedValue`] elements.
        found: usize,
    },
}

impl Display for UntypedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UntypedError::InvalidLen { expected, found } => {
                write!(
                    f,
                    "encountered invalid length for untyped slice. \
                     expected: {expected}, found: {found}"
                )
            }
        }
    }
}

impl UntypedValue {
    /// Decodes the slice of [`UntypedValue`] as a value of type `T`.
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

    /// Encodes the slice of [`UntypedValue`] from the given value of type `T`.
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

/// Tuple types that allow to decode a slice of [`UntypedValue`].
pub trait DecodeUntypedSlice: Sized {
    /// Decodes the slice of [`UntypedValue`] as a value of type `Self`.
    ///
    /// # Note
    ///
    /// `Self` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `Self` and the length of `slice` does not match.
    fn decode_untyped_slice(params: &[UntypedValue]) -> Result<Self, UntypedError>;
}

impl<T1> DecodeUntypedSlice for T1
where
    T1: From<UntypedValue>,
{
    fn decode_untyped_slice(results: &[UntypedValue]) -> Result<Self, UntypedError> {
        if results.len() != 1 {
            return Err(UntypedError::InvalidLen {
                expected: 1,
                found: results.len(),
            });
        }
        Ok(<T1 as From<UntypedValue>>::from(results[0]))
    }
}

macro_rules! impl_decode_untyped_slice {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> DecodeUntypedSlice for ($($tuple,)*)
        where
            $(
                $tuple: From<UntypedValue>
            ),*
        {
            #[allow(non_snake_case)]
            fn decode_untyped_slice(results: &[UntypedValue]) -> Result<Self, UntypedError> {
                match results {
                    &[$($tuple),*] => Ok((
                        $(
                            <$tuple as From<UntypedValue>>::from($tuple),
                        )*
                    )),
                    _unexpected => {
                        Err(UntypedError::InvalidLen {
                            expected: $n,
                            found: results.len(),
                        })
                    }
                }
            }
        }
    };
}
for_each_tuple!(impl_decode_untyped_slice);

/// Tuple types that allow to encode a slice of [`UntypedValue`].
pub trait EncodeUntypedSlice {
    /// Encodes the slice of [`UntypedValue`] from the given value of type `Self`.
    ///
    /// # Note
    ///
    /// `Self` can either be a single type or a tuple of types depending
    /// on the length of the `slice`.
    ///
    /// # Errors
    ///
    /// If the tuple length of `Self` and the length of `slice` does not match.
    fn encode_untyped_slice(self, results: &mut [UntypedValue]) -> Result<(), UntypedError>;
}

impl<T1> EncodeUntypedSlice for T1
where
    T1: Into<UntypedValue>,
{
    fn encode_untyped_slice(self, results: &mut [UntypedValue]) -> Result<(), UntypedError> {
        if results.len() != 1 {
            return Err(UntypedError::InvalidLen {
                expected: 1,
                found: results.len(),
            });
        }
        results[0] = self.into();
        Ok(())
    }
}

macro_rules! impl_encode_untyped_slice {
    ( $n:literal $( $tuple:ident )* ) => {
        impl<$($tuple),*> EncodeUntypedSlice for ($($tuple,)*)
        where
            $(
                $tuple: Into<UntypedValue>
            ),*
        {
            #[allow(non_snake_case)]
            fn encode_untyped_slice(self, results: &mut [UntypedValue]) -> Result<(), UntypedError> {
                if results.len() != $n {
                    return Err(UntypedError::InvalidLen {
                        expected: $n,
                        found: results.len(),
                    })
                }
                let ($($tuple,)*) = self;
                let converted: [UntypedValue; $n] = [
                    $(
                        <$tuple as Into<UntypedValue>>::into($tuple)
                    ),*
                ];
                results.copy_from_slice(&converted);
                Ok(())
            }
        }
    };
}
for_each_tuple!(impl_encode_untyped_slice);
