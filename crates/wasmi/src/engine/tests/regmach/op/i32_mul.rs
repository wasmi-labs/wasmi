use super::*;

const WASM_OP: &str = "mul";

#[test]
fn i32_mul() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_mul)
}

#[test]
fn i32_mul_imm16() {
    test_binary_reg_imm16(WASM_OP, Instruction::i32_mul_imm16)
}

#[test]
fn i32_mul_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, Instruction::i32_mul_imm16)
}

#[test]
fn i32_mul_imm() {
    test_binary_reg_imm(WASM_OP, Instruction::i32_mul_imm)
}

#[test]
fn i32_mul_imm_rev() {
    test_binary_reg_imm_rev(WASM_OP, Instruction::i32_mul_imm)
}

#[test]
fn i32_mul_zero() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_u32(0),
    }];
    test_binary_reg_imm_with(WASM_OP, 0_i32, expected)
}

#[test]
fn i32_mul_zero_rev() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_u32(0),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, 0_i32, expected)
}

#[test]
fn i32_mul_one() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with(WASM_OP, 1_i32, expected)
}

#[test]
fn i32_mul_one_rev() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, 1_i32, expected)
}

#[test]
fn i32_mul_consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from_i32(lhs * rhs),
        }],
    )
}
