use super::*;
use crate::{
    core::UntypedVal,
    engine::{
        bytecode::{BranchOffset, BranchOffset16, RegisterSpan},
        translator::tests::wasm_type::WasmTy,
    },
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
        TranslationTest::from_wat(&wasm)
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
            true => Register::from_i16(0),
            false => Register::from_i16(1),
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
        TranslationTest::from_wat(&wasm)
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
        TranslationTest::from_wat(&wasm)
            .expect_func(
                ExpectedFunc::new([Instruction::return_reg(Register::from_i16(-1))])
                    .consts([expected]),
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
        TranslationTest::from_wat(&wasm)
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
        TranslationTest::from_wat(&wasm)
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
        TranslationTest::from_wat(&wasm)
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
    TranslationTest::from_wat(wasm)
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy(3, 0),
            Instruction::return_reg(Register::from_i16(1)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez(Register::from_i16(0)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_reg(Register::from_i16(1), Register::from_i16(0)),
            Instruction::return_reg(Register::from_i16(0)),
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
        TranslationTest::from_wat(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_nez_reg(Register::from_i16(0), Register::from_i16(-1)),
                    Instruction::return_reg(Register::from_i16(-1)),
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
        let const32: AnyConst32 = returned_value.into();
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs([
                Instruction::return_nez_imm32(Register::from_i16(0), const32),
                Instruction::return_imm32(const32),
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
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs([
                return_nez_i64imm32_instr(Register::from_i16(0), returned_value),
                return_i64imm32_instr(returned_value),
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
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs([
                return_nez_f64imm32_instr(Register::from_i16(0), returned_value),
                return_f64imm32_instr(returned_value),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_reg2(Register::from_i16(2), 0, 1),
            Instruction::return_reg2(0, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_2_rev() {
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_reg2(Register::from_i16(2), 1, 0),
            Instruction::return_reg2(1, 0),
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
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_nez_reg2(Register::from_i16(0), -1, -2),
                Instruction::return_reg2(-1, -2),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_span(
                Register::from_i16(3),
                RegisterSpan::new(Register::from_i16(0)).iter(3),
            ),
            Instruction::return_reg3(0, 1, 2),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_many(Register::from_i16(2), 0, 1),
            Instruction::register(0),
            Instruction::return_reg3(0, 1, 0),
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
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_nez_many(Register::from_i16(0), -1, -2),
                Instruction::register(-3),
                Instruction::return_reg3(-1, -2, -3),
            ])
            .consts([10_i32, 20, 30]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_4_span() {
    let wasm = r"
        (module
            (func (param i32 i32 i32 i32 i32) (result i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (br_if 0
                    (local.get 4) ;; br_if condition
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_span(
                Register::from_i16(4),
                RegisterSpan::new(Register::from_i16(0)).iter(4),
            ),
            Instruction::return_span(RegisterSpan::new(Register::from_i16(0)).iter(4)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_4() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (br_if 0
                    (local.get 2) ;; br_if condition
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_many(Register::from_i16(2), 0, 1),
            Instruction::register2(0, 1),
            Instruction::return_many(0, 1, 0),
            Instruction::register(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_4_imm() {
    let wasm = r"
        (module
            (func (param i32) (result i32 i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 10)
                (i32.const 20)
                (br_if 0
                    (local.get 0) ;; br_if condition
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_nez_many(Register::from_i16(0), -1, -2),
                Instruction::register2(-1, -2),
                Instruction::return_many(-1, -2, -1),
                Instruction::register(-2),
            ])
            .consts([10_i32, 20]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_5() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (br_if 0
                    (local.get 2) ;; br_if condition
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_many(Register::from_i16(2), 0, 1),
            Instruction::register3(0, 1, 0),
            Instruction::return_many(0, 1, 0),
            Instruction::register2(1, 0),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_5_imm() {
    let wasm = r"
        (module
            (func (param i32) (result i32 i32 i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 10)
                (i32.const 20)
                (i32.const 10)
                (br_if 0
                    (local.get 0) ;; br_if condition
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_nez_many(Register::from_i16(0), -1, -2),
                Instruction::register3(-1, -2, -1),
                Instruction::return_many(-1, -2, -1),
                Instruction::register2(-2, -1),
            ])
            .consts([10_i32, 20]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_6() {
    let wasm = r"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (br_if 0
                    (local.get 2) ;; br_if condition
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_nez_many(Register::from_i16(2), 0, 1),
            Instruction::register_list(0, 1, 0),
            Instruction::register(1),
            Instruction::return_many(0, 1, 0),
            Instruction::register3(1, 0, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_if_results_6_imm() {
    let wasm = r"
        (module
            (func (param i32) (result i32 i32 i32 i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 10)
                (i32.const 20)
                (i32.const 10)
                (i32.const 20)
                (br_if 0
                    (local.get 0) ;; br_if condition
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_nez_many(Register::from_i16(0), -1, -2),
                Instruction::register_list(-1, -2, -1),
                Instruction::register(-2),
                Instruction::return_many(-1, -2, -1),
                Instruction::register3(-2, -1, -2),
            ])
            .consts([10_i32, 20]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn branch_if_results_0() {
    let wasm = r"
        (module
            (func (param i32)
                (local.get 0)
                (block (param i32)
                    (br_if 0)
                )
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy(1, 0),
            Instruction::branch_i32_nez(Register::from_i16(1), BranchOffset16::from(1)),
            Instruction::Return,
        ])
        .run()
}

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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegisterSpan::new(Register::from_i16(3)), 0, 1),
            Instruction::branch_i32_eqz(Register::from_i16(4), BranchOffset16::from(3)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::i32_clz(Register::from_i16(2), Register::from_i16(0)),
            Instruction::i32_ctz(Register::from_i16(3), Register::from_i16(1)),
            Instruction::branch_i32_nez(Register::from_i16(3), BranchOffset16::from(1)),
            Instruction::return_reg(Register::from_i16(2)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::copy_span_non_overlapping(
                RegisterSpan::new(Register::from_i16(5)),
                RegisterSpan::new(Register::from_i16(0)),
                3,
            ),
            Instruction::branch_i32_eqz(Register::from_i16(7), BranchOffset16::from(3)),
            Instruction::copy2(RegisterSpan::new(Register::from_i16(3)), 5, 6),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy2(RegisterSpan::new(Register::from_i16(3)), 5, 6),
            Instruction::i32_add(
                Register::from_i16(3),
                Register::from_i16(3),
                Register::from_i16(4),
            ),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::i32_clz(Register::from_i16(3), Register::from_i16(0)),
            Instruction::i32_ctz(Register::from_i16(4), Register::from_i16(1)),
            Instruction::branch_i32_nez(Register::from_i16(2), BranchOffset16::from(1)),
            Instruction::i32_add(
                Register::from_i16(3),
                Register::from_i16(3),
                Register::from_i16(4),
            ),
            Instruction::return_reg(Register::from_i16(3)),
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
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::branch_i32_eqz(Register::from_i16(2), BranchOffset16::from(4)),
                Instruction::copy_many_non_overlapping(
                    RegisterSpan::new(Register::from_i16(3)),
                    -1,
                    0,
                ),
                Instruction::register2(1, -2),
                Instruction::branch(BranchOffset::from(3)),
                Instruction::copy_many_non_overlapping(
                    RegisterSpan::new(Register::from_i16(3)),
                    -1,
                    0,
                ),
                Instruction::register2(1, -2),
                Instruction::return_span(RegisterSpan::new(Register::from_i16(3)).iter(4)),
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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eqz(Register::from_i16(2), BranchOffset16::from(4)),
            Instruction::copy_many_non_overlapping(RegisterSpan::new(Register::from_i16(3)), 0, 0),
            Instruction::register2(1, 1),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::copy_many_non_overlapping(RegisterSpan::new(Register::from_i16(3)), 0, 0),
            Instruction::register2(1, 1),
            Instruction::return_span(RegisterSpan::new(Register::from_i16(3)).iter(4)),
        ])
        .run()
}
