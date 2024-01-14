use super::*;
use crate::{
    core::TrapCode,
    engine::{
        bytecode::{BranchOffset, BranchOffset16, RegisterSpan},
        CompiledFunc,
    },
};
use wasmi_core::F32;

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_0() {
    let wat = include_str!("fuzz_0.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::copy_imm32(Register::from_i16(0), 13.0_f32),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_1() {
    let wat = include_str!("fuzz_1.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::copy_f64imm32(Register::from_i16(0), 13.0_f32),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_2() {
    let wat = include_str!("fuzz_2.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm(Register::from_i16(0), 0, BranchOffset16::from(2)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_3() {
    let wat = include_str!("fuzz_3.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_internal_0(
                RegisterSpan::new(Register::from_i16(0)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::call_internal_0(
                RegisterSpan::new(Register::from_i16(3)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::copy_span_non_overlapping(
                RegisterSpan::new(Register::from_i16(0)),
                RegisterSpan::new(Register::from_i16(2)),
                3,
            ),
            Instruction::branch_table(Register::from_i16(5), 2),
            Instruction::return_span(RegisterSpan::new(Register::from_i16(0)).iter_u16(3)),
            Instruction::return_span(RegisterSpan::new(Register::from_i16(0)).iter_u16(3)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_4() {
    let wat = include_str!("fuzz_4.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 1),
            Instruction::copy(1, 0),
            Instruction::branch_i32_eq_imm(Register::from_i16(1), 0, BranchOffset16::from(2)),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_5() {
    let wat = include_str!("fuzz_5.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_internal(
                RegisterSpan::new(Register::from_i16(1)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::register(Register::from_i16(0)),
            Instruction::branch_i32_eq_imm(Register::from_i16(3), 0, BranchOffset16::from(5)),
            Instruction::call_internal(
                RegisterSpan::new(Register::from_i16(2)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::register(Register::from_i16(2)),
            Instruction::branch_i32_eq_imm(Register::from_i16(4), 0, BranchOffset16::from(1)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy_imm32(Register::from_i16(3), 0),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_6() {
    let wat = include_str!("fuzz_6.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm(Register::from_i16(0), 0, BranchOffset16::from(4)),
            Instruction::branch_i32_eq_imm(Register::from_i16(0), 0, BranchOffset16::from(1)),
            Instruction::copy(1, 0),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy(1, 0),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_7() {
    let wat = include_str!("fuzz_7.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::copy_imm32(Register::from_i16(0), 1),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_8() {
    let wat = include_str!("fuzz_8.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(4, 1),
            Instruction::copy_imm32(Register::from_i16(1), 10),
            Instruction::copy(3, 0),
            Instruction::copy_imm32(Register::from_i16(0), 20),
            Instruction::copy(2, 4),
            Instruction::return_reg2(3, 2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_9() {
    let wat = include_str!("fuzz_9.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(6, 1),
            Instruction::copy_imm32(Register::from_i16(1), 10),
            Instruction::copy(5, 0),
            Instruction::copy_imm32(Register::from_i16(0), 20),
            Instruction::branch_i32_eq_imm(Register::from_i16(0), 0, BranchOffset16::from(5)),
            Instruction::i32_add(
                Register::from_i16(3),
                Register::from_i16(5),
                Register::from_i16(6),
            ),
            Instruction::copy(4, 2),
            Instruction::copy_imm32(Register::from_i16(2), 30),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::i32_mul(
                Register::from_i16(3),
                Register::from_i16(5),
                Register::from_i16(6),
            ),
            Instruction::return_reg(3),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_10() {
    let wat = include_str!("fuzz_10.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm(Register::from_i16(0), 0, BranchOffset16::from(3)),
            Instruction::copy_imm32(Register::from_i16(1), 10),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy_imm32(Register::from_i16(1), 20),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_11() {
    let wat = include_str!("fuzz_11.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_and_imm16(Register::from_i16(1), Register::from_i16(0), 2),
            Instruction::i32_eq_imm16(Register::from_i16(0), Register::from_i16(0), 0),
            Instruction::i32_and(
                Register::from_i16(1),
                Register::from_i16(1),
                Register::from_i16(0),
            ),
            Instruction::return_nez(1),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_12_f32() {
    let wat = include_str!("fuzz_12_f32.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func(ExpectedFunc::new([
            Instruction::copy_imm32(Register::from_i16(0), u32::MAX),
            Instruction::f32_le(
                Register::from_i16(1),
                Register::from_i16(0),
                Register::from_i16(0),
            ),
            Instruction::return_nez(1),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
        ]))
        .expect_func(ExpectedFunc::new([
            Instruction::copy_imm32(Register::from_i16(0), u32::MAX),
            Instruction::f32_ge(
                Register::from_i16(1),
                Register::from_i16(0),
                Register::from_i16(0),
            ),
            Instruction::return_nez(1),
            Instruction::Trap(TrapCode::UnreachableCodeReached),
        ]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_12_f64() {
    let wat = include_str!("fuzz_12_f64.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::copy(0, -1),
                Instruction::f64_le(
                    Register::from_i16(1),
                    Register::from_i16(0),
                    Register::from_i16(0),
                ),
                Instruction::return_nez(1),
                Instruction::Trap(TrapCode::UnreachableCodeReached),
            ])
            .consts([u64::MAX]),
        )
        .expect_func(
            ExpectedFunc::new([
                Instruction::copy(0, -1),
                Instruction::f64_ge(
                    Register::from_i16(1),
                    Register::from_i16(0),
                    Register::from_i16(0),
                ),
                Instruction::return_nez(1),
                Instruction::Trap(TrapCode::UnreachableCodeReached),
            ])
            .consts([u64::MAX]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_13_codegen() {
    let wat = include_str!("fuzz_13.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_nez_many(0, 0, 0),
            Instruction::Register(Register::from_i16(0)),
            Instruction::return_reg3(0, 0, 0),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_13_execute() {
    use crate::{Engine, Linker, Store};
    let wat = include_str!("fuzz_13.wat");
    let wasm = wat2wasm(wat);
    let engine = Engine::default();
    let mut store = <Store<()>>::new(&engine, ());
    let linker = Linker::new(&engine);
    let module = Module::new(&engine, &wasm[..]).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let func = instance
        .get_func(&store, "")
        .unwrap()
        .typed::<(), (i32, i32, i32)>(&store)
        .unwrap();
    let (x, y, z) = func.call(&mut store, ()).unwrap();
    assert!(x == 0 && y == 0 && z == 0);
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_14() {
    let wat = include_str!("fuzz_14.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::i32_and(
                    Register::from_i16(2),
                    Register::from_i16(0),
                    Register::from_i16(1),
                ),
                Instruction::return_reg2(2, -1),
            ])
            .consts([0_i32]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_15_01() {
    let wat = include_str!("fuzz_15_01.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func(
            // Note:
            //
            // - The bug is that `copy_imm32` overwrites `i32_wrap_i64` which is the `index` of the `br_table`.
            // - Furthermore `br_table` somehow uses `reg(0)` for `index` instead of `reg(1)` where `i32_wrap_i64`
            //   stores its `index` result.
            ExpectedFunc::new([
                Instruction::i32_wrap_i64(Register::from_i16(1), Register::from_i16(0)),
                Instruction::branch_table(Register::from_i16(0), 3),
                Instruction::copy_imm32(Register::from_i16(1), 10.0_f32),
                Instruction::branch(BranchOffset::from(3)),
                Instruction::return_reg(1),
                Instruction::branch(BranchOffset::from(1)),
                Instruction::Trap(TrapCode::UnreachableCodeReached),
            ]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_15_01_execute() {
    // Note: we can remove this test case once the bug is fixed
    //       since this is a codegen bug and not an executor bug.
    use crate::{Engine, Linker, Store};
    let wat = include_str!("fuzz_15_01.wat");
    let wasm = wat2wasm(wat);
    let engine = Engine::default();
    let mut store = <Store<()>>::new(&engine, ());
    let linker = Linker::new(&engine);
    let module = Module::new(&engine, &wasm[..]).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let func = instance
        .get_func(&store, "")
        .unwrap()
        .typed::<i64, F32>(&store)
        .unwrap();
    let result = func.call(&mut store, 1).unwrap();
    assert_eq!(result, 10.0);
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_regression_15_02() {
    let wat = include_str!("fuzz_15_02.wat");
    let wasm = wat2wasm(wat);
    TranslationTest::new(wasm)
        .expect_func(
            // Note: The bug is that `copy2` overwrites `i32_wrap_i64` which is the `index` of the `br_table`.
            ExpectedFunc::new([
                Instruction::i32_wrap_i64(Register::from_i16(1), Register::from_i16(0)),
                Instruction::copy2(
                    RegisterSpan::new(Register::from_i16(1)),
                    Register::from_i16(-1),
                    Register::from_i16(-2),
                ),
                Instruction::branch_table(Register::from_i16(1), 3),
                Instruction::branch(BranchOffset::from(3)),
                Instruction::return_span(RegisterSpan::new(Register::from_i16(1)).iter_u16(2)),
                Instruction::branch(BranchOffset::from(1)),
                Instruction::Trap(TrapCode::UnreachableCodeReached),
            ])
            .consts([10.0_f32, 20.0_f32]),
        )
        .run()
}
