use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F32, "add");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_add)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_add_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_add_imm)
}

#[test]
fn reg_nan() {
    test_binary_reg_imm_with(WASM_OP, f32::NAN, [Instruction::return_imm32(f32::NAN)]).run()
}

#[test]
fn nan_reg() {
    test_binary_reg_imm_rev_with(WASM_OP, f32::NAN, [Instruction::return_imm32(f32::NAN)]).run()
}

#[test]
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0.0_f32, expected).run()
}

#[test]
fn reg_zero_rev() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_rev_with(WASM_OP, 0.0_f32, expected).run()
}

#[test]
fn consteval() {
    let lhs = 1.0_f32;
    let rhs = 2.0;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from(lhs + rhs),
        }],
    )
}
