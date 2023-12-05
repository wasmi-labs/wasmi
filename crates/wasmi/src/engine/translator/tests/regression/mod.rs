use super::*;
use crate::engine::{bytecode::RegisterSpan, CompiledFunc};

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_0() {
    let wat = include_str!("fuzz_0.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::copy_imm32(Register::from_i16(0), 13.0_f32),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_1() {
    let wat = include_str!("fuzz_1.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::copy_f64imm32(Register::from_i16(0), 13.0_f32),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_3() {
    let wat = include_str!("fuzz_3.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_internal_0(
                RegisterSpan::new(Register::from_i16(0)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::call_internal_0(
                RegisterSpan::new(Register::from_i16(3)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::copy_span_non_overlapping(
                RegisterSpan::new(Register::from_i16(0)),
                RegisterSpan::new(Register::from_i16(2)),
                3,
            ),
            Instruction::branch_table(Register::from_i16(5), 2),
            Instruction::return_span(RegisterSpan::new(Register::from_i16(0)).iter_u16(3)),
            Instruction::return_span(RegisterSpan::new(Register::from_i16(0)).iter_u16(3)),
        ])
        .run()
}
