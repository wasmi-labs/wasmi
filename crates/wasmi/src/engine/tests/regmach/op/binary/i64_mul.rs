use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "mul");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_mul)
}

#[test]
fn reg_imm16() {
    test_binary_reg_imm16(WASM_OP, Instruction::i64_mul_imm16)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, swap_ops!(Instruction::i64_mul_imm16))
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, i64::MAX, Instruction::i64_mul_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, i64::MAX, Instruction::i64_mul_imm)
}

#[test]
fn reg_zero() {
    let expected = [Instruction::return_i64imm32(0)];
    test_binary_reg_imm_with(WASM_OP, 0_i64, expected).run()
}

#[test]
fn reg_zero_rev() {
    let expected = [Instruction::return_i64imm32(0)];
    test_binary_reg_imm_rev_with(WASM_OP, 0_i64, expected).run()
}

#[test]
fn reg_one() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 1_i32, expected).run()
}

#[test]
fn reg_one_rev() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_rev_with(WASM_OP, 1_i32, expected).run()
}

#[test]
fn consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(WASM_OP, lhs, rhs, [Instruction::return_i64imm32(lhs * rhs)])
}
