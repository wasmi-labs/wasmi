use super::*;
use crate::ValType;

fn test_reg(ty: ValType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $index i32) (param $value {display_ty})
                (local.get $index)
                (local.get $value)
                (table.set $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_set(Reg::from(0), Reg::from(1)),
            Instruction::table_index(0),
            Instruction::Return,
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_reg(ValType::FuncRef);
    test_reg(ValType::ExternRef);
}

fn test_reg_at(index: u32, value_type: ValType) {
    let display_ty = DisplayValueType::from(value_type);
    let display_index = DisplayWasm::from(index);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty})
                (i32.const {display_index})
                (local.get $value)
                (table.set $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_set_at(Reg::from(0), index),
            Instruction::table_index(0),
            Instruction::Return,
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_at() {
    fn test_for(index: u32) {
        test_reg_at(index, ValType::FuncRef);
        test_reg_at(index, ValType::ExternRef);
    }
    test_for(0);
    test_for(u32::MAX);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_funcref() {
    let wasm = r"
        (module
            (table $t 10 funcref)
            (elem declare func $f)
            (func $f (param $index i32)
                (table.set $t (local.get $index) (ref.func $f))
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::ref_func(Reg::from(1), 0),
            Instruction::table_set(Reg::from(0), Reg::from(1)),
            Instruction::table_index(0),
            Instruction::Return,
        ])
        .run();
}

fn test_at_imm_funcref(index: u32) {
    let wasm = format!(
        r"
        (module
            (table $t 10 funcref)
            (elem declare func $f)
            (func $f
                (table.set $t (i32.const {index}) (ref.func $f))
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::ref_func(Reg::from(0), 0),
            Instruction::table_set_at(Reg::from(0), index),
            Instruction::table_index(0),
            Instruction::Return,
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_funcref() {
    test_at_imm_funcref(0);
    test_at_imm_funcref(u32::MAX);
}

fn test_imm_null(value_type: ValType) {
    let display_ty = DisplayValueType::from(value_type);
    let ref_id = match value_type {
        ValType::FuncRef => "func",
        ValType::ExternRef => "extern",
        _ => panic!("invalid Wasm reftype"),
    };
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func $f (param $index i32)
                (table.set $t (local.get $index) (ref.null {ref_id}))
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_set(Reg::from(0), Reg::from(-1)),
                Instruction::table_index(0),
                Instruction::Return,
            ])
            .consts([0]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_null() {
    test_imm_null(ValType::FuncRef);
    test_imm_null(ValType::ExternRef);
}

fn test_at_imm_null(index: u32, value_type: ValType) {
    let display_ty = DisplayValueType::from(value_type);
    let ref_id = match value_type {
        ValType::FuncRef => "func",
        ValType::ExternRef => "extern",
        _ => panic!("invalid Wasm reftype"),
    };
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func $f
                (table.set $t (i32.const {index}) (ref.null {ref_id}))
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_set_at(Reg::from(-1), index),
                Instruction::table_index(0),
                Instruction::Return,
            ])
            .consts([0]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_null() {
    fn test_for(index: u32) {
        test_at_imm_null(index, ValType::FuncRef);
        test_at_imm_null(index, ValType::ExternRef);
    }
    test_for(0);
    test_for(u32::MAX);
}
