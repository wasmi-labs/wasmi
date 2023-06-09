//! Tests for the register-machine `wasmi` engine translation implementation.

#![allow(unused_imports)] // TODO: remove

mod op;

use super::{create_module, wat2wasm};
use crate::{
    engine::{
        bytecode2::{BinInstr, BinInstrImm16, Const16, Const32, Instruction, Register, UnaryInstr},
        CompiledFunc,
        DedupFuncType,
    },
    Config,
    Engine,
    Module,
};
use std::fmt::Display;
use wasmi_core::UntypedValue;

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
    let mut config = Config::default();
    config.set_register_machine_translation(true);
    assert_func_bodies_with_config(wasm_bytes, &config, expected)
}

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
fn assert_func_bodies_with_config<E, T>(wasm_bytes: impl AsRef<[u8]>, config: &Config, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = Instruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let wasm_bytes = wasm_bytes.as_ref();
    let module = create_module(config, wasm_bytes);
    let engine = module.engine();
    for ((func_type, func_body), expected) in module.internal_funcs().zip(expected) {
        assert_func_body(engine, func_type, func_body, expected);
    }
}

/// Asserts that the given `func_body` consists of the expected instructions.
///
/// # Note
///
/// This enables the register machine bytecode translation.
///
/// # Panics
///
/// If there is an instruction mismatch between the actual instructions in
/// `func_body` and the `expected_instructions`.
fn assert_func_body<E>(
    engine: &Engine,
    func_type: DedupFuncType,
    func_body: CompiledFunc,
    expected_instructions: E,
) where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let expected_instructions = expected_instructions.into_iter();
    let len_expected = expected_instructions.len();
    for (index, actual, expected) in
        expected_instructions
            .into_iter()
            .enumerate()
            .map(|(index, expected)| {
                (
                    index,
                    engine.resolve_instr_2(func_body, index).unwrap_or_else(|| {
                        panic!("encountered missing instruction at position {index}")
                    }),
                    expected,
                )
            })
    {
        assert_eq!(
            actual,
            expected,
            "encountered instruction mismatch for {:?} at position {index}",
            engine.resolve_func_type(&func_type, Clone::clone),
        );
    }
    if let Some(unexpected) = engine.resolve_instr_2(func_body, len_expected) {
        panic!("encountered unexpected instruction at position {len_expected}: {unexpected:?}",);
    }
}

/// Identifier for a Wasm operator.
///
/// # Note
///
/// This type is mainly used for test Wasm blob generation.
#[derive(Debug, Copy, Clone)]
pub enum WasmOp {
    I32(&'static str),
    I64(&'static str),
    F32(&'static str),
    F64(&'static str),
}

impl WasmOp {
    /// Returns the [`WasmType`] of the [`WasmOp`].
    pub fn ty(&self) -> WasmType {
        match self {
            WasmOp::I32(_) => WasmType::I32,
            WasmOp::I64(_) => WasmType::I64,
            WasmOp::F32(_) => WasmType::F32,
            WasmOp::F64(_) => WasmType::F64,
        }
    }
}

impl Display for WasmOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::I32(op) => write!(f, "{}.{op}", self.ty()),
            Self::I64(op) => write!(f, "{}.{op}", self.ty()),
            Self::F32(op) => write!(f, "{}.{op}", self.ty()),
            Self::F64(op) => write!(f, "{}.{op}", self.ty()),
        }
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
    let wasm_ty = wasm_op.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {wasm_ty}) (param {wasm_ty}) (result {wasm_ty})
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
        Instruction::ReturnReg {
            value: Register::from_u16(2),
        },
    ];
    assert_func_bodies(wasm, [expected]);
}

fn test_binary_reg_imm16(
    wasm_op: WasmOp,
    make_instr: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
) {
    /// This constant value fits into 16 bit and is kinda uninteresting for optimizations.
    const VALUE: i16 = 100;
    let wasm_ty = wasm_op.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {wasm_ty}) (result {wasm_ty})
                local.get 0
                {wasm_ty}.const {VALUE}
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
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    assert_func_bodies(wasm, [expected]);
}

/// Variant of [`test_binary_reg_imm16`] where both operands are swapped.
fn test_binary_reg_imm16_rev(
    wasm_op: WasmOp,
    make_instr: fn(result: Register, lhs: Register, rhs: Const16) -> Instruction,
) {
    /// This constant value fits into 16 bit and is kinda uninteresting for optimizations.
    const VALUE: i16 = 100;
    let wasm_ty = wasm_op.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {wasm_ty}) (result {wasm_ty})
                {wasm_ty}.const {VALUE}
                local.get 0
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
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    assert_func_bodies(wasm, [expected]);
}

fn test_binary_reg_imm(
    wasm_op: WasmOp,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) {
    /// Does not fit into 16 bit value.
    const VALUE: i32 = i32::MAX;
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::Const32(Const32::from_i32(VALUE)),
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    test_binary_reg_imm_with(wasm_op, VALUE, expected)
}

/// Variant of [`test_binary_reg_imm`] where both operands are swapped.
fn test_binary_reg_imm_rev(
    wasm_op: WasmOp,
    make_instr: fn(result: Register, lhs: Register) -> Instruction,
) {
    /// Does not fit into 16 bit value.
    const VALUE: i32 = i32::MAX;
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::Const32(Const32::from_i32(VALUE)),
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    test_binary_reg_imm_rev_with(wasm_op, VALUE, expected)
}

fn test_binary_reg_imm_with<V, E>(wasm_op: WasmOp, value: V, expected: E)
where
    V: Copy + Display,
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let wasm_ty = wasm_op.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {wasm_ty}) (result {wasm_ty})
                local.get 0
                {wasm_ty}.const {value}
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
    let wasm_ty = wasm_op.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {wasm_ty}) (result {wasm_ty})
                {wasm_ty}.const {value}
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
    let wasm_ty = wasm_op.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (result {wasm_ty})
                {wasm_ty}.const {lhs}
                {wasm_ty}.const {rhs}
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
    let wasm_ty = wasm_op.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {wasm_ty}) (result {wasm_ty})
                local.get 0
                local.get 0
                {wasm_op}
            )
        )
    "#,
    ));
    assert_func_bodies(wasm, [expected]);
}
