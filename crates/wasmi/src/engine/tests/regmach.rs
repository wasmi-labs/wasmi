//! Tests for the register-machine `wasmi` engine translation implementation.

#![allow(unused_imports)] // TODO: remove

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

#[test]
fn i32_add() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (param i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#,
    );
    let expected = [
        Instruction::I32Add(BinInstr {
            result: Register::from_u16(2),
            lhs: Register::from_u16(0),
            rhs: Register::from_u16(1),
        }),
        Instruction::ReturnReg {
            value: Register::from_u16(2),
        },
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn i32_add_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add
            )
        )
    "#,
    );
    let expected = [
        Instruction::I32AddImm16(BinInstrImm16 {
            result: Register::from_u16(1),
            reg_in: Register::from_u16(0),
            imm_in: Const16::from_i16(1),
        }),
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn i32_add_imm_rev() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32)
                i32.const 1
                local.get 0
                i32.add
            )
        )
    "#,
    );
    let expected = [
        Instruction::I32AddImm16(BinInstrImm16 {
            result: Register::from_u16(1),
            reg_in: Register::from_u16(0),
            imm_in: Const16::from_i16(1),
        }),
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn i32_add_imm_big() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32)
                local.get 0
                i32.const 65535 ;; u16::MAX
                i32.add
            )
        )
    "#,
    );
    let expected = [
        Instruction::I32AddImm(UnaryInstr {
            result: Register::from_u16(1),
            input: Register::from_u16(0),
        }),
        Instruction::Const32(Const32::from_i32(u16::MAX as i32)),
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn i32_add_imm_big_rev() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32)
                i32.const 65535 ;; u16::MAX
                local.get 0
                i32.add
            )
        )
    "#,
    );
    let expected = [
        Instruction::I32AddImm(UnaryInstr {
            result: Register::from_u16(1),
            input: Register::from_u16(0),
        }),
        Instruction::Const32(Const32::from_i32(u16::MAX as i32)),
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn i32_add_zero() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.add
            )
        )
    "#,
    );
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn i32_add_zero_rev() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32)
                i32.const 0
                local.get 0
                i32.add
            )
        )
    "#,
    );
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn i32_add_consteval() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (result i32)
                i32.const 1
                i32.const 2
                i32.add
            )
        )
    "#,
    );
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_u32(3),
    }];
    assert_func_bodies(wasm, [expected]);
}
