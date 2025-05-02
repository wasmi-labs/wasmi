//! Tests for the register-machine Wasmi engine translation implementation.

mod display_wasm;
pub mod driver;
mod fuzz;
mod op;
pub mod wasm_type;

use self::{
    display_wasm::DisplayWasm,
    driver::{ExpectedFunc, TranslationTest},
};
use crate::{
    core::UntypedVal,
    ir::{AnyConst32, BoundedRegSpan, Const16, Const32, Instruction, Reg, RegSpan},
    Config,
    Engine,
    Module,
};
use std::{fmt::Display, format};

/// Compiles the `wasm` encoded bytes into a [`Module`].
///
/// # Panics
///
/// If an error occurred upon module compilation, validation or translation.
fn create_module(config: &Config, bytes: &[u8]) -> Module {
    let engine = Engine::new(config);
    Module::new(&engine, bytes).unwrap()
}

/// Used to swap operands of a `rev` variant [`Instruction`] constructor.
macro_rules! swap_ops {
    ($fn_name:path) => {
        |result: Reg, lhs, rhs| -> Instruction { $fn_name(result, rhs, lhs) }
    };
}
use swap_ops;

/// Used to swap `lhs` and `rhs` operands of a fused `cmp+branch` instruction.
macro_rules! swap_cmp_br_ops {
    ($fn_name:path) => {
        |lhs, rhs, offset: BranchOffset16| -> Instruction { $fn_name(rhs, lhs, offset) }
    };
}
use swap_cmp_br_ops;

/// Used to swap `lhs` and `rhs` operands of a fused `cmp+select` instruction.
macro_rules! swap_cmp_select_ops {
    ($fn_name:path) => {
        |result, lhs, rhs| -> Instruction { $fn_name(result, rhs, lhs) }
    };
}
use swap_cmp_select_ops;

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
fn assert_func_bodies<E, T>(wasm: &str, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = Instruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let mut testcase = TranslationTest::new(wasm);
    for instrs in expected {
        testcase.expect_func_instrs(instrs);
    }
    testcase.run();
}

/// Creates a new [`BoundedRegSpan`] starting with `reg` and with `len`.
fn bspan(reg: impl Into<Reg>, len: u16) -> BoundedRegSpan {
    BoundedRegSpan::new(RegSpan::new(reg.into()), len)
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
    /// Create a new binary [`WasmOp`] for the given [`ValType`]: `fn(T, T) -> T`
    pub const fn binary(ty: WasmType, op: &'static str) -> Self {
        Self::Binary { ty, op }
    }

    /// Create a new compare [`WasmOp`] for the given [`ValType`]: `fn(T, T) -> i32`
    pub const fn cmp(ty: WasmType, op: &'static str) -> Self {
        Self::Cmp { ty, op }
    }

    /// Create a new `load` [`WasmOp`] for the given [`ValType`].
    pub const fn load(ty: WasmType, op: &'static str) -> Self {
        Self::Load { ty, op }
    }

    /// Create a new `store` [`WasmOp`] for the given [`ValType`].
    pub const fn store(ty: WasmType, op: &'static str) -> Self {
        Self::Store { ty, op }
    }

    /// Returns the parameter [`ValType`] of the [`WasmOp`].
    pub fn param_ty(&self) -> WasmType {
        match self {
            Self::Binary { ty, op: _ } => *ty,
            Self::Cmp { ty, op: _ } => *ty,
            Self::Load { .. } => panic!("load instructions have no parameters"),
            Self::Store { ty, op: _ } => *ty,
        }
    }

    /// Returns the result [`ValType`] of the [`WasmOp`].
    pub fn result_ty(&self) -> WasmType {
        match self {
            Self::Binary { ty, op: _ } => *ty,
            Self::Cmp { ty: _, op: _ } => WasmType::I32,
            Self::Load { ty, op: _ } => *ty,
            Self::Store { .. } => panic!("store instructions have no results"),
        }
    }

    /// Returns the display [`ValType`] of the [`WasmOp`].
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
    make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
) {
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (func (param {param_ty}) (param {param_ty}) (result {result_ty})
                local.get 0
                local.get 1
                {wasm_op}
            )
        )
    "#,
    );
    let expected = [
        make_instr(Reg::from(2), Reg::from(0), Reg::from(1)),
        Instruction::return_reg(2),
    ];
    assert_func_bodies(&wasm, [expected]);
}

fn testcase_binary_reg_imm<T>(wasm_op: WasmOp, value: T) -> TranslationTest
where
    T: Copy,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {param_ty}.const {display_value}
                {wasm_op}
            )
        )
    "#,
    );
    TranslationTest::new(&wasm)
}

