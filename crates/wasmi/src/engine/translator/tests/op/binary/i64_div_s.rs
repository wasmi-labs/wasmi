use super::*;
use crate::TrapCode;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "div_s");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    // Note: we cannot optimize for `x / x` since `x == 0` has to trap.
    let expected = [
        Instruction::i64_div_s(Reg::from(1), Reg::from(0), Reg::from(0)),
        Instruction::return_reg(Reg::from(1)),
    ];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_div_s)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    test_binary_reg_imm16_rhs::<NonZeroI64>(
        WASM_OP,
        nonzero_i64(100),
        Instruction::i64_div_s_imm16_rhs,
    )
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_lhs() {
    test_binary_reg_imm16_lhs::<i64>(WASM_OP, 100, Instruction::i64_div_s_imm16_lhs)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, i64::MAX, Instruction::i64_div_s)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs(WASM_OP, i64::MAX, Instruction::i64_div_s)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero() {
    let expected = [Instruction::trap(TrapCode::IntegerDivisionByZero)];
    test_binary_reg_imm_with(WASM_OP, 0_i64, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_one() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 1_i64, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = -4;
    let rhs = 2;
    test_binary_consteval(WASM_OP, lhs, rhs, [return_i64imm32_instr(lhs / rhs)])
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_div_by_zero() {
    let lhs = -4;
    let rhs = 0;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::trap(TrapCode::IntegerDivisionByZero)],
    )
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_overflow() {
    let lhs = i64::MIN;
    let rhs = -1;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::trap(TrapCode::IntegerOverflow)],
    )
}
