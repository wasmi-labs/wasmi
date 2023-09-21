use super::*;
use crate::{core::ValueType, engine::regmach::tests::display_wasm::DisplayValueType};

fn test_reg(ty: ValueType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (param $index i32) (result i32)
                (local.get $value)
                (local.get $index)
                (table.grow $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_grow(
                Register::from_i16(2),
                Register::from_i16(1),
                Register::from_i16(0),
            ),
            Instruction::table_idx(0),
            Instruction::return_reg(Register::from_i16(2)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_reg(ValueType::FuncRef);
    test_reg(ValueType::ExternRef);
}

fn test_imm16(ty: ValueType, delta: u32) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (result i32)
                (local.get $value)
                (i32.const {delta})
                (table.grow $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_grow_imm(
                Register::from_i16(1),
                u32imm16(delta),
                Register::from_i16(0),
            ),
            Instruction::table_idx(0),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm16() {
    fn test_for(delta: u32) {
        test_imm16(ValueType::FuncRef, delta);
        test_imm16(ValueType::ExternRef, delta);
    }
    test_for(1);
    test_for(42);
    test_for(u32::from(u16::MAX) - 1);
    test_for(u32::from(u16::MAX));
}

fn test_imm_zero(ty: ValueType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (result i32)
                (local.get $value)
                (i32.const 0)
                (table.grow $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_size(Register::from_i16(1), 0),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_zero() {
    test_imm_zero(ValueType::FuncRef);
    test_imm_zero(ValueType::ExternRef);
}

fn test_imm_value_and_zero(ty: ValueType) {
    let display_ty: DisplayValueType = DisplayValueType::from(ty);
    let ref_ty = match ty {
        ValueType::FuncRef => "func",
        ValueType::ExternRef => "extern",
        _ => panic!("invalid Wasm reftype"),
    };
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (result i32)
                (ref.null {ref_ty})
                (i32.const 0)
                (table.grow $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_size(Register::from_i16(1), 0),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_value_and_zero() {
    test_imm_value_and_zero(ValueType::FuncRef);
    test_imm_value_and_zero(ValueType::ExternRef);
}

fn test_imm(ty: ValueType, delta: u32) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (result i32)
                (local.get $value)
                (i32.const {delta})
                (table.grow $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_grow(
                    Register::from_i16(1),
                    Register::from_i16(-1),
                    Register::from_i16(0),
                ),
                Instruction::table_idx(0),
                Instruction::return_reg(Register::from_i16(1)),
            ])
            .consts([delta]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    fn test_for(delta: u32) {
        test_imm(ValueType::FuncRef, delta);
        test_imm(ValueType::ExternRef, delta);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX - 1);
    test_for(u32::MAX);
}
