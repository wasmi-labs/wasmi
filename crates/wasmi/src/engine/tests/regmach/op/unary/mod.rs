mod conversion;
mod op;

use super::*;
use crate::engine::{const_pool::ConstRef, tests::regmach::driver::TranslationTest};
use std::fmt::Display;
use wasmi_core::{UntypedValue, F32};

pub trait WasmType: Copy + Display + Into<UntypedValue> + From<UntypedValue> {
    const NAME: &'static str;

    fn return_imm_instr(&self) -> Instruction;
}

impl WasmType for i32 {
    const NAME: &'static str = "i32";

    fn return_imm_instr(&self) -> Instruction {
        Instruction::return_imm32(*self)
    }
}

impl WasmType for i64 {
    const NAME: &'static str = "i64";

    fn return_imm_instr(&self) -> Instruction {
        match i32::try_from(*self) {
            Ok(value) => Instruction::return_i64imm32(value),
            Err(_) => Instruction::return_cref(0),
        }
    }
}

impl WasmType for f32 {
    const NAME: &'static str = "f32";

    fn return_imm_instr(&self) -> Instruction {
        Instruction::ReturnImm32 {
            value: Const32::from_f32(F32::from(*self)),
        }
    }
}

impl WasmType for f64 {
    const NAME: &'static str = "f64";

    fn return_imm_instr(&self) -> Instruction {
        Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }
    }
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn conversion_reg<I, O>(
    wasm_op: &str,
    make_instr: fn(result: Register, input: Register) -> Instruction,
) where
    I: WasmType,
    O: WasmType,
{
    let param_ty = <I as WasmType>::NAME;
    let result_ty = <O as WasmType>::NAME;
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {result_ty}.{wasm_op}
            )
        )
    "#,
    ));
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::return_reg(1),
    ];
    TranslationTest::new(wasm).expect_func(expected).run();
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn unary_reg<T>(wasm_op: &str, make_instr: fn(result: Register, input: Register) -> Instruction)
where
    T: WasmType,
{
    conversion_reg::<T, T>(wasm_op, make_instr)
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn conversion_imm<I, O>(wasm_op: &str, input: I, eval: fn(input: I) -> O)
where
    I: WasmType,
    O: WasmType,
{
    let param_ty = <I as WasmType>::NAME;
    let result_ty = <O as WasmType>::NAME;
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (result {result_ty})
                {param_ty}.const {input}
                {result_ty}.{wasm_op}
            )
        )
    "#,
    ));
    let instr = <O as WasmType>::return_imm_instr(&eval(input));
    let mut testcase = TranslationTest::new(wasm);
    if let Instruction::ReturnImm { value } = &instr {
        testcase.expect_const(*value, eval(input));
    }
    testcase.expect_func([instr]).run();
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn unary_imm<T>(wasm_op: &str, input: T, eval: fn(input: T) -> T)
where
    T: WasmType,
{
    conversion_imm::<T, T>(wasm_op, input, eval)
}
