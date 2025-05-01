use super::*;
use core::fmt::Display;
use wasm_type::WasmTy;

macro_rules! test_for {
    ( $( ($op:literal, $make_instr:expr) ),* $(,)?
    ) => {
        $( test_for($op, $make_instr); )*
    };
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_i64_eqz() {
    fn test_for(op: &str, expect_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction) {
        let wasm = &format!(
            r"
            (module
                (func (param i64 i64) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i64.{op})
                    (i64.eqz)
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
        ("and", Instruction::i64_nand),
        ("or", Instruction::i64_nor),
        ("xor", Instruction::i64_xnor),
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
fn binop_imm_i64_eqz_rhs() {
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
                    (i64.eqz)
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
        (i64, "and", Instruction::i64_nand_imm16),
        (i64, "or", Instruction::i64_nor_imm16),
        (i64, "xor", Instruction::i64_xnor_imm16),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_imm_i64_eqz_lhs() {
    fn test_for<T>(
        op: &str,
        value: T,
        expect_instr: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
    ) where
        T: Display + WasmTy,
        Const16<T>: TryFrom<T>,
        DisplayWasm<T>: Display,
    {
        let display_value = DisplayWasm::from(value);
        let wasm = &format!(
            r"
            (module
                (func (param i64 i64) (result i32)
                    (i64.const {display_value})
                    (local.get 0)
                    (i64.{op})
                    (i64.eqz)
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
        (i64, "and", Instruction::i64_nand_imm16),
        (i64, "or", Instruction::i64_nor_imm16),
        (i64, "xor", Instruction::i64_xnor_imm16),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_i64_eqz_double() {
    fn test_for(op: &str, expect_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction) {
        let wasm = &format!(
            r"
            (module
                (func (param i64 i64) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i64.{op})
                    (i64.eqz)
                    (i32.eqz)
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
        ("and", Instruction::i64_and),
        ("or", Instruction::i64_or),
        ("xor", Instruction::i64_xor),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_imm_i64_eqz_rhs_double() {
    fn test_for<T>(
        op: &str,
        value: T,
        expect_instr: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
    ) where
        T: Display + WasmTy,
        Const16<T>: TryFrom<T>,
        DisplayWasm<T>: Display,
    {
        let display_value = DisplayWasm::from(value);
        let wasm = &format!(
            r"
            (module
                (func (param i64 i64) (result i32)
                    (local.get 0)
                    (i64.const {display_value})
                    (i64.{op})
                    (i64.eqz)
                    (i32.eqz)
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
        (i64, "and", Instruction::i64_and_imm16),
        (i64, "or", Instruction::i64_or_imm16),
        (i64, "xor", Instruction::i64_xor_imm16),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn binop_imm_i64_eqz_lhs_double() {
    fn test_for<T>(
        op: &str,
        value: T,
        expect_instr: fn(result: Reg, lhs: Reg, rhs: Const16<T>) -> Instruction,
    ) where
        T: Display + WasmTy,
        Const16<T>: TryFrom<T>,
        DisplayWasm<T>: Display,
    {
        let display_value = DisplayWasm::from(value);
        let wasm = &format!(
            r"
            (module
                (func (param i64 i64) (result i32)
                    (i64.const {display_value})
                    (local.get 0)
                    (i64.{op})
                    (i64.eqz)
                    (i32.eqz)
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
        (i64, "and", Instruction::i64_and_imm16),
        (i64, "or", Instruction::i64_or_imm16),
        (i64, "xor", Instruction::i64_xor_imm16),
    );
}
