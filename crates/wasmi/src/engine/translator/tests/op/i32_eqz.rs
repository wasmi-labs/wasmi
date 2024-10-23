use super::*;
use core::fmt::Display;
use wasm_type::WasmTy;

macro_rules! test_for {
    ( $( ($input_ty:literal, $op:literal, $make_instr:expr) ),* $(,)?
    ) => {
        $( test_for($input_ty, $op, $make_instr); )*
    };
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_i32_eqz_i64() {
    fn test_for(
        input_ty: &str,
        op: &str,
        expect_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
    ) {
        let wasm = &format!(
            r"
            (module
                (func (param {input_ty} {input_ty}) (result i32)
                    (local.get 0)
                    (local.get 1)
                    ({input_ty}.{op})
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
    test_for!(
        ("i32", "and", Instruction::i32_and_eqz),
        ("i32", "or", Instruction::i32_or_eqz),
        ("i32", "xor", Instruction::i32_xor_eqz),
        ("i32", "lt_s", swap_ops!(Instruction::i32_le_s)),
        ("i32", "lt_u", swap_ops!(Instruction::i32_le_u)),
        ("i32", "le_s", swap_ops!(Instruction::i32_lt_s)),
        ("i32", "le_u", swap_ops!(Instruction::i32_lt_u)),
        ("i32", "gt_s", Instruction::i32_le_s),
        ("i32", "gt_u", Instruction::i32_le_u),
        ("i32", "ge_s", Instruction::i32_lt_s),
        ("i32", "ge_u", Instruction::i32_lt_u),
        ("i64", "lt_s", swap_ops!(Instruction::i64_le_s)),
        ("i64", "lt_u", swap_ops!(Instruction::i64_le_u)),
        ("i64", "le_s", swap_ops!(Instruction::i64_lt_s)),
        ("i64", "le_u", swap_ops!(Instruction::i64_lt_u)),
        ("i64", "gt_s", Instruction::i64_le_s),
        ("i64", "gt_u", Instruction::i64_le_u),
        ("i64", "ge_s", Instruction::i64_lt_s),
        ("i64", "ge_u", Instruction::i64_lt_u),
        ("f32", "eq", Instruction::f32_ne),
        ("f32", "ne", Instruction::f32_eq),
        ("f64", "eq", Instruction::f64_ne),
        ("f64", "ne", Instruction::f64_eq),
    );
}

macro_rules! test_for_imm {
    ( $( ($input_ty:ty, $op:literal, $make_instr:expr) ),* $(,)?
    ) => {
        $( test_for::<$input_ty>($op, 1, $make_instr); )*
    };
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_imm_i32_eqz() {
    fn test_for<T>(
        op: &str,
        value: T,
        expect_instr: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
    ) where
        T: Display + WasmTy,
        Const16<T>: TryFrom<T>,
        DisplayWasm<T>: Display,
    {
        let input_ty = T::NAME;
        let display_value = DisplayWasm::from(value);
        let wasm = &format!(
            r"
            (module
                (func (param {input_ty} {input_ty}) (result i32)
                    (local.get 0)
                    ({input_ty}.const {display_value})
                    ({input_ty}.{op})
                    (i32.eqz)
                )
            )",
        );
        TranslationTest::from_wat(wasm)
            .expect_func_instrs([
                expect_instr(
                    Reg::from(2),
                    Reg::from(0),
                    Const16::try_from(value).ok().unwrap(),
                ),
                Instruction::return_reg(2),
            ])
            .run()
    }
    test_for_imm!(
        (i32, "and", Instruction::i32_and_eqz_imm16),
        (i32, "or", Instruction::i32_or_eqz_imm16),
        (i32, "xor", Instruction::i32_xor_eqz_imm16),
        (i32, "lt_s", swap_ops!(Instruction::i32_le_s_imm16_lhs)),
        (u32, "lt_u", swap_ops!(Instruction::i32_le_u_imm16_lhs)),
        (i32, "le_s", swap_ops!(Instruction::i32_lt_s_imm16_lhs)),
        (u32, "le_u", swap_ops!(Instruction::i32_lt_u_imm16_lhs)),
        (i32, "gt_s", Instruction::i32_le_s_imm16_rhs),
        (u32, "gt_u", Instruction::i32_le_u_imm16_rhs),
        (i32, "ge_s", Instruction::i32_lt_s_imm16_rhs),
        (u32, "ge_u", Instruction::i32_lt_u_imm16_rhs),
        (i64, "lt_s", swap_ops!(Instruction::i64_le_s_imm16_lhs)),
        (u64, "lt_u", swap_ops!(Instruction::i64_le_u_imm16_lhs)),
        (i64, "le_s", swap_ops!(Instruction::i64_lt_s_imm16_lhs)),
        (u64, "le_u", swap_ops!(Instruction::i64_lt_u_imm16_lhs)),
        (i64, "gt_s", Instruction::i64_le_s_imm16_rhs),
        (u64, "gt_u", Instruction::i64_le_u_imm16_rhs),
        (i64, "ge_s", Instruction::i64_lt_s_imm16_rhs),
        (u64, "ge_u", Instruction::i64_lt_u_imm16_rhs),
    );
}
