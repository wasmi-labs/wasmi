use super::*;
use crate::core::ValueType;

fn test_reg(ty: ValueType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $index i32) (result {display_ty})
                (local.get $index)
                (table.get $t)
            )
        )",
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            Instruction::table_get(Register::from_i16(1), Register::from_i16(0)),
            Instruction::table_idx(0),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_reg(ValueType::FuncRef);
    test_reg(ValueType::ExternRef);
}

fn test_imm(ty: ValueType, index: u32) {
    let display_ty = DisplayValueType::from(ty);
    let display_index = DisplayWasm::from(index);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (result {display_ty})
                (i32.const {display_index})
                (table.get $t)
            )
        )",
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            Instruction::table_get_imm(Register::from_i16(0), index),
            Instruction::table_idx(0),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    fn test_for(index: u32) {
        test_imm(ValueType::FuncRef, index);
        test_imm(ValueType::ExternRef, index);
    }
    test_for(0);
    test_for(1);
    test_for(u32::MAX);
}
