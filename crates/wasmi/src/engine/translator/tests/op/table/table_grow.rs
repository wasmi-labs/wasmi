use super::*;
use crate::ValType;

fn test_reg(ty: ValType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (param $index i32) (result i32)
                (local.get $value)
                (local.get $index)
                (table.grow $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_grow(Reg::from(2), Reg::from(1), Reg::from(0)),
            Instruction::table_index(0),
            Instruction::return_reg(Reg::from(2)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_reg(ValType::FuncRef);
    test_reg(ValType::ExternRef);
}

fn test_imm16(ty: ValType, delta: u64) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (result i32)
                (local.get $value)
                (i32.const {delta})
                (table.grow $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_grow_imm(Reg::from(1), u64imm16(delta), Reg::from(0)),
            Instruction::table_index(0),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm16() {
    fn test_for(delta: u64) {
        test_imm16(ValType::FuncRef, delta);
        test_imm16(ValType::ExternRef, delta);
    }
    test_for(1);
    test_for(42);
    test_for(u64::from(u16::MAX) - 1);
    test_for(u64::from(u16::MAX));
}

fn test_imm_zero(ty: ValType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (result i32)
                (local.get $value)
                (i32.const 0)
                (table.grow $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_size(Reg::from(1), 0),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_zero() {
    test_imm_zero(ValType::FuncRef);
    test_imm_zero(ValType::ExternRef);
}

fn test_imm_value_and_zero(ty: ValType) {
    let display_ty: DisplayValueType = DisplayValueType::from(ty);
    let ref_ty = match ty {
        ValType::FuncRef => "func",
        ValType::ExternRef => "extern",
        _ => panic!("invalid Wasm reftype"),
    };
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (result i32)
                (ref.null {ref_ty})
                (i32.const 0)
                (table.grow $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_size(Reg::from(1), 0),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_value_and_zero() {
    test_imm_value_and_zero(ValType::FuncRef);
    test_imm_value_and_zero(ValType::ExternRef);
}

fn test_imm(ty: ValType, delta: u32) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (result i32)
                (local.get $value)
                (i32.const {delta})
                (table.grow $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_grow(Reg::from(1), Reg::from(-1), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::return_reg(Reg::from(1)),
            ])
            .consts([delta]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    fn test_for(delta: u32) {
        test_imm(ValType::FuncRef, delta);
        test_imm(ValType::ExternRef, delta);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX - 1);
    test_for(u32::MAX);
}
