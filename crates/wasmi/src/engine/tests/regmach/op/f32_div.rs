use super::*;

const WASM_OP: WasmOp = WasmOp::F32("div");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_div)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_div_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_div_imm_rev)
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
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(f32::NAN),
    }];
    assert_func_bodies(wasm, [expected]);
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
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(f32::NAN),
    }];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn consteval() {
    let lhs = 13.0_f32;
    let rhs = 5.5;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from(lhs / rhs),
        }],
    )
}
