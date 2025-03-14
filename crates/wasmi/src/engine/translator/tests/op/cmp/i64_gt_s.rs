use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::I64, "gt_s");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: AnyConst32::from(false),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, swap_ops!(Instruction::i64_lt_s))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    test_binary_reg_imm16_rhs::<i64>(WASM_OP, 100, swap_ops!(Instruction::i64_lt_s_imm16_lhs))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_lhs() {
    test_binary_reg_imm16_lhs::<i64>(WASM_OP, 100, swap_ops!(Instruction::i64_lt_s_imm16_rhs))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 100_000, swap_ops!(Instruction::i64_lt_s))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs(WASM_OP, 100_000, swap_ops!(Instruction::i64_lt_s))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_max() {
    let expected = [Instruction::ReturnImm32 {
        value: AnyConst32::from(false),
    }];
    test_binary_reg_imm_with(WASM_OP, i64::MAX, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn min_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: AnyConst32::from(false),
    }];
    test_binary_reg_imm_lhs_with(WASM_OP, i64::MIN, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 1_i64;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(lhs > rhs),
        }],
    )
}
