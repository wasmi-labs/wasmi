use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F64, "sub");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_sub)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f64, Instruction::f64_sub)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f64, Instruction::f64_sub)
}

#[test]
fn reg_nan() {
    testcase_binary_reg_imm(WASM_OP, f64::NAN)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(f64::NAN)]))
        .run();
}

#[test]
fn nan_reg() {
    testcase_binary_imm_reg(WASM_OP, f64::NAN)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(f64::NAN)]))
        .run();
}

#[test]
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0.0_f64, expected).run()
}

#[test]
fn consteval() {
    let lhs = 13.0_f64;
    let rhs = 5.5;
    let result = lhs - rhs;
    testcase_binary_consteval(WASM_OP, lhs, rhs)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(result)]))
        .run();
}
