use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I32, "ge_s");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(true),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_ge_s)
}

#[test]
fn reg_imm16() {
    test_binary_reg_imm16(WASM_OP, Instruction::i32_ge_s_imm16)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, swap_ops!(Instruction::i32_le_s_imm16))
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 100_000, Instruction::i32_ge_s_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 100_000, Instruction::i32_le_s_imm)
}

#[test]
fn reg_min() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(true),
    }];
    test_binary_reg_imm_with(WASM_OP, i32::MIN, expected)
}

#[test]
fn max_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(true),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, i32::MAX, expected)
}

#[test]
fn consteval() {
    let lhs = 1_i32;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from(lhs >= rhs),
        }],
    )
}
