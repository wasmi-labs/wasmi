//! Tests for the register-machine `wasmi` engine translation implementation.

#![allow(unused_imports)] // TODO: remove
#![cfg(not(miri))]

mod display_wasm;
pub mod driver;
mod op;
pub mod wasm_type;

use self::{display_wasm::DisplayWasm, driver::TranslationTest};
use super::{create_module, wat2wasm};
use crate::{
    engine::{
        bytecode2::{BinInstr, BinInstrImm16, Const16, Const32, Instruction, Register, UnaryInstr},
        const_pool::ConstRef,
        CompiledFunc,
        DedupFuncType,
    },
    Config,
    Engine,
    EngineBackend,
    Module,
};
use std::fmt::Display;
use wasmi_core::{UntypedValue, ValueType};

/// Used to swap operands of a `rev` variant [`Instruction`] constructor.
macro_rules! swap_ops {
    ($fn_name:path) => {
        |result: Register, lhs: Const16, rhs: Register| -> Instruction {
            $fn_name(result, rhs, lhs)
        }
    };
}

use swap_ops;

/// Asserts that the given `wasm` bytes yield functions with expected instructions.
///
/// Uses the given [`Config`] to configure the [`Engine`] that the tests are run on.
///
/// # Note
///
/// This enables the register machine bytecode translation.
///
/// # Panics
///
/// If any of the yielded functions consists of instruction different from the
/// expected instructions for that function.
fn assert_func_bodies<E, T>(wasm_bytes: impl AsRef<[u8]>, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = Instruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let mut testcase = TranslationTest::new(wasm_bytes.as_ref());
    for instrs in expected {
        testcase.expect_func(instrs);
    }
    testcase.run();
}

/// Identifier for a Wasm operator.
///
/// # Note
///
/// This type is mainly used for test Wasm blob generation.
#[derive(Debug, Copy, Clone)]
pub enum WasmOp {
    /// For Wasm functions with signature: `fn(T, T) -> T`
    Binary { ty: WasmType, op: &'static str },
    /// For Wasm functions with signature: `fn(T, T) -> i32`
    Cmp { ty: WasmType, op: &'static str },
    /// For Wasm `load` instructions.
    Load { ty: WasmType, op: &'static str },
    /// For Wasm `store` instructions.
    Store { ty: WasmType, op: &'static str },
}

impl WasmOp {
    /// Create a new binary [`WasmOp`] for the given [`ValueType`]: `fn(T, T) -> T`
    pub const fn binary(ty: WasmType, op: &'static str) -> Self {
        Self::Binary { ty, op }
    }

    /// Create a new compare [`WasmOp`] for the given [`ValueType`]: `fn(T, T) -> i32`
    pub const fn cmp(ty: WasmType, op: &'static str) -> Self {
        Self::Cmp { ty, op }
    }

    /// Create a new `load` [`WasmOp`] for the given [`ValueType`].
    pub const fn load(ty: WasmType, op: &'static str) -> Self {
        Self::Load { ty, op }
    }

    /// Create a new `store` [`WasmOp`] for the given [`ValueType`].
    pub const fn store(ty: WasmType, op: &'static str) -> Self {
        Self::Store { ty, op }
    }

    /// Returns the parameter [`ValueType`] of the [`WasmOp`].
    pub fn param_ty(&self) -> WasmType {
        match self {
            Self::Binary { ty, op: _ } => *ty,
            Self::Cmp { ty, op: _ } => *ty,
            Self::Load { .. } => panic!("load instructions have no parameters"),
            Self::Store { ty, op: _ } => *ty,
        }
    }

    /// Returns the result [`ValueType`] of the [`WasmOp`].
    pub fn result_ty(&self) -> WasmType {
        match self {
            Self::Binary { ty, op: _ } => *ty,
            Self::Cmp { ty: _, op: _ } => WasmType::I32,
            Self::Load { ty, op: _ } => *ty,
            Self::Store { .. } => panic!("store instructions have no results"),
        }
    }

    /// Returns the display [`ValueType`] of the [`WasmOp`].
    pub fn display_ty(&self) -> WasmType {
        match self {
            Self::Binary { .. } => self.param_ty(),
            Self::Cmp { .. } => self.param_ty(),
            Self::Load { .. } => self.result_ty(),
            Self::Store { .. } => self.param_ty(),
        }
    }

    /// Returns the operator identifier of the [`WasmOp`].
    pub fn op(&self) -> &'static str {
        match self {
            WasmOp::Binary { ty: _, op } => op,
            WasmOp::Cmp { ty: _, op } => op,
            WasmOp::Load { ty: _, op } => op,
            WasmOp::Store { ty: _, op } => op,
        }
    }
}

impl Display for WasmOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}", self.display_ty(), self.op())
    }
}

