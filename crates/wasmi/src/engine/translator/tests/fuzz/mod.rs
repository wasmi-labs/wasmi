//! This module contains translation test cases found via Wasmi fuzzing.

use super::*;
use crate::{
    core::TrapCode,
    engine::EngineFunc,
    ir::{index::Global, Address, Address32, BranchOffset, BranchOffset16, RegSpan},
    tests::{AssertResults, AssertTrap, ExecutionTest},
};

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_0() {
    let wasm = include_str!("wat/fuzz_0.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::copy_imm32(Reg::from(0), 13.0_f32),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_1() {
    let wasm = include_str!("wat/fuzz_1.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::copy_f64imm32(Reg::from(0), 13.0_f32),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_2() {
    let wasm = include_str!("wat/fuzz_2.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(2)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_3() {
    let wasm = include_str!("wat/fuzz_3.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_internal_0(RegSpan::new(Reg::from(0)), EngineFunc::from_u32(0)),
            Instruction::call_internal_0(RegSpan::new(Reg::from(3)), EngineFunc::from_u32(0)),
            Instruction::return_reg3_ext(2, 3, 4),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_4() {
    let wasm = include_str!("wat/fuzz_4.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 1),
            Instruction::copy(1, 0),
            Instruction::branch_i32_eq_imm16(Reg::from(1), 0, BranchOffset16::from(2)),
            Instruction::trap(TrapCode::UnreachableCodeReached),
            Instruction::trap(TrapCode::UnreachableCodeReached),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_5() {
    let wasm = include_str!("wat/fuzz_5.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_internal(RegSpan::new(Reg::from(1)), EngineFunc::from_u32(0)),
            Instruction::register(Reg::from(0)),
            Instruction::branch_i32_eq_imm16(Reg::from(3), 0, BranchOffset16::from(5)),
            Instruction::call_internal(RegSpan::new(Reg::from(2)), EngineFunc::from_u32(0)),
            Instruction::register(Reg::from(2)),
            Instruction::branch_i32_eq_imm16(Reg::from(4), 0, BranchOffset16::from(1)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy_imm32(Reg::from(3), 0),
            Instruction::trap(TrapCode::UnreachableCodeReached),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_6() {
    let wasm = include_str!("wat/fuzz_6.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 0),
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(4)),
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(1)),
            Instruction::copy(1, 2),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy(1, 2),
            Instruction::trap(TrapCode::UnreachableCodeReached),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_7() {
    let wasm = include_str!("wat/fuzz_7.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::copy_imm32(Reg::from(0), 1),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_8() {
    let wasm = include_str!("wat/fuzz_8.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(4, 1),
            Instruction::copy_imm32(Reg::from(1), 10),
            Instruction::copy(3, 0),
            Instruction::copy_imm32(Reg::from(0), 20),
            Instruction::copy(2, 4),
            Instruction::return_reg2_ext(3, 2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_9() {
    let wasm = include_str!("wat/fuzz_9.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(6, 1),
            Instruction::copy_imm32(Reg::from(1), 10),
            Instruction::copy(5, 0),
            Instruction::copy_imm32(Reg::from(0), 20),
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(5)),
            Instruction::i32_add(Reg::from(3), Reg::from(5), Reg::from(6)),
            Instruction::copy(4, 2),
            Instruction::copy_imm32(Reg::from(2), 30),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::i32_mul(Reg::from(3), Reg::from(5), Reg::from(6)),
            Instruction::return_reg(3),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_10() {
    let wasm = include_str!("wat/fuzz_10.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(0), 0, BranchOffset16::from(3)),
            Instruction::copy_imm32(Reg::from(1), 10),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy_imm32(Reg::from(1), 20),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_11() {
    let wasm = include_str!("wat/fuzz_11.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_bitand_imm16(Reg::from(1), Reg::from(0), 2),
            Instruction::i32_eq_imm16(Reg::from(0), Reg::from(0), 0),
            Instruction::branch_i32_and(Reg::from(1), Reg::from(0), 2),
            Instruction::trap(TrapCode::UnreachableCodeReached),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_12_f32() {
    let wasm = include_str!("wat/fuzz_12_f32.wat");
    TranslationTest::new(wasm)
        .expect_func(ExpectedFunc::new([
            Instruction::copy_imm32(Reg::from(0), u32::MAX),
            Instruction::branch_f32_le(Reg::from(0), Reg::from(0), 2),
            Instruction::trap(TrapCode::UnreachableCodeReached),
            Instruction::Return,
        ]))
        .expect_func(ExpectedFunc::new([
            Instruction::copy_imm32(Reg::from(0), u32::MAX),
            Instruction::branch_f32_le(Reg::from(0), Reg::from(0), 2),
            Instruction::trap(TrapCode::UnreachableCodeReached),
            Instruction::Return,
        ]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_12_f64() {
    let wasm = include_str!("wat/fuzz_12_f64.wat");
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::copy(0, -1),
                Instruction::branch_f64_le(Reg::from(0), Reg::from(0), 2),
                Instruction::trap(TrapCode::UnreachableCodeReached),
                Instruction::Return,
            ])
            .consts([u64::MAX]),
        )
        .expect_func(
            ExpectedFunc::new([
                Instruction::copy(0, -1),
                Instruction::branch_f64_le(Reg::from(0), Reg::from(0), 2),
                Instruction::trap(TrapCode::UnreachableCodeReached),
                Instruction::Return,
            ])
            .consts([u64::MAX]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_13_codegen() {
    let wasm = include_str!("wat/fuzz_13.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(0, 0_i16, 3),
            Instruction::copy2_ext(RegSpan::new(Reg::from(1)), 0, 0),
            Instruction::branch(2),
            Instruction::copy2_ext(RegSpan::new(Reg::from(1)), 0, 0),
            Instruction::return_reg3_ext(0, 1, 2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_13_execute() {
    let wasm = include_str!("wat/fuzz_13.wat");
    ExecutionTest::default()
        .wasm(wasm)
        .call::<(), (i32, i32, i32)>("", ())
        .assert_results((0, 0, 0));
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_14() {
    let wasm = include_str!("wat/fuzz_14.wat");
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::i32_bitand(Reg::from(2), Reg::from(0), Reg::from(1)),
                Instruction::return_reg2_ext(2, -1),
            ])
            .consts([0_i32]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_15_01_codegen() {
    let wasm = include_str!("wat/fuzz_15_01.wat");
    TranslationTest::new(wasm)
        .expect_func(
            // Note:
            //
            // - The bug is that `copy_imm32` overwrites `i32_wrap_i64` which is the `index` of the `br_table`.
            // - Furthermore `br_table` somehow uses `reg(0)` for `index` instead of `reg(1)` where `i32_wrap_i64`
            //   stores its `index` result.
            ExpectedFunc::new([
                Instruction::i32_wrap_i64(Reg::from(1), Reg::from(0)),
                Instruction::branch_table_1(Reg::from(1), 3_u32),
                Instruction::const32(10.0_f32),
                Instruction::branch_table_target(RegSpan::new(Reg::from(1)), BranchOffset::from(3)),
                Instruction::return_imm32(10.0_f32),
                Instruction::branch_table_target(RegSpan::new(Reg::from(1)), BranchOffset::from(1)),
                Instruction::trap(TrapCode::UnreachableCodeReached),
            ]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_15_01_execute() {
    let wasm = include_str!("wat/fuzz_15_01.wat");
    ExecutionTest::default()
        .wasm(wasm)
        .call::<i64, f32>("", 1)
        .assert_results(10.0);
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_15_02() {
    let wasm = include_str!("wat/fuzz_15_02.wat");
    TranslationTest::new(wasm)
        .expect_func(
            // Note: The bug is that `copy2` overwrites `i32_wrap_i64` which is the `index` of the `br_table`.
            ExpectedFunc::new([
                Instruction::i32_wrap_i64(Reg::from(1), Reg::from(0)),
                Instruction::branch_table_2(Reg::from(1), 3_u32),
                Instruction::register2_ext(Reg::from(-1), Reg::from(-2)),
                Instruction::branch_table_target(RegSpan::new(Reg::from(1)), BranchOffset::from(3)),
                Instruction::return_reg2_ext(Reg::from(-1), Reg::from(-2)),
                Instruction::branch_table_target(RegSpan::new(Reg::from(1)), BranchOffset::from(1)),
                Instruction::trap(TrapCode::UnreachableCodeReached),
            ])
            .consts([10.0_f32, 20.0_f32]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_15_03() {
    let wasm = include_str!("wat/fuzz_15_03.wat");
    TranslationTest::new(wasm)
        .expect_func(
            // Note: The bug is that `copy2` overwrites `i32_wrap_i64` which is the `index` of the `br_table`.
            ExpectedFunc::new([
                Instruction::global_get(Reg::from(1), Global::from(0)),
                Instruction::global_get(Reg::from(2), Global::from(0)),
                Instruction::i32_wrap_i64(Reg::from(3), Reg::from(0)),
                Instruction::branch_table_2(Reg::from(3), 4_u32),
                Instruction::register2_ext(Reg::from(-1), Reg::from(-2)),
                Instruction::branch_table_target(RegSpan::new(Reg::from(3)), BranchOffset::from(4)),
                Instruction::branch_table_target(RegSpan::new(Reg::from(2)), BranchOffset::from(5)),
                Instruction::branch_table_target(RegSpan::new(Reg::from(3)), BranchOffset::from(2)),
                Instruction::branch_table_target(RegSpan::new(Reg::from(1)), BranchOffset::from(5)),
                Instruction::i32_add(Reg::from(3), Reg::from(3), Reg::from(4)),
                Instruction::return_reg(Reg::from(3)),
                Instruction::i32_mul(Reg::from(2), Reg::from(2), Reg::from(3)),
                Instruction::return_reg(Reg::from(2)),
                Instruction::i32_bitxor(Reg::from(1), Reg::from(1), Reg::from(2)),
                Instruction::return_reg(Reg::from(1)),
            ])
            .consts([10_i32, 20_i32]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_16() {
    // The bug in this regression test was a forgotten adjustment
    // for the preserved local value causing the `value` register
    // of the `i64_store_at` instruction to be 32676 instead of 2.
    let wasm = include_str!("wat/fuzz_16.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 0),
            Instruction::global_get(Reg::from(0), Global::from(0)),
            Instruction::global_set(Reg::from(0), Global::from(0)),
            Instruction::trap(TrapCode::MemoryOutOfBounds),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_17() {
    // The bug in this regression test was a forgotten adjustment
    // for the preserved local value causing the `value` register
    // of the `i64_store_at` instruction to be 32676 instead of 2.
    let wasm = include_str!("wat/fuzz_17.wat");
    let addr = Address::try_from(4294967295_u64).unwrap();
    let addr32 = Address32::try_from(addr).unwrap();
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 0),
            Instruction::copy_i64imm32(Reg::from(0), 2),
            Instruction::copy_imm32(Reg::from(1), -1.0_f32),
            Instruction::store64_at(Reg::from(2), addr32),
            Instruction::trap(TrapCode::UnreachableCodeReached),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn audit_0_codegen() {
    let wasm = include_str!("wat/audit_0.wat");
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_many_ext(-1, -2, -1),
                Instruction::register(-2),
            ])
            .consts([1, 0]),
        )
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal_0(RegSpan::new(Reg::from(0)), EngineFunc::from_u32(0)),
                Instruction::branch_table_many(Reg::from(3), 3_u32),
                Instruction::register_list_ext(-1, 0, 1),
                Instruction::register(2),
                Instruction::branch_table_target(RegSpan::new(Reg::from(0)), BranchOffset::from(3)),
                Instruction::Return,
                Instruction::Return,
                Instruction::return_span(bspan(0, 4)),
            ])
            .consts([0]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn audit_0_execution() {
    let wasm = include_str!("wat/audit_0.wat");
    ExecutionTest::default()
        .wasm(wasm)
        .call::<(), (i32, i32, i32, i32)>("", ())
        .assert_results((0, 1, 0, 1));
}

#[test]
#[cfg_attr(miri, ignore)]
fn audit_1_codegen() {
    let wasm = include_str!("wat/audit_1.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span_non_overlapping(
                RegSpan::new(Reg::from(6)),
                RegSpan::new(Reg::from(0)),
                3_u16,
            ),
            Instruction::trap(TrapCode::IntegerOverflow),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn audit_1_execution() {
    let wasm = include_str!("wat/audit_1.wat");
    ExecutionTest::default()
        .wasm(wasm)
        .call::<(), (i32, i32, i32)>("", ())
        .assert_trap(TrapCode::IntegerOverflow);
}

#[test]
#[cfg_attr(miri, ignore)]
fn audit_2_codegen() {
    let wasm = include_str!("wat/audit_2.wat");
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 0),
            Instruction::copy(0, 2),
            Instruction::copy(1, 0),
            Instruction::return_many_ext(2, 1, 0),
            Instruction::register(0),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn audit_2_execution() {
    let wasm = include_str!("wat/audit_2.wat");
    ExecutionTest::default()
        .wasm(wasm)
        .call::<i32, (i32, i32, i32, i32)>("", 1)
        .assert_results((1, 1, 1, 1));
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_br_if() {
    let wasm = r#"
        (module
            (func (export "") (param f32 f32) (result f32)
                loop ;; label = @1
                    f32.const 0
                    local.get 1
                    i32.reinterpret_f32
                    br_if 1
                    local.set 0
                    local.get 1
                    i32.reinterpret_f32
                    br_if 0 (;@1;)
                    unreachable
                end
                unreachable
            )
        )
    "#;
    ExecutionTest::default()
        .wasm(wasm)
        .call::<(f32, f32), f32>("", (0.0, 0.0))
        .assert_trap(TrapCode::UnreachableCodeReached);
}
