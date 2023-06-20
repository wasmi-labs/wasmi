use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F64, "ge");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from(true),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_ge)
}

#[test]
fn reg_imm() {
    test_binary_reg_imm64(WASM_OP, 1.0, Instruction::f64_ge_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, 1.0, Instruction::f64_le_imm)
}

#[test]
fn reg_nan() {
    test_reg_nan(WASM_OP, [Instruction::return_imm32(false)]);
}

#[test]
fn nan_reg() {
    test_nan_reg(WASM_OP, [Instruction::return_imm32(false)]);
}

#[test]
fn consteval() {
    fn test_with(lhs: f64, rhs: f64, result: bool) {
        test_binary_consteval(
            WASM_OP,
            lhs,
            rhs,
            [Instruction::ReturnImm32 {
                value: Const32::from(result),
            }],
        );
    }
    test_with(1.0, 1.0, true);
    test_with(1.0, 2.0, false);
    test_with(2.0, 1.0, true);
}
