use super::*;
use crate::ir::RegSpan;

#[test]
#[cfg_attr(miri, ignore)]
fn merge_2_copy_instrs_0() {
    let wasm = r"
        (module
            (func (param i32 i32 i32 i32 i32)
                (local.set 0 (local.get 2)) ;; copy 0 <- 2
                (local.set 1 (local.get 4)) ;; copy 1 <- 4
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), Reg::from(2), Reg::from(4)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn merge_2_copy_instrs_1() {
    let wasm = r"
        (module
            (func (param i32 i32 i32 i32 i32)
                (local.set 0 (local.get 4)) ;; copy 1 <- 4
                (local.set 1 (local.get 2)) ;; copy 0 <- 2
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), Reg::from(4), Reg::from(2)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn merge_2_copy_instrs_2() {
    let wasm = r"
        (module
            (func (param i32 i32 i32 i32 i32)
                (local.set 1 (local.get 4)) ;; copy 1 <- 4
                (local.set 0 (local.get 2)) ;; copy 0 <- 2
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), Reg::from(2), Reg::from(4)),
            Instruction::Return,
        ])
        .run()
}
