use super::*;
use crate::engine::bytecode2::Sign;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::F64, "copysign");

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f64_copysign)
}

#[test]
fn reg_imm() {
    fn make_instrs(sign: Sign) -> [Instruction; 2] {
        [
            Instruction::f64_copysign_imm(Register::from_u16(1), Register::from_u16(0), sign),
            Instruction::return_reg(1),
        ]
    }
    test_binary_reg_imm_with(WASM_OP, 1.0_f64, make_instrs(Sign::Pos));
    test_binary_reg_imm_with(WASM_OP, -1.0_f64, make_instrs(Sign::Neg));
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm64_rev(WASM_OP, 1.0_f64, Instruction::f64_copysign_imm_rev)
}

#[test]
fn consteval() {
    let lhs = 13.0_f32;
    test_binary_consteval(
        WASM_OP,
        lhs,
        1.0,
        [Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        lhs,
        -1.0,
        [Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }],
    );
}
