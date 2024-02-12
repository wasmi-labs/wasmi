use super::*;

use crate::engine::{
    bytecode::GlobalIdx,
    translator::tests::{display_wasm::DisplayValueType, driver::ExpectedFunc},
};
use core::fmt::Display;
use wasm_type::WasmType;
use wasmi_core::ValueType;

fn test_reg<T>()
where
    T: WasmType + Default,
    DisplayWasm<T>: Display,
{
    let ty = T::NAME;
    let display_value = DisplayWasm::from(T::default());
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $g (mut {ty}) ({ty}.const {display_value}))
            (func (param $value {ty})
                local.get $value
                global.set $g
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_set(GlobalIdx::from(0), Register::from_i16(0)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_reg::<i32>();
    test_reg::<i64>();
    test_reg::<f32>();
    test_reg::<f64>();
}

fn test_imm<T>(value: T)
where
    T: WasmType + Default,
    DisplayWasm<T>: Display,
{
    let display_ty = DisplayValueType::from(T::VALUE_TYPE);
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $g (mut {display_ty}) ({display_ty}.const {display_value}))
            (func (param $value {display_ty})
                {display_ty}.const {display_value}
                global.set $g
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::global_set(GlobalIdx::from(0), Register::from_i16(-1)),
                Instruction::Return,
            ])
            .consts([value]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    test_imm::<i32>(i32::from(i16::MAX) + 1);
    test_imm::<i32>(i32::from(i16::MIN) - 1);
    test_imm::<i64>(i64::from(i16::MAX) + 1);
    test_imm::<i64>(i64::from(i16::MIN) - 1);
    test_imm::<f32>(0.0);
    test_imm::<f32>(-1.0);
    test_imm::<f64>(0.0);
    test_imm::<f64>(-1.0);
}

fn test_i32imm16(value: i32) {
    let display_ty = DisplayValueType::from(ValueType::I32);
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $g (mut {display_ty}) ({display_ty}.const {display_value}))
            (func (param $value {display_ty})
                {display_ty}.const {display_value}
                global.set $g
            )
        )
    "#,
    ));
    let imm16 = <Const16<i32>>::try_from(value)
        .unwrap_or_else(|_| panic!("cannot convert `value` to 16-bit encoding: {value}"));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_set_i32imm16(GlobalIdx::from(0), imm16),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn i32imm16() {
    test_i32imm16(0);
    test_i32imm16(1);
    test_i32imm16(-1);
    test_i32imm16(i32::from(i16::MAX));
    test_i32imm16(i32::from(i16::MIN));
}

fn test_i64imm16(value: i64) {
    let display_ty = DisplayValueType::from(ValueType::I64);
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $g (mut {display_ty}) ({display_ty}.const {display_value}))
            (func (param $value {display_ty})
                {display_ty}.const {display_value}
                global.set $g
            )
        )
    "#,
    ));
    let imm16 = <Const16<i64>>::try_from(value)
        .unwrap_or_else(|_| panic!("cannot convert `value` to 16-bit encoding: {value}"));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_set_i64imm16(GlobalIdx::from(0), imm16),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn i64imm16() {
    test_i64imm16(0);
    test_i64imm16(1);
    test_i64imm16(-1);
    test_i64imm16(i64::from(i16::MAX));
    test_i64imm16(i64::from(i16::MIN));
}

