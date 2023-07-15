use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F64, "max");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_max)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f64, Instruction::f64_max)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev_commutative(WASM_OP, 1.0_f64, Instruction::f64_max)
}

#[test]
fn reg_nan() {
    testcase_binary_reg_imm(WASM_OP, f64::NAN)
        .expect_func(
            ExpectedFunc::new([Instruction::return_reg(Register::from_i16(-1))]).consts([f64::NAN]),
        )
        .run();
}

#[test]
fn nan_reg() {
    testcase_binary_imm_reg(WASM_OP, f64::NAN)
        .expect_func(
            ExpectedFunc::new([Instruction::return_reg(Register::from_i16(-1))]).consts([f64::NAN]),
        )
        .run();
}

#[test]
fn reg_neg_infinity() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, f64::NEG_INFINITY, expected).run()
}

#[test]
fn reg_neg_infinity_rev() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_rev_with(WASM_OP, f64::NEG_INFINITY, expected).run()
}

#[test]
fn consteval() {
    let lhs = 1.0_f64;
    let rhs = 2.0;
    let result = if rhs > lhs { rhs } else { lhs };
    testcase_binary_consteval(WASM_OP, lhs, rhs)
        .expect_func(
            ExpectedFunc::new([Instruction::return_reg(Register::from_i16(-1))]).consts([result]),
        )
        .run();
}
