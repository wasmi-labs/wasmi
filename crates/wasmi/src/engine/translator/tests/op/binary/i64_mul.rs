use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "mul");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_mul)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    test_binary_reg_imm16::<i64>(WASM_OP, 100, Instruction::i64_mul_imm16)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev::<i64>(WASM_OP, 100, swap_ops!(Instruction::i64_mul_imm16))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, i64::MAX, Instruction::i64_mul)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev_commutative(WASM_OP, i64::MAX, Instruction::i64_mul)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero() {
    let expected = [return_i64imm32_instr(0)];
    test_binary_reg_imm_with(WASM_OP, 0_i64, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero_rev() {
    let expected = [return_i64imm32_instr(0)];
    test_binary_reg_imm_rev_with(WASM_OP, 0_i64, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_one() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 1_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_one_rev() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_rev_with(WASM_OP, 1_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(WASM_OP, lhs, rhs, [return_i64imm32_instr(lhs * rhs)])
}
