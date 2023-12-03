use super::*;
use wasmi_core::TrapCode;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I32, "div_s");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    // Note: we cannot optimize for `x / x` since `x == 0` has to trap.
    let expected = [
        Instruction::i32_div_s(
            Register::from_i16(1),
            Register::from_i16(0),
            Register::from_i16(0),
        ),
        Instruction::return_reg(Register::from_i16(1)),
    ];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_div_s)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    test_binary_reg_imm16::<NonZeroI32>(WASM_OP, nonzero_i32(100), Instruction::i32_div_s_imm16)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev::<i32>(WASM_OP, 100, Instruction::i32_div_s_imm16_rev)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, i32::MAX, Instruction::i32_div_s)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, i32::MAX, Instruction::i32_div_s)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero() {
    let expected = [Instruction::Trap(TrapCode::IntegerDivisionByZero)];
    test_binary_reg_imm_with(WASM_OP, 0_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_one() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 1_i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = -4;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(lhs / rhs),
        }],
    )
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
        [Instruction::Trap(TrapCode::IntegerDivisionByZero)],
    )
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval_overflow() {
    let lhs = i32::MIN;
    let rhs = -1;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::Trap(TrapCode::IntegerOverflow)],
    )
}
