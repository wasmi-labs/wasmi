use super::*;
use crate::{
    core::ValueType,
    engine::tests::regmach::{
        display_wasm::{DisplayValue, DisplayValueType},
        wasm_type::WasmType,
    },
    ExternRef,
    FuncRef,
    Value,
};

fn test_reg(ty: ValueType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $index i32) (param $value {display_ty})
                (local.get $index)
                (local.get $value)
                (table.set $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_set(Register::from_i16(0), Register::from_i16(1)),
            Instruction::table_idx(0),
            Instruction::Return,
        ])
        .run();
}

#[test]
fn reg() {
    test_reg(ValueType::FuncRef);
    test_reg(ValueType::ExternRef);
}

fn test_reg_at(index: u32, value_type: ValueType) {
    let display_ty = DisplayValueType::from(value_type);
    let display_index = DisplayWasm::from(index);
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty})
                (i32.const {display_index})
                (local.get $value)
                (table.set $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_set_at(index, Register::from_i16(0)),
            Instruction::table_idx(0),
            Instruction::Return,
        ])
        .run();
}

#[test]
fn reg_at() {
    fn test_for(index: u32) {
        test_reg_at(index, ValueType::FuncRef);
        test_reg_at(index, ValueType::ExternRef);
    }
    test_for(0);
    test_for(u32::MAX);
}

#[test]
fn imm_funcref() {
    let wasm = wat2wasm(
        r"
        (module
            (table $t 10 funcref)
            (elem declare func $f)
            (func $f (param $index i32)
                (table.set $t (local.get $index) (ref.func $f))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::ref_func(Register::from_i16(1), 0),
            Instruction::table_set(Register::from_i16(0), Register::from_i16(1)),
            Instruction::table_idx(0),
            Instruction::Return,
        ])
        .run();
}

fn test_at_imm_funcref(index: u32) {
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 funcref)
            (elem declare func $f)
            (func $f
                (table.set $t (i32.const {index}) (ref.func $f))
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::ref_func(Register::from_i16(0), 0),
            Instruction::table_set_at(index, Register::from_i16(0)),
            Instruction::table_idx(0),
            Instruction::Return,
        ])
        .run();
}

#[test]
fn at_imm_funcref() {
    test_at_imm_funcref(0);
    test_at_imm_funcref(u32::MAX);
}

fn test_imm_null(value_type: ValueType) {
    let display_ty = DisplayValueType::from(value_type);
    let ref_id = match value_type {
        ValueType::FuncRef => "func",
        ValueType::ExternRef => "extern",
        _ => panic!("invalid Wasm reftype"),
    };
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func $f (param $index i32)
                (table.set $t (local.get $index) (ref.null {ref_id}))
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_set(Register::from_i16(0), Register::from_i16(-1)),
                Instruction::table_idx(0),
                Instruction::Return,
            ])
            .consts([0]),
        )
        .run();
}

#[test]
fn imm_null() {
    test_imm_null(ValueType::FuncRef);
    test_imm_null(ValueType::ExternRef);
}

fn test_at_imm_null(index: u32, value_type: ValueType) {
    let display_ty = DisplayValueType::from(value_type);
    let ref_id = match value_type {
        ValueType::FuncRef => "func",
        ValueType::ExternRef => "extern",
        _ => panic!("invalid Wasm reftype"),
    };
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func $f
                (table.set $t (i32.const {index}) (ref.null {ref_id}))
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_set_at(index, Register::from_i16(-1)),
                Instruction::table_idx(0),
                Instruction::Return,
            ])
            .consts([0]),
        )
        .run();
}

#[test]
fn at_imm_null() {
    fn test_for(index: u32) {
        test_at_imm_null(index, ValueType::FuncRef);
        test_at_imm_null(index, ValueType::ExternRef);
    }
    test_for(0);
    test_for(u32::MAX);
}
