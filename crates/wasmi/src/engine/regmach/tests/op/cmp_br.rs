use super::{wasm_type::WasmType, *};
use crate::{
    core::ValueType,
    engine::{
        bytecode::{BranchOffset, GlobalIdx},
        regmach::{bytecode::BranchOffset16, tests::display_wasm::DisplayValueType},
    },
};
use std::fmt::{Debug, Display};

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward() {
    fn test_for(
        ty: ValueType,
        op: &str,
        expect_instr: fn(Register, Register, BranchOffset16) -> Instruction,
    ) {
        let ty = DisplayValueType::from(ty);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param {ty} {ty})
                    (loop
                        (local.get 0)
                        (local.get 1)
                        ({ty}.{op})
                        (br_if 0)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    BranchOffset16::from(0),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for(ValueType::I32, "and", Instruction::branch_i32_and);
    test_for(ValueType::I32, "or", Instruction::branch_i32_or);
    test_for(ValueType::I32, "xor", Instruction::branch_i32_xor);
    test_for(ValueType::I32, "eq", Instruction::branch_i32_eq);
    test_for(ValueType::I32, "ne", Instruction::branch_i32_ne);
    test_for(ValueType::I32, "lt_s", Instruction::branch_i32_lt_s);
    test_for(ValueType::I32, "lt_u", Instruction::branch_i32_lt_u);
    test_for(ValueType::I32, "le_s", Instruction::branch_i32_le_s);
    test_for(ValueType::I32, "le_u", Instruction::branch_i32_le_u);
    test_for(ValueType::I32, "gt_s", Instruction::branch_i32_gt_s);
    test_for(ValueType::I32, "gt_u", Instruction::branch_i32_gt_u);
    test_for(ValueType::I32, "ge_s", Instruction::branch_i32_ge_s);
    test_for(ValueType::I32, "ge_u", Instruction::branch_i32_ge_u);

    test_for(ValueType::I64, "eq", Instruction::branch_i64_eq);
    test_for(ValueType::I64, "ne", Instruction::branch_i64_ne);
    test_for(ValueType::I64, "lt_s", Instruction::branch_i64_lt_s);
    test_for(ValueType::I64, "lt_u", Instruction::branch_i64_lt_u);
    test_for(ValueType::I64, "le_s", Instruction::branch_i64_le_s);
    test_for(ValueType::I64, "le_u", Instruction::branch_i64_le_u);
    test_for(ValueType::I64, "gt_s", Instruction::branch_i64_gt_s);
    test_for(ValueType::I64, "gt_u", Instruction::branch_i64_gt_u);
    test_for(ValueType::I64, "ge_s", Instruction::branch_i64_ge_s);
    test_for(ValueType::I64, "ge_u", Instruction::branch_i64_ge_u);

    test_for(ValueType::F32, "eq", Instruction::branch_f32_eq);
    test_for(ValueType::F32, "ne", Instruction::branch_f32_ne);
    test_for(ValueType::F32, "lt", Instruction::branch_f32_lt);
    test_for(ValueType::F32, "le", Instruction::branch_f32_le);
    test_for(ValueType::F32, "gt", Instruction::branch_f32_gt);
    test_for(ValueType::F32, "ge", Instruction::branch_f32_ge);

    test_for(ValueType::F64, "eq", Instruction::branch_f64_eq);
    test_for(ValueType::F64, "ne", Instruction::branch_f64_ne);
    test_for(ValueType::F64, "lt", Instruction::branch_f64_lt);
    test_for(ValueType::F64, "le", Instruction::branch_f64_le);
    test_for(ValueType::F64, "gt", Instruction::branch_f64_gt);
    test_for(ValueType::F64, "ge", Instruction::branch_f64_ge);
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm() {
    fn test_for<T>(
        op: &str,
        value: T,
        expect_instr: fn(Register, Const16<T>, BranchOffset16) -> Instruction,
    ) where
        T: WasmType,
        Const16<T>: TryFrom<T> + Debug,
        DisplayWasm<T>: Display,
    {
        let ty = T::NAME;
        let display_value = DisplayWasm::from(value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param {ty} {ty})
                    (loop
                        (local.get 0)
                        ({ty}.const {display_value})
                        ({ty}.{op})
                        (br_if 0)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(
                    Register::from_i16(0),
                    <Const16<T>>::try_from(value).ok().unwrap(),
                    BranchOffset16::from(0),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for::<i32>("and", 1, Instruction::branch_i32_and_imm);
    test_for::<i32>("or", 1, Instruction::branch_i32_or_imm);
    test_for::<i32>("xor", 1, Instruction::branch_i32_xor_imm);
    test_for::<i32>("eq", 1, Instruction::branch_i32_eq_imm);
    test_for::<i32>("ne", 1, Instruction::branch_i32_ne_imm);
    test_for::<i32>("lt_s", 1, Instruction::branch_i32_lt_s_imm);
    test_for::<u32>("lt_u", 1, Instruction::branch_i32_lt_u_imm);
    test_for::<i32>("le_s", 1, Instruction::branch_i32_le_s_imm);
    test_for::<u32>("le_u", 1, Instruction::branch_i32_le_u_imm);
    test_for::<i32>("gt_s", 1, Instruction::branch_i32_gt_s_imm);
    test_for::<u32>("gt_u", 1, Instruction::branch_i32_gt_u_imm);
    test_for::<i32>("ge_s", 1, Instruction::branch_i32_ge_s_imm);
    test_for::<u32>("ge_u", 1, Instruction::branch_i32_ge_u_imm);

    test_for::<i64>("eq", 1, Instruction::branch_i64_eq_imm);
    test_for::<i64>("ne", 1, Instruction::branch_i64_ne_imm);
    test_for::<i64>("lt_s", 1, Instruction::branch_i64_lt_s_imm);
    test_for::<u64>("lt_u", 1, Instruction::branch_i64_lt_u_imm);
    test_for::<i64>("le_s", 1, Instruction::branch_i64_le_s_imm);
    test_for::<u64>("le_u", 1, Instruction::branch_i64_le_u_imm);
    test_for::<i64>("gt_s", 1, Instruction::branch_i64_gt_s_imm);
    test_for::<u64>("gt_u", 1, Instruction::branch_i64_gt_u_imm);
    test_for::<i64>("ge_s", 1, Instruction::branch_i64_ge_s_imm);
    test_for::<u64>("ge_u", 1, Instruction::branch_i64_ge_u_imm);
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm_eqz() {
    fn test_for(op: &str, expect_instr: fn(Register, BranchOffset) -> Instruction) {
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32 i32)
                    (loop
                        (local.get 0)
                        (i32.const 0)
                        (i32.{op})
                        (br_if 0)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(Register::from_i16(0), BranchOffset::from(0_i32)),
                Instruction::Return,
            ])
            .run()
    }
    test_for("eq", Instruction::branch_i32_eqz);
    test_for("ne", Instruction::branch_i32_nez);
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward() {
    fn test_for(
        ty: ValueType,
        op: &str,
        expect_instr: fn(Register, Register, BranchOffset16) -> Instruction,
    ) {
        let ty = DisplayValueType::from(ty);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param {ty} {ty})
                    (block
                        (local.get 0)
                        (local.get 1)
                        ({ty}.{op})
                        (br_if 0)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    BranchOffset16::from(1),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for(ValueType::I32, "and", Instruction::branch_i32_and);
    test_for(ValueType::I32, "or", Instruction::branch_i32_or);
    test_for(ValueType::I32, "xor", Instruction::branch_i32_xor);
    test_for(ValueType::I32, "eq", Instruction::branch_i32_eq);
    test_for(ValueType::I32, "ne", Instruction::branch_i32_ne);
    test_for(ValueType::I32, "lt_s", Instruction::branch_i32_lt_s);
    test_for(ValueType::I32, "lt_u", Instruction::branch_i32_lt_u);
    test_for(ValueType::I32, "le_s", Instruction::branch_i32_le_s);
    test_for(ValueType::I32, "le_u", Instruction::branch_i32_le_u);
    test_for(ValueType::I32, "gt_s", Instruction::branch_i32_gt_s);
    test_for(ValueType::I32, "gt_u", Instruction::branch_i32_gt_u);
    test_for(ValueType::I32, "ge_s", Instruction::branch_i32_ge_s);
    test_for(ValueType::I32, "ge_u", Instruction::branch_i32_ge_u);

    test_for(ValueType::I64, "eq", Instruction::branch_i64_eq);
    test_for(ValueType::I64, "ne", Instruction::branch_i64_ne);
    test_for(ValueType::I64, "lt_s", Instruction::branch_i64_lt_s);
    test_for(ValueType::I64, "lt_u", Instruction::branch_i64_lt_u);
    test_for(ValueType::I64, "le_s", Instruction::branch_i64_le_s);
    test_for(ValueType::I64, "le_u", Instruction::branch_i64_le_u);
    test_for(ValueType::I64, "gt_s", Instruction::branch_i64_gt_s);
    test_for(ValueType::I64, "gt_u", Instruction::branch_i64_gt_u);
    test_for(ValueType::I64, "ge_s", Instruction::branch_i64_ge_s);
    test_for(ValueType::I64, "ge_u", Instruction::branch_i64_ge_u);

    test_for(ValueType::F32, "eq", Instruction::branch_f32_eq);
    test_for(ValueType::F32, "ne", Instruction::branch_f32_ne);
    test_for(ValueType::F32, "lt", Instruction::branch_f32_lt);
    test_for(ValueType::F32, "le", Instruction::branch_f32_le);
    test_for(ValueType::F32, "gt", Instruction::branch_f32_gt);
    test_for(ValueType::F32, "ge", Instruction::branch_f32_ge);

    test_for(ValueType::F64, "eq", Instruction::branch_f64_eq);
    test_for(ValueType::F64, "ne", Instruction::branch_f64_ne);
    test_for(ValueType::F64, "lt", Instruction::branch_f64_lt);
    test_for(ValueType::F64, "le", Instruction::branch_f64_le);
    test_for(ValueType::F64, "gt", Instruction::branch_f64_gt);
    test_for(ValueType::F64, "ge", Instruction::branch_f64_ge);
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward_nop_copy() {
    fn test_for(
        ty: ValueType,
        op: &str,
        expect_instr: fn(Register, Register, BranchOffset16) -> Instruction,
    ) {
        let ty = DisplayValueType::from(ty);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (global $g (mut {ty}) ({ty}.const 10))
                (func (param {ty} {ty}) (result {ty})
                    (global.get $g)
                    (block (param {ty}) (result {ty})
                        (local.get 0)
                        (local.get 1)
                        ({ty}.{op})
                        (br_if 0)
                        (drop)
                        (local.get 0)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::global_get(Register::from_i16(2), GlobalIdx::from(0)),
                expect_instr(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    BranchOffset16::from(2),
                ),
                Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
                Instruction::return_reg(2),
            ])
            .run()
    }

    test_for(ValueType::I32, "and", Instruction::branch_i32_and);
    test_for(ValueType::I32, "or", Instruction::branch_i32_or);
    test_for(ValueType::I32, "xor", Instruction::branch_i32_xor);
    test_for(ValueType::I32, "eq", Instruction::branch_i32_eq);
    test_for(ValueType::I32, "ne", Instruction::branch_i32_ne);
    test_for(ValueType::I32, "lt_s", Instruction::branch_i32_lt_s);
    test_for(ValueType::I32, "lt_u", Instruction::branch_i32_lt_u);
    test_for(ValueType::I32, "le_s", Instruction::branch_i32_le_s);
    test_for(ValueType::I32, "le_u", Instruction::branch_i32_le_u);
    test_for(ValueType::I32, "gt_s", Instruction::branch_i32_gt_s);
    test_for(ValueType::I32, "gt_u", Instruction::branch_i32_gt_u);
    test_for(ValueType::I32, "ge_s", Instruction::branch_i32_ge_s);
    test_for(ValueType::I32, "ge_u", Instruction::branch_i32_ge_u);

    test_for(ValueType::I64, "eq", Instruction::branch_i64_eq);
    test_for(ValueType::I64, "ne", Instruction::branch_i64_ne);
    test_for(ValueType::I64, "lt_s", Instruction::branch_i64_lt_s);
    test_for(ValueType::I64, "lt_u", Instruction::branch_i64_lt_u);
    test_for(ValueType::I64, "le_s", Instruction::branch_i64_le_s);
    test_for(ValueType::I64, "le_u", Instruction::branch_i64_le_u);
    test_for(ValueType::I64, "gt_s", Instruction::branch_i64_gt_s);
    test_for(ValueType::I64, "gt_u", Instruction::branch_i64_gt_u);
    test_for(ValueType::I64, "ge_s", Instruction::branch_i64_ge_s);
    test_for(ValueType::I64, "ge_u", Instruction::branch_i64_ge_u);

    test_for(ValueType::F32, "eq", Instruction::branch_f32_eq);
    test_for(ValueType::F32, "ne", Instruction::branch_f32_ne);
    test_for(ValueType::F32, "lt", Instruction::branch_f32_lt);
    test_for(ValueType::F32, "le", Instruction::branch_f32_le);
    test_for(ValueType::F32, "gt", Instruction::branch_f32_gt);
    test_for(ValueType::F32, "ge", Instruction::branch_f32_ge);

    test_for(ValueType::F64, "eq", Instruction::branch_f64_eq);
    test_for(ValueType::F64, "ne", Instruction::branch_f64_ne);
    test_for(ValueType::F64, "lt", Instruction::branch_f64_lt);
    test_for(ValueType::F64, "le", Instruction::branch_f64_le);
    test_for(ValueType::F64, "gt", Instruction::branch_f64_gt);
    test_for(ValueType::F64, "ge", Instruction::branch_f64_ge);
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward_multi_value() {
    fn test_for(
        ty: ValueType,
        op: &str,
        expect_instr: fn(Register, Register, BranchOffset16) -> Instruction,
    ) {
        let ty = DisplayValueType::from(ty);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param {ty} {ty}) (result {ty})
                    (block (result {ty})
                        (local.get 0) ;; returned from block if `local.get 0 != 0`
                        (local.get 0)
                        (local.get 1)
                        ({ty}.{op})
                        (br_if 0)
                        (drop)
                        (local.get 1) ;; returned from block if `local.get 0 == 0`
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    BranchOffset16::from(3),
                ),
                Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
                Instruction::branch(BranchOffset::from(2)),
                Instruction::copy(Register::from_i16(2), Register::from_i16(1)),
                Instruction::return_reg(2),
            ])
            .run()
    }

    test_for(ValueType::I32, "and", Instruction::branch_i32_and_eqz);
    test_for(ValueType::I32, "or", Instruction::branch_i32_or_eqz);
    test_for(ValueType::I32, "xor", Instruction::branch_i32_xor_eqz);
    test_for(ValueType::I32, "eq", Instruction::branch_i32_ne);
    test_for(ValueType::I32, "ne", Instruction::branch_i32_eq);
    test_for(ValueType::I32, "lt_s", Instruction::branch_i32_ge_s);
    test_for(ValueType::I32, "lt_u", Instruction::branch_i32_ge_u);
    test_for(ValueType::I32, "le_s", Instruction::branch_i32_gt_s);
    test_for(ValueType::I32, "le_u", Instruction::branch_i32_gt_u);
    test_for(ValueType::I32, "gt_s", Instruction::branch_i32_le_s);
    test_for(ValueType::I32, "gt_u", Instruction::branch_i32_le_u);
    test_for(ValueType::I32, "ge_s", Instruction::branch_i32_lt_s);
    test_for(ValueType::I32, "ge_u", Instruction::branch_i32_lt_u);

    test_for(ValueType::I64, "eq", Instruction::branch_i64_ne);
    test_for(ValueType::I64, "ne", Instruction::branch_i64_eq);
    test_for(ValueType::I64, "lt_s", Instruction::branch_i64_ge_s);
    test_for(ValueType::I64, "lt_u", Instruction::branch_i64_ge_u);
    test_for(ValueType::I64, "le_s", Instruction::branch_i64_gt_s);
    test_for(ValueType::I64, "le_u", Instruction::branch_i64_gt_u);
    test_for(ValueType::I64, "gt_s", Instruction::branch_i64_le_s);
    test_for(ValueType::I64, "gt_u", Instruction::branch_i64_le_u);
    test_for(ValueType::I64, "ge_s", Instruction::branch_i64_lt_s);
    test_for(ValueType::I64, "ge_u", Instruction::branch_i64_lt_u);
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward() {
    fn test_for(
        ty: ValueType,
        op: &str,
        expect_instr: fn(Register, Register, BranchOffset16) -> Instruction,
    ) {
        let ty = DisplayValueType::from(ty);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param {ty} {ty})
                    (if
                        ({ty}.{op}
                            (local.get 0)
                            (local.get 1)
                        )
                        (then)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    BranchOffset16::from(1),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for(ValueType::I32, "and", Instruction::branch_i32_and_eqz);
    test_for(ValueType::I32, "or", Instruction::branch_i32_or_eqz);
    test_for(ValueType::I32, "xor", Instruction::branch_i32_xor_eqz);
    test_for(ValueType::I32, "eq", Instruction::branch_i32_ne);
    test_for(ValueType::I32, "ne", Instruction::branch_i32_eq);
    test_for(ValueType::I32, "lt_s", Instruction::branch_i32_ge_s);
    test_for(ValueType::I32, "lt_u", Instruction::branch_i32_ge_u);
    test_for(ValueType::I32, "le_s", Instruction::branch_i32_gt_s);
    test_for(ValueType::I32, "le_u", Instruction::branch_i32_gt_u);
    test_for(ValueType::I32, "gt_s", Instruction::branch_i32_le_s);
    test_for(ValueType::I32, "gt_u", Instruction::branch_i32_le_u);
    test_for(ValueType::I32, "ge_s", Instruction::branch_i32_lt_s);
    test_for(ValueType::I32, "ge_u", Instruction::branch_i32_lt_u);

    test_for(ValueType::I64, "eq", Instruction::branch_i64_ne);
    test_for(ValueType::I64, "ne", Instruction::branch_i64_eq);
    test_for(ValueType::I64, "lt_s", Instruction::branch_i64_ge_s);
    test_for(ValueType::I64, "lt_u", Instruction::branch_i64_ge_u);
    test_for(ValueType::I64, "le_s", Instruction::branch_i64_gt_s);
    test_for(ValueType::I64, "le_u", Instruction::branch_i64_gt_u);
    test_for(ValueType::I64, "gt_s", Instruction::branch_i64_le_s);
    test_for(ValueType::I64, "gt_u", Instruction::branch_i64_le_u);
    test_for(ValueType::I64, "ge_s", Instruction::branch_i64_lt_s);
    test_for(ValueType::I64, "ge_u", Instruction::branch_i64_lt_u);
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_i32_eqz_fuse() {
    fn test_for(op: &str, expect_instr: fn(Register, Register, BranchOffset16) -> Instruction) {
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32 i32)
                    (block
                        (local.get 0)
                        (local.get 1)
                        (i32.{op})
                        (i32.eqz)
                        (br_if 0)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    BranchOffset16::from(1),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for("and", Instruction::branch_i32_and_eqz);
    test_for("or", Instruction::branch_i32_or_eqz);
    test_for("xor", Instruction::branch_i32_xor_eqz);
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_i32_eqz_fuse() {
    fn test_for(op: &str, expect_instr: fn(Register, Register, BranchOffset16) -> Instruction) {
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32 i32)
                    (if
                        (i32.eqz (i32.{op} (local.get 0) (local.get 1)))
                        (then)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    BranchOffset16::from(1),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for("and", Instruction::branch_i32_and);
    test_for("or", Instruction::branch_i32_or);
    test_for("xor", Instruction::branch_i32_xor);
}
