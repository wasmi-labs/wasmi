use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F64, "lt");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(false),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_lt)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, 1.0_f64, Instruction::f64_lt_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, 1.0_f64, Instruction::f64_gt_imm)
}

#[test]
fn reg_neg_inf() {
    test_binary_reg_imm_with(
        WASM_OP,
        f64::NEG_INFINITY,
        [Instruction::ReturnImm32 {
            value: Const32::from(false),
        }],
    )
    .run()
}

#[test]
fn pos_inf_reg() {
    test_binary_reg_imm_rev_with(
        WASM_OP,
        f64::INFINITY,
        [Instruction::ReturnImm32 {
            value: Const32::from(false),
        }],
    )
    .run()
}

#[test]
fn reg_nan() {
    test_binary_reg_imm_with(WASM_OP, f64::NAN, [Instruction::return_imm32(false)]).run()
}

#[test]
fn nan_reg() {
    test_binary_reg_imm_rev_with(WASM_OP, f64::NAN, [Instruction::return_imm32(false)]).run()
}

#[test]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1.0,
        2.0,
        [Instruction::ReturnImm32 {
            value: Const32::from(true),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        2.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: Const32::from(false),
        }],
    );
}
