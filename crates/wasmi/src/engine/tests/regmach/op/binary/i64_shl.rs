use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "shl");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_shl)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, i64::MAX, Instruction::i64_shl_imm_rev)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, Instruction::i64_shl_imm16_rev)
}

#[test]
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0_i32, expected)
}

#[test]
fn reg_0_after_mod32() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0_i32, expected);
    test_binary_reg_imm_with(WASM_OP, 64_i32, expected);
    test_binary_reg_imm_with(WASM_OP, 128_i32, expected);
}

#[test]
fn reg_1_after_mod32() {
    let expected = [
        Instruction::i64_shl_imm(
            Register::from_u16(1),
            Register::from_u16(0),
            Const16::from_i16(1),
        ),
        Instruction::return_reg(1),
    ];
    test_binary_reg_imm_with(WASM_OP, 1_i32, expected);
    test_binary_reg_imm_with(WASM_OP, 65_i32, expected);
    test_binary_reg_imm_with(WASM_OP, 129_i32, expected);
}

#[test]
fn zero_reg() {
    let expected = [Instruction::return_i64imm32(0)];
    test_binary_reg_imm_rev_with(WASM_OP, 0_i32, expected)
}

#[test]
fn consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::return_i64imm32(lhs << rhs)],
    )
}
