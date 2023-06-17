use super::*;

const WASM_OP: WasmOp = WasmOp::F64("min");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_min)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, 1.0_f64, Instruction::f64_min_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, 1.0_f64, Instruction::f64_min_imm)
}

#[test]
fn reg_nan() {
    // Note: Unfortunately we cannot use convenience functions
    //       for test case since f32 NaN `Display` implementation
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
    //       for test case since f32 NaN `Display` implementation
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
fn reg_pos_infinity() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with(WASM_OP, f64::INFINITY, expected)
}

#[test]
fn reg_pos_infinity_rev() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, f64::INFINITY, expected)
}

#[test]
fn consteval() {
    let lhs = 1.0_f64;
    let rhs = 2.0;
    // let result = if rhs < lhs { rhs } else { lhs };
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }],
    )
}
