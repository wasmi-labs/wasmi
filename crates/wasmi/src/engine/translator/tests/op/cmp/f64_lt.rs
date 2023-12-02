use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F64, "lt");

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
    test_binary_reg_reg(WASM_OP, Instruction::f64_lt)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f64, Instruction::f64_lt)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f64, Instruction::f64_lt)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_neg_inf() {
    test_binary_reg_imm_with(
        WASM_OP,
        f64::NEG_INFINITY,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(false),
        }],
    )
    .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn pos_inf_reg() {
    test_binary_reg_imm_rev_with(
        WASM_OP,
        f64::INFINITY,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(false),
        }],
    )
    .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_nan() {
    test_binary_reg_imm_with(WASM_OP, f64::NAN, [Instruction::return_imm32(false)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nan_reg() {
    test_binary_reg_imm_rev_with(WASM_OP, f64::NAN, [Instruction::return_imm32(false)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1.0,
        2.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(true),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        2.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(false),
        }],
    );
}
