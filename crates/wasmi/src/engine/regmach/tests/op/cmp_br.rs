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
