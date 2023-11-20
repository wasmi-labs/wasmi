use super::*;
use crate::engine::{
    bytecode::{BranchOffset, GlobalIdx},
    regmach::bytecode::BranchOffset16,
};

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (loop
                    (local.get 0)
                    (local.get 1)
                    (i32.eq)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(0),
            ),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (loop
                    (local.get 0)
                    (i32.const 1)
                    (i32.eq)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm(
                Register::from_i16(0),
                i32imm16(1_i32),
                BranchOffset16::from(0),
            ),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm_eqz() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (loop
                    (local.get 0)
                    (i32.const 0)
                    (i32.eq)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_eqz(Register::from_i16(0), BranchOffset::from(0_i32)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (block
                    (local.get 0)
                    (local.get 1)
                    (i32.eq)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(1),
            ),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward_nop_copy() {
    let wasm = wat2wasm(
        r"
        (module
            (global $g (mut i32) (i32.const 10))
            (func (param i32 i32) (result i32)
                (global.get $g)
                (block (param i32) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.eq)
                    (br_if 0)
                    (drop)
                    (local.get 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_get(Register::from_i16(2), GlobalIdx::from(0)),
            Instruction::branch_i32_eq(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(2),
            ),
            Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward_multi_value() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (block (result i32)
                    (local.get 0) ;; returned from block if `local.get 0 != 0`
                    (local.get 0)
                    (local.get 1)
                    (i32.eq)
                    (br_if 0)
                    (drop)
                    (local.get 1) ;; returned from block if `local.get 0 == 0`
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_ne(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(3),
            ),
            Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy(Register::from_i16(2), Register::from_i16(1)),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (if
                    (i32.eq
                        (local.get 0)
                        (local.get 1)
                    )
                    (then)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_ne(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(1),
            ),
            Instruction::Return,
        ])
        .run()
}
