use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F32, "add");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_local_reg(WASM_OP, Instruction::f32_add)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_local_imm32(WASM_OP, 1.0_f32, Instruction::f32_add)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_local_imm32_lhs_commutative(WASM_OP, 1.0_f32, Instruction::f32_add)
}

#[test]
#[cfg_attr(miri, ignore)]
fn loc_nan() {
    test_binary_local_imm_with(WASM_OP, f32::NAN, [Instruction::return_imm32(f32::NAN)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nan_reg() {
    test_binary_local_imm_lhs_with(WASM_OP, f32::NAN, [Instruction::return_imm32(f32::NAN)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero() {
    // We cannot optimize `x + 0` -> `x` because `-0 + 0` -> `0` according to IEEE.
    let expected = [
        Instruction::f32_add(Local::from(1), Local::from(0), Local::from(-1)),
        Instruction::return_reg(1),
    ];
    testcase_binary_local_imm(WASM_OP, 0.0_f32)
        .expect_func(ExpectedFunc::new(expected).consts([0.0_f32]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero_lhs() {
    // We cannot optimize `0 + x` -> `x` because `0 + -0` -> `0` according to IEEE.
    let expected = [
        Instruction::f32_add(Local::from(1), Local::from(0), Local::from(-1)),
        Instruction::return_reg(1),
    ];
    testcase_binary_imm_reg(WASM_OP, 0.0_f32)
        .expect_func(ExpectedFunc::new(expected).consts([0.0_f32]))
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 1.0_f32;
    let rhs = 2.0;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(lhs + rhs),
        }],
    )
}
