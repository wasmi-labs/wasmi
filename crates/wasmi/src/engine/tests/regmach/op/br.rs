use super::*;
use crate::engine::tests::regmach::{
    display_wasm::DisplayValueType,
    driver::ExpectedFunc,
    wasm_type::WasmType,
};
use core::fmt::Display;

#[test]
fn as_return() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (br 0)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::Return])
        .run()
}

#[test]
fn as_return_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (br 0)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(Register::from_i16(0))])
        .run()
}

#[test]
fn as_return_1_imm() {
    fn test_for<T>(value: T)
    where
        T: WasmType,
        DisplayWasm<T>: Display,
    {
        let display_ty = DisplayValueType::from(<T as WasmType>::VALUE_TYPE);
        let display_value = DisplayWasm::from(value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result {display_ty})
                    ({display_ty}.const {display_value})
                    (br 0)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func(
                ExpectedFunc::new([Instruction::return_reg(Register::from_i16(-1))])
                    .consts([value]),
            )
            .run()
    }
    test_for::<i64>(i64::MIN);
    test_for::<i64>(i64::MAX);
    test_for::<i64>(i64::from(i32::MIN) - 1);
    test_for::<i64>(i64::from(i32::MAX) + 1);
    test_for::<f64>(0.3);
    test_for::<f64>(0.123456789);
    test_for::<f64>(-0.123456789);
    test_for::<f64>(0.987654321);
    test_for::<f64>(-0.987654321);
}

#[test]
fn as_return_1_imm32() {
    fn test_for<T>(value: T)
    where
        T: WasmType + Into<AnyConst32>,
        DisplayWasm<T>: Display,
    {
        let display_ty = DisplayValueType::from(<T as WasmType>::VALUE_TYPE);
        let display_value = DisplayWasm::from(value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result {display_ty})
                    ({display_ty}.const {display_value})
                    (br 0)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([Instruction::return_imm32(value)])
            .run()
    }
    test_for::<i32>(5);
    test_for::<i32>(42);
    test_for::<f32>(5.5);
    test_for::<f32>(-42.25);
}

#[test]
fn as_return_1_i64imm32() {
    fn test_for(value: i64) {
        let display_value = DisplayWasm::from(value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result i64)
                    (i64.const {display_value})
                    (br 0)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([return_i64imm32_instr(value)])
            .run()
    }
    test_for(0);
    test_for(1);
    test_for(-1);
    test_for(5);
    test_for(-42);
    test_for(i64::from(i32::MIN));
    test_for(i64::from(i32::MAX));
}

#[test]
fn as_return_1_f64imm32() {
    fn test_for(value: f64) {
        let display_value = DisplayWasm::from(value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result f64)
                    (f64.const {display_value})
                    (br 0)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([return_f64imm32_instr(value)])
            .run()
    }
    test_for(0.0);
    test_for(0.25);
    test_for(-0.25);
    test_for(0.5);
    test_for(-0.5);
    test_for(1.0);
    test_for(-1.0);
    test_for(-42.25);
    test_for(f64::NEG_INFINITY);
    test_for(f64::INFINITY);
    test_for(f64::NAN);
    test_for(f64::EPSILON);
}
