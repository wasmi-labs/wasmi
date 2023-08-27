use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F64, "add");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_add)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f64, Instruction::f64_add)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev_commutative(WASM_OP, 1.0_f64, Instruction::f64_add)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_nan() {
    testcase_binary_reg_imm(WASM_OP, f64::NAN)
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
fn reg_zero() {
    // We cannot optimize `x + 0` -> `x` because `-0 + 0` -> `0` according to IEEE.
    let expected = [
        Instruction::f64_add(
            Register::from_i16(1),
            Register::from_i16(0),
            Register::from_i16(-1),
        ),
        Instruction::return_reg(1),
    ];
    testcase_binary_reg_imm(WASM_OP, 0.0_f64)
        .expect_func(ExpectedFunc::new(expected).consts([0.0_f64]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero_rev() {
    // We cannot optimize `0 + x` -> `x` because `0 + -0` -> `0` according to IEEE.
    let expected = [
        Instruction::f64_add(
            Register::from_i16(1),
            Register::from_i16(0),
            Register::from_i16(-1),
        ),
        Instruction::return_reg(1),
    ];
    testcase_binary_imm_reg(WASM_OP, 0.0_f64)
        .expect_func(ExpectedFunc::new(expected).consts([0.0_f64]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 1.0_f64;
    let rhs = 2.0;
    let result = lhs + rhs;
    testcase_binary_consteval(WASM_OP, lhs, rhs)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(result)]))
        .run();
}