/// A Wasm operator type.
///
/// # Note
///
/// This type is mainly used for test Wasm blob generation.
#[derive(Debug, Copy, Clone)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

impl Display for WasmType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
        }
    }
}

fn test_binary_reg_reg(
    wasm_op: WasmOp,
    make_instr: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
) {
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (param {param_ty}) (result {result_ty})
                local.get 0
                local.get 1
                {wasm_op}
            )
        )
    "#,
    ));
    let expected = [
        make_instr(
            Register::from_u16(2),
            Register::from_u16(0),
            Register::from_u16(1),
        ),
        Instruction::return_reg(2),
    ];
    assert_func_bodies(wasm, [expected]);
}

fn test_binary_reg_imm16<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
) where
    T: Copy + Into<Const16>,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {param_ty}.const {display_value}
                {wasm_op}
            )
        )
    "#,
    ));
    let immediate: Const16 = value.into();
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0), immediate),
        Instruction::return_reg(1),
    ];
    assert_func_bodies(wasm, [expected]);
}

/// Variant of [`test_binary_reg_imm16`] where both operands are swapped.
fn test_binary_reg_imm16_rev<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
) where
    T: Copy + Into<Const16>,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                {param_ty}.const {display_value}
                local.get 0
                {wasm_op}
            )
        )
    "#,
    ));
    let immediate: Const16 = value.into();
    let expected = [
        make_instr(Register::from_u16(1), immediate, Register::from_u16(0)),
        Instruction::return_reg(1),
    ];
    assert_func_bodies(wasm, [expected]);
}

fn test_binary_reg_imm32<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) where
    T: Copy + Into<Const32>,
    DisplayWasm<T>: Display,
{
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::const32(value),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(wasm_op, value, expected).run()
}

/// Variant of [`test_binary_reg_imm32`] where both operands are swapped.
fn test_binary_reg_imm32_rev<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) where
    T: Copy + Into<Const32>,
    DisplayWasm<T>: Display,
{
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::const32(value),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_rev_with(wasm_op, value, expected).run()
}

fn test_binary_reg_imm64<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) where
    T: Copy,
    DisplayWasm<T>: Display,
{
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::ConstRef(ConstRef::from_u32(0)),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(wasm_op, value, expected).run()
}

/// Variant of [`test_binary_reg_imm64`] where both operands are swapped.
fn test_binary_reg_imm64_rev<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) where
    T: Copy,
    DisplayWasm<T>: Display,
{
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::ConstRef(ConstRef::from_u32(0)),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_rev_with(wasm_op, value, expected).run()
}

fn test_binary_reg_imm_with<T, E>(wasm_op: WasmOp, value: T, expected: E) -> TranslationTest
where
    T: Copy,
    DisplayWasm<T>: Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {param_ty}.const {display_value}
                {wasm_op}
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    testcase.expect_func(expected);
    testcase
}

fn test_binary_reg_imm_rev_with<T, E>(wasm_op: WasmOp, value: T, expected: E) -> TranslationTest
where
    T: Copy,
    DisplayWasm<T>: Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                {param_ty}.const {display_value}
                local.get 0
                {wasm_op}
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    testcase.expect_func(expected);
    testcase
}

fn test_binary_consteval<T, E>(wasm_op: WasmOp, lhs: T, rhs: T, expected: E)
where
    T: Copy,
    DisplayWasm<T>: Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let display_lhs = DisplayWasm::from(lhs);
    let display_rhs = DisplayWasm::from(rhs);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (result {result_ty})
                {param_ty}.const {display_lhs}
                {param_ty}.const {display_rhs}
                {wasm_op}
            )
        )
    "#,
    ));
    assert_func_bodies(wasm, [expected]);
}

fn test_binary_same_reg<E>(wasm_op: WasmOp, expected: E)
where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                local.get 0
                {wasm_op}
            )
        )
    "#,
    ));
    assert_func_bodies(wasm, [expected]);
}
