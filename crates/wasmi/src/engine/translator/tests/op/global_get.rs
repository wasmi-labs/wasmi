use super::*;

use crate::engine::bytecode::GlobalIdx;
use core::fmt::Display;
use wasm_type::WasmTy;

/// Test for `global.get` of internally defined mutable global variables.
///
/// # Note
///
/// Optimization to replace `global.get` with the underlying initial value
/// of the global variable cannot be done since the value might change
/// during program execution.
fn test_mutable<T>(value: T)
where
    T: WasmTy,
    DisplayWasm<T>: Display,
{
    let ty = T::NAME;
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (global $g (mut {ty}) ({ty}.const {display_value}))
            (func (result {ty})
                global.get $g
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            Instruction::global_get(Reg::from_i16(0), GlobalIdx::from(0)),
            Instruction::return_reg(Reg::from_i16(0)),
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
    T: WasmTy,
    DisplayWasm<T>: Display,
{
    let ty = T::NAME;
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (global $g {ty} ({ty}.const {display_value}))
            (func (result {ty})
                global.get $g
            )
        )
    "#,
    );
    let mut testcase = TranslationTest::from_wat(&wasm);
    let instr = <T as WasmTy>::return_imm_instr(&value);
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
    T: WasmTy,
{
    let ty = T::NAME;
    let wasm = format!(
        r#"
        (module
            (import "host" "g" (global $g {ty}))
            (func (result {ty})
                global.get $g
            )
        )
    "#,
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            Instruction::global_get(Reg::from_i16(0), GlobalIdx::from(0)),
            Instruction::return_reg(Reg::from_i16(0)),
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
    let wasm = r#"
        (module
            (global $g (mut i64) (i64.const 0))
            (func (result i32 i64)
                (i32.const 2)
                (global.get $g)
            )
        )
        "#;
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::global_get(Reg::from_i16(0), GlobalIdx::from(0)),
                Instruction::return_reg2(-1, 0),
            ])
            .consts([2_i32]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_global_get_as_return_values_1() {
    let wasm = r#"
        (module
            (global $g (mut i64) (i64.const 0))
            (func (result i32 i64)
                (block (result i32 i64)
                    (i32.const 2)
                    (global.get $g)
                )
            )
        )
        "#;
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::global_get(Reg::from_i16(0), GlobalIdx::from(0)),
                Instruction::return_reg2(-1, 0),
            ])
            .consts([2_i32]),
        )
        .run()
}
