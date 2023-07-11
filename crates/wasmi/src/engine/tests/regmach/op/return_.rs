use super::*;
use crate::engine::tests::regmach::{display_wasm::DisplayValueType, wasm_type::WasmType};
use core::fmt::Display;

#[test]
fn as_return() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::Return])
        .run()
}

#[test]
fn as_return_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::return_reg(Register::from_u16(0))])
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
                    (return)
                )
            )",
        ));
        let cref = ConstRef::from_u32(0);
        TranslationTest::new(wasm)
            .expect_func([Instruction::return_imm(cref)])
            .expect_const(cref, value.into())
            .run()
    }
    test_for::<i64>(i64::MIN);
    test_for::<i64>(i64::MAX);
    test_for::<i64>(i64::from(i32::MIN) - 1);
    test_for::<i64>(i64::from(i32::MAX) + 1);
    test_for::<f64>(0.0);
    test_for::<f64>(-1.0);
    test_for::<f64>(5.5);
    test_for::<f64>(-42.25);
}

#[test]
fn as_return_1_imm32() {
    fn test_for<T>(value: T)
    where
        T: WasmType + Into<Const32>,
        DisplayWasm<T>: Display,
    {
        let display_ty = DisplayValueType::from(<T as WasmType>::VALUE_TYPE);
        let display_value = DisplayWasm::from(value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result {display_ty})
                    ({display_ty}.const {display_value})
                    (return)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func([Instruction::return_imm32(value)])
            .run()
    }
    test_for::<i32>(5);
    test_for::<i32>(42);
    test_for::<f32>(5.5);
    test_for::<f32>(-42.25);
}

#[test]
fn as_return_1_i64imm32() {
    fn test_for(value: i32) {
        let display_value = DisplayWasm::from(value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result i64)
                    (i64.const {display_value})
                    (return)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func([Instruction::return_i64imm32(value)])
            .run()
    }
    test_for(0);
    test_for(1);
    test_for(-1);
    test_for(5);
    test_for(-42);
}
