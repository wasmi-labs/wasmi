/// Macro that turns an iterator over `Option<T>` into an iterator over `T`.
///
/// - Filters out all the `None` items yielded by the input iterator.
/// - Allows to specify `Some` items as just `T` as convenience.
macro_rules! iter_filter_opts {
    [ $($item:expr),* $(,)? ] => {{
        [ $( ::core::option::Option::from($item) ),* ].into_iter().filter_map(|x| x)
    }};
}

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
mod wide_arithmetic;

use super::{
    bspan,
    display_wasm::DisplayValueType,
    driver::ExpectedFunc,
    swap_cmp_br_ops,
    swap_ops,
    test_binary_consteval,
    test_binary_reg_imm16_lhs,
    test_binary_reg_imm16_rhs,
    test_binary_reg_imm32,
    test_binary_reg_imm32_lhs,
    test_binary_reg_imm32_lhs_commutative,
    test_binary_reg_imm_lhs_with,
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
use crate::ir::{Address, Address32, Offset16};
use std::{fmt, format};

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

/// Creates an [`Const32<u64>`] from the given `u64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `u64` losslessly.
#[track_caller]
fn u64imm16(value: u64) -> Const16<u64> {
    <Const16<u64>>::try_from(value)
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

/// Creates an [`Offset16`] from the given `offset`.
fn offset16(offset: u16) -> Offset16 {
    Offset16::try_from(u64::from(offset)).unwrap()
}

/// Adjusts a translation test to use memories with that specified index type.
#[derive(Copy, Clone)]
enum IndexType {
    /// The 32-bit index type.
    ///
    /// This is WebAssembly's default.
    Memory32,
    /// The 64-bit index type.
    ///
    /// This got introduced by the Wasm `memory64` proposal.
    Memory64,
}

impl IndexType {
    /// Returns the `.wat` string reprensetation for the [`IndexType`] of a `memory` declaration.
    fn wat(&self) -> &'static str {
        match self {
            Self::Memory32 => "i32",
            Self::Memory64 => "i64",
        }
    }
}

/// Convenience type to create Wat memories with a tagged memory index.
#[derive(Copy, Clone)]
pub struct MemIdx(u32);

impl fmt::Display for MemIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "$mem{}", self.0)
    }
}

impl MemIdx {
    /// Returns `true` if [`MemIdx`] refers to the default Wasm memory index.
    fn is_default(&self) -> bool {
        self.0 == 0
    }

    /// Returns the `$mem{n}` memory index used by some Wasm memory instructions.
    fn instr(&self) -> Option<Instruction> {
        match self.0 {
            0 => None,
            n => Some(Instruction::memory_index(n)),
        }
    }
}

/// Asserts that `ptr+offset` overflow either `i32` or `i64` depending on `index_ty`.
fn assert_overflowing_ptr_offset(index_ty: IndexType, ptr: u64, offset: u64) {
    match index_ty {
        IndexType::Memory32 => {
            let Ok(ptr32) = u32::try_from(ptr) else {
                panic!("ptr must be a 32-bit value but found: {ptr}");
            };
            let Ok(offset32) = u32::try_from(offset) else {
                panic!("offset must be a 32-bit value but found: {offset}");
            };
            assert!(
                ptr32.checked_add(offset32).is_none(),
                "ptr+offset must overflow in this testcase (32-bit)"
            );
        }
        IndexType::Memory64 => {
            assert!(
                ptr.checked_add(offset).is_none(),
                "ptr+offset must overflow in this testcase (64-bit)"
            );
        }
    }
}

/// Returns the effective 32-bit address for `ptr+offset`.
fn effective_address32(ptr: u64, offset: u64) -> Address32 {
    let Some(addr) = ptr.checked_add(offset) else {
        panic!("ptr+offset must not overflow in this testcase")
    };
    let Ok(addr) = Address::try_from(addr) else {
        panic!("ptr+offset must fit in a `usize` for this testcase")
    };
    let Ok(addr32) = Address32::try_from(addr) else {
        panic!("ptr+offset must fit in a `u32` for this testcase")
    };
    addr32
}
