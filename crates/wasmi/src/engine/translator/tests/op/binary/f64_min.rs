use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F64, "min");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_local_reg(WASM_OP, Instruction::f64_min)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_local_imm32(WASM_OP, 1.0_f64, Instruction::f64_min)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_local_imm32_lhs_commutative(WASM_OP, 1.0_f64, Instruction::f64_min)
}

#[test]
#[cfg_attr(miri, ignore)]
fn loc_nan() {
    testcase_binary_local_imm(WASM_OP, f64::NAN)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(f64::NAN)]))
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn nan_reg() {
    testcase_binary_imm_reg(WASM_OP, f64::NAN)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(f64::NAN)]))
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_pos_infinity() {
    let expected = [Instruction::return_reg(0)];
    test_binary_local_imm_with(WASM_OP, f64::INFINITY, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_pos_infinity_lhs() {
    let expected = [Instruction::return_reg(0)];
    test_binary_local_imm_lhs_with(WASM_OP, f64::INFINITY, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 1.0_f64;
    let rhs = 2.0;
    let result = if rhs < lhs { rhs } else { lhs };
    testcase_binary_consteval(WASM_OP, lhs, rhs)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(result)]))
        .run();
}
