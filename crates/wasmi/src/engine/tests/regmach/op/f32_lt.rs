use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F32, "lt");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(false),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_lt)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_lt_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_gt_imm)
}

#[test]
fn reg_neg_inf() {
    test_binary_reg_imm_with(
        WASM_OP,
        f32::NEG_INFINITY,
        [Instruction::ReturnImm32 {
            value: Const32::from(false),
        }],
    )
}

#[test]
fn pos_inf_reg() {
    test_binary_reg_imm_rev_with(
        WASM_OP,
        f32::INFINITY,
        [Instruction::ReturnImm32 {
            value: Const32::from(false),
        }],
    )
}

#[test]
fn reg_nan() {
    test_reg_nan(WASM_OP, [Instruction::return_imm32(false)]);
}

#[test]
fn nan_reg() {
    test_nan_reg(WASM_OP, [Instruction::return_imm32(false)]);
}

#[test]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1.0,
        2.0,
        [Instruction::ReturnImm32 {
            value: Const32::from(true),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        2.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: Const32::from(false),
        }],
    );
}
