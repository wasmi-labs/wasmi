use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F32, "eq");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    // We cannot optimize `x == x` to `true` since `x == Nan` or `Nan == x` is always `false`.
    let expected = [
        Instruction::f32_eq(Local::from(1), Local::from(0), Local::from(0)),
        Instruction::return_reg(Local::from(1)),
    ];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_local_reg(WASM_OP, Instruction::f32_eq)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_local_imm32(WASM_OP, 1.0_f32, Instruction::f32_eq)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_local_imm32_lhs_commutative(WASM_OP, 1.0_f32, Instruction::f32_eq)
}

#[test]
#[cfg_attr(miri, ignore)]
fn loc_nan() {
    test_binary_local_imm_with(WASM_OP, f32::NAN, [Instruction::return_imm32(false)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nan_reg() {
    test_binary_local_imm_lhs_with(WASM_OP, f32::NAN, [Instruction::return_imm32(false)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(true),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        0.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(false),
        }],
    );
}
