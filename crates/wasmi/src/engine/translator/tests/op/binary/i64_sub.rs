use super::*;

const WASM_OP: WasmOp = WasmOp::binary(WasmType::I64, "sub");

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    let expected = [return_i64imm32_instr(0)];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i64_sub)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16() {
    let value = 100;
    let rhs = <Const16<i64>>::from(-value);
    test_binary_reg_imm_with::<i64, _>(
        WASM_OP,
        i64::from(value),
        [
            Instruction::i64_add_imm16(Reg::from(1), Reg::from(0), rhs),
            Instruction::return_reg(1),
        ],
    )
    .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm16_lhs() {
    test_binary_reg_imm16_lhs::<i64>(WASM_OP, 100, Instruction::i64_sub_imm16_lhs)
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    test_reg_imm(i64::MAX);
    test_reg_imm(i64::MAX - 1);
    test_reg_imm(i64::MIN);
    test_reg_imm(i64::MIN + 1);
    test_reg_imm(i64::from(i16::MIN));
    test_reg_imm(i64::from(i16::MAX) + 2);
}

fn test_reg_imm(value: i64) {
    let mut testcase = testcase_binary_reg_imm(WASM_OP, value);
    testcase.expect_func(
        ExpectedFunc::new([
            Instruction::i64_add(Reg::from(1), Reg::from(0), Reg::from(-1)),
            Instruction::return_reg(Reg::from(1)),
        ])
        .consts([value.wrapping_neg()]),
    );
    testcase.run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm_lhs() {
    test_binary_reg_imm32_lhs(WASM_OP, i64::MAX, Instruction::i64_sub)
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
    test_binary_consteval(WASM_OP, lhs, rhs, [return_i64imm32_instr(lhs - rhs)])
}
