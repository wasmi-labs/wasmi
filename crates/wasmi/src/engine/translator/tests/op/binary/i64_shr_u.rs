use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "shr_u");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_shr_u)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, i64::MAX, Instruction::i64_shr_u)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev::<i64>(WASM_OP, 100, Instruction::i64_shr_u_imm16_rev)
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
        Instruction::i64_shr_u_imm(Reg::from_i16(1), Reg::from_i16(0), <Const16<i64>>::from(1)),
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
    test_binary_reg_imm_rev_with(WASM_OP, 0_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(WASM_OP, lhs, rhs, [return_i64imm32_instr(lhs >> rhs)])
}
