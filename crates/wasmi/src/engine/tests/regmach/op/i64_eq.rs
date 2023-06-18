use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::I64, "eq");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_i32(1),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_eq)
}

#[test]
fn reg_imm16() {
    test_binary_reg_imm16(WASM_OP, Instruction::i64_eq_imm16)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, swap_ops!(Instruction::i64_eq_imm16))
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, i64::MAX, Instruction::i64_eq_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, i64::MAX, Instruction::i64_eq_imm)
}

#[test]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1,
        1,
        [Instruction::ReturnImm32 {
            value: Const32::from_i32(1),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        42,
        5,
        [Instruction::ReturnImm32 {
            value: Const32::from_i32(0),
        }],
    );
}
