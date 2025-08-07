//! Translation tests for all unary Wasm instructions.

mod conversion;
mod op;

use super::*;
use crate::{core::UntypedVal, TrapCode};
use std::fmt::Display;
use wasm_type::WasmTy;

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary Wasmi instruction.
fn conversion_reg_with<I, O, E>(wasm_op: &str, expected: E)
where
    I: WasmTy,
    O: WasmTy,
    E: IntoIterator<Item = Instruction>,
{
    let param_ty = <I as WasmTy>::NAME;
    let result_ty = <O as WasmTy>::NAME;
    let wasm = format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {result_ty}.{wasm_op}
            )
        )
    "#,
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs(expected)
        .run();
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary Wasmi instruction.
fn conversion_reg<I, O>(wasm_op: &str, make_instr: fn(result: Reg, input: Reg) -> Instruction)
where
    I: WasmTy,
    O: WasmTy,
{
    let expected = [
        make_instr(Reg::from(1), Reg::from(0)),
        Instruction::return_reg(1),
    ];
    conversion_reg_with::<I, O, _>(wasm_op, expected)
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary Wasmi instruction.
fn unary_reg<T>(wasm_op: &str, make_instr: fn(result: Reg, input: Reg) -> Instruction)
where
    T: WasmTy,
{
    conversion_reg::<T, T>(wasm_op, make_instr)
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary Wasmi instruction.
fn conversion_imm<I, O>(wasm_op: &str, input: I, eval: fn(input: I) -> O)
where
    I: WasmTy,
    O: WasmTy,
    DisplayWasm<I>: Display,
{
    let param_ty = <I as WasmTy>::NAME;
    let result_ty = <O as WasmTy>::NAME;
    let wasm_input = DisplayWasm::from(input);
    let wasm = format!(
        r#"
        (module
            (func (result {result_ty})
                {param_ty}.const {wasm_input}
                {result_ty}.{wasm_op}
            )
        )
    "#,
    );
    let result = eval(input);
    let instr = <O as WasmTy>::return_imm_instr(&result);
    let mut testcase = TranslationTest::new(&wasm);
    if let Instruction::ReturnReg { value } = &instr {
        assert!(value.is_const());
        testcase.expect_func(ExpectedFunc::new([instr]).consts([result]));
    } else {
        testcase.expect_func_instrs([instr]);
    }
    testcase.run();
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary Wasmi instruction.
fn unary_imm<T>(wasm_op: &str, input: T, eval: fn(input: T) -> T)
where
    T: WasmTy,
    DisplayWasm<T>: Display,
{
    conversion_imm::<T, T>(wasm_op, input, eval)
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary Wasmi instruction.
fn fallible_conversion_imm_err<I, O>(wasm_op: &str, input: I, eval: fn(input: I) -> TrapCode)
where
    I: WasmTy,
    O: WasmTy,
    DisplayWasm<I>: Display,
{
    let param_ty = <I as WasmTy>::NAME;
    let result_ty = <O as WasmTy>::NAME;
    let wasm_input = DisplayWasm::from(input);
    let wasm = format!(
        r#"
        (module
            (func (result {result_ty})
                {param_ty}.const {wasm_input}
                {result_ty}.{wasm_op}
            )
        )
    "#,
    );
    let trap_code = eval(input);
    TranslationTest::new(&wasm)
        .expect_func_instrs([Instruction::trap(trap_code)])
        .run();
}
