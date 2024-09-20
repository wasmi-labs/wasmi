mod binary;
mod block;
mod br;
mod br_if;
mod br_table;
mod call;
mod cmp;
mod cmp_br;
mod copy;
mod global_get;
mod global_set;
mod i32_eqz;
mod if_;
mod load;
mod local_preserve;
mod local_set;
mod loop_;
mod memory;
mod ref_;
mod return_;
mod return_call;
mod select;
mod store;
mod table;
mod unary;

use super::{
    bspan,
    display_wasm::DisplayValueType,
    driver::ExpectedFunc,
    swap_ops,
    test_binary_consteval,
    test_binary_reg_imm16,
    test_binary_reg_imm16_rev,
    test_binary_reg_imm32,
    test_binary_reg_imm32_rev,
    test_binary_reg_imm32_rev_commutative,
    test_binary_reg_imm_rev_with,
    test_binary_reg_imm_with,
    test_binary_reg_reg,
    test_binary_same_reg,
    testcase_binary_consteval,
    testcase_binary_imm_reg,
    testcase_binary_reg_imm,
    wasm_type,
    AnyConst32,
    Const16,
    Const32,
    DisplayWasm,
    Instruction,
    Reg,
    TranslationTest,
    WasmOp,
    WasmType,
};
use std::format;

/// Creates an [`Const32<i32>`] from the given `i32` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `i32` losslessly.
#[track_caller]
#[allow(dead_code)] // might be useful later
fn i32imm16(value: i32) -> Const16<i32> {
    <Const16<i32>>::try_from(value)
        .unwrap_or_else(|_| panic!("value must be 16-bit encodable: {}", value))
}

/// Creates an [`Const32<u32>`] from the given `u32` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `u32` losslessly.
#[track_caller]
fn u32imm16(value: u32) -> Const16<u32> {
    <Const16<u32>>::try_from(value)
        .unwrap_or_else(|_| panic!("value must be 16-bit encodable: {}", value))
}

/// Creates an [`Const32<i64>`] from the given `i64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `i32` losslessly.
#[track_caller]
fn i64imm32(value: i64) -> Const32<i64> {
    <Const32<i64>>::try_from(value)
        .unwrap_or_else(|_| panic!("value must be 32-bit encodable: {}", value))
}

/// Creates an [`Const32<f64>`] from the given `i64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `i32` losslessly.
#[track_caller]
fn f64imm32(value: f64) -> Const32<f64> {
    <Const32<f64>>::try_from(value)
        .unwrap_or_else(|_| panic!("value must be 32-bit encodable: {}", value))
}

/// Creates an [`Instruction::ReturnI64Imm32`] from the given `i64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `i32` losslessly.
#[track_caller]
fn return_i64imm32_instr(value: i64) -> Instruction {
    Instruction::return_i64imm32(i64imm32(value))
}

/// Creates an [`Instruction::ReturnNezI64Imm32`] from the given `i64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `i32` losslessly.
#[track_caller]
fn return_nez_i64imm32_instr(condition: Reg, value: i64) -> Instruction {
    Instruction::return_nez_i64imm32(condition, i64imm32(value))
}

/// Creates an [`Instruction::ReturnF64Imm32`] from the given `f64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `f32` losslessly.
#[track_caller]
fn return_f64imm32_instr(value: f64) -> Instruction {
    Instruction::return_f64imm32(f64imm32(value))
}

/// Creates an [`Instruction::ReturnNezF64Imm32`] from the given `f64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `f32` losslessly.
#[track_caller]
fn return_nez_f64imm32_instr(condition: Reg, value: f64) -> Instruction {
    Instruction::return_nez_f64imm32(condition, f64imm32(value))
}
