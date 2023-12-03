use super::*;

const WASM_OP: WasmOp = WasmOp::cmp(WasmType::F64, "eq");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    // We cannot optimize `x == x` to `true` since `x == Nan` or `Nan == x` is always `false`.
    let expected = [
        Instruction::f64_eq(
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
    test_binary_reg_reg(WASM_OP, Instruction::f64_eq)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, 1.0_f64, Instruction::f64_eq)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev_commutative(WASM_OP, 1.0_f64, Instruction::f64_eq)
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
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(1),
        }],
    );
    test_binary_consteval(
        WASM_OP,
        0.0,
        1.0,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(0),
        }],
    );
}
