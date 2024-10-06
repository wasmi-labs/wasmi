use super::*;

#[test]
#[cfg_attr(miri, ignore)]
fn binop_i32_eqz() {
    fn test_for(op: &str, expect_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction) {
        let wasm = &format!(
            r"
            (module
                (func (param i32 i32) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.{op})
                    (i32.eqz)
                )
            )",
        );
        TranslationTest::from_wat(wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(2), Reg::from(0), Reg::from(1)),
                Instruction::return_reg(2),
            ])
            .run()
    }
    test_for("and", Instruction::i32_and_eqz);
    test_for("or", Instruction::i32_or_eqz);
    test_for("xor", Instruction::i32_xor_eqz);
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_imm_i32_eqz() {
    fn test_for(
        op: &str,
        expect_instr: fn(result: Reg, lhs: Reg, rhs: Const16<i32>) -> Instruction,
    ) {
        let wasm = &format!(
            r"
            (module
                (func (param i32 i32) (result i32)
                    (local.get 0)
                    (i32.const 1)
                    (i32.{op})
                    (i32.eqz)
                )
            )",
        );
        TranslationTest::from_wat(wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(2), Reg::from(0), Const16::from(1)),
                Instruction::return_reg(2),
            ])
            .run()
    }
    test_for("and", Instruction::i32_and_eqz_imm16);
    test_for("or", Instruction::i32_or_eqz_imm16);
    test_for("xor", Instruction::i32_xor_eqz_imm16);
}
