use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F32, "mul");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_local_reg(WASM_OP, Instruction::f32_mul)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_local_imm32(WASM_OP, 1.0_f32, Instruction::f32_mul)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_local_imm32_lhs_commutative(WASM_OP, 1.0_f32, Instruction::f32_mul)
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
fn consteval() {
    let lhs = 5.0_f32;
    let rhs = 13.0;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(lhs * rhs),
        }],
    )
}
