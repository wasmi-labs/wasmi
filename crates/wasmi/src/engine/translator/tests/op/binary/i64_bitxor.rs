use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "xor");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    let expected = [return_i64imm32_instr(0)];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_bitxor)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    test_binary_reg_imm16_rhs::<i64>(WASM_OP, 100, Instruction::i64_bitxor_imm16)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_lhs() {
    test_binary_reg_imm16_lhs::<i64>(WASM_OP, 100, swap_ops!(Instruction::i64_bitxor_imm16))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, i64::MAX, Instruction::i64_bitxor)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs_commutative(WASM_OP, i64::MAX, Instruction::i64_bitxor)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0i64, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero_lhs() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_lhs_with(WASM_OP, 0i64, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 10;
    let rhs = 20;
    test_binary_consteval(WASM_OP, lhs, rhs, [return_i64imm32_instr(lhs ^ rhs)])
}
