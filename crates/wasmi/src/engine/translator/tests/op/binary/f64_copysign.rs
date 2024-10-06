use super::*;
use crate::ir::Sign;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F64, "copysign");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_copysign)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    fn make_instrs(sign: Sign<f64>) -> [Instruction; 2] {
        [
            Instruction::f64_copysign_imm(Reg::from(1), Reg::from(0), sign),
            Instruction::return_reg(1),
        ]
    }
    test_binary_reg_imm_with(WASM_OP, 1.0_f64, make_instrs(Sign::pos())).run();
    test_binary_reg_imm_with(WASM_OP, -1.0_f64, make_instrs(Sign::neg())).run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs(WASM_OP, 1.0_f64, Instruction::f64_copysign)
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 13.0_f64;
    testcase_binary_consteval(WASM_OP, lhs, 1.0)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(lhs)]))
        .run();
    testcase_binary_consteval(WASM_OP, lhs, -1.0)
        .expect_func(ExpectedFunc::new([return_f64imm32_instr(-lhs)]))
        .run();
}
