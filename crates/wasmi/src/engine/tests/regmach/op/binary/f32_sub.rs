use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F32, "sub");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_sub)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_sub_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_sub_imm_rev)
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
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0.0_f32, expected)
}

#[test]
fn consteval() {
    let lhs = 13.0_f32;
    let rhs = 5.5;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from(lhs - rhs),
        }],
    )
}
