use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::I32, "eq");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(true),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_eq)
}

#[test]
fn reg_imm16() {
    test_binary_reg_imm16::<i16>(WASM_OP, 100, Instruction::i32_eq_imm16)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev::<i16>(WASM_OP, 100, swap_ops!(Instruction::i32_eq_imm16))
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, i32::MAX, Instruction::i32_eq_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, i32::MAX, Instruction::i32_eq_imm)
}

#[test]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1,
        1,
        [Instruction::ReturnImm32 {
            value: Const32::from(true),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        42,
        5,
        [Instruction::ReturnImm32 {
            value: Const32::from(false),
        }],
    );
}
