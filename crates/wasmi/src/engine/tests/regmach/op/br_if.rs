use super::*;
use crate::engine::bytecode::BranchOffset;

#[test]
fn consteval_return() {
    fn test_for(condition: bool) {
        let expected = match condition {
            true => Register::from_u16(0),
            false => Register::from_u16(1),
        };
        let condition = DisplayWasm::from(i32::from(condition));
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32 i32) (result i32)
                    (local.get 0)
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func([Instruction::return_reg(expected)])
            .run()
    }
    test_for(true);
    test_for(false);
}

#[test]
fn consteval_branch_always() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (block (param i32) (result i32)
                    (i32.const 1) ;; br_if condition: true
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::return_reg(Register::from_u16(2)),
        ])
        .run()
}

#[test]
fn consteval_branch_never() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (block (param i32) (result i32)
                    (i32.const 0) ;; br_if condition: false
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::return_reg(Register::from_u16(1))])
        .run()
}

#[test]
fn return_if_results_0() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (local.get 0)
                (br_if 0)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::return_nez(Register::from_u16(0)),
            Instruction::Return,
        ])
        .run()
}

#[test]
fn return_if_results_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (br_if 0)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::return_nez_reg(Register::from_u16(1), Register::from_u16(0)),
            Instruction::return_reg(Register::from_u16(0)),
        ])
        .run()
}

#[test]
fn branch_if_results_0() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (local.get 0)
                (block (param i32)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_nez(Register::from_u16(0), BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
fn branch_if_results_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (block (param i32 i32) (result i32)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_eqz(Register::from_u16(1), BranchOffset::from(3)),
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::return_reg(Register::from_u16(2)),
        ])
        .run()
}

#[test]
fn branch_if_results_2() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (block (param i32 i32 i32) (result i32 i32)
                    (br_if 0)
                )
                (i32.add)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_eqz(Register::from_u16(2), BranchOffset::from(4)),
            Instruction::copy(Register::from_u16(3), Register::from_u16(0)),
            Instruction::copy(Register::from_u16(4), Register::from_u16(1)),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::copy(Register::from_u16(3), Register::from_u16(0)),
            Instruction::copy(Register::from_u16(4), Register::from_u16(1)),
            Instruction::i32_add(
                Register::from_u16(3),
                Register::from_u16(3),
                Register::from_u16(4),
            ),
            Instruction::return_reg(Register::from_u16(3)),
        ])
        .run()
}
