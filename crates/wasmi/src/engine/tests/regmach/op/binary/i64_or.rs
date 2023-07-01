use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "or");

#[test]
fn same_reg() {
    let expected = [Instruction::return_reg(0)];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_or)
}

#[test]
fn reg_imm16() {
    test_binary_reg_imm16(WASM_OP, Instruction::i64_or_imm16)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, swap_ops!(Instruction::i64_or_imm16))
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, i64::MAX, Instruction::i64_or_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, i64::MAX, Instruction::i64_or_imm)
}

#[test]
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0i64, expected).run()
}

#[test]
fn reg_zero_rev() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_rev_with(WASM_OP, 0i64, expected).run()
}

#[test]
fn reg_ones() {
    let expected = [Instruction::return_i64imm32(-1)];
    test_binary_reg_imm_with(WASM_OP, -1_i64, expected).run()
}

#[test]
fn reg_ones_rev() {
    let expected = [Instruction::return_i64imm32(-1)];
    test_binary_reg_imm_rev_with(WASM_OP, -1_i64, expected).run()
}

#[test]
fn consteval() {
    let lhs = 10;
    let rhs = 20;
    test_binary_consteval(WASM_OP, lhs, rhs, [Instruction::return_i64imm32(lhs | rhs)])
}
