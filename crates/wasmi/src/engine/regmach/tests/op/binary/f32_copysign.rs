use super::*;
use crate::engine::bytecode::Sign;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F32, "copysign");

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_copysign)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    fn make_instrs(sign: Sign) -> [Instruction; 2] {
        [
            Instruction::f32_copysign_imm(Register::from_i16(1), Register::from_i16(0), sign),
            Instruction::return_reg(1),
        ]
    }
    test_binary_reg_imm_with(WASM_OP, 1.0_f32, make_instrs(Sign::Pos)).run();
    test_binary_reg_imm_with(WASM_OP, -1.0_f32, make_instrs(Sign::Neg)).run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, 1.0_f32, Instruction::f32_copysign)
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 13.0_f32;
    test_binary_consteval(
        WASM_OP,
        lhs,
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(lhs),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        lhs,
        -1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(-lhs),
        }],
    );
}
