use super::*;

use crate::engine::bytecode::GlobalIdx;
use core::fmt::Display;
use wasm_type::WasmType;

/// Test for `global.get` of internally defined mutable global variables.
///
/// # Note
///
/// Optimization to replace `global.get` with the underlying initial value
/// of the global variable cannot be done since the value might change
/// during program execution.
fn test_mutable<T>(value: T)
where
    T: WasmType,
    DisplayWasm<T>: Display,
{
    let ty = T::NAME;
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $g (mut {ty}) ({ty}.const {display_value}))
            (func (result {ty})
                global.get $g
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::global_get(Register::from_u16(0), GlobalIdx::from(0)),
            Instruction::return_reg(Register::from_u16(0)),
        ])
        .run()
}

#[test]
fn mutable_i32() {
    test_mutable::<i32>(42);
}

#[test]
fn mutable_i64() {
    test_mutable::<i64>(42);
}

#[test]
fn mutable_f32() {
    test_mutable::<f32>(42.5);
}

#[test]
fn mutable_f64() {
    test_mutable::<f64>(42.5);
}

/// Test for `global.get` of internally defined immutable global variables.
///
/// # Note
///
/// In this case optimization to replace the `global.get` with the constant
/// value of the global variable can be applied always.
fn test_immutable<T>(value: T)
where
    T: WasmType,
    DisplayWasm<T>: Display,
{
    let ty = T::NAME;
    let display_value = DisplayWasm::from(value);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (global $g {ty} ({ty}.const {display_value}))
            (func (result {ty})
                global.get $g
            )
        )
    "#,
    ));
    let instr = <T as WasmType>::return_imm_instr(&value);
    TranslationTest::new(wasm).expect_func([instr]).run()
}

#[test]
fn immutable_i32() {
    test_immutable::<i32>(42);
}

#[test]
fn immutable_i64() {
    test_immutable::<i64>(42);
}

#[test]
fn immutable_f32() {
    test_immutable::<f32>(42.5);
}

#[test]
fn immutable_f64() {
    test_immutable::<f64>(42.5);
}

/// Test for `global.get` of immutable imported global variables.
///
/// # Note
///
/// Even though the accessed global variable is immutable no
/// optimization can be applied since its value is unknown due
/// to being imported.
fn test_imported<T>()
where
    T: WasmType,
{
    let ty = T::NAME;
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (import "host" "g" (global $g {ty}))
            (func (result {ty})
                global.get $g
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::global_get(Register::from_u16(0), GlobalIdx::from(0)),
            Instruction::return_reg(Register::from_u16(0)),
        ])
        .run()
}

#[test]
fn imported_i32() {
    test_imported::<i32>();
}

#[test]
fn imported_i64() {
    test_imported::<i64>();
}

#[test]
fn imported_f32() {
    test_imported::<f32>();
}

#[test]
fn imported_f64() {
    test_imported::<f64>();
}
