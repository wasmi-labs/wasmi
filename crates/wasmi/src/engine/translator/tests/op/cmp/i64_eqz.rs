use super::*;

const PARAM: WasmType = WasmType::I64;

#[test] #[cfg_attr(miri, ignore)]
fn reg() {
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {PARAM}) (result i32)
                local.get 0
                {PARAM}.eqz
            )
        )
        "#
    ));
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::i64_eq_imm16(
                Reg::from_u16(1),
                Reg::from_u16(0),
                Const16::from_i16(0),
            ),
            Instruction::return_reg(1),
        ])
        .run();
}

fn imm_with(value: i64) {
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (result i32)
                {PARAM}.const {value}
                {PARAM}.eqz
            )
        )
        "#
    ));
    TranslationTest::new(wasm)
        .expect_func([Instruction::ReturnImm32 {
            value: Const32::from(value == 0),
        }])
        .run();
}

#[test] #[cfg_attr(miri, ignore)]
fn imm() {
    imm_with(0);
    imm_with(1);
}
