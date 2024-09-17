use super::*;
use crate::engine::bytecode::{BranchOffset, RegSpan};

#[test]
#[cfg_attr(miri, ignore)]
fn empty_loop() {
    let wasm = r"
        (module
            (func (loop))
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([Instruction::Return])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nested_empty_loop() {
    let wasm = r"
        (module
            (func (loop (loop)))
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([Instruction::Return])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_1() {
    let wasm = r"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (loop (param i32) (result i32))
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_1_nested() {
    let wasm = r"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (loop (param i32) (result i32)
                    (loop (param i32) (result i32))
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_2() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (loop (param i32 i32) (result i32 i32))
                (i32.add)
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegSpan::new(Reg::from(2)), 0, 1),
            Instruction::i32_add(Reg::from(2), Reg::from(2), Reg::from(3)),
            Instruction::return_reg(Reg::from(2)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_2_nested() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (loop (param i32 i32) (result i32 i32)
                    (loop (param i32 i32) (result i32 i32))
                )
                (i32.add)
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegSpan::new(Reg::from(2)), 0, 1),
            Instruction::i32_add(Reg::from(2), Reg::from(2), Reg::from(3)),
            Instruction::return_reg(Reg::from(2)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn repeat_loop() {
    let wasm = r"
        (module
            (func
                (loop (br 0))
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([Instruction::branch(BranchOffset::from(0))])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn repeat_loop_1() {
    let wasm = r"
        (module
            (func (param i32)
                (local.get 0)
                (loop (param i32) (br 0))
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::branch(BranchOffset::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn repeat_loop_1_copy() {
    let wasm = r"
        (module
            (func (param i32 i32)
                (local.get 0)
                (loop (param i32)
                    (drop)
                    (local.get 1)
                    (br 0)
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(2), Reg::from(0)),
            Instruction::copy(Reg::from(2), Reg::from(1)),
            Instruction::branch(BranchOffset::from(-1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_4_mixed_1() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32)
                (i32.const 10)
                (local.get 0)
                (local.get 1)
                (i32.const 20)
                (loop (param i32 i32 i32 i32) (result i32 i32 i32 i32))
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::copy_many_non_overlapping(RegSpan::new(Reg::from(2)), -1, 0),
                Instruction::register2(1, -2),
                Instruction::return_span(bspan(2, 4)),
            ])
            .consts([10_i32, 20]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_4_mixed_2() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32)
                (local.get 0)
                (local.get 0)
                (local.get 1)
                (local.get 1)
                (loop (param i32 i32 i32 i32) (result i32 i32 i32 i32))
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy_many_non_overlapping(RegSpan::new(Reg::from(2)), 0, 0),
            Instruction::register2(1, 1),
            Instruction::return_span(bspan(2, 4)),
        ])
        .run()
}
