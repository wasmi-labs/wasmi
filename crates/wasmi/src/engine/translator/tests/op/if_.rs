use super::*;
use crate::{
    core::UntypedVal,
    engine::EngineFunc,
    ir::{index::Global, BranchOffset, BranchOffset16, RegSpan},
    TrapCode,
};

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_then() {
    let wasm = r"
        (module
            (func (param i32)
                (if (local.get 0)
                    (then)
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_then_nested() {
    let wasm = r"
        (module
            (func (param i32 i32)
                (if (local.get 0)
                    (then
                        (if (local.get 1)
                            (then)
                        )
                    )
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(2)),
            Instruction::branch_i32_eq_imm16(Reg::from(1), 0, BranchOffset16::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_then_global_set() {
    let wasm = r"
        (module
            (global $g (mut i32) (i32.const 0))
            (func (param i32 i32) (result i32)
                (if (local.get 0)
                    (then
                        (global.set $g (local.get 1))
                    )
                )
                (i32.const 10)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(2_i16)),
            Instruction::global_set(Reg::from(1), Global::from(0)),
            Instruction::return_imm32(AnyConst32::from(10_i32)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_then_return() {
    let wasm = r"
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
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(3)),
            Instruction::i32_add(Reg::from(3), Reg::from(1), Reg::from(2)),
            Instruction::return_reg(Reg::from(3)),
            Instruction::return_imm32(AnyConst32::from(0_i32)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_then_else_return() {
    let wasm = r"
        (module
            (func (param i32) (result i32)
                (if (local.get 0)
                    (then
                        (return (i32.const 10))
                    )
                    (else
                        (return (i32.const 20))
                    )
                )
                (i32.const 30)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(2)),
            Instruction::return_imm32(AnyConst32::from(10_i32)),
            Instruction::return_imm32(AnyConst32::from(20_i32)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_then_br_else() {
    let wasm = r"
        (module
            (func (param i32) (result i32)
                (if (local.get 0)
                    (then
                        (br 0)
                    )
                    (else
                        (return (i32.const 10))
                    )
                )
                (i32.const 20)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(2)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::return_imm32(AnyConst32::from(10_i32)),
            Instruction::return_imm32(AnyConst32::from(20_i32)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_then_else_br() {
    let wasm = r"
        (module
            (func (param i32) (result i32)
                (if (local.get 0)
                    (then
                        (return (i32.const 10))
                    )
                    (else
                        (br 0)
                    )
                )
                (i32.const 20)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(2)),
            Instruction::return_imm32(AnyConst32::from(10_i32)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::return_imm32(AnyConst32::from(20_i32)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_then_else() {
    let wasm = r"
        (module
            (func (param i32)
                (if (local.get 0)
                    (then)
                    (else
                        (nop) ;; without the `nop` there would be no `else` branch
                    )
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(2)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_then_else_nested() {
    let wasm = r"
        (module
            (func (param i32 i32)
                (if (local.get 0)
                    (then
                        (if (local.get 1)
                            (then)
                            (else (nop)) ;; `nop` required so that `else` is not dropped
                        )
                    )
                    (else
                        (if (local.get 1)
                            (then)
                            (else (nop)) ;; `nop` required so that `else` is not dropped
                        )
                    )
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(4)),
            Instruction::branch_i32_eq_imm16(Reg::from(1), 0, BranchOffset16::from(2)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::branch_i32_eq_imm16(Reg::from(1), 0, BranchOffset16::from(2)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_then_else_with_params() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32)
                (local.get 1)
                (local.get 2)
                (if (param i32 i32) (result i32) (local.get 0)
                    (then (i32.add))
                    (else (i32.mul))
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(RegSpan::new(Reg::from(4)), 1, 2),
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(3)),
            Instruction::i32_add(Reg::from(3), Reg::from(4), Reg::from(5)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::i32_mul(Reg::from(3), Reg::from(4), Reg::from(5)),
            Instruction::return_reg(Reg::from(3)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn const_condition() {
    fn test_for(condition: bool) {
        let true_value = 10_i32;
        let false_value = 20_i32;
        let expected = match condition {
            true => true_value,
            false => false_value,
        };
        let condition = i32::from(condition);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([Instruction::return_imm32(AnyConst32::from(expected))])
            .run()
    }
    test_for(true);
    test_for(false);
}

#[test]
#[cfg_attr(miri, ignore)]
fn const_condition_trap_then() {
    fn test_for<I>(condition: bool, instrs: I)
    where
        I: IntoIterator<Item = Instruction>,
    {
        let condition = i32::from(condition);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm).expect_func_instrs(instrs).run()
    }
    test_for(true, [Instruction::trap(TrapCode::UnreachableCodeReached)]);
    test_for(
        false,
        [
            Instruction::i32_add(Reg::from(2), Reg::from(0), Reg::from(1)),
            Instruction::return_reg(Reg::from(2)),
        ],
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn const_condition_trap_else() {
    fn test_for<I>(condition: bool, instrs: I)
    where
        I: IntoIterator<Item = Instruction>,
    {
        let condition = i32::from(condition);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm).expect_func_instrs(instrs).run()
    }
    test_for(
        true,
        [
            Instruction::i32_add(Reg::from(2), Reg::from(0), Reg::from(1)),
            Instruction::return_reg(Reg::from(2)),
        ],
    );
    test_for(false, [Instruction::trap(TrapCode::UnreachableCodeReached)]);
}

#[test]
#[cfg_attr(miri, ignore)]
fn const_condition_br_if_then() {
    fn test_for<I>(condition: bool, instrs: I)
    where
        I: IntoIterator<Item = Instruction>,
    {
        let condition = i32::from(condition);
        let wasm = format!(
            r"
            (module
                (func (param i32) (result i32)
                    (i32.const {condition})
                    (if
                        (then
                            (unreachable)
                        )
                        (else
                            (local.get 0) ;; br_if condition
                            (br_if 0)
                            (unreachable)
                        )
                    )
                    (i32.const 1)
                )
            )",
        );
        TranslationTest::new(&wasm).expect_func_instrs(instrs).run()
    }
    test_for(true, [Instruction::trap(TrapCode::UnreachableCodeReached)]);
    test_for(
        false,
        [
            Instruction::branch_i32_ne_imm16(Reg::from(0), 0, BranchOffset16::from(2)),
            Instruction::trap(TrapCode::UnreachableCodeReached),
            Instruction::return_imm32(AnyConst32::from(1_i32)),
        ],
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn const_condition_br_if_else() {
    fn test_for<I>(condition: bool, instrs: I)
    where
        I: IntoIterator<Item = Instruction>,
    {
        let condition = i32::from(condition);
        let wasm = format!(
            r"
            (module
                (func (param i32) (result i32)
                    (i32.const {condition})
                    (if
                        (then
                            (local.get 0) ;; br_if condition
                            (br_if 0)
                            (unreachable)
                        )
                        (else
                            (unreachable)
                        )
                    )
                    (i32.const 1)
                )
            )",
        );
        TranslationTest::new(&wasm).expect_func_instrs(instrs).run()
    }
    test_for(
        true,
        [
            Instruction::branch_i32_ne_imm16(Reg::from(0), 0, BranchOffset16::from(2)),
            Instruction::trap(TrapCode::UnreachableCodeReached),
            Instruction::return_imm32(AnyConst32::from(1_i32)),
        ],
    );
    test_for(false, [Instruction::trap(TrapCode::UnreachableCodeReached)]);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_if_false_without_else_block_0() {
    let wasm = r#"
        (module
            (func
                (if
                    (i32.const 0) ;; false
                    (then
                        (return)
                    )
                )
            )
        )
        "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::Return])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_if_false_without_else_block_1() {
    let wasm = r#"
        (module
            (func (result i32)
                (if
                    (i32.const 0) ;; false
                    (then
                        (return (i32.const 0))
                    )
                )
                (i32.const 1)
            )
        )
        "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_imm32(1)])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_if_without_else_has_result() {
    let wasm = r#"
        (module
            (func $f (result i64 i32)
                (i64.const 1)
                (i32.const 0)
            )
            (func (result i64)
                (call $f)
                (if (param i64) (result i64)
                    (then
                        (drop)
                        (i64.const -1)
                    )
                )
            )
        )
        "#;
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([Instruction::return_reg2_ext(-1, -2)])
                .consts([UntypedVal::from(1_i64), UntypedVal::from(0_i32)]),
        )
        .expect_func_instrs([
            Instruction::call_internal_0(RegSpan::new(Reg::from(0)), EngineFunc::from_u32(0)),
            Instruction::branch_i32_eq_imm16(Reg::from(1), 0, BranchOffset16::from(3)),
            Instruction::copy_i64imm32(Reg::from(0), -1),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run()
}
