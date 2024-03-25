use super::*;
use crate::engine::bytecode::RegisterSpan;

#[test]
#[cfg_attr(miri, ignore)]
fn merge_copy_0() {
    let wasm = r"
        (module
            (func (param i32 i32 i32 i32 i32)
                (local.set 0 (local.get 2)) ;; copy 0 <- 2
                (local.set 1 (local.get 4)) ;; copy 1 <- 4
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegisterSpan::new(Register::from_i16(0)), Register::from_i16(2), Register::from_i16(4)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn merge_copy_1() {
    let wasm = r"
        (module
            (func (param i32 i32 i32 i32 i32)
                (local.set 0 (local.get 4)) ;; copy 1 <- 4
                (local.set 1 (local.get 2)) ;; copy 0 <- 2
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegisterSpan::new(Register::from_i16(0)), Register::from_i16(4), Register::from_i16(2)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn merge_copy_2() {
    let wasm = r"
        (module
            (func (param i32 i32 i32 i32 i32)
                (local.set 1 (local.get 4)) ;; copy 1 <- 4
                (local.set 0 (local.get 2)) ;; copy 0 <- 2
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegisterSpan::new(Register::from_i16(0)), Register::from_i16(2), Register::from_i16(4)),
            Instruction::Return,
        ])
        .run()
}
