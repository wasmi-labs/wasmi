use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "rotl");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_rotl)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs(WASM_OP, i64::MAX, Instruction::i64_rotl)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_lhs() {
    test_binary_reg_imm16_lhs::<i64>(WASM_OP, 100, Instruction::i64_rotl_imm16)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_0_after_mod32() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 64_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 128_i32, expected).run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_1_after_mod32() {
    let expected = [
        Instruction::i64_rotl_by(Local::from(1), Local::from(0), shamt::<i64>(1)),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(WASM_OP, 1_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 65_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 129_i32, expected).run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn zero_reg() {
    let expected = [return_i64imm32_instr(0)];
    test_binary_reg_imm_lhs_with(WASM_OP, 0_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn minus_one_reg() {
    let expected = [return_i64imm32_instr(-1)];
    test_binary_reg_imm_lhs_with(WASM_OP, -1_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 10_i64;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [return_i64imm32_instr(lhs.rotate_left(rhs as u32))],
    )
}
