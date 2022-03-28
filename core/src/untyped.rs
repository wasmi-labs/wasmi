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
use core::ops::{Neg, Shl, Shr};

/// An untyped [`Value`].
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
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

macro_rules! impl_from_prim {
    ( $( $prim:ty ),* $(,)? ) => {
        $(
            impl From<$prim> for UntypedValue {
                fn from(value: $prim) -> Self {
                    Self { bits: value as u64 }
                }
            }
        )*
    };
}
#[rustfmt::skip]
impl_from_prim!(
    bool,
    i8, i16, i32, i64,
    u8, u16, u32, u64,
    f32, f64,
);

impl From<F32> for UntypedValue {
    fn from(value: F32) -> Self {
        Self {
            bits: value.to_bits() as u64,
        }
    }
}

impl From<F64> for UntypedValue {
    fn from(value: F64) -> Self {
        Self {
            bits: value.to_bits() as u64,
        }
    }
}

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
