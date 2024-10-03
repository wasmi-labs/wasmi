use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::I64, "ne");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: AnyConst32::from(0),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_ne)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    test_binary_reg_imm16::<i64>(WASM_OP, 100, Instruction::i64_ne_imm16)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_lhs() {
    test_binary_reg_imm16_lhs::<i64>(WASM_OP, 100, swap_ops!(Instruction::i64_ne_imm16))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, i64::MAX, Instruction::i64_ne)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs_commutative(WASM_OP, i64::MAX, Instruction::i64_ne)
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1,
        1,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(0),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        42,
        5,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(1),
        }],
    );
}
