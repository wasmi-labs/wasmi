//! Tests for the register-machine `wasmi` engine translation implementation.

#![allow(unused_imports)] // TODO: remove

pub mod driver;
mod op;

use self::driver::TranslationTest;

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

    /// Returns the parameter [`ValueType`] of the [`WasmOp`].
    pub fn param_ty(&self) -> WasmType {
        match self {
            Self::Binary { ty, op: _ } => *ty,
            Self::Cmp { ty, op: _ } => *ty,
        }
    }

    /// Returns the result [`ValueType`] of the [`WasmOp`].
    pub fn result_ty(&self) -> WasmType {
        match self {
            Self::Binary { ty, op: _ } => *ty,
            Self::Cmp { ty: _, op: _ } => WasmType::I32,
        }
    }

    /// Returns the display [`ValueType`] of the [`WasmOp`].
    pub fn display_ty(&self) -> WasmType {
        self.param_ty()
    }

    /// Returns the operator identifier of the [`WasmOp`].
    pub fn op(&self) -> &'static str {
        match self {
            WasmOp::Binary { ty: _, op } => op,
            WasmOp::Cmp { ty: _, op } => op,
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

fn test_binary_reg_imm16(
    wasm_op: WasmOp,
    make_instr: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
) {
    /// This constant value fits into 16 bit and is kinda uninteresting for optimizations.
    const VALUE: i16 = 100;
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {param_ty}.const {VALUE}
                {wasm_op}
            )
        )
    "#,
    ));
    let expected = [
        make_instr(
            Register::from_u16(1),
            Register::from_u16(0),
            Const16::from_i16(VALUE),
        ),
        Instruction::return_reg(1),
    ];
    assert_func_bodies(wasm, [expected]);
}

/// Variant of [`test_binary_reg_imm16`] where both operands are swapped.
fn test_binary_reg_imm16_rev(
    wasm_op: WasmOp,
    make_instr: fn(result: Register, lhs: Const16, rhs: Register) -> Instruction,
) {
    /// This constant value fits into 16 bit and is kinda uninteresting for optimizations.
    const VALUE: i16 = 100;
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                {param_ty}.const {VALUE}
                local.get 0
                {wasm_op}
            )
        )
    "#,
    ));
    let expected = [
        make_instr(
            Register::from_u16(1),
            Const16::from_i16(VALUE),
            Register::from_u16(0),
        ),
        Instruction::return_reg(1),
    ];
    assert_func_bodies(wasm, [expected]);
}

fn test_binary_reg_imm32<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) where
    T: Copy + Display + Into<Const32>,
{
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::const32(value),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(wasm_op, value, expected)
}

/// Variant of [`test_binary_reg_imm32`] where both operands are swapped.
fn test_binary_reg_imm32_rev<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) where
    T: Copy + Display + Into<Const32>,
{
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::const32(value),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_rev_with(wasm_op, value, expected)
}

fn test_binary_reg_imm64<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) where
    T: Copy + Display,
{
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::ConstRef(ConstRef::from_u32(0)),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(wasm_op, value, expected)
}

/// Variant of [`test_binary_reg_imm64`] where both operands are swapped.
fn test_binary_reg_imm64_rev<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) where
    T: Copy + Display,
{
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::ConstRef(ConstRef::from_u32(0)),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_rev_with(wasm_op, value, expected)
}

fn test_binary_reg_imm_with<V, E>(wasm_op: WasmOp, value: V, expected: E)
where
    V: Copy + Display,
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
                {param_ty}.const {value}
                {wasm_op}
            )
        )
    "#,
    ));
    assert_func_bodies(wasm, [expected]);
}

fn test_binary_reg_imm_rev_with<T, E>(wasm_op: WasmOp, value: T, expected: E)
where
    T: Copy + Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                {param_ty}.const {value}
                local.get 0
                {wasm_op}
            )
        )
    "#,
    ));
    assert_func_bodies(wasm, [expected]);
}

fn test_binary_consteval<T, E>(wasm_op: WasmOp, lhs: T, rhs: T, expected: E)
where
    T: Copy + Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (result {result_ty})
                {param_ty}.const {lhs}
                {param_ty}.const {rhs}
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

fn test_reg_nan<E>(wasm_op: WasmOp, expected: E)
where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    test_reg_nan_ext(wasm_op, expected).run()
}

fn test_reg_nan_ext<E>(wasm_op: WasmOp, expected: E) -> TranslationTest
where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    assert!(matches!(wasm_op.param_ty(), WasmType::F32 | WasmType::F64));
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {param_ty}.const nan
                {wasm_op}
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    testcase.expect_func(expected);
    testcase
}

fn test_nan_reg<E>(wasm_op: WasmOp, expected: E)
where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    test_nan_reg_ext(wasm_op, expected).run()
}

fn test_nan_reg_ext<E>(wasm_op: WasmOp, expected: E) -> TranslationTest
where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    assert!(matches!(wasm_op.param_ty(), WasmType::F32 | WasmType::F64));
    let param_ty = wasm_op.param_ty();
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {param_ty}.const nan
                {wasm_op}
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    testcase.expect_func(expected);
    testcase
}
