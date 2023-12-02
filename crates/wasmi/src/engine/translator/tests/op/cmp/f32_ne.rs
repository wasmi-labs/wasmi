use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F32, "ne");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    // We cannot optimize `x != x` to `false` since `x != Nan` or `Nan != x` is always `true`.
    let expected = [
        Instruction::f32_ne(
            Register::from_i16(1),
            Register::from_i16(0),
            Register::from_i16(0),
        ),
        Instruction::return_reg(Register::from_i16(1)),
    ];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::f32_ne)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f32, Instruction::f32_ne)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev_commutative(WASM_OP, 1.0_f32, Instruction::f32_ne)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_nan() {
    test_binary_reg_imm_with(WASM_OP, f32::NAN, [Instruction::return_imm32(true)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nan_reg() {
    test_binary_reg_imm_rev_with(WASM_OP, f32::NAN, [Instruction::return_imm32(true)]).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    test_binary_consteval(
        WASM_OP,
        1.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(false),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        0.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(true),
        }],
    );
}
