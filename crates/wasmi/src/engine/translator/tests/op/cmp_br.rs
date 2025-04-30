use super::{wasm_type::WasmTy, *};
use crate::{
    core::{UntypedVal, ValType},
    ir::{index::Global, BranchOffset, BranchOffset16, Comparator, ComparatorAndOffset},
};
use std::{
    fmt,
    fmt::{Debug, Display},
    string::String,
};

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward() {
    fn test_for(ty: ValType, op: &str, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let ty = DisplayValueType::from(ty);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(0), Reg::from(1), BranchOffset16::from(0)),
                Instruction::Return,
            ])
            .run()
    }

    test_for(ValType::I32, "and", Instruction::branch_i32_and);
    test_for(ValType::I32, "or", Instruction::branch_i32_or);
    test_for(ValType::I32, "xor", Instruction::branch_i32_xor);
    test_for(ValType::I32, "eq", Instruction::branch_i32_eq);
    test_for(ValType::I32, "ne", Instruction::branch_i32_ne);
    test_for(ValType::I32, "lt_s", Instruction::branch_i32_lt_s);
    test_for(ValType::I32, "lt_u", Instruction::branch_i32_lt_u);
    test_for(ValType::I32, "le_s", Instruction::branch_i32_le_s);
    test_for(ValType::I32, "le_u", Instruction::branch_i32_le_u);
    test_for(
        ValType::I32,
        "gt_s",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        ValType::I32,
        "gt_u",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(
        ValType::I32,
        "ge_s",
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        ValType::I32,
        "ge_u",
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );

    test_for(ValType::I64, "eq", Instruction::branch_i64_eq);
    test_for(ValType::I64, "ne", Instruction::branch_i64_ne);
    test_for(ValType::I64, "lt_s", Instruction::branch_i64_lt_s);
    test_for(ValType::I64, "lt_u", Instruction::branch_i64_lt_u);
    test_for(ValType::I64, "le_s", Instruction::branch_i64_le_s);
    test_for(ValType::I64, "le_u", Instruction::branch_i64_le_u);
    test_for(
        ValType::I64,
        "gt_s",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        ValType::I64,
        "gt_u",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(
        ValType::I64,
        "ge_s",
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        ValType::I64,
        "ge_u",
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );

    test_for(ValType::F32, "eq", Instruction::branch_f32_eq);
    test_for(ValType::F32, "ne", Instruction::branch_f32_ne);
    test_for(ValType::F32, "lt", Instruction::branch_f32_lt);
    test_for(ValType::F32, "le", Instruction::branch_f32_le);
    test_for(
        ValType::F32,
        "gt",
        swap_cmp_br_ops!(Instruction::branch_f32_lt),
    );
    test_for(
        ValType::F32,
        "ge",
        swap_cmp_br_ops!(Instruction::branch_f32_le),
    );

    test_for(ValType::F64, "eq", Instruction::branch_f64_eq);
    test_for(ValType::F64, "ne", Instruction::branch_f64_ne);
    test_for(ValType::F64, "lt", Instruction::branch_f64_lt);
    test_for(ValType::F64, "le", Instruction::branch_f64_le);
    test_for(
        ValType::F64,
        "gt",
        swap_cmp_br_ops!(Instruction::branch_f64_lt),
    );
    test_for(
        ValType::F64,
        "ge",
        swap_cmp_br_ops!(Instruction::branch_f64_le),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm_rhs() {
    fn test_for<T>(
        op: &str,
        value: T,
        expect_instr: fn(Reg, Const16<T>, BranchOffset16) -> Instruction,
    ) where
        T: WasmTy,
        Const16<T>: TryFrom<T> + Debug,
        DisplayWasm<T>: Display,
    {
        let ty = T::NAME;
        let display_value = DisplayWasm::from(value);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(
                    Reg::from(0),
                    <Const16<T>>::try_from(value).ok().unwrap(),
                    BranchOffset16::from(0),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for::<i32>("and", 1, Instruction::branch_i32_and_imm16);
    test_for::<i32>("or", 1, Instruction::branch_i32_or_imm16);
    test_for::<i32>("xor", 1, Instruction::branch_i32_xor_imm16);
    test_for::<i32>("eq", 1, Instruction::branch_i32_eq_imm16);
    test_for::<i32>("ne", 1, Instruction::branch_i32_ne_imm16);
    test_for::<i32>("lt_s", 1, Instruction::branch_i32_lt_s_imm16_rhs);
    test_for::<u32>("lt_u", 1, Instruction::branch_i32_lt_u_imm16_rhs);
    test_for::<i32>("le_s", 1, Instruction::branch_i32_le_s_imm16_rhs);
    test_for::<u32>("le_u", 1, Instruction::branch_i32_le_u_imm16_rhs);
    test_for::<i32>(
        "gt_s",
        1,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s_imm16_lhs),
    );
    test_for::<u32>(
        "gt_u",
        1,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u_imm16_lhs),
    );
    test_for::<i32>(
        "ge_s",
        1,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s_imm16_lhs),
    );
    test_for::<u32>(
        "ge_u",
        1,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u_imm16_lhs),
    );

    test_for::<i64>("eq", 1, Instruction::branch_i64_eq_imm16);
    test_for::<i64>("ne", 1, Instruction::branch_i64_ne_imm16);
    test_for::<i64>("lt_s", 1, Instruction::branch_i64_lt_s_imm16_rhs);
    test_for::<u64>("lt_u", 1, Instruction::branch_i64_lt_u_imm16_rhs);
    test_for::<i64>("le_s", 1, Instruction::branch_i64_le_s_imm16_rhs);
    test_for::<u64>("le_u", 1, Instruction::branch_i64_le_u_imm16_rhs);
    test_for::<i64>(
        "gt_s",
        1,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s_imm16_lhs),
    );
    test_for::<u64>(
        "gt_u",
        1,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u_imm16_lhs),
    );
    test_for::<i64>(
        "ge_s",
        1,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s_imm16_lhs),
    );
    test_for::<u64>(
        "ge_u",
        1,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u_imm16_lhs),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm_lhs() {
    fn test_for<T>(
        op: &str,
        value: T,
        expect_instr: fn(Reg, Const16<T>, BranchOffset16) -> Instruction,
    ) where
        T: WasmTy,
        Const16<T>: TryFrom<T> + Debug,
        DisplayWasm<T>: Display,
    {
        let ty = T::NAME;
        let display_value = DisplayWasm::from(value);
        let wasm = format!(
            r"
            (module
                (func (param {ty} {ty})
                    (loop
                        ({ty}.const {display_value})
                        (local.get 0)
                        ({ty}.{op})
                        (br_if 0)
                    )
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(
                    Reg::from(0),
                    <Const16<T>>::try_from(value).ok().unwrap(),
                    BranchOffset16::from(0),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for::<i32>("and", 1, Instruction::branch_i32_and_imm16);
    test_for::<i32>("or", 1, Instruction::branch_i32_or_imm16);
    test_for::<i32>("xor", 1, Instruction::branch_i32_xor_imm16);
    test_for::<i32>("eq", 1, Instruction::branch_i32_eq_imm16);
    test_for::<i32>("ne", 1, Instruction::branch_i32_ne_imm16);
    test_for::<i32>(
        "lt_s",
        1,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s_imm16_lhs),
    );
    test_for::<u32>(
        "lt_u",
        1,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u_imm16_lhs),
    );
    test_for::<i32>(
        "le_s",
        1,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s_imm16_lhs),
    );
    test_for::<u32>(
        "le_u",
        1,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u_imm16_lhs),
    );
    test_for::<i32>("gt_s", 1, Instruction::branch_i32_lt_s_imm16_rhs);
    test_for::<u32>("gt_u", 1, Instruction::branch_i32_lt_u_imm16_rhs);
    test_for::<i32>("ge_s", 1, Instruction::branch_i32_le_s_imm16_rhs);
    test_for::<u32>("ge_u", 1, Instruction::branch_i32_le_u_imm16_rhs);

    test_for::<i64>("eq", 1, Instruction::branch_i64_eq_imm16);
    test_for::<i64>("ne", 1, Instruction::branch_i64_ne_imm16);
    test_for::<i64>(
        "lt_s",
        1,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s_imm16_lhs),
    );
    test_for::<u64>(
        "lt_u",
        1,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u_imm16_lhs),
    );
    test_for::<i64>(
        "le_s",
        1,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s_imm16_lhs),
    );
    test_for::<u64>(
        "le_u",
        1,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u_imm16_lhs),
    );
    test_for::<i64>("gt_s", 1, Instruction::branch_i64_lt_s_imm16_rhs);
    test_for::<u64>("gt_u", 1, Instruction::branch_i64_lt_u_imm16_rhs);
    test_for::<i64>("ge_s", 1, Instruction::branch_i64_le_s_imm16_rhs);
    test_for::<u64>("ge_u", 1, Instruction::branch_i64_le_u_imm16_rhs);
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward() {
    fn test_for(ty: ValType, op: &str, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let ty = DisplayValueType::from(ty);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(0), Reg::from(1), BranchOffset16::from(1)),
                Instruction::Return,
            ])
            .run()
    }

    test_for(ValType::I32, "and", Instruction::branch_i32_and);
    test_for(ValType::I32, "or", Instruction::branch_i32_or);
    test_for(ValType::I32, "xor", Instruction::branch_i32_xor);
    test_for(ValType::I32, "eq", Instruction::branch_i32_eq);
    test_for(ValType::I32, "ne", Instruction::branch_i32_ne);
    test_for(ValType::I32, "lt_s", Instruction::branch_i32_lt_s);
    test_for(ValType::I32, "lt_u", Instruction::branch_i32_lt_u);
    test_for(ValType::I32, "le_s", Instruction::branch_i32_le_s);
    test_for(ValType::I32, "le_u", Instruction::branch_i32_le_u);
    test_for(
        ValType::I32,
        "gt_s",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        ValType::I32,
        "gt_u",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(
        ValType::I32,
        "ge_s",
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        ValType::I32,
        "ge_u",
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );

    test_for(ValType::I64, "eq", Instruction::branch_i64_eq);
    test_for(ValType::I64, "ne", Instruction::branch_i64_ne);
    test_for(ValType::I64, "lt_s", Instruction::branch_i64_lt_s);
    test_for(ValType::I64, "lt_u", Instruction::branch_i64_lt_u);
    test_for(ValType::I64, "le_s", Instruction::branch_i64_le_s);
    test_for(ValType::I64, "le_u", Instruction::branch_i64_le_u);
    test_for(
        ValType::I64,
        "gt_s",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        ValType::I64,
        "gt_u",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(
        ValType::I64,
        "ge_s",
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        ValType::I64,
        "ge_u",
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );

    test_for(ValType::F32, "eq", Instruction::branch_f32_eq);
    test_for(ValType::F32, "ne", Instruction::branch_f32_ne);
    test_for(ValType::F32, "lt", Instruction::branch_f32_lt);
    test_for(ValType::F32, "le", Instruction::branch_f32_le);
    test_for(
        ValType::F32,
        "gt",
        swap_cmp_br_ops!(Instruction::branch_f32_lt),
    );
    test_for(
        ValType::F32,
        "ge",
        swap_cmp_br_ops!(Instruction::branch_f32_le),
    );

    test_for(ValType::F64, "eq", Instruction::branch_f64_eq);
    test_for(ValType::F64, "ne", Instruction::branch_f64_ne);
    test_for(ValType::F64, "lt", Instruction::branch_f64_lt);
    test_for(ValType::F64, "le", Instruction::branch_f64_le);
    test_for(
        ValType::F64,
        "gt",
        swap_cmp_br_ops!(Instruction::branch_f64_lt),
    );
    test_for(
        ValType::F64,
        "ge",
        swap_cmp_br_ops!(Instruction::branch_f64_le),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward_nop_copy() {
    fn test_for(ty: ValType, op: &str, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let ty = DisplayValueType::from(ty);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::global_get(Reg::from(2), Global::from(0)),
                expect_instr(Reg::from(0), Reg::from(1), BranchOffset16::from(2)),
                Instruction::copy(Reg::from(2), Reg::from(0)),
                Instruction::return_reg(2),
            ])
            .run()
    }

    test_for(ValType::I32, "and", Instruction::branch_i32_and);
    test_for(ValType::I32, "or", Instruction::branch_i32_or);
    test_for(ValType::I32, "xor", Instruction::branch_i32_xor);
    test_for(ValType::I32, "eq", Instruction::branch_i32_eq);
    test_for(ValType::I32, "ne", Instruction::branch_i32_ne);
    test_for(ValType::I32, "lt_s", Instruction::branch_i32_lt_s);
    test_for(ValType::I32, "lt_u", Instruction::branch_i32_lt_u);
    test_for(ValType::I32, "le_s", Instruction::branch_i32_le_s);
    test_for(ValType::I32, "le_u", Instruction::branch_i32_le_u);
    test_for(
        ValType::I32,
        "gt_s",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        ValType::I32,
        "gt_u",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(
        ValType::I32,
        "ge_s",
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        ValType::I32,
        "ge_u",
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );

    test_for(ValType::I64, "eq", Instruction::branch_i64_eq);
    test_for(ValType::I64, "ne", Instruction::branch_i64_ne);
    test_for(ValType::I64, "lt_s", Instruction::branch_i64_lt_s);
    test_for(ValType::I64, "lt_u", Instruction::branch_i64_lt_u);
    test_for(ValType::I64, "le_s", Instruction::branch_i64_le_s);
    test_for(ValType::I64, "le_u", Instruction::branch_i64_le_u);
    test_for(
        ValType::I64,
        "gt_s",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        ValType::I64,
        "gt_u",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(
        ValType::I64,
        "ge_s",
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        ValType::I64,
        "ge_u",
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );

    test_for(ValType::F32, "eq", Instruction::branch_f32_eq);
    test_for(ValType::F32, "ne", Instruction::branch_f32_ne);
    test_for(ValType::F32, "lt", Instruction::branch_f32_lt);
    test_for(ValType::F32, "le", Instruction::branch_f32_le);
    test_for(
        ValType::F32,
        "gt",
        swap_cmp_br_ops!(Instruction::branch_f32_lt),
    );
    test_for(
        ValType::F32,
        "ge",
        swap_cmp_br_ops!(Instruction::branch_f32_le),
    );

    test_for(ValType::F64, "eq", Instruction::branch_f64_eq);
    test_for(ValType::F64, "ne", Instruction::branch_f64_ne);
    test_for(ValType::F64, "lt", Instruction::branch_f64_lt);
    test_for(ValType::F64, "le", Instruction::branch_f64_le);
    test_for(
        ValType::F64,
        "gt",
        swap_cmp_br_ops!(Instruction::branch_f64_lt),
    );
    test_for(
        ValType::F64,
        "ge",
        swap_cmp_br_ops!(Instruction::branch_f64_le),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward_multi_value() {
    fn test_for(ty: ValType, op: &str, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let ty = DisplayValueType::from(ty);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(0), Reg::from(1), BranchOffset16::from(3)),
                Instruction::copy(Reg::from(2), Reg::from(0)),
                Instruction::branch(BranchOffset::from(2)),
                Instruction::copy(Reg::from(2), Reg::from(1)),
                Instruction::return_reg(2),
            ])
            .run()
    }

    test_for(ValType::I32, "and", Instruction::branch_i32_and_eqz);
    test_for(ValType::I32, "or", Instruction::branch_i32_or_eqz);
    test_for(ValType::I32, "xor", Instruction::branch_i32_xor_eqz);
    test_for(ValType::I32, "eq", Instruction::branch_i32_ne);
    test_for(ValType::I32, "ne", Instruction::branch_i32_eq);
    test_for(
        ValType::I32,
        "lt_s",
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        ValType::I32,
        "lt_u",
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );
    test_for(
        ValType::I32,
        "le_s",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        ValType::I32,
        "le_u",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(ValType::I32, "gt_s", Instruction::branch_i32_le_s);
    test_for(ValType::I32, "gt_u", Instruction::branch_i32_le_u);
    test_for(ValType::I32, "ge_s", Instruction::branch_i32_lt_s);
    test_for(ValType::I32, "ge_u", Instruction::branch_i32_lt_u);

    test_for(ValType::I64, "eq", Instruction::branch_i64_ne);
    test_for(ValType::I64, "ne", Instruction::branch_i64_eq);
    test_for(
        ValType::I64,
        "lt_s",
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        ValType::I64,
        "lt_u",
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );
    test_for(
        ValType::I64,
        "le_s",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        ValType::I64,
        "le_u",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(ValType::I64, "gt_s", Instruction::branch_i64_le_s);
    test_for(ValType::I64, "gt_u", Instruction::branch_i64_le_u);
    test_for(ValType::I64, "ge_s", Instruction::branch_i64_lt_s);
    test_for(ValType::I64, "ge_u", Instruction::branch_i64_lt_u);
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward() {
    fn test_for(ty: ValType, op: &str, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let ty = DisplayValueType::from(ty);
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(0), Reg::from(1), BranchOffset16::from(1)),
                Instruction::Return,
            ])
            .run()
    }

    test_for(ValType::I32, "and", Instruction::branch_i32_and_eqz);
    test_for(ValType::I32, "or", Instruction::branch_i32_or_eqz);
    test_for(ValType::I32, "xor", Instruction::branch_i32_xor_eqz);
    test_for(ValType::I32, "eq", Instruction::branch_i32_ne);
    test_for(ValType::I32, "ne", Instruction::branch_i32_eq);
    test_for(
        ValType::I32,
        "lt_s",
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        ValType::I32,
        "lt_u",
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );
    test_for(
        ValType::I32,
        "le_s",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        ValType::I32,
        "le_u",
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(ValType::I32, "gt_s", Instruction::branch_i32_le_s);
    test_for(ValType::I32, "gt_u", Instruction::branch_i32_le_u);
    test_for(ValType::I32, "ge_s", Instruction::branch_i32_lt_s);
    test_for(ValType::I32, "ge_u", Instruction::branch_i32_lt_u);

    test_for(ValType::I64, "eq", Instruction::branch_i64_ne);
    test_for(ValType::I64, "ne", Instruction::branch_i64_eq);
    test_for(
        ValType::I64,
        "lt_s",
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        ValType::I64,
        "lt_u",
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );
    test_for(
        ValType::I64,
        "le_s",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        ValType::I64,
        "le_u",
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(ValType::I64, "gt_s", Instruction::branch_i64_le_s);
    test_for(ValType::I64, "gt_u", Instruction::branch_i64_le_u);
    test_for(ValType::I64, "ge_s", Instruction::branch_i64_lt_s);
    test_for(ValType::I64, "ge_u", Instruction::branch_i64_lt_u);
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_i32_eqz_fuse() {
    fn test_for(op: &str, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let wasm = format!(
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
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(0), Reg::from(1), BranchOffset16::from(1)),
                Instruction::Return,
            ])
            .run()
    }

    test_for("eq", Instruction::branch_i32_ne);
    test_for("ne", Instruction::branch_i32_eq);
    test_for("and", Instruction::branch_i32_and_eqz);
    test_for("or", Instruction::branch_i32_or_eqz);
    test_for("xor", Instruction::branch_i32_xor_eqz);
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_i32_eqz_fuse() {
    fn test_for(op: &str, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let wasm = format!(
            r"
            (module
                (func (param i32 i32)
                    (if
                        (i32.eqz (i32.{op} (local.get 0) (local.get 1)))
                        (then)
                    )
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(Reg::from(0), Reg::from(1), BranchOffset16::from(1)),
                Instruction::Return,
            ])
            .run()
    }

    test_for("eq", Instruction::branch_i32_eq);
    test_for("ne", Instruction::branch_i32_ne);
    test_for("and", Instruction::branch_i32_and);
    test_for("or", Instruction::branch_i32_or);
    test_for("xor", Instruction::branch_i32_xor);
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_i64_eqz_fuse() {
    let wasm = r"
        (module
            (func (param i64)
                (block
                    (i64.eqz (local.get 0))
                    (br_if 0)
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i64_eq_imm16(Reg::from(0), 0, BranchOffset16::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_i64_eqz_fuse() {
    let wasm = r"
        (module
            (func (param i64)
                (if
                    (i64.eqz (local.get 0))
                    (then)
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i64_ne_imm16(Reg::from(0), 0, BranchOffset16::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn cmp_br_fallback() {
    // Required amount of instructions to trigger the `cmp+br` fallback instruction generation.
    let len_adds = (1 << 15) + 1;
    let wasm = generate_cmp_br_fallback_wasm(len_adds).unwrap();
    let expected_instrs = {
        let mut instrs = std::vec![
            Instruction::branch_cmp_fallback(0, -1, -3),
            Instruction::i32_add_imm16(1, 0, 1),
        ];
        instrs.extend((0..(len_adds - 2)).map(|_| Instruction::i32_add_imm16(1, 1, 1)));
        instrs.extend([
            Instruction::i32_add_imm16(0, 1, 1),
            Instruction::branch_cmp_fallback(0, -1, -2),
            Instruction::r#return(),
        ]);
        instrs
    };
    let offset = len_adds as i32 + 1;
    let param0: ComparatorAndOffset =
        ComparatorAndOffset::new(Comparator::I32Ne, BranchOffset::from(offset));
    let param1 = ComparatorAndOffset::new(Comparator::I32Ne, BranchOffset::from(-offset));
    TranslationTest::new(&wasm)
        .expect_func(ExpectedFunc::new(expected_instrs).consts([
            UntypedVal::from(0_i64),  // reg(-1)
            UntypedVal::from(param1), // reg(-2)
            UntypedVal::from(param0), // reg(-3)
        ]))
        .run()
}

fn generate_cmp_br_fallback_wasm(len_adds: usize) -> Result<String, fmt::Error> {
    use fmt::Write as _;
    let mut wasm = String::new();
    writeln!(
        wasm,
        r"
        (module
            (func (param i32)
                (loop $continue
                    (block $skip
                        (br_if $skip (local.get 0))

                        (local.get 0)"
    )?;
    for _ in 0..len_adds {
        writeln!(
            wasm,
            "\
                        (i32.add (i32.const 1))"
        )?;
    }
    writeln!(
        wasm,
        "\
                        (local.set 0)
                    )
                    (br_if $continue (local.get 0))
                )
            )
        )"
    )?;
    Ok(wasm)
}
