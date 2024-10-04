use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::I32, "eq");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: AnyConst32::from(true),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_eq)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    test_binary_reg_imm16_rhs::<i32>(WASM_OP, 100, Instruction::i32_eq_imm16)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_lhs() {
    test_binary_reg_imm16_lhs::<i32>(WASM_OP, 100, swap_ops!(Instruction::i32_eq_imm16))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, i32::MAX, Instruction::i32_eq)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs_commutative(WASM_OP, i32::MAX, Instruction::i32_eq)
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1,
        1,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(true),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        42,
        5,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(false),
        }],
    );
}
