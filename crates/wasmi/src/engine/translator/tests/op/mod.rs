use crate::core::ValType;

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
mod i32_nez;
mod i64_eqz;
mod i64_nez;
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
    swap_cmp_select_ops,
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

/// Creates an [`Instruction::ReturnF64Imm32`] from the given `f64` value.
///
/// # Panics
///
/// If the `value` cannot be converted into `f32` losslessly.
#[track_caller]
fn return_f64imm32_instr(value: f64) -> Instruction {
    Instruction::return_f64imm32(f64imm32(value))
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

#[derive(Debug, Copy, Clone)]
pub enum CmpOp {
    // i32
    I32And,
    I32Or,
    I32Xor,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32LeS,
    I32LeU,
    I32GtS,
    I32GtU,
    I32GeS,
    I32GeU,
    // i64
    I64And,
    I64Or,
    I64Xor,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64LeS,
    I64LeU,
    I64GtS,
    I64GtU,
    I64GeS,
    I64GeU,
    // f32
    F32Eq,
    F32Ne,
    F32Lt,
    F32Le,
    F32Gt,
    F32Ge,
    // f64
    F64Eq,
    F64Ne,
    F64Lt,
    F64Le,
    F64Gt,
    F64Ge,
}

impl CmpOp {
    /// Returns the Wasm parameter type of the [`CmpOp`].
    pub fn param_ty(self) -> ValType {
        match self {
            CmpOp::I32And
            | CmpOp::I32Or
            | CmpOp::I32Xor
            | CmpOp::I32Eq
            | CmpOp::I32Ne
            | CmpOp::I32LtS
            | CmpOp::I32LtU
            | CmpOp::I32LeS
            | CmpOp::I32LeU
            | CmpOp::I32GtS
            | CmpOp::I32GtU
            | CmpOp::I32GeS
            | CmpOp::I32GeU => ValType::I32,
            CmpOp::I64And
            | CmpOp::I64Or
            | CmpOp::I64Xor
            | CmpOp::I64Eq
            | CmpOp::I64Ne
            | CmpOp::I64LtS
            | CmpOp::I64LtU
            | CmpOp::I64LeS
            | CmpOp::I64LeU
            | CmpOp::I64GtS
            | CmpOp::I64GtU
            | CmpOp::I64GeS
            | CmpOp::I64GeU => ValType::I64,
            CmpOp::F32Eq
            | CmpOp::F32Ne
            | CmpOp::F32Lt
            | CmpOp::F32Le
            | CmpOp::F32Gt
            | CmpOp::F32Ge => ValType::F32,
            CmpOp::F64Eq
            | CmpOp::F64Ne
            | CmpOp::F64Lt
            | CmpOp::F64Le
            | CmpOp::F64Gt
            | CmpOp::F64Ge => ValType::F64,
        }
    }

    /// Returns the Wasm result type of the [`CmpOp`].
    pub fn result_ty(self) -> ValType {
        match self {
            CmpOp::I64And | CmpOp::I64Or | CmpOp::I64Xor => ValType::I64,
            _ => ValType::I32,
        }
    }

    /// Returns a string representation of the Wasm operator without type annotation.
    pub fn op_str(self) -> &'static str {
        match self {
            CmpOp::I32And => "and",
            CmpOp::I32Or => "or",
            CmpOp::I32Xor => "xor",
            CmpOp::I32Eq => "eq",
            CmpOp::I32Ne => "ne",
            CmpOp::I32LtS => "lt_s",
            CmpOp::I32LtU => "lt_u",
            CmpOp::I32LeS => "le_s",
            CmpOp::I32LeU => "le_u",
            CmpOp::I32GtS => "gt_s",
            CmpOp::I32GtU => "gt_u",
            CmpOp::I32GeS => "ge_s",
            CmpOp::I32GeU => "ge_u",
            CmpOp::I64And => "and",
            CmpOp::I64Or => "or",
            CmpOp::I64Xor => "xor",
            CmpOp::I64Eq => "eq",
            CmpOp::I64Ne => "ne",
            CmpOp::I64LtS => "lt_s",
            CmpOp::I64LtU => "lt_u",
            CmpOp::I64LeS => "le_s",
            CmpOp::I64LeU => "le_u",
            CmpOp::I64GtS => "gt_s",
            CmpOp::I64GtU => "gt_u",
            CmpOp::I64GeS => "ge_s",
            CmpOp::I64GeU => "ge_u",
            CmpOp::F32Eq => "eq",
            CmpOp::F32Ne => "ne",
            CmpOp::F32Lt => "lt",
            CmpOp::F32Le => "le",
            CmpOp::F32Gt => "gt",
            CmpOp::F32Ge => "ge",
            CmpOp::F64Eq => "eq",
            CmpOp::F64Ne => "ne",
            CmpOp::F64Lt => "lt",
            CmpOp::F64Le => "le",
            CmpOp::F64Gt => "gt",
            CmpOp::F64Ge => "ge",
        }
    }
}
