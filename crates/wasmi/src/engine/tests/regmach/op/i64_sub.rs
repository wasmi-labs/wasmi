use super::*;

const WASM_OP: WasmOp = WasmOp::I64("sub");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnI64Imm32 {
        value: Const32::from_i32(0),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_sub)
}

#[test]
fn reg_imm16() {
    test_binary_reg_imm16(WASM_OP, Instruction::i64_sub_imm16)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, Instruction::i64_sub_imm16_rev)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, Instruction::i64_sub_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, Instruction::i64_sub_imm)
}

#[test]
fn reg_zero() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with(WASM_OP, 0i32, expected)
}

#[test]
fn consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnI64Imm32 {
            value: Const32::from_i32(lhs - rhs),
        }],
    )
}
