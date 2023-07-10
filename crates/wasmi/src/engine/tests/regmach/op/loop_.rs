use super::*;
use crate::engine::bytecode::BranchOffset;

#[test]
fn empty_loop() {
    let wasm = wat2wasm(
        r"
        (module
            (func (loop))
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::Return])
        .run()
}

#[test]
fn nested_empty_loop() {
    let wasm = wat2wasm(
        r"
        (module
            (func (loop (loop)))
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::Return])
        .run()
}

#[test]
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
        .expect_func([
            Instruction::copy(Register::from_u16(1), Register::from_u16(0)),
            Instruction::return_reg(Register::from_u16(1)),
        ])
        .run()
}

#[test]
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
        .expect_func([
            Instruction::copy(Register::from_u16(1), Register::from_u16(0)),
            Instruction::return_reg(Register::from_u16(1)),
        ])
        .run()
}

#[test]
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
        .expect_func([
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::copy(Register::from_u16(3), Register::from_u16(1)),
            Instruction::i32_add(
                Register::from_u16(2),
                Register::from_u16(2),
                Register::from_u16(3),
            ),
            Instruction::return_reg(Register::from_u16(2)),
        ])
        .run()
}

#[test]
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
        .expect_func([
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::copy(Register::from_u16(3), Register::from_u16(1)),
            Instruction::i32_add(
                Register::from_u16(2),
                Register::from_u16(2),
                Register::from_u16(3),
            ),
            Instruction::return_reg(Register::from_u16(2)),
        ])
        .run()
}

#[test]
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
        .expect_func([Instruction::branch(BranchOffset::from(0))])
        .run()
}

#[test]
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
        .expect_func([
            Instruction::copy(Register::from_u16(1), Register::from_u16(0)),
            Instruction::branch(BranchOffset::from(0)),
        ])
        .run()
}

#[test]
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
        .expect_func([
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::copy(Register::from_u16(2), Register::from_u16(1)),
            Instruction::branch(BranchOffset::from(-1)),
        ])
        .run()
}
