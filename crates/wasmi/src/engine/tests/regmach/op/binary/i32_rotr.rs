use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I32, "rotr");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_rotr)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, i32::MAX, Instruction::i32_rotr_imm_rev)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, Instruction::i32_rotr_imm16_rev)
}

#[test]
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0_i32, expected).run()
}

#[test]
fn reg_0_after_mod32() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 32_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 64_i32, expected).run();
}

#[test]
fn reg_1_after_mod32() {
    let expected = [
        Instruction::i32_rotr_imm(
            Register::from_u16(1),
            Register::from_u16(0),
            Const16::from_i16(1),
        ),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(WASM_OP, 1_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 33_i32, expected).run();
    test_binary_reg_imm_with(WASM_OP, 65_i32, expected).run();
}

#[test]
fn zero_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_i32(0_i32),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, 0_i32, expected).run()
}

#[test]
fn minus_one_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_i32(-1_i32),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, -1_i32, expected).run()
}

#[test]
fn consteval() {
    let lhs = -10_i32;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from_i32(lhs.rotate_right(rhs as u32)),
        }],
    )
}
