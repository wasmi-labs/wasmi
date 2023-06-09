use super::*;

#[test]
fn i32_mul() {
    test_binary_reg_reg("mul", Instruction::i32_mul)
}

#[test]
fn i32_mul_imm16() {
    test_binary_reg_imm16("mul", Instruction::i32_mul_imm16)
}

#[test]
fn i32_mul_imm16_rev() {
    test_binary_reg_imm16_rev("mul", Instruction::i32_mul_imm16)
}

#[test]
fn i32_mul_imm() {
    test_binary_reg_imm("mul", Instruction::i32_mul_imm)
}

#[test]
fn i32_mul_imm_rev() {
    test_binary_reg_imm_rev("mul", Instruction::i32_mul_imm)
}

#[test]
fn i32_mul_zero() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_u32(0),
    }];
    test_binary_reg_imm_with("mul", 0_i32, expected)
}

#[test]
fn i32_mul_zero_rev() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_u32(0),
    }];
    test_binary_reg_imm_rev_with("mul", 0_i32, expected)
}

#[test]
fn i32_mul_one() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with("mul", 1_i32, expected)
}

#[test]
fn i32_mul_one_rev() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_rev_with("mul", 1_i32, expected)
}

#[test]
fn i32_mul_consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(
        "mul",
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from_i32(lhs * rhs),
        }],
    )
}