fn testcase_binary_imm_reg<T>(wasm_op: WasmOp, value: T) -> TranslationTest
where
    T: Copy,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                {param_ty}.const {display_value}
                local.get 0
                {wasm_op}
            )
        )
    "#,
    );
    TranslationTest::new(&wasm)
}

/// Variant of [`test_binary_reg_imm16`] where the `rhs` operand is an immediate value.
fn test_binary_reg_imm16_rhs<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
) where
    T: Copy + TryInto<Const16<T>>,
    DisplayWasm<T>: Display,
{
    let immediate: Const16<T> = value
        .try_into()
        .unwrap_or_else(|_| panic!("failed to convert {} to Const16", DisplayWasm::from(value)));
    let expected = [
        make_instr(Reg::from(1), Reg::from(0), immediate),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(wasm_op, value, expected).run()
}

/// Variant of [`test_binary_reg_imm16`] where the `lhs` operand is an immediate value.
fn test_binary_reg_imm16_lhs<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Reg, lhs: Const16<T>, rhs: Reg) -> Instruction,
) where
    T: Copy + TryInto<Const16<T>>,
    DisplayWasm<T>: Display,
{
    let immediate: Const16<T> = value
        .try_into()
        .unwrap_or_else(|_| panic!("failed to convert {} to Const16", DisplayWasm::from(value)));
    let expected = [
        make_instr(Reg::from(1), immediate, Reg::from(0)),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_lhs_with(wasm_op, value, expected).run()
}

fn test_binary_reg_imm32<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let expected = [
        make_instr(Reg::from(1), Reg::from(0), Reg::from(-1)),
        Instruction::return_reg(1),
    ];
    let mut testcase = testcase_binary_reg_imm(wasm_op, value);
    testcase.expect_func(ExpectedFunc::new(expected).consts([value.into()]));
    testcase.run()
}

/// Variant of [`test_binary_reg_imm32`] where both operands are swapped.
fn test_binary_reg_imm32_lhs<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let expected = [
        make_instr(Reg::from(1), Reg::from(-1), Reg::from(0)),
        Instruction::return_reg(1),
    ];
    let mut testcase = testcase_binary_imm_reg(wasm_op, value);
    testcase.expect_func(ExpectedFunc::new(expected).consts([value.into()]));
    testcase.run()
}

/// Variant of [`test_binary_reg_imm32`] where both operands are swapped.
fn test_binary_reg_imm32_lhs_commutative<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let expected = [
        make_instr(Reg::from(1), Reg::from(0), Reg::from(-1)),
        Instruction::return_reg(1),
    ];
    let mut testcase = testcase_binary_imm_reg(wasm_op, value);
    testcase.expect_func(ExpectedFunc::new(expected).consts([value.into()]));
    testcase.run()
}

fn test_binary_reg_imm_with<T, E>(wasm_op: WasmOp, value: T, expected: E) -> TranslationTest
where
    T: Copy,
    DisplayWasm<T>: Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let mut testcase = testcase_binary_reg_imm(wasm_op, value);
    testcase.expect_func_instrs(expected);
    testcase
}

fn test_binary_reg_imm_lhs_with<T, E>(wasm_op: WasmOp, value: T, expected: E) -> TranslationTest
where
    T: Copy,
    DisplayWasm<T>: Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let mut testcase = testcase_binary_imm_reg(wasm_op, value);
    testcase.expect_func_instrs(expected);
    testcase
}

fn testcase_binary_consteval<T>(wasm_op: WasmOp, lhs: T, rhs: T) -> TranslationTest
where
    T: Copy,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let display_lhs = DisplayWasm::from(lhs);
    let display_rhs = DisplayWasm::from(rhs);
    let wasm = format!(
        r#"
        (module
            (func (result {result_ty})
                {param_ty}.const {display_lhs}
                {param_ty}.const {display_rhs}
                {wasm_op}
            )
        )
    "#,
    );
    TranslationTest::new(&wasm)
}

fn test_binary_consteval<T, E>(wasm_op: WasmOp, lhs: T, rhs: T, expected: E)
where
    T: Copy,
    DisplayWasm<T>: Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    testcase_binary_consteval(wasm_op, lhs, rhs)
        .expect_func_instrs(expected)
        .run()
}

fn test_binary_same_reg<E>(wasm_op: WasmOp, expected: E)
where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                local.get 0
                {wasm_op}
            )
        )
    "#,
    );
    assert_func_bodies(&wasm, [expected]);
}
