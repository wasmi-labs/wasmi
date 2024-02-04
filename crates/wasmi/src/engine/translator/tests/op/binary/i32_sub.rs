use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I32, "sub");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: AnyConst32::from(0),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_sub)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    let value = 100;
    let rhs = <Const16<i32>>::from(-value);
    test_binary_reg_imm_with::<i32, _>(
        WASM_OP,
        value as _,
        [
            Instruction::i32_add_imm16(Register::from_i16(1), Register::from_i16(0), rhs),
            Instruction::return_reg(1),
        ],
    )
    .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev::<i32>(WASM_OP, 100, Instruction::i32_sub_imm16_rev)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, i32::MAX, Instruction::i32_sub)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, i32::MAX, Instruction::i32_sub)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_zero() {
    let expected = [Instruction::return_reg(0)];
    test_binary_reg_imm_with(WASM_OP, 0i32, expected).run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn consteval() {
    let lhs = 1;
    let rhs = 2;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: AnyConst32::from(lhs - rhs),
        }],
    )
}
