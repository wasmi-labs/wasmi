use super::*;
use crate::engine::{
    bytecode::RegisterSpan,
    regmach::translator::tests::{
        display_wasm::DisplayValueType,
        driver::ExpectedFunc,
        wasm_type::WasmType,
    },
};
use core::fmt::Display;

#[test]
#[cfg_attr(miri, ignore)]
fn return_0() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::Return])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_1() {
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
        .expect_func_instrs([Instruction::return_reg(Register::from_i16(0))])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_1_imm() {
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
        TranslationTest::new(wasm)
            .expect_func(
                ExpectedFunc::new([Instruction::return_reg(Register::from_i16(-1))])
                    .consts([value.into()]),
            )
            .run()
    }
    test_for::<i64>(i64::MIN);
    test_for::<i64>(i64::MAX);
    test_for::<i64>(i64::from(i32::MIN) - 1);
    test_for::<i64>(i64::from(i32::MAX) + 1);
    test_for::<f64>(0.3);
    test_for::<f64>(-0.3);
    test_for::<f64>(0.123456789);
    test_for::<f64>(0.987654321);
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_1_imm32() {
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
                    (return)
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
fn return_1_i64imm32() {
    fn test_for(value: i64) {
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
            .expect_func_instrs([return_i64imm32_instr(value)])
            .run()
    }
    test_for(0);
    test_for(1);
    test_for(-1);
    test_for(5);
    test_for(-42);
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_1_f64imm32() {
    fn test_for(value: f64) {
        let display_value = DisplayWasm::from(value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result f64)
                    (f64.const {display_value})
                    (return)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([return_f64imm32_instr(value)])
            .run()
    }
    test_for(0.0);
    test_for(1.0);
    test_for(-1.0);
    test_for(5.5);
    test_for(-42.25);
    test_for(f64::NEG_INFINITY);
    test_for(f64::INFINITY);
    test_for(f64::NAN);
    test_for(f64::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_2() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32 i32)
                (local.get 0)
                (local.get 0)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg2(0, 0)])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_2_imm() {
    let wasm = wat2wasm(
        r"
        (module
            (func (result i32 i32)
                (i32.const 10)
                (i32.const 20)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func(ExpectedFunc::new([Instruction::return_reg2(-1, -2)]).consts([10_i32, 20]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_2_mixed() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32 i32)
                (i32.const 10)
                (local.get 0)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func(ExpectedFunc::new([Instruction::return_reg2(-1, 0)]).consts([10_i32]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_3() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 0)
                (local.get 0)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg3(0, 0, 0)])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_3_imm() {
    let wasm = wat2wasm(
        r"
        (module
            (func (result i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 30)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([Instruction::return_reg3(-1, -2, -3)]).consts([10_i32, 20, 30]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_3_mixed() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32 i32 i32)
                (i32.const 10)
                (local.get 0)
                (i32.const 10)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func(ExpectedFunc::new([Instruction::return_reg3(-1, 0, -1)]).consts([10_i32]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_4_span() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32 i32 i32) (result i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(
            RegisterSpan::new(Register::from_i16(0)).iter(4),
        )])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_4() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32 i32 i32 i32)
                (local.get 0)
                (local.get 0)
                (local.get 0)
                (local.get 0)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_many(0, 0, 0), Instruction::register(0)])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_5_span() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(
            RegisterSpan::new(Register::from_i16(0)).iter(5),
        )])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_5() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_many(0, 1, 0),
            Instruction::register2(1, 0),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_6() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_many(0, 1, 0),
            Instruction::register3(1, 0, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_7() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_many(0, 1, 0),
            Instruction::register_list(1, 0, 1),
            Instruction::register(0),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_8() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_many(0, 1, 0),
            Instruction::register_list(1, 0, 1),
            Instruction::register2(0, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn return_9() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (local.get 1)
                (local.get 0)
                (return)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_many(0, 1, 0),
            Instruction::register_list(1, 0, 1),
            Instruction::register3(0, 1, 0),
        ])
        .run()
}
