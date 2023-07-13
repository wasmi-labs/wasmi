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

#[test]
fn const_condition() {
    fn test_for(condition: bool) {
        let true_value = 10_i32;
        let false_value = 20_i32;
        let expected = match condition {
            true => true_value,
            false => false_value,
        };
        let condition = i32::from(condition);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result i32)
                    (i32.const {condition})
                    (if (result i32)
                        (then (i32.const {true_value}))
                        (else (i32.const {false_value}))
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func([Instruction::return_imm32(Const32::from(expected))])
            .run()
    }
    test_for(true);
    test_for(false);
}
