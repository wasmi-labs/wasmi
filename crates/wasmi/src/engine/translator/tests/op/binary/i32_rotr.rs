use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I32, "rotr");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_rotr)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs(WASM_OP, i32::MAX, Instruction::i32_rotr)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_lhs() {
    test_binary_reg_imm16_lhs::<i32>(WASM_OP, 100, Instruction::i32_rotr_imm16)
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
    test_binary_reg_imm_with(WASM_OP, 32_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 64_i32, expected).run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_1_after_mod32() {
    let expected = [
        Instruction::i32_rotr_by(Reg::from(1), Reg::from(0), shamt::<i32>(1)),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(WASM_OP, 1_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 33_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 65_i32, expected).run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn zero_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: AnyConst32::from(0_i32),
    }];
    test_binary_reg_imm_lhs_with(WASM_OP, 0_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn minus_one_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: AnyConst32::from(-1_i32),
    }];
    test_binary_reg_imm_lhs_with(WASM_OP, -1_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = -10_i32;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(lhs.rotate_right(rhs as u32)),
        }],
    )
}
