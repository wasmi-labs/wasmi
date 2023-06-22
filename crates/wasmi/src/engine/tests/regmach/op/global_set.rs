use super::*;

use crate::engine::bytecode::GlobalIdx;
use core::fmt::Display;
use wasm_type::WasmType;

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
        .expect_func([
            Instruction::global_set(GlobalIdx::from(0), Register::from_u16(0)),
            Instruction::Return,
        ])
        .run()
}

#[test]
fn reg() {
    test_reg::<i32>();
    test_reg::<i64>();
    test_reg::<f32>();
    test_reg::<f64>();
}

fn test_imm32<T>(value: T)
where
    T: WasmType + Default + Into<Const32>,
    DisplayWasm<T>: Display,
{
    let ty = T::NAME;
    let init_value = DisplayWasm::from(T::default());
    let new_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $g (mut {ty}) ({ty}.const {init_value}))
            (func
                {ty}.const {new_value}
                global.set $g
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::global_set_imm32(GlobalIdx::from(0)),
            Instruction::const32(value),
            Instruction::Return,
        ])
        .run()
}

#[test]
fn imm_i32() {
    test_imm32::<i32>(42);
}

#[test]
fn imm_f32() {
    test_imm32::<f32>(42.5);
}

fn test_imm64<T>(value: T)
where
    T: WasmType + Default,
    DisplayWasm<T>: Display,
{
    let ty = T::NAME;
    let init_value = DisplayWasm::from(T::default());
    let new_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $g (mut {ty}) ({ty}.const {init_value}))
            (func
                {ty}.const {new_value}
                global.set $g
            )
        )
    "#,
    ));
    let cref = ConstRef::from_u32(0);
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::global_set_imm(GlobalIdx::from(0)),
            Instruction::const_ref(cref),
            Instruction::Return,
        ])
        .expect_const(cref, value)
        .run()
}

#[test]
fn imm_i64() {
    test_imm64::<i64>(42);
}

#[test]
fn imm_f64() {
    test_imm64::<f64>(42.5);
}
