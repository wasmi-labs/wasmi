use super::*;

use crate::engine::{
    bytecode::{BranchOffset, GlobalIdx},
    regmach::bytecode::RegisterSpan,
};
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
        .expect_func_instrs([
            Instruction::global_get(Register::from_i16(0), GlobalIdx::from(0)),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn mutable_i32() {
    test_mutable::<i32>(42);
}

#[test]
#[cfg_attr(miri, ignore)]
fn mutable_i64() {
    test_mutable::<i64>(42);
}

#[test]
#[cfg_attr(miri, ignore)]
fn mutable_f32() {
    test_mutable::<f32>(42.5);
}

#[test]
#[cfg_attr(miri, ignore)]
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
    let mut testcase = TranslationTest::new(wasm);
    let instr = <T as WasmType>::return_imm_instr(&value);
    match instr {
        Instruction::ReturnReg { value: register } => {
            assert!(register.is_const());
            testcase.expect_func(ExpectedFunc::new([instr]).consts([value]));
        }
        instr => {
            testcase.expect_func_instrs([instr]);
        }
    }
    testcase.run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn immutable_i32() {
    test_immutable::<i32>(42);
}

#[test]
#[cfg_attr(miri, ignore)]
fn immutable_i64() {
    test_immutable::<i64>(42);
}

#[test]
#[cfg_attr(miri, ignore)]
fn immutable_f32() {
    test_immutable::<f32>(42.5);
}

#[test]
#[cfg_attr(miri, ignore)]
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
        .expect_func_instrs([
            Instruction::global_get(Register::from_i16(0), GlobalIdx::from(0)),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn imported_i32() {
    test_imported::<i32>();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imported_i64() {
    test_imported::<i64>();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imported_f32() {
    test_imported::<f32>();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imported_f64() {
    test_imported::<f64>();
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_global_get_as_return_values_0() {
    let wasm = wat2wasm(
        r#"
        (module
            (global $g (mut i64) (i64.const 0))
            (func (result i32 i64)
                (i32.const 2)
                (global.get $g)
            )
        )
        "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_get(Register::from_i16(0), GlobalIdx::from(0)),
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 2_i32),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(0)).iter(2)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_global_get_as_return_values_1() {
    let wasm = wat2wasm(
        r#"
        (module
            (global $g (mut i64) (i64.const 0))
            (func (result i32 i64)
                (block (result i32 i64)
                    (i32.const 2)
                    (global.get $g)
                )
            )
        )
        "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_get(Register::from_i16(0), GlobalIdx::from(0)),
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 2_i32),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(0)).iter(2)),
        ])
        .run()
}
