use super::*;
use crate::engine::regmach::bytecode::BranchOffset16;

#[test]
#[cfg_attr(miri, ignore)]
fn it_works() {
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
fn it_works_imm() {
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
