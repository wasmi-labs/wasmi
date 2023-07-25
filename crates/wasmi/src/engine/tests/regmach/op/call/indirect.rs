use super::*;
use crate::engine::{
    bytecode::{SignatureIdx, TableIdx},
    CompiledFunc,
    RegisterSpan,
};

#[test]
fn no_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func))
            (func (param $index i32)
                (local.get $index)
                (call_indirect (type $type))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_indirect_0(
                RegisterSpan::new(Register::from_i16(1)),
                SignatureIdx::from(0),
            ),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::Return,
        ])
        .run();
}

#[test]
fn no_params_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func))
                (func (param $index i32)
                    (i32.const {index})
                    (call_indirect (type $type))
                )
            )
        "#,
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::call_indirect_0(
                    RegisterSpan::new(Register::from_i16(1)),
                    SignatureIdx::from(0),
                ),
                Instruction::call_indirect_params_imm16(u32imm16(index), TableIdx::from(0)),
                Instruction::Return,
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
fn one_reg_param_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32) (result i32)))
            (func (param $index i32) (param i32) (result i32)
                (local.get 1)
                (local.get $index)
                (call_indirect (type $type))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_indirect(
                RegisterSpan::new(Register::from_i16(2)),
                SignatureIdx::from(0),
            ),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(1)).iter(1), 1),
            Instruction::return_reg(Register::from_i16(2)),
        ])
        .run();
}

#[test]
fn one_reg_param_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32) (result i32)))
                (func (param $index i32) (param i32) (result i32)
                    (local.get 1)
                    (i32.const {index})
                    (call_indirect (type $type))
                )
            )
        "#,
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::call_indirect(
                    RegisterSpan::new(Register::from_i16(2)),
                    SignatureIdx::from(0),
                ),
                Instruction::call_indirect_params_imm16(u32imm16(index), TableIdx::from(0)),
                Instruction::call_params(RegisterSpan::new(Register::from_i16(1)).iter(1), 1),
                Instruction::return_reg(Register::from_i16(2)),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
fn one_imm_param_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32) (result i32)))
            (func (param $index i32) (result i32)
                (i32.const 10)
                (local.get $index)
                (call_indirect (type $type))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(1), 10),
            Instruction::call_indirect(
                RegisterSpan::new(Register::from_i16(1)),
                SignatureIdx::from(0),
            ),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(1)).iter(1), 1),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

#[test]
fn one_imm_param_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32) (result i32)))
                (func (param $index i32) (result i32)
                    (i32.const 10)
                    (i32.const {index})
                    (call_indirect (type $type))
                )
            )
        "#,
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::copy_imm32(Register::from_i16(1), 10),
                Instruction::call_indirect(
                    RegisterSpan::new(Register::from_i16(1)),
                    SignatureIdx::from(0),
                ),
                Instruction::call_indirect_params_imm16(u32imm16(index), TableIdx::from(0)),
                Instruction::call_params(RegisterSpan::new(Register::from_i16(1)).iter(1), 1),
                Instruction::return_reg(Register::from_i16(1)),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
fn two_reg_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32) (result i32 i32)))
            (func (param $index i32) (param i32 i32) (result i32 i32)
                (local.get 1)
                (local.get 2)
                (local.get $index)
                (call_indirect (type $type))
            )
        )
    "#,
    );
    let result_reg = Register::from_i16(3);
    let results = RegisterSpan::new(result_reg);
    let params = RegisterSpan::new(Register::from_i16(1)).iter(2);
    let elem_index = Register::from_i16(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_indirect(results, SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::call_params(params, 2),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(3)).iter(2)),
        ])
        .run();
}

#[test]
fn two_reg_params_reg_rev() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32) (result i32 i32)))
            (func (param $index i32) (param i32 i32) (result i32 i32)
                (local.get 2)
                (local.get 1)
                (local.get $index)
                (call_indirect (type $type))
            )
        )
    "#,
    );
    let result_reg = Register::from_i16(3);
    let results = RegisterSpan::new(result_reg);
    let params = RegisterSpan::new(Register::from_i16(3)).iter(2);
    let elem_index = Register::from_i16(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(3), Register::from_i16(2)),
            Instruction::copy(Register::from_i16(4), Register::from_i16(1)),
            Instruction::call_indirect(results, SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::call_params(params, 2),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(3)).iter(2)),
        ])
        .run();
}

#[test]
fn two_imm_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32) (result i32 i32)))
            (func (param $index i32) (result i32 i32)
                (i32.const 10)
                (i32.const 20)
                (local.get $index)
                (call_indirect (type $type))
            )
        )
    "#,
    );
    let result_reg = Register::from_i16(1);
    let results = RegisterSpan::new(result_reg);
    let params = RegisterSpan::new(Register::from_i16(1)).iter(2);
    let elem_index = Register::from_i16(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(1), 10),
            Instruction::copy_imm32(Register::from_i16(2), 20),
            Instruction::call_indirect(results, SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::call_params(params, 2),
            Instruction::return_many(RegisterSpan::new(Register::from_i16(1)).iter(2)),
        ])
        .run();
}

#[test]
fn two_reg_params_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32 i32) (result i32 i32)))
                (func (param i32 i32) (result i32 i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.const {index})
                    (call_indirect (type $type))
                )
            )
        "#,
        ));
        let result_reg = Register::from_i16(2);
        let results = RegisterSpan::new(result_reg);
        let params = RegisterSpan::new(Register::from_i16(0)).iter(2);
        let elem_index = u32imm16(index);
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::call_indirect(results, SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                Instruction::call_params(params, 2),
                Instruction::return_many(RegisterSpan::new(Register::from_i16(2)).iter(2)),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
fn two_reg_params_rev_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32 i32) (result i32 i32)))
                (func (param i32 i32) (result i32 i32)
                    (local.get 1)
                    (local.get 0)
                    (i32.const {index})
                    (call_indirect (type $type))
                )
            )
        "#,
        ));
        let result_reg = Register::from_i16(2);
        let results = RegisterSpan::new(result_reg);
        let params = RegisterSpan::new(Register::from_i16(2)).iter(2);
        let elem_index = u32imm16(index);
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::copy(Register::from_i16(2), Register::from_i16(1)),
                Instruction::copy(Register::from_i16(3), Register::from_i16(0)),
                Instruction::call_indirect(results, SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                Instruction::call_params(params, 2),
                Instruction::return_many(RegisterSpan::new(Register::from_i16(2)).iter(2)),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
fn two_imm_params_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32 i32) (result i32 i32)))
                (func (result i32 i32)
                    (i32.const 10)
                    (i32.const 20)
                    (i32.const {index})
                    (call_indirect (type $type))
                )
            )
        "#,
        ));
        let result_reg = Register::from_i16(0);
        let results = RegisterSpan::new(result_reg);
        let params = RegisterSpan::new(Register::from_i16(0)).iter(2);
        let elem_index = u32imm16(index);
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::copy_imm32(Register::from_i16(0), 10),
                Instruction::copy_imm32(Register::from_i16(1), 20),
                Instruction::call_indirect(results, SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                Instruction::call_params(params, 2),
                Instruction::return_many(RegisterSpan::new(Register::from_i16(0)).iter(2)),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}
