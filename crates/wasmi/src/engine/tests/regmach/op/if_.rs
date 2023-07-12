use super::*;
use crate::engine::bytecode::BranchOffset;

#[test]
fn simple_if_then() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (if (local.get 0)
                    (then)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_eqz(Register::from_u16(0), BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
fn simple_if_then_else() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (if (local.get 0)
                    (then)
                    (else
                        (nop) ;; without the `nop` there would be no `else` branch
                    )
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_eqz(Register::from_u16(0), BranchOffset::from(2)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}
