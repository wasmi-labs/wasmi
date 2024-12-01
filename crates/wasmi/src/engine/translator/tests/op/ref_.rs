use super::*;

#[test]
#[cfg_attr(miri, ignore)]
fn const_prop() {
    let wasm = r"
        (module
            (func (result i32)
                ref.null func
                ref.is_null
            )
        )
    ";
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_imm32(1_i32)])
        .run()
}
