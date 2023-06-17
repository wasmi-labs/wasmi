use super::*;

const WASM_OP: WasmOp = WasmOp::F64("div");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_div)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, 1.0_f32, Instruction::f64_div_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, 1.0_f32, Instruction::f64_div_imm_rev)
}

#[test]
fn reg_nan() {
    // Note: Unfortunately we cannot use convenience functions
    //       for test case since NaN `Display` implementation
    //       differs from what the `wat2wasm` parser expects.
    let ty = WASM_OP.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {ty}) (result {ty})
                local.get 0
                {ty}.const nan
                {WASM_OP}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }])
        .expect_const(ConstRef::from_u32(0), f64::NAN)
        .run();
}

#[test]
fn nan_reg() {
    // Note: Unfortunately we cannot use convenience functions
    //       for test case since NaN `Display` implementation
    //       differs from what the `wat2wasm` parser expects.
    let ty = WASM_OP.ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {ty}) (result {ty})
                {ty}.const nan
                local.get 0
                {WASM_OP}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }])
        .expect_const(ConstRef::from_u32(0), f64::NAN)
        .run();
}

#[test]
fn consteval() {
    let lhs = 13.0_f64;
    let rhs = 5.5;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }],
    )
}
