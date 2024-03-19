use super::*;
use crate::core::ValueType;

fn test_reg(ty: ValueType) {
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
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            Instruction::table_size(Register::from_i16(0), 0),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_reg(ValueType::FuncRef);
    test_reg(ValueType::ExternRef);
}
