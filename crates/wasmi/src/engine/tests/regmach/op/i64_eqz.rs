use super::*;

const PARAM: WasmType = WasmType::I64;

#[test]
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
                Register::from_u16(1),
                Register::from_u16(0),
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

#[test]
fn imm() {
    imm_with(0);
    imm_with(1);
}
