use super::*;
use crate::engine::{CompiledFunc, RegisterSpan};

#[test]
fn no_params() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f)
            (func
                (call $f)
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::Return])
        .expect_func_instrs([
            Instruction::call_internal_0(
                RegisterSpan::new(Register::from_i16(0)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::Return,
        ])
        .run();
}

#[test]
fn one_param_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32) (result i32)
                (local.get 0)
            )
            (func (param i32) (result i32)
                (call $f (local.get 0))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(Register::from_i16(0))])
        .expect_func_instrs([
            Instruction::call_internal(
                RegisterSpan::new(Register::from_i16(1)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::Register(Register::from_i16(0)),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

#[test]
fn one_param_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32) (result i32)
                (local.get 0)
            )
            (func (result i32)
                (call $f (i32.const 0))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(Register::from_i16(0))])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(
                    RegisterSpan::new(Register::from_i16(0)),
                    CompiledFunc::from_u32(0),
                ),
                Instruction::Register(Register::from_i16(-1)),
                Instruction::return_reg(Register::from_i16(0)),
            ])
            .consts([0]),
        )
        .run();
}

#[test]
fn two_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
            )
            (func (param i32 i32) (result i32 i32)
                (call $f (local.get 0) (local.get 1))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg_2(
            Register::from_i16(0),
            Register::from_i16(1),
        )])
        .expect_func_instrs([
            Instruction::call_internal(
                RegisterSpan::new(Register::from_i16(2)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::register_2(Register::from_i16(0), Register::from_i16(1)),
            Instruction::return_reg_2(Register::from_i16(2), Register::from_i16(3)),
        ])
        .run();
}

#[test]
fn two_params_imm() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
            )
            (func (result i32 i32)
                (call $f (i32.const 10) (i32.const 20))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg_2(
            Register::from_i16(0),
            Register::from_i16(1),
        )])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(
                    RegisterSpan::new(Register::from_i16(0)),
                    CompiledFunc::from_u32(0),
                ),
                Instruction::register_2(Register::from_i16(-1), Register::from_i16(-2)),
                Instruction::return_reg_2(Register::from_i16(0), Register::from_i16(1)),
            ])
            .consts([10, 20]),
        )
        .run();
}

#[test]
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
                (call $f (local.get 0) (local.get 1) (local.get 2))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg_3(
            Register::from_i16(0),
            Register::from_i16(1),
            Register::from_i16(2),
        )])
        .expect_func_instrs([
            Instruction::call_internal(
                RegisterSpan::new(Register::from_i16(3)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::register_3(
                Register::from_i16(0),
                Register::from_i16(1),
                Register::from_i16(2),
            ),
            Instruction::return_reg_3(
                Register::from_i16(3),
                Register::from_i16(4),
                Register::from_i16(5),
            ),
        ])
        .run();
}

#[test]
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
                (call $f (i32.const 10) (i32.const 20) (i32.const 30))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg_3(
            Register::from_i16(0),
            Register::from_i16(1),
            Register::from_i16(2),
        )])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(
                    RegisterSpan::new(Register::from_i16(0)),
                    CompiledFunc::from_u32(0),
                ),
                Instruction::register_3(
                    Register::from_i16(-1),
                    Register::from_i16(-2),
                    Register::from_i16(-3),
                ),
                Instruction::return_reg_3(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    Register::from_i16(2),
                ),
            ])
            .consts([10, 20, 30]),
        )
        .run();
}
