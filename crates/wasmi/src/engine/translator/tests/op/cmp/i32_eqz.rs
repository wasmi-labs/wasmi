use super::*;

const PARAM: WasmType = WasmType::I32;

#[test] #[cfg_attr(miri, ignore)]
fn reg() {
    let wasm = format!(
        r#"
        (module
            (func (param {PARAM}) (result i32)
                local.get 0
                {PARAM}.eqz
            )
        )
        "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            Instruction::i32_eq_imm16(
                Reg::from(1),
                Reg::from(0),
                0,
            ),
            Instruction::return_reg(1),
        ])
        .run();
}

fn imm_with(value: i32) {
    let wasm = format!(
        r#"
        (module
            (func (result i32)
                {PARAM}.const {value}
                {PARAM}.eqz
            )
        )
        "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([Instruction::return_imm32((value == 0) as u32)])
        .run();
}

#[test] #[cfg_attr(miri, ignore)]
fn imm() {
    imm_with(0);
    imm_with(1);
}
