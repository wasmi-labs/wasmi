use super::*;
use crate::{
    core::UntypedValue,
    engine::{bytecode::BranchOffset, translator::tests::wasm_type::WasmType},
};
use core::fmt::Display;

#[test]
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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

#[test]
#[cfg_attr(miri, ignore)]
fn test_br_as_return_values() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (result i32 i64)
                (i32.const 2)
                (block (result i64)
                    (return (br 0 (i32.const 1) (i64.const 7)))
                )
            )
        )
        "#,
    );
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::copy_i64imm32(Register::from_i16(0), 7),
                Instruction::branch(BranchOffset::from(1)),
                Instruction::return_reg2(-1, 0),
            ])
            .consts([UntypedValue::from(2_i32)]),
        )
        .run()
}