fn test_shadow_stack_in_v0(value: i32) {
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $__shadow_stack (mut i32) (i32.const 1000))
            (func
                (local $v i32)
                global.get $__shadow_stack
                i32.const {value}
                i32.sub
                local.tee $v
                global.set $__shadow_stack
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_add_imm_inout_global_0(Register::from_i16(0), value.wrapping_neg()),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn shadow_stack_in_v0() {
    test_shadow_stack_in_v0(-4);
    test_shadow_stack_in_v0(4);
    test_shadow_stack_in_v0(i32::from(i16::MIN));
    test_shadow_stack_in_v0(i32::from(i16::MIN) - 1);
    test_shadow_stack_in_v0(i32::from(i16::MAX));
    test_shadow_stack_in_v0(i32::from(i16::MAX) + 1);
    test_shadow_stack_in_v0(i32::MIN);
    test_shadow_stack_in_v0(i32::MIN + 1);
    test_shadow_stack_in_v0(i32::MAX);
    test_shadow_stack_in_v0(i32::MAX - 1);
}

fn test_shadow_stack_in_v1(value: i32) {
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $__shadow_stack (mut i32) (i32.const 1000))
            (func
                (local $v i32)
                global.get $__shadow_stack
                i32.const {value}
                i32.add
                local.tee $v
                global.set $__shadow_stack
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_add_imm_inout_global_0(Register::from_i16(0), value),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn shadow_stack_in_v1() {
    test_shadow_stack_in_v1(-4);
    test_shadow_stack_in_v1(4);
    test_shadow_stack_in_v1(i32::from(i16::MIN));
    test_shadow_stack_in_v1(i32::from(i16::MIN) - 1);
    test_shadow_stack_in_v1(i32::from(i16::MAX));
    test_shadow_stack_in_v1(i32::from(i16::MAX) + 1);
    test_shadow_stack_in_v1(i32::MIN);
    test_shadow_stack_in_v1(i32::MIN + 1);
    test_shadow_stack_in_v1(i32::MAX);
    test_shadow_stack_in_v1(i32::MAX - 1);
}

fn test_shadow_stack_out_v0(value: i32) {
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $__shadow_stack (mut i32) (i32.const 1000))
            (func (param $v i32)
                local.get $v
                i32.const {value}
                i32.add
                global.set $__shadow_stack
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_add_imm_into_global_0(Register::from_i16(0), value),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn shadow_stack_out_v0() {
    test_shadow_stack_out_v0(-4);
    test_shadow_stack_out_v0(4);
    test_shadow_stack_out_v0(i32::from(i16::MIN));
    test_shadow_stack_out_v0(i32::from(i16::MIN) + 1);
    test_shadow_stack_out_v0(i32::from(i16::MAX) - 1);
    test_shadow_stack_out_v0(i32::from(i16::MAX));
}

fn test_shadow_stack_out_v0_big(value: i32) {
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $__shadow_stack (mut i32) (i32.const 1000))
            (func (param $v i32)
                local.get $v
                i32.const {value}
                i32.add
                global.set $__shadow_stack
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::i32_add_imm_into_global_0(Register::from_i16(0), value),
                Instruction::Return,
            ])
            .consts([value]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn shadow_stack_out_v0_big() {
    test_shadow_stack_out_v0_big(i32::from(i16::MIN) - 1);
    test_shadow_stack_out_v0_big(i32::from(i16::MAX) + 1);
    test_shadow_stack_out_v0_big(i32::MIN);
    test_shadow_stack_out_v0_big(i32::MIN + 1);
    test_shadow_stack_out_v0_big(i32::MAX);
    test_shadow_stack_out_v0_big(i32::MAX - 1);
}

fn test_shadow_stack_out_v1(value: i32) {
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $__shadow_stack (mut i32) (i32.const 1000))
            (func (param $v i32)
                local.get $v
                i32.const {value}
                i32.sub
                global.set $__shadow_stack
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_add_imm_into_global_0(Register::from_i16(0), value.wrapping_neg()),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn shadow_stack_out_v1() {
    test_shadow_stack_out_v1(-4);
    test_shadow_stack_out_v1(4);
    test_shadow_stack_out_v1(i32::from(i16::MIN) + 1);
    test_shadow_stack_out_v1(i32::from(i16::MAX) - 1);
    test_shadow_stack_out_v1(i32::from(i16::MAX));
    test_shadow_stack_out_v1(i32::from(i16::MAX) + 1);
}

fn test_shadow_stack_out_v1_big(value: i32) {
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $__shadow_stack (mut i32) (i32.const 1000))
            (func (param $v i32)
                local.get $v
                i32.const {value}
                i32.sub
                global.set $__shadow_stack
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::i32_add_imm_into_global_0(Register::from_i16(0), value.wrapping_neg()),
                Instruction::Return,
            ])
            .consts([value.wrapping_neg()]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn shadow_stack_out_v1_big() {
    test_shadow_stack_out_v1_big(i32::from(i16::MIN));
    test_shadow_stack_out_v1_big(i32::from(i16::MAX) + 2);
    test_shadow_stack_out_v1_big(i32::MIN);
    test_shadow_stack_out_v1_big(i32::MIN + 1);
    test_shadow_stack_out_v1_big(i32::MAX - 1);
    test_shadow_stack_out_v1_big(i32::MAX);
}
