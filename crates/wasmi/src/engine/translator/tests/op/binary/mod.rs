//! Translation tests for all generic binary Wasm instructions that do not fit a certain group.
//!
//! # Note
//!
//! These tests include Wasm arithmetic, logical, bitwise, shift and rotate instructions.

use super::*;
use crate::ir::IntoShiftAmount;
use core::num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64};

mod f32_add;
mod f32_copysign;
mod f32_div;
mod f32_max;
mod f32_min;
mod f32_mul;
mod f32_sub;
mod f64_add;
mod f64_copysign;
mod f64_div;
mod f64_max;
mod f64_min;
mod f64_mul;
mod f64_sub;
mod i32_add;
mod i32_bitand;
mod i32_bitor;
mod i32_bitxor;
mod i32_div_s;
mod i32_div_u;
mod i32_mul;
mod i32_rem_s;
mod i32_rem_u;
mod i32_rotl;
mod i32_rotr;
mod i32_shl;
mod i32_shr_s;
mod i32_shr_u;
mod i32_sub;
mod i64_add;
mod i64_bitand;
mod i64_bitor;
mod i64_bitxor;
mod i64_div_s;
mod i64_div_u;
mod i64_mul;
mod i64_rem_s;
mod i64_rem_u;
mod i64_rotl;
mod i64_rotr;
mod i64_shl;
mod i64_shr_s;
mod i64_shr_u;
mod i64_sub;

/// Helper to create a [`NonZeroI32`].
fn nonzero_i32(value: i32) -> NonZeroI32 {
    NonZeroI32::new(value).unwrap()
}

/// Helper to create a [`NonZeroU32`].
fn nonzero_u32(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

/// Helper to create a [`NonZeroI64`].
fn nonzero_i64(value: i64) -> NonZeroI64 {
    NonZeroI64::new(value).unwrap()
}

/// Helper to create a [`NonZeroU64`].
fn nonzero_u64(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}

/// Helper to create a [`ShiftAmount`].
fn shamt<T>(value: <T as IntoShiftAmount>::Input) -> <T as IntoShiftAmount>::Output
where
    T: IntoShiftAmount,
{
    T::into_shift_amount(value).unwrap()
}

/// Creates an [`Instruction::ReturnF64Imm32`] from the given `f64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `f32` losslessly.
fn return_f64imm32_instr(value: f64) -> Instruction {
    let const32 = <Const32<f64>>::try_from(value).expect("value must be 32-bit encodable");
    Instruction::return_f64imm32(const32)
}
