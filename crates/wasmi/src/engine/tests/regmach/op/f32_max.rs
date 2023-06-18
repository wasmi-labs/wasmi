use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F32, "max");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_max)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_max_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_max_imm)
}

#[test]
fn reg_nan() {
    // Note: Unfortunately we cannot use convenience functions
    //       for test case since f32 NaN `Display` implementation
    //       differs from what the `wat2wasm` parser expects.
    let ty = WASM_OP.result_ty();
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
    //       for test case since f32 NaN `Display` implementation
    //       differs from what the `wat2wasm` parser expects.
    let ty = WASM_OP.result_ty();
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
fn reg_neg_infinity() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with(WASM_OP, f32::NEG_INFINITY, expected)
}

#[test]
fn reg_neg_infinity_rev() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, f32::NEG_INFINITY, expected)
}

#[test]
fn consteval() {
    let lhs = 1.0_f32;
    let rhs = 2.0;
    let result = if rhs > lhs { rhs } else { lhs };
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from(result),
        }],
    )
}
