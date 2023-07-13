use super::*;
use crate::engine::bytecode::BranchOffset;
use wasmi_core::TrapCode;

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
fn if_then_return() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32 i32) (result i32)
                (if (local.get 0)
                    (then
                        (return
                            (i32.add
                                (local.get 1)
                                (local.get 2)
                            )
                        )
                    )
                )
                (i32.const 0)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_eqz(Register::from_u16(0), BranchOffset::from(3)),
            Instruction::i32_add(Register::from_u16(3), Register::from_u16(1), Register::from_u16(2)),
            Instruction::return_reg(Register::from_u16(3)),
            Instruction::return_imm32(Const32::from(0_i32)),
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

#[test]
fn const_condition_trap_then() {
    fn test_for<I>(condition: bool, instrs: I)
    where
        I: IntoIterator<Item = Instruction>,
    {
        let condition = i32::from(condition);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32 i32) (result i32)
                    (i32.const {condition})
                    (if (result i32 i32)
                        (then
                            (unreachable)
                        )
                        (else
                            (local.get 0)
                            (local.get 1)
                        )
                    )
                    (i32.add)
                )
            )",
        ));
        TranslationTest::new(wasm).expect_func(instrs).run()
    }
    test_for(true, [Instruction::Trap(TrapCode::UnreachableCodeReached)]);
    test_for(
        false,
        [
            Instruction::i32_add(
                Register::from_u16(2),
                Register::from_u16(0),
                Register::from_u16(1),
            ),
            Instruction::return_reg(Register::from_u16(2)),
        ],
    );
}

#[test]
fn const_condition_trap_else() {
    fn test_for<I>(condition: bool, instrs: I)
    where
        I: IntoIterator<Item = Instruction>,
    {
        let condition = i32::from(condition);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32 i32) (result i32)
                    (i32.const {condition})
                    (if (result i32 i32)
                        (then
                            (local.get 0)
                            (local.get 1)
                        )
                        (else
                            (unreachable)
                        )
                    )
                    (i32.add)
                )
            )",
        ));
        TranslationTest::new(wasm).expect_func(instrs).run()
    }
    test_for(
        true,
        [
            Instruction::i32_add(
                Register::from_u16(2),
                Register::from_u16(0),
                Register::from_u16(1),
            ),
            Instruction::return_reg(Register::from_u16(2)),
        ],
    );
    test_for(false, [Instruction::Trap(TrapCode::UnreachableCodeReached)]);
}
