use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F64, "add");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_add)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, 1.0_f64, Instruction::f64_add_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, 1.0_f64, Instruction::f64_add_imm)
}

#[test]
fn reg_nan() {
    test_reg_nan_ext(WASM_OP, [Instruction::return_cref(0)])
        .expect_const(ConstRef::from_u32(0), f64::NAN)
        .run();
}

#[test]
fn nan_reg() {
    test_nan_reg_ext(WASM_OP, [Instruction::return_cref(0)])
        .expect_const(ConstRef::from_u32(0), f64::NAN)
        .run();
}

#[test]
fn reg_zero() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with(WASM_OP, 0.0_f64, expected)
}

#[test]
fn reg_zero_rev() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, 0.0_f64, expected)
}

#[test]
fn consteval() {
    let lhs = 1.0_f64;
    let rhs = 2.0;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }],
    )
}
