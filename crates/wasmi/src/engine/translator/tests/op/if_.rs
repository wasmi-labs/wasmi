use super::*;
use crate::{
    core::{TrapCode, UntypedVal},
    engine::{
        bytecode::{BranchOffset, BranchOffset16, GlobalIdx, RegSpan},
        EngineFunc,
    },
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(1)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(2)),
            Instruction::branch_i32_eqz(Reg::from_i16(1), BranchOffset16::from(1)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(2)),
            Instruction::global_set(GlobalIdx::from(0), Reg::from_i16(1)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(3)),
            Instruction::i32_add(Reg::from_i16(3), Reg::from_i16(1), Reg::from_i16(2)),
            Instruction::return_reg(Reg::from_i16(3)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(2)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(2)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(2)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(2)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(4)),
            Instruction::branch_i32_eqz(Reg::from_i16(1), BranchOffset16::from(2)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::branch_i32_eqz(Reg::from_i16(1), BranchOffset16::from(2)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegSpan::new(Reg::from_i16(4)), 1, 2),
            Instruction::branch_i32_eqz(Reg::from_i16(0), BranchOffset16::from(3)),
            Instruction::i32_add(Reg::from_i16(3), Reg::from_i16(4), Reg::from_i16(5)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::i32_mul(Reg::from_i16(3), Reg::from_i16(4), Reg::from_i16(5)),
            Instruction::return_reg(Reg::from_i16(3)),
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
        TranslationTest::from_wat(&wasm)
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
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs(instrs)
            .run()
    }
    test_for(true, [Instruction::Trap(TrapCode::UnreachableCodeReached)]);
    test_for(
        false,
        [
            Instruction::i32_add(Reg::from_i16(2), Reg::from_i16(0), Reg::from_i16(1)),
            Instruction::return_reg(Reg::from_i16(2)),
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
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs(instrs)
            .run()
    }
    test_for(
        true,
        [
            Instruction::i32_add(Reg::from_i16(2), Reg::from_i16(0), Reg::from_i16(1)),
            Instruction::return_reg(Reg::from_i16(2)),
        ],
    );
    test_for(false, [Instruction::Trap(TrapCode::UnreachableCodeReached)]);
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
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs(instrs)
            .run()
    }
    test_for(true, [Instruction::Trap(TrapCode::UnreachableCodeReached)]);
    test_for(
        false,
        [
            Instruction::branch_i32_nez(Reg::from_i16(0), BranchOffset16::from(2)),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
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
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs(instrs)
            .run()
    }
    test_for(
        true,
        [
            Instruction::branch_i32_nez(Reg::from_i16(0), BranchOffset16::from(2)),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
            Instruction::return_imm32(AnyConst32::from(1_i32)),
        ],
    );
    test_for(false, [Instruction::Trap(TrapCode::UnreachableCodeReached)]);
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
    TranslationTest::from_wat(wasm)
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
    TranslationTest::from_wat(wasm)
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
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([Instruction::return_reg2(-1, -2)])
                .consts([UntypedVal::from(1_i64), UntypedVal::from(0_i32)]),
        )
        .expect_func_instrs([
            Instruction::call_internal_0(RegSpan::new(Reg::from_i16(0)), EngineFunc::from_u32(0)),
            Instruction::branch_i32_eqz(Reg::from_i16(1), BranchOffset16::from(3)),
            Instruction::copy_i64imm32(Reg::from_i16(0), -1),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::return_reg(Reg::from_i16(0)),
        ])
        .run()
}
