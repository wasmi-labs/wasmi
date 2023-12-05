use super::*;
use crate::engine::bytecode::{BranchOffset, BranchOffset16};

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
fn fuzz_regression_2() {
    let wat = include_str!("fuzz_2.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm(Register::from_i16(0), 0, BranchOffset16::from(2)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}
