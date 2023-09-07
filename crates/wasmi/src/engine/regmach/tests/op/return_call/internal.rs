use super::*;
use crate::engine::{CompiledFunc, RegisterSpan};

#[test]
#[cfg_attr(miri, ignore)]
fn no_params() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f)
            (func
                (return_call $f)
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::Return])
        .expect_func_instrs([Instruction::return_call_internal_0(CompiledFunc::from_u32(
            0,
        ))])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_param_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32) (result i32)
                (local.get 0)
            )
            (func (param i32) (result i32)
                (return_call $f (local.get 0))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(Register::from_i16(0))])
        .expect_func_instrs([
            Instruction::return_call_internal(CompiledFunc::from_u32(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(1), 1),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_param_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32) (result i32)
                (local.get 0)
            )
            (func (result i32)
                (return_call $f (i32.const 10))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(Register::from_i16(0))])
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::return_call_internal(CompiledFunc::from_u32(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(1), 1),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
            )
            (func (param i32 i32) (result i32 i32)
                (return_call $f (local.get 0) (local.get 1))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_many(
            RegisterSpan::new(Register::from_i16(0)).iter(2),
        )])
        .expect_func_instrs([
            Instruction::return_call_internal(CompiledFunc::from_u32(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(2), 2),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_reg_rev() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
            )
            (func (param i32 i32) (result i32 i32)
                (return_call $f (local.get 1) (local.get 0))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_many(
            RegisterSpan::new(Register::from_i16(0)).iter(2),
        )])
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(2), Register::from(1)),
            Instruction::copy(Register::from_i16(3), Register::from(0)),
            Instruction::return_call_internal(CompiledFunc::from_u32(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(2)).iter(2), 2),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
            )
            (func (result i32 i32)
                (return_call $f (i32.const 10) (i32.const 20))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_many(
            RegisterSpan::new(Register::from_i16(0)).iter(2),
        )])
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy_imm32(Register::from_i16(1), 20_i32),
            Instruction::return_call_internal(CompiledFunc::from_u32(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(2), 2),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
            )
            (func (param i32 i32 i32) (result i32 i32 i32)
                (return_call $f (local.get 0) (local.get 1) (local.get 2))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_many(
            RegisterSpan::new(Register::from_i16(0)).iter(3),
        )])
        .expect_func_instrs([
            Instruction::return_call_internal(CompiledFunc::from_u32(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(3), 3),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_reg_rev() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
            )
            (func (param i32 i32 i32) (result i32 i32 i32)
                (return_call $f (local.get 2) (local.get 1) (local.get 0))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_many(
            RegisterSpan::new(Register::from_i16(0)).iter(3),
        )])
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(3), Register::from(2)),
            Instruction::copy(Register::from_i16(4), Register::from(1)),
            Instruction::copy(Register::from_i16(5), Register::from(0)),
            Instruction::return_call_internal(CompiledFunc::from_u32(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(3)).iter(3), 3),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
            )
            (func (result i32 i32 i32)
                (return_call $f (i32.const 10) (i32.const 20) (i32.const 30))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_many(
            RegisterSpan::new(Register::from_i16(0)).iter(3),
        )])
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy_imm32(Register::from_i16(1), 20_i32),
            Instruction::copy_imm32(Register::from_i16(2), 30_i32),
            Instruction::return_call_internal(CompiledFunc::from_u32(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(0)).iter(3), 3),
        ])
        .run();
}
