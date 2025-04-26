use super::*;
use crate::{
    core::UntypedVal,
    engine::translator::tests::wasm_type::WasmTy,
    ir::{BranchOffset, BranchOffset16, RegSpan},
};
use core::fmt::Display;

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_return() {
    fn test_for(condition: bool) {
        let condition = DisplayWasm::from(i32::from(condition));
        let wasm = format!(
            r"
            (module
                (func (param i32)
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([Instruction::Return])
            .run()
    }
    test_for(true);
    test_for(false);
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_return_1() {
    fn test_for(condition: bool) {
        let expected = match condition {
            true => Reg::from(0),
            false => Reg::from(1),
        };
        let condition = DisplayWasm::from(i32::from(condition));
        let wasm = format!(
            r"
            (module
                (func (param i32 i32) (result i32)
                    (local.get 0)
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([Instruction::return_reg(expected)])
            .run()
    }
    test_for(true);
    test_for(false);
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_return_1_imm() {
    fn test_for<T>(condition: bool, if_true: T, if_false: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        let expected: UntypedVal = match condition {
            true => if_true.into(),
            false => if_false.into(),
        };
        let condition = DisplayWasm::from(i32::from(condition));
        let display_ty = DisplayValueType::from(<T as WasmTy>::VALUE_TYPE);
        let display_if_true = DisplayWasm::from(if_true);
        let display_if_false = DisplayWasm::from(if_false);
        let wasm = format!(
            r"
            (module
                (func (result {display_ty})
                    ({display_ty}.const {display_if_true})
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                    (drop)
                    ({display_ty}.const {display_if_false})
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func(
                ExpectedFunc::new([Instruction::return_reg(Reg::from(-1))]).consts([expected]),
            )
            .run()
    }
    /// Run the test for both sign polarities of the `br_if` condition.
    fn test_for_both<T>(if_true: T, if_false: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        test_for::<T>(true, if_true, if_false);
        test_for::<T>(false, if_true, if_false);
    }
    test_for_both::<i64>(i64::MIN, i64::MAX);
    test_for_both::<i64>(i64::from(i32::MIN) - 1, i64::from(i32::MAX) + 1);
    test_for_both::<f64>(0.3, -0.3);
    test_for_both::<f64>(0.123456789, -0.987654321);
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_return_1_imm32() {
    fn test_for<T>(condition: bool, if_true: T, if_false: T)
    where
        T: WasmTy + Into<AnyConst32>,
        DisplayWasm<T>: Display,
    {
        let expected: AnyConst32 = match condition {
            true => if_true.into(),
            false => if_false.into(),
        };
        let condition = DisplayWasm::from(i32::from(condition));
        let display_ty = DisplayValueType::from(<T as WasmTy>::VALUE_TYPE);
        let display_if_true = DisplayWasm::from(if_true);
        let display_if_false = DisplayWasm::from(if_false);
        let wasm = format!(
            r"
            (module
                (func (result {display_ty})
                    ({display_ty}.const {display_if_true})
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                    (drop)
                    ({display_ty}.const {display_if_false})
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([Instruction::return_imm32(expected)])
            .run()
    }
    /// Run the test for both sign polarities of the `br_if` condition.
    fn test_for_both<T>(if_true: T, if_false: T)
    where
        T: WasmTy + Into<AnyConst32>,
        DisplayWasm<T>: Display,
    {
        test_for::<T>(true, if_true, if_false);
        test_for::<T>(false, if_true, if_false);
    }
    test_for_both::<i32>(5, 42);
    test_for_both::<f32>(5.5, -42.25);
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_return_1_i64imm32() {
    fn test_for(condition: bool, if_true: i64, if_false: i64) {
        let expected: i64 = match condition {
            true => if_true,
            false => if_false,
        };
        let condition = DisplayWasm::from(i32::from(condition));
        let display_if_true = DisplayWasm::from(if_true);
        let display_if_false = DisplayWasm::from(if_false);
        let wasm = format!(
            r"
            (module
                (func (result i64)
                    (i64.const {display_if_true})
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                    (drop)
                    (i64.const {display_if_false})
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([return_i64imm32_instr(expected)])
            .run()
    }
    /// Run the test for both sign polarities of the `br_if` condition.
    fn test_for_both(if_true: i64, if_false: i64) {
        test_for(true, if_true, if_false);
        test_for(false, if_true, if_false);
    }
    test_for_both(0, -1);
    test_for_both(5, 42);
    test_for_both(i64::from(i32::MIN), i64::from(i32::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_return_1_f64imm32() {
    fn test_for(condition: bool, if_true: f64, if_false: f64) {
        let expected: f64 = match condition {
            true => if_true,
            false => if_false,
        };
        let condition = DisplayWasm::from(i32::from(condition));
        let display_if_true = DisplayWasm::from(if_true);
        let display_if_false = DisplayWasm::from(if_false);
        let wasm = format!(
            r"
            (module
                (func (result f64)
                    (f64.const {display_if_true})
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                    (drop)
                    (f64.const {display_if_false})
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([return_f64imm32_instr(expected)])
            .run()
    }
    /// Run the test for both sign polarities of the `br_if` condition.
    fn test_for_both(if_true: f64, if_false: f64) {
        test_for(true, if_true, if_false);
        test_for(false, if_true, if_false);
    }
    test_for_both(0.0, -1.0);
    test_for_both(5.5, 42.25);
    test_for_both(f64::NEG_INFINITY, f64::INFINITY);
    test_for_both(f64::NAN, f64::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_branch_always() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (block (param i32) (result i32)
                    (i32.const 1) ;; br_if condition: true
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(3, 0),
            Instruction::copy(2, 3),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_branch_never() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (block (param i32) (result i32)
                    (i32.const 0) ;; br_if condition: false
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(3, 0),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_0() {
    let wasm = r"
        (module
            (func (param i32)
                (local.get 0)
                (br_if 0)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_ne_imm16(0, 0_i16, 1),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_1() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (br_if 0)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_ne_imm16(1, 0_i16, 1),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_1_imm() {
    fn test_for<T>(returned_value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        let display_ty = DisplayValueType::from(<T as WasmTy>::VALUE_TYPE);
        let display_value = DisplayWasm::from(returned_value);
        let wasm = format!(
            r"
            (module
                (func (param i32) (result {display_ty})
                    ({display_ty}.const {display_value})
                    (local.get 0) ;; br_if condition
                    (br_if 0)
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::branch_i32_eq_imm16(0, 0_i16, 3),
                    Instruction::copy(0, -1),
                    Instruction::branch(2),
                    Instruction::copy(0, -1),
                    Instruction::return_reg(0),
                ])
                .consts([returned_value]),
            )
            .run()
    }

    test_for::<i64>(i64::from(i32::MIN) - 1);
    test_for::<i64>(i64::from(i32::MAX) + 1);
    test_for::<i64>(i64::MIN);
    test_for::<i64>(i64::MAX);

    test_for::<f64>(0.3);
    test_for::<f64>(-0.3);
    test_for::<f64>(0.123456789);
    test_for::<f64>(0.987654321);
    test_for::<f64>(-0.123456789);
    test_for::<f64>(-0.987654321);
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_1_imm32() {
    fn test_for<T>(returned_value: T)
    where
        T: WasmTy + Into<AnyConst32>,
        DisplayWasm<T>: Display,
    {
        let display_ty = DisplayValueType::from(<T as WasmTy>::VALUE_TYPE);
        let display_value = DisplayWasm::from(returned_value);
        let wasm = format!(
            r"
            (module
                (func (param i32) (result {display_ty})
                    ({display_ty}.const {display_value})
                    (local.get 0) ;; br_if condition
                    (br_if 0)
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::branch_i32_eq_imm16(0, 0_i16, 3),
                Instruction::copy_imm32(0, returned_value),
                Instruction::branch(2),
                Instruction::copy_imm32(0, returned_value),
                Instruction::return_reg(0),
            ])
            .run()
    }
    test_for::<i32>(0);
    test_for::<i32>(1);
    test_for::<i32>(-1);
    test_for::<i32>(42);
    test_for::<f32>(0.0);
    test_for::<f32>(5.5);
    test_for::<f32>(42.25);
    test_for::<f32>(f32::NAN);
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_1_i64imm32() {
    fn test_for(returned_value: i64) {
        let display_value = DisplayWasm::from(returned_value);
        let wasm = format!(
            r"
            (module
                (func (param i32) (result i64)
                    (i64.const {display_value})
                    (local.get 0) ;; br_if condition
                    (br_if 0)
                )
            )",
        );
        let returned_value32 = i32::try_from(returned_value).unwrap();
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::branch_i32_eq_imm16(0, 0_i16, 3),
                Instruction::copy_i64imm32(0, returned_value32),
                Instruction::branch(2),
                Instruction::copy_i64imm32(0, returned_value32),
                Instruction::return_reg(0),
            ])
            .run()
    }

    test_for(0);
    test_for(1);
    test_for(-1);
    test_for(i64::from(i32::MIN) + 1);
    test_for(i64::from(i32::MIN));
    test_for(i64::from(i32::MAX) - 1);
    test_for(i64::from(i32::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_1_f64imm32() {
    fn test_for(returned_value: f64) {
        let display_value = DisplayWasm::from(returned_value);
        let wasm = format!(
            r"
            (module
                (func (param i32) (result f64)
                    (f64.const {display_value})
                    (local.get 0) ;; br_if condition
                    (br_if 0)
                )
            )",
        );
        let returned_value32 = returned_value as f32;
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::branch_i32_eq_imm16(0, 0_i16, 3),
                Instruction::copy_f64imm32(0, returned_value32),
                Instruction::branch(2),
                Instruction::copy_f64imm32(0, returned_value32),
                Instruction::return_reg(0),
            ])
            .run()
    }

    test_for(0.0);
    test_for(0.25);
    test_for(-0.25);
    test_for(0.5);
    test_for(-0.5);
    test_for(1.0);
    test_for(-1.0);
    test_for(f64::NEG_INFINITY);
    test_for(f64::INFINITY);
    test_for(f64::NAN);
    test_for(f64::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_2() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
                (br_if 0
                    (local.get 2) ;; br_if condition
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_ne_imm16(2, 0_i16, 1),
            Instruction::return_reg2_ext(0, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_2_lhs() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32 i32)
                (local.get 1)
                (local.get 0)
                (br_if 0
                    (local.get 2) ;; br_if condition
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(2, 0_i16, 3),
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), 1, 0),
            Instruction::branch(2),
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), 1, 0),
            Instruction::return_reg2_ext(0, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_2_imm() {
    let wasm = r"
        (module
            (func (param i32) (result i32 i32)
                (i32.const 10)
                (i32.const 20)
                (br_if 0
                    (local.get 0) ;; br_if condition
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::branch_i32_eq_imm16(0, 0_i16, 3),
                Instruction::copy2_ext(RegSpan::new(Reg::from(0)), -1, -2),
                Instruction::branch(2),
                Instruction::copy2_ext(RegSpan::new(Reg::from(0)), -1, -2),
                Instruction::return_reg2_ext(0, 1),
            ])
            .consts([10_i32, 20]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_3_span() {
    let wasm = r"
        (module
            (func (param i32 i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (br_if 0
                    (local.get 3) ;; br_if condition
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_ne_imm16(3, 0_i16, 1),
            Instruction::return_reg3_ext(0, 1, 2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_3() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (br_if 0
                    (local.get 2) ;; br_if condition
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(2, 0_i16, 3),
            Instruction::copy(2, 0),
            Instruction::branch(2),
            Instruction::copy(2, 0),
            Instruction::return_reg3_ext(0, 1, 2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_3_imm() {
    let wasm = r"
        (module
            (func (param i32) (result i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 30)
                (br_if 0
                    (local.get 0) ;; br_if condition
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::branch_i32_eq_imm16(0, 0_i16, 4),
                Instruction::copy_many_non_overlapping_ext(RegSpan::new(Reg::from(0)), -1, -2),
                Instruction::register(-3),
                Instruction::branch(3),
                Instruction::copy_many_non_overlapping_ext(RegSpan::new(Reg::from(0)), -1, -2),
                Instruction::register(-3),
                Instruction::return_reg3_ext(0, 1, 2),
            ])
            .consts([10_i32, 20, 30]),
        )
        .run()
}

// #[test]
// #[cfg_attr(miri, ignore)]
// fn return_if_results_4_span() {
//     let wasm = r"
//         (module
//             (func (param i32 i32 i32 i32 i32) (result i32 i32 i32 i32)
//                 (local.get 0)
//                 (local.get 1)
//                 (local.get 2)
//                 (local.get 3)
//                 (br_if 0
//                     (local.get 4) ;; br_if condition
//                 )
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func_instrs([
//             Instruction::return_nez_span(Reg::from(4), bspan(0, 4)),
//             Instruction::return_span(bspan(0, 4)),
//         ])
//         .run()
// }

// #[test]
// #[cfg_attr(miri, ignore)]
// fn return_if_results_4() {
//     let wasm = r"
//         (module
//             (func (param i32 i32 i32) (result i32 i32 i32 i32)
//                 (local.get 0)
//                 (local.get 1)
//                 (local.get 0)
//                 (local.get 1)
//                 (br_if 0
//                     (local.get 2) ;; br_if condition
//                 )
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func_instrs([
//             Instruction::return_nez_many_ext(Reg::from(2), 0, 1),
//             Instruction::register2_ext(0, 1),
//             Instruction::return_many_ext(0, 1, 0),
//             Instruction::register(1),
//         ])
//         .run()
// }

// #[test]
// #[cfg_attr(miri, ignore)]
// fn return_if_results_4_imm() {
//     let wasm = r"
//         (module
//             (func (param i32) (result i32 i32 i32 i32)
//                 (i32.const 10)
//                 (i32.const 20)
//                 (i32.const 10)
//                 (i32.const 20)
//                 (br_if 0
//                     (local.get 0) ;; br_if condition
//                 )
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func(
//             ExpectedFunc::new([
//                 Instruction::return_nez_many_ext(Reg::from(0), -1, -2),
//                 Instruction::register2_ext(-1, -2),
//                 Instruction::return_many_ext(-1, -2, -1),
//                 Instruction::register(-2),
//             ])
//             .consts([10_i32, 20]),
//         )
//         .run()
// }

// #[test]
// #[cfg_attr(miri, ignore)]
// fn return_if_results_5() {
//     let wasm = r"
//         (module
//             (func (param i32 i32 i32) (result i32 i32 i32 i32 i32)
//                 (local.get 0)
//                 (local.get 1)
//                 (local.get 0)
//                 (local.get 1)
//                 (local.get 0)
//                 (br_if 0
//                     (local.get 2) ;; br_if condition
//                 )
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func_instrs([
//             Instruction::return_nez_many_ext(Reg::from(2), 0, 1),
//             Instruction::register3_ext(0, 1, 0),
//             Instruction::return_many_ext(0, 1, 0),
//             Instruction::register2_ext(1, 0),
//         ])
//         .run()
// }

// #[test]
// #[cfg_attr(miri, ignore)]
// fn return_if_results_5_imm() {
//     let wasm = r"
//         (module
//             (func (param i32) (result i32 i32 i32 i32 i32)
//                 (i32.const 10)
//                 (i32.const 20)
//                 (i32.const 10)
//                 (i32.const 20)
//                 (i32.const 10)
//                 (br_if 0
//                     (local.get 0) ;; br_if condition
//                 )
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func(
//             ExpectedFunc::new([
//                 Instruction::return_nez_many_ext(Reg::from(0), -1, -2),
//                 Instruction::register3_ext(-1, -2, -1),
//                 Instruction::return_many_ext(-1, -2, -1),
//                 Instruction::register2_ext(-2, -1),
//             ])
//             .consts([10_i32, 20]),
//         )
//         .run()
// }

// #[test]
// #[cfg_attr(miri, ignore)]
// fn return_if_results_6() {
//     let wasm = r"
//         (module
//             (func (param i32 i32 i32) (result i32 i32 i32 i32 i32 i32)
//                 (local.get 0)
//                 (local.get 1)
//                 (local.get 0)
//                 (local.get 1)
//                 (local.get 0)
//                 (local.get 1)
//                 (br_if 0
//                     (local.get 2) ;; br_if condition
//                 )
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func_instrs([
//             Instruction::return_nez_many_ext(Reg::from(2), 0, 1),
//             Instruction::register_list_ext(0, 1, 0),
//             Instruction::register(1),
//             Instruction::return_many_ext(0, 1, 0),
//             Instruction::register3_ext(1, 0, 1),
//         ])
//         .run()
// }

// #[test]
// #[cfg_attr(miri, ignore)]
// fn return_if_results_6_imm() {
//     let wasm = r"
//         (module
//             (func (param i32) (result i32 i32 i32 i32 i32 i32)
//                 (i32.const 10)
//                 (i32.const 20)
//                 (i32.const 10)
//                 (i32.const 20)
//                 (i32.const 10)
//                 (i32.const 20)
//                 (br_if 0
//                     (local.get 0) ;; br_if condition
//                 )
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func(
//             ExpectedFunc::new([
//                 Instruction::return_nez_many_ext(Reg::from(0), -1, -2),
//                 Instruction::register_list_ext(-1, -2, -1),
//                 Instruction::register(-2),
//                 Instruction::return_many_ext(-1, -2, -1),
//                 Instruction::register3_ext(-2, -1, -2),
//             ])
//             .consts([10_i32, 20]),
//         )
//         .run()
// }

// #[test]
// #[cfg_attr(miri, ignore)]
// fn branch_if_results_0() {
//     let wasm = r"
//         (module
//             (func (param i32)
//                 (local.get 0)
//                 (block (param i32)
//                     (br_if 0)
//                 )
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func_instrs([
//             Instruction::copy(1, 0),
//             Instruction::branch_i32_ne_imm16(Reg::from(1), 0, BranchOffset16::from(1)),
//             Instruction::Return,
//         ])
//         .run()
// }

#[test]
#[cfg_attr(miri, ignore)]
fn branch_if_results_1() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (block (param i32 i32) (result i32)
                    (br_if 0)
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(RegSpan::new(Reg::from(3)), 0, 1),
            Instruction::branch_i32_eq_imm16(Reg::from(4), 0, BranchOffset16::from(3)),
            Instruction::copy(2, 3),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy(2, 3),
            Instruction::return_reg(2),
        ])
        .run()
}

/// Variant of the [`branch_if_results_1`] test where it is possible to avoid copies.
///
/// # Note
///
/// Copy elision is possible since the registers on top of the stack
/// are the same as the expected block results when translating the Wasm `br_if`.
/// We achieve this by using expressions as inputs such as `(i32.clz (local.get 0))`.
#[test]
#[cfg_attr(miri, ignore)]
fn branch_if_results_1_avoid_copy() {
    let wasm = r"
        (module
            (func (param i32 i32) (result i32)
                (i32.clz (local.get 0))
                (i32.ctz (local.get 1))
                (block (param i32 i32) (result i32)
                    (br_if 0)
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_clz(Reg::from(2), Reg::from(0)),
            Instruction::i32_ctz(Reg::from(3), Reg::from(1)),
            Instruction::branch_i32_ne_imm16(Reg::from(3), 0, BranchOffset16::from(1)),
            Instruction::return_reg(Reg::from(2)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn branch_if_results_2() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (block (param i32 i32 i32) (result i32 i32)
                    (br_if 0)
                )
                (i32.add)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span_non_overlapping(
                RegSpan::new(Reg::from(5)),
                RegSpan::new(Reg::from(0)),
                3_u16,
            ),
            Instruction::branch_i32_eq_imm16(Reg::from(7), 0, BranchOffset16::from(3)),
            Instruction::copy2_ext(RegSpan::new(Reg::from(3)), 5, 6),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy2_ext(RegSpan::new(Reg::from(3)), 5, 6),
            Instruction::i32_add(Reg::from(3), Reg::from(3), Reg::from(4)),
            Instruction::return_reg(3),
        ])
        .run()
}

/// Variant of the [`branch_if_results_2`] test where it is possible to avoid copies.
///
/// # Note
///
/// Read the docs on [`branch_if_results_1_avoid_copy`] test for more information.
#[test]
#[cfg_attr(miri, ignore)]
fn branch_if_results_2_avoid_copy() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32)
                (i32.clz (local.get 0)) ;; on dynamic register stack
                (i32.ctz (local.get 1)) ;; on dynamic register stack
                (block (param i32 i32) (result i32 i32)
                    (br_if 0
                        (local.get 2) ;; br_if condition
                    )
                )
                (i32.add)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_clz(Reg::from(3), Reg::from(0)),
            Instruction::i32_ctz(Reg::from(4), Reg::from(1)),
            Instruction::branch_i32_ne_imm16(Reg::from(2), 0, BranchOffset16::from(1)),
            Instruction::i32_add(Reg::from(3), Reg::from(3), Reg::from(4)),
            Instruction::return_reg(Reg::from(3)),
        ])
        .run()
}

/// This test case was design to specifically test the `copy_span` optimizations.
#[test]
#[cfg_attr(miri, ignore)]
fn branch_if_results_4_mixed_1() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32 i32)
                (block (result i32 i32 i32 i32)
                    (i32.const 10)
                    (local.get 0)
                    (local.get 1)
                    (i32.const 20)
                    (br_if 0
                        (local.get 2) ;; br_if condition
                    )
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::branch_i32_eq_imm16(Reg::from(2), 0, BranchOffset16::from(4)),
                Instruction::copy_many_non_overlapping_ext(RegSpan::new(Reg::from(3)), -1, 0),
                Instruction::register2_ext(1, -2),
                Instruction::branch(BranchOffset::from(3)),
                Instruction::copy_many_non_overlapping_ext(RegSpan::new(Reg::from(3)), -1, 0),
                Instruction::register2_ext(1, -2),
                Instruction::return_span(bspan(3, 4)),
            ])
            .consts([10_i32, 20]),
        )
        .run()
}

/// This test case was design to specifically test the `copy_span` optimizations.
#[test]
#[cfg_attr(miri, ignore)]
fn branch_if_results_4_mixed_2() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32 i32)
                (block (result i32 i32 i32 i32)
                    (local.get 0)
                    (local.get 0)
                    (local.get 1)
                    (local.get 1)
                    (br_if 0
                        (local.get 2) ;; br_if condition
                    )
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(Reg::from(2), 0, BranchOffset16::from(4)),
            Instruction::copy_many_non_overlapping_ext(RegSpan::new(Reg::from(3)), 0, 0),
            Instruction::register2_ext(1, 1),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::copy_many_non_overlapping_ext(RegSpan::new(Reg::from(3)), 0, 0),
            Instruction::register2_ext(1, 1),
            Instruction::return_span(bspan(3, 4)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn branch_if_i32_eqz() {
    let wasm = r"
        (module
            (func (param i32)
                (block $exit
                    (i32.eqz (local.get 0)) ;; br_if condition
                    (br_if $exit)
                    (drop (i32.add (local.get 0) (i32.const 1)))
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm16(0, 0_i16, BranchOffset16::from(2)),
            Instruction::i32_add_imm16(1, 0, 1),
            Instruction::r#return(),
        ])
        .run()
}

// #[test]
// #[cfg_attr(miri, ignore)]
// fn return_if_i32_eqz() {
//     let wasm = r"
//         (module
//             (func (param i32)
//                 (i32.eqz (local.get 0)) ;; br_if condition
//                 (br_if 0)
//                 (drop (i32.add (local.get 0) (i32.const 1)))
//             )
//         )";
//     TranslationTest::new(wasm)
//         .expect_func_instrs([
//             Instruction::i32_eq_imm16(1, 0, 0),
//             Instruction::return_nez(1),
//             Instruction::i32_add_imm16(1, 0, 1),
//             Instruction::r#return(),
//         ])
//         .run()
// }
