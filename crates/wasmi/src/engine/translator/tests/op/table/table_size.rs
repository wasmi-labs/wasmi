use super::*;
use crate::ValType;

fn test_reg(ty: ValType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (result i32)
                (table.size $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_size(Reg::from(0), 0),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_reg(ValType::FuncRef);
    test_reg(ValType::ExternRef);
}
