use super::*;
use crate::engine::{bytecode::FuncIdx, RegisterSpan};

#[test]
#[cfg_attr(miri, ignore)]
fn no_params() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f))
            (func
                (call $f)
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_imported_0(
                RegisterSpan::new(Register::from_i16(0)),
                FuncIdx::from(0),
            ),
            Instruction::Return,
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_param_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32) (result i32)))
            (func (param i32) (result i32)
                (call $f (local.get 0))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(1)), FuncIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(1), 1),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_param_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32) (result i32)))
            (func (result i32)
                (call $f (i32.const 10))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(0)), FuncIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(1), 1),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32 i32) (result i32 i32)))
            (func (param i32 i32) (result i32 i32)
                (call $f (local.get 0) (local.get 1))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(2)), FuncIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(2), 2),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(2)).iter(2)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_reg_rev() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32 i32) (result i32 i32)))
            (func (param i32 i32) (result i32 i32)
                (call $f (local.get 1) (local.get 0))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(2), Register::from(1)),
            Instruction::copy(Register::from_i16(3), Register::from(0)),
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(2)), FuncIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(2)).iter(2), 2),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(2)).iter(2)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32 i32) (result i32 i32)))
            (func (result i32 i32)
                (call $f (i32.const 10) (i32.const 20))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy_imm32(Register::from_i16(1), 20_i32),
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(0)), FuncIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(2), 2),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(0)).iter(2)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32 i32 i32) (result i32 i32 i32)))
            (func (param i32 i32 i32) (result i32 i32 i32)
                (call $f (local.get 0) (local.get 1) (local.get 2))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(3)), FuncIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(3), 3),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(3)).iter(3)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_reg_rev() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32 i32 i32) (result i32 i32 i32)))
            (func (param i32 i32 i32) (result i32 i32 i32)
                (call $f (local.get 2) (local.get 1) (local.get 0))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(3), Register::from(2)),
            Instruction::copy(Register::from_i16(4), Register::from(1)),
            Instruction::copy(Register::from_i16(5), Register::from(0)),
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(3)), FuncIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(3)).iter(3), 3),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(3)).iter(3)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32 i32 i32) (result i32 i32 i32)))
            (func (result i32 i32 i32)
                (call $f (i32.const 10) (i32.const 20) (i32.const 30))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy_imm32(Register::from_i16(1), 20_i32),
            Instruction::copy_imm32(Register::from_i16(2), 30_i32),
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(0)), FuncIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(3), 3),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(0)).iter(3)),
        ])
        .run();
}
