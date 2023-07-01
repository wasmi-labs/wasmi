use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F32, "ne");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(false),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_ne)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_ne_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_ne_imm)
}

#[test]
fn reg_nan() {
    test_reg_nan(WASM_OP, [Instruction::return_imm32(true)]);
}

#[test]
fn nan_reg() {
    test_nan_reg(WASM_OP, [Instruction::return_imm32(true)]);
}

#[test]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: Const32::from(false),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        0.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: Const32::from(true),
        }],
    );
}
