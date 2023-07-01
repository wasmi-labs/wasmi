use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F32, "mul");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_mul)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_mul_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_mul_imm)
}

#[test]
fn reg_nan() {
    test_reg_nan(WASM_OP, [Instruction::return_imm32(f32::NAN)]);
}

#[test]
fn nan_reg() {
    test_nan_reg(WASM_OP, [Instruction::return_imm32(f32::NAN)]);
}

#[test]
fn consteval() {
    let lhs = 5.0_f32;
    let rhs = 13.0;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from(lhs * rhs),
        }],
    )
}
