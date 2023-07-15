use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F64, "add");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_add)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f64, Instruction::f64_add)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev_commutative(WASM_OP, 1.0_f64, Instruction::f64_add)
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
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0.0_f64, expected).run()
}

#[test]
fn reg_zero_rev() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_rev_with(WASM_OP, 0.0_f64, expected).run()
}

#[test]
fn consteval() {
    let lhs = 1.0_f64;
    let rhs = 2.0;
    testcase_binary_consteval(WASM_OP, lhs, rhs)
        .expect_func(
            ExpectedFunc::new([Instruction::return_reg(Register::from_i16(-1))])
                .consts([lhs + rhs]),
        )
        .run();
}
