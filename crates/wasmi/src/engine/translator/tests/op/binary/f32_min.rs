use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F32, "min");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_local_reg(WASM_OP, Instruction::f32_min)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_local_imm32(WASM_OP, 1.0_f32, Instruction::f32_min)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_local_imm32_lhs_commutative(WASM_OP, 1.0_f32, Instruction::f32_min)
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
fn reg_pos_infinity() {
    let expected = [Instruction::return_reg(0)];
    test_binary_local_imm_with(WASM_OP, f32::INFINITY, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_pos_infinity_lhs() {
    let expected = [Instruction::return_reg(0)];
    test_binary_local_imm_lhs_with(WASM_OP, f32::INFINITY, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 1.0_f32;
    let rhs = 2.0;
    let result = if rhs < lhs { rhs } else { lhs };
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(result),
        }],
    )
}
