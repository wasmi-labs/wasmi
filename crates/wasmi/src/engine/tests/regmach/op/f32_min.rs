use super::*;

const WASM_OP: WasmOp = WasmOp::F32("min");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_min)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_min_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_min_imm)
}

#[test]
fn reg_nan() {
    // Note: Unfortunately we cannot use convenience functions
    //       for test case since f32 NaN `Display` implementation
    //       differs from what the `wat2wasm` parser expects.
    let wasm = wat2wasm(
        r#"
        (module
            (func (param f32) (result f32)
                local.get 0
                f32.const nan
                f32.add
            )
        )
    "#,
    );
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
    let wasm = wat2wasm(
        r#"
        (module
            (func (param f32) (result f32)
                f32.const nan
                local.get 0
                f32.add
            )
        )
    "#,
    );
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(f32::NAN),
    }];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn reg_pos_infinity() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with(WASM_OP, f32::INFINITY, expected)
}

#[test]
fn reg_pos_infinity_rev() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, f32::INFINITY, expected)
}

#[test]
fn consteval() {
    let lhs = 1.0_f32;
    let rhs = 2.0;
    let result = if rhs < lhs { rhs } else { lhs };
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from(result),
        }],
    )
}
