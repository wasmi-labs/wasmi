use super::*;
use crate::{
    core::ValueType,
    engine::tests::regmach::{display_wasm::DisplayValueType, wasm_type::WasmType},
    ExternRef,
    FuncRef,
};

fn test_reg(ty: ValueType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $index i32) (result {display_ty})
                (local.get $index)
                (table.get $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_get(Register::from_i16(1), Register::from_i16(0)),
            Instruction::table_idx(0),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

#[test]
fn reg() {
    test_reg(ValueType::FuncRef);
    test_reg(ValueType::ExternRef);
}

fn test_imm(ty: ValueType, index: u32) {
    let display_ty = DisplayValueType::from(ty);
    let display_index = DisplayWasm::from(index);
    let wasm = wat2wasm(&format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (result {display_ty})
                (i32.const {display_index})
                (table.get $t)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_get_imm(Register::from_i16(0), index),
            Instruction::table_idx(0),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run();
}

#[test]
fn imm() {
    fn test_for(index: u32) {
        test_imm(ValueType::FuncRef, index);
        test_imm(ValueType::ExternRef, index);
    }
    test_for(0);
    test_for(1);
    test_for(u32::MAX);
}
