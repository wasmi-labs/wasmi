use super::*;

#[test]
fn reg_reg() {
    test_binary_reg_reg("add", Instruction::i32_add)
}

#[test]
fn reg_imm16() {
    test_binary_reg_imm16("add", Instruction::i32_add_imm16)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev("add", Instruction::i32_add_imm16)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm("add", Instruction::i32_add_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm_rev("add", Instruction::i32_add_imm)
}

#[test]
fn reg_zero() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with("add", 0i32, expected)
}

#[test]
fn reg_zero_rev() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_rev_with("add", 0i32, expected)
}

#[test]
fn consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(
        "add",
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from_i32(lhs + rhs),
        }],
    )
}
