use super::*;
use crate::engine::{bytecode::BranchOffset, bytecode2::RegisterSpan};

#[test]
#[cfg_attr(miri, ignore)]
fn empty_loop() {
    let wasm = wat2wasm(
        r"
        (module
            (func (loop))
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::Return])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nested_empty_loop() {
    let wasm = wat2wasm(
        r"
        (module
            (func (loop (loop)))
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::Return])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (loop (param i32) (result i32))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_1_nested() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (loop (param i32) (result i32)
                    (loop (param i32) (result i32))
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_2() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (loop (param i32 i32) (result i32 i32))
                (i32.add)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span(
                RegisterSpan::new(Register::from_i16(2)),
                RegisterSpan::new(Register::from_i16(0)),
                2,
            ),
            Instruction::i32_add(
                Register::from_i16(2),
                Register::from_i16(2),
                Register::from_i16(3),
            ),
            Instruction::return_reg(Register::from_i16(2)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_2_nested() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (loop (param i32 i32) (result i32 i32)
                    (loop (param i32 i32) (result i32 i32))
                )
                (i32.add)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span(
                RegisterSpan::new(Register::from_i16(2)),
                RegisterSpan::new(Register::from_i16(0)),
                2,
            ),
            Instruction::i32_add(
                Register::from_i16(2),
                Register::from_i16(2),
                Register::from_i16(3),
            ),
            Instruction::return_reg(Register::from_i16(2)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn repeat_loop() {
    let wasm = wat2wasm(
        r"
        (module
            (func
                (loop (br 0))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::branch(BranchOffset::from(0))])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn repeat_loop_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (local.get 0)
                (loop (param i32) (br 0))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::branch(BranchOffset::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn repeat_loop_1_copy() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (local.get 0)
                (loop (param i32)
                    (drop)
                    (local.get 1)
                    (br 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
            Instruction::copy(Register::from_i16(2), Register::from_i16(1)),
            Instruction::branch(BranchOffset::from(-1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_4_mixed_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32)
                (i32.const 10)
                (local.get 0)
                (local.get 1)
                (i32.const 20)
                (loop (param i32 i32 i32 i32) (result i32 i32 i32 i32))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span(
                RegisterSpan::new(Register::from_i16(3)),
                RegisterSpan::new(Register::from_i16(0)),
                2,
            ),
            Instruction::copy_imm32(Register::from_i16(2), 10),
            Instruction::copy_imm32(Register::from_i16(5), 20),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(2)).iter(4)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn identity_loop_4_mixed_2() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32)
                (local.get 0)
                (local.get 0)
                (local.get 1)
                (local.get 1)
                (loop (param i32 i32 i32 i32) (result i32 i32 i32 i32))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(5), Register::from_i16(1)),
            Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
            Instruction::copy_span(
                RegisterSpan::new(Register::from_i16(3)),
                RegisterSpan::new(Register::from_i16(0)),
                2,
            ),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(2)).iter(4)),
        ])
        .run()
}
