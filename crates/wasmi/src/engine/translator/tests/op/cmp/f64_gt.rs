use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F64, "gt");

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
    test_binary_local_reg(WASM_OP, swap_ops!(Instruction::f64_lt))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_local_imm32(WASM_OP, 1.0, swap_ops!(Instruction::f64_lt))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_local_imm32_lhs(WASM_OP, 1.0, swap_ops!(Instruction::f64_lt))
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_pos_inf() {
    test_binary_local_imm_with(
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
fn neg_inf_reg() {
    test_binary_local_imm_lhs_with(
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
fn loc_nan() {
    test_binary_local_imm_with(WASM_OP, f64::NAN, [Instruction::return_imm32(false)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nan_reg() {
    test_binary_local_imm_lhs_with(WASM_OP, f64::NAN, [Instruction::return_imm32(false)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1.0,
        2.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(false),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        2.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(true),
        }],
    );
}
