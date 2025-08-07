use super::*;
use crate::ValType;

fn test_reg(ty: ValType) {
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
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_get(Reg::from(1), Reg::from(0)),
            Instruction::table_index(0),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_reg(ValType::FuncRef);
    test_reg(ValType::ExternRef);
}

fn test_imm(ty: ValType, index: u32) {
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
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_get_imm(Reg::from(0), index),
            Instruction::table_index(0),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    fn test_for(index: u32) {
        test_imm(ValType::FuncRef, index);
        test_imm(ValType::ExternRef, index);
    }
    test_for(0);
    test_for(1);
    test_for(u32::MAX);
}
