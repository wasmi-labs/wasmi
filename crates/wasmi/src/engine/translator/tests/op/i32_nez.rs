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
fn binop_i32_nez() {
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
                    (i32.const 0)
                    (i32.ne)
                )
            )",
        );
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(2), Reg::from(0), Reg::from(1)),
                Instruction::return_reg(2),
            ])
            .run()
    }
    test_for!(
        ("i32", "eq", Instruction::i32_eq),
        ("i32", "ne", Instruction::i32_ne),
        ("i32", "and", Instruction::i32_and),
        ("i32", "or", Instruction::i32_or),
        ("i32", "xor", Instruction::i32_xor),
        ("i32", "lt_s", Instruction::i32_lt_s),
        ("i32", "lt_u", Instruction::i32_lt_u),
        ("i32", "le_s", Instruction::i32_le_s),
        ("i32", "le_u", Instruction::i32_le_u),
        ("i32", "gt_s", swap_ops!(Instruction::i32_lt_s)),
        ("i32", "gt_u", swap_ops!(Instruction::i32_lt_u)),
        ("i32", "ge_s", swap_ops!(Instruction::i32_le_s)),
        ("i32", "ge_u", swap_ops!(Instruction::i32_le_u)),
        ("i64", "eq", Instruction::i64_ne),
        ("i64", "ne", Instruction::i64_eq),
        ("i64", "lt_s", Instruction::i64_lt_s),
        ("i64", "lt_u", Instruction::i64_lt_u),
        ("i64", "le_s", Instruction::i64_le_s),
        ("i64", "le_u", Instruction::i64_le_u),
        ("i64", "gt_s", swap_ops!(Instruction::i64_lt_s)),
        ("i64", "gt_u", swap_ops!(Instruction::i64_lt_u)),
        ("i64", "ge_s", swap_ops!(Instruction::i64_le_s)),
        ("i64", "ge_u", swap_ops!(Instruction::i64_le_u)),
        ("f32", "eq", Instruction::f32_eq),
        ("f32", "ne", Instruction::f32_ne),
        ("f64", "eq", Instruction::f64_eq),
        ("f64", "ne", Instruction::f64_ne),
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
fn binop_i32_nez_imm_rhs() {
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
                    (i32.const 0)
                    (i32.ne)
                )
            )",
        );
        TranslationTest::new(wasm)
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
        (i32, "eq", Instruction::i32_eq_imm16),
        (i32, "ne", Instruction::i32_ne_imm16),
        (i32, "and", Instruction::i32_and_imm16),
        (i32, "or", Instruction::i32_or_imm16),
        (i32, "xor", Instruction::i32_xor_imm16),
        (i32, "lt_s", Instruction::i32_lt_s_imm16_rhs),
        (u32, "lt_u", Instruction::i32_lt_u_imm16_rhs),
        (i32, "le_s", Instruction::i32_le_s_imm16_rhs),
        (u32, "le_u", Instruction::i32_le_u_imm16_rhs),
        (i32, "gt_s", swap_ops!(Instruction::i32_lt_s_imm16_lhs)),
        (u32, "gt_u", swap_ops!(Instruction::i32_lt_u_imm16_lhs)),
        (i32, "ge_s", swap_ops!(Instruction::i32_le_s_imm16_lhs)),
        (u32, "ge_u", swap_ops!(Instruction::i32_le_u_imm16_lhs)),
        (i64, "eq", Instruction::i64_eq_imm16),
        (i64, "ne", Instruction::i64_ne_imm16),
        (i64, "lt_s", Instruction::i64_lt_s_imm16_rhs),
        (u64, "lt_u", Instruction::i64_lt_u_imm16_rhs),
        (i64, "le_s", Instruction::i64_le_s_imm16_rhs),
        (u64, "le_u", Instruction::i64_le_u_imm16_rhs),
        (i64, "gt_s", swap_ops!(Instruction::i64_lt_s_imm16_lhs)),
        (u64, "gt_u", swap_ops!(Instruction::i64_lt_u_imm16_lhs)),
        (i64, "ge_s", swap_ops!(Instruction::i64_le_s_imm16_lhs)),
        (u64, "ge_u", swap_ops!(Instruction::i64_le_u_imm16_lhs)),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_i32_nez_imm_lhs() {
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
                    ({input_ty}.const {display_value})
                    (local.get 0)
                    ({input_ty}.{op})
                    (i32.const 0)
                    (i32.ne)
                )
            )",
        );
        TranslationTest::new(wasm)
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
        (i32, "eq", Instruction::i32_eq_imm16),
        (i32, "ne", Instruction::i32_ne_imm16),
        (i32, "and", Instruction::i32_and_imm16),
        (i32, "or", Instruction::i32_or_imm16),
        (i32, "xor", Instruction::i32_xor_imm16),
        (i32, "lt_s", swap_ops!(Instruction::i32_lt_s_imm16_lhs)),
        (u32, "lt_u", swap_ops!(Instruction::i32_lt_u_imm16_lhs)),
        (i32, "le_s", swap_ops!(Instruction::i32_le_s_imm16_lhs)),
        (u32, "le_u", swap_ops!(Instruction::i32_le_u_imm16_lhs)),
        (i32, "gt_s", Instruction::i32_lt_s_imm16_rhs),
        (u32, "gt_u", Instruction::i32_lt_u_imm16_rhs),
        (i32, "ge_s", Instruction::i32_le_s_imm16_rhs),
        (u32, "ge_u", Instruction::i32_le_u_imm16_rhs),
        (i64, "eq", Instruction::i64_eq_imm16),
        (i64, "ne", Instruction::i64_ne_imm16),
        (i64, "lt_s", swap_ops!(Instruction::i64_lt_s_imm16_lhs)),
        (u64, "lt_u", swap_ops!(Instruction::i64_lt_u_imm16_lhs)),
        (i64, "le_s", swap_ops!(Instruction::i64_le_s_imm16_lhs)),
        (u64, "le_u", swap_ops!(Instruction::i64_le_u_imm16_lhs)),
        (i64, "gt_s", Instruction::i64_lt_s_imm16_rhs),
        (u64, "gt_u", Instruction::i64_lt_u_imm16_rhs),
        (i64, "ge_s", Instruction::i64_le_s_imm16_rhs),
        (u64, "ge_u", Instruction::i64_le_u_imm16_rhs),
    );
}
