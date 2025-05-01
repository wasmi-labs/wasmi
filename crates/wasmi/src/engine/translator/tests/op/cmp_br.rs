use super::{wasm_type::WasmTy, *};
use crate::{
    core::{UntypedVal, ValType},
    ir::{index::Global, BranchOffset, BranchOffset16, Comparator, ComparatorAndOffset},
};
use std::{fmt, fmt::Debug, string::String};

#[derive(Debug, Copy, Clone)]
pub enum CmpOp {
    // i32
    I32And,
    I32Or,
    I32Xor,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32LeS,
    I32LeU,
    I32GtS,
    I32GtU,
    I32GeS,
    I32GeU,
    // i64
    I64And,
    I64Or,
    I64Xor,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64LeS,
    I64LeU,
    I64GtS,
    I64GtU,
    I64GeS,
    I64GeU,
    // f32
    F32Eq,
    F32Ne,
    F32Lt,
    F32Le,
    F32Gt,
    F32Ge,
    // f64
    F64Eq,
    F64Ne,
    F64Lt,
    F64Le,
    F64Gt,
    F64Ge,
}

impl CmpOp {
    /// Returns the Wasm parameter type of the [`CmpOp`].
    pub fn param_ty(self) -> ValType {
        match self {
            CmpOp::I32And
            | CmpOp::I32Or
            | CmpOp::I32Xor
            | CmpOp::I32Eq
            | CmpOp::I32Ne
            | CmpOp::I32LtS
            | CmpOp::I32LtU
            | CmpOp::I32LeS
            | CmpOp::I32LeU
            | CmpOp::I32GtS
            | CmpOp::I32GtU
            | CmpOp::I32GeS
            | CmpOp::I32GeU => ValType::I32,
            CmpOp::I64And
            | CmpOp::I64Or
            | CmpOp::I64Xor
            | CmpOp::I64Eq
            | CmpOp::I64Ne
            | CmpOp::I64LtS
            | CmpOp::I64LtU
            | CmpOp::I64LeS
            | CmpOp::I64LeU
            | CmpOp::I64GtS
            | CmpOp::I64GtU
            | CmpOp::I64GeS
            | CmpOp::I64GeU => ValType::I64,
            CmpOp::F32Eq
            | CmpOp::F32Ne
            | CmpOp::F32Lt
            | CmpOp::F32Le
            | CmpOp::F32Gt
            | CmpOp::F32Ge => ValType::F32,
            CmpOp::F64Eq
            | CmpOp::F64Ne
            | CmpOp::F64Lt
            | CmpOp::F64Le
            | CmpOp::F64Gt
            | CmpOp::F64Ge => ValType::F64,
        }
    }

    /// Returns the Wasm result type of the [`CmpOp`].
    pub fn result_ty(self) -> ValType {
        match self {
            CmpOp::I64And | CmpOp::I64Or | CmpOp::I64Xor => ValType::I64,
            _ => ValType::I32,
        }
    }

    /// Returns a string representation of the Wasm operator without type annotation.
    pub fn op_str(self) -> &'static str {
        match self {
            CmpOp::I32And => "and",
            CmpOp::I32Or => "or",
            CmpOp::I32Xor => "xor",
            CmpOp::I32Eq => "eq",
            CmpOp::I32Ne => "ne",
            CmpOp::I32LtS => "lt_s",
            CmpOp::I32LtU => "lt_u",
            CmpOp::I32LeS => "le_s",
            CmpOp::I32LeU => "le_u",
            CmpOp::I32GtS => "gt_s",
            CmpOp::I32GtU => "gt_u",
            CmpOp::I32GeS => "ge_s",
            CmpOp::I32GeU => "ge_u",
            CmpOp::I64And => "and",
            CmpOp::I64Or => "or",
            CmpOp::I64Xor => "xor",
            CmpOp::I64Eq => "eq",
            CmpOp::I64Ne => "ne",
            CmpOp::I64LtS => "lt_s",
            CmpOp::I64LtU => "lt_u",
            CmpOp::I64LeS => "le_s",
            CmpOp::I64LeU => "le_u",
            CmpOp::I64GtS => "gt_s",
            CmpOp::I64GtU => "gt_u",
            CmpOp::I64GeS => "ge_s",
            CmpOp::I64GeU => "ge_u",
            CmpOp::F32Eq => "eq",
            CmpOp::F32Ne => "ne",
            CmpOp::F32Lt => "lt",
            CmpOp::F32Le => "le",
            CmpOp::F32Gt => "gt",
            CmpOp::F32Ge => "ge",
            CmpOp::F64Eq => "eq",
            CmpOp::F64Ne => "ne",
            CmpOp::F64Lt => "lt",
            CmpOp::F64Le => "le",
            CmpOp::F64Gt => "gt",
            CmpOp::F64Ge => "ge",
        }
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward() {
    fn test_for(op: CmpOp, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (func (param {input_ty} {input_ty})
                    (loop $continue
                        local.get 0
                        local.get 1
                        {input_ty}.{op_str}
                        {result_ty}.const 0
                        {result_ty}.ne
                        br_if $continue
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

    test_for(CmpOp::I32And, Instruction::branch_i32_and);
    test_for(CmpOp::I32Or, Instruction::branch_i32_or);
    test_for(CmpOp::I32Xor, Instruction::branch_i32_xor);
    test_for(CmpOp::I32Eq, Instruction::branch_i32_eq);
    test_for(CmpOp::I32Ne, Instruction::branch_i32_ne);
    test_for(CmpOp::I32LtS, Instruction::branch_i32_lt_s);
    test_for(CmpOp::I32LtU, Instruction::branch_i32_lt_u);
    test_for(CmpOp::I32LeS, Instruction::branch_i32_le_s);
    test_for(CmpOp::I32LeU, Instruction::branch_i32_le_u);
    test_for(
        CmpOp::I32GtS,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        CmpOp::I32GtU,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(
        CmpOp::I32GeS,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        CmpOp::I32GeU,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );

    test_for(CmpOp::I64And, Instruction::branch_i64_and);
    test_for(CmpOp::I64Or, Instruction::branch_i64_or);
    test_for(CmpOp::I64Xor, Instruction::branch_i64_xor);
    test_for(CmpOp::I64Eq, Instruction::branch_i64_eq);
    test_for(CmpOp::I64Ne, Instruction::branch_i64_ne);
    test_for(CmpOp::I64LtS, Instruction::branch_i64_lt_s);
    test_for(CmpOp::I64LtU, Instruction::branch_i64_lt_u);
    test_for(CmpOp::I64LeS, Instruction::branch_i64_le_s);
    test_for(CmpOp::I64LeU, Instruction::branch_i64_le_u);
    test_for(
        CmpOp::I64GtS,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        CmpOp::I64GtU,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(
        CmpOp::I64GeS,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        CmpOp::I64GeU,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );

    test_for(CmpOp::F32Eq, Instruction::branch_f32_eq);
    test_for(CmpOp::F32Ne, Instruction::branch_f32_ne);
    test_for(CmpOp::F32Lt, Instruction::branch_f32_lt);
    test_for(CmpOp::F32Le, Instruction::branch_f32_le);
    test_for(CmpOp::F32Gt, swap_cmp_br_ops!(Instruction::branch_f32_lt));
    test_for(CmpOp::F32Ge, swap_cmp_br_ops!(Instruction::branch_f32_le));

    test_for(CmpOp::F64Eq, Instruction::branch_f64_eq);
    test_for(CmpOp::F64Ne, Instruction::branch_f64_ne);
    test_for(CmpOp::F64Lt, Instruction::branch_f64_lt);
    test_for(CmpOp::F64Le, Instruction::branch_f64_le);
    test_for(CmpOp::F64Gt, swap_cmp_br_ops!(Instruction::branch_f64_lt));
    test_for(CmpOp::F64Ge, swap_cmp_br_ops!(Instruction::branch_f64_le));
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm_rhs() {
    fn test_for<T>(op: CmpOp, expect_instr: fn(Reg, Const16<T>, BranchOffset16) -> Instruction)
    where
        T: WasmTy,
        Const16<T>: TryFrom<T> + Debug,
    {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (func (param {input_ty} {input_ty})
                    (loop $continue
                        local.get 0
                        {input_ty}.const 1
                        {input_ty}.{op_str}
                        {result_ty}.const 0
                        {result_ty}.ne
                        br_if $continue
                    )
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(
                    Reg::from(0),
                    <Const16<T>>::try_from(T::from(1)).ok().unwrap(),
                    BranchOffset16::from(0),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for::<i32>(CmpOp::I32And, Instruction::branch_i32_and_imm16);
    test_for::<i32>(CmpOp::I32Or, Instruction::branch_i32_or_imm16);
    test_for::<i32>(CmpOp::I32Xor, Instruction::branch_i32_xor_imm16);
    test_for::<i32>(CmpOp::I32Eq, Instruction::branch_i32_eq_imm16);
    test_for::<i32>(CmpOp::I32Ne, Instruction::branch_i32_ne_imm16);
    test_for::<i32>(CmpOp::I32LtS, Instruction::branch_i32_lt_s_imm16_rhs);
    test_for::<u32>(CmpOp::I32LtU, Instruction::branch_i32_lt_u_imm16_rhs);
    test_for::<i32>(CmpOp::I32LeS, Instruction::branch_i32_le_s_imm16_rhs);
    test_for::<u32>(CmpOp::I32LeU, Instruction::branch_i32_le_u_imm16_rhs);
    test_for::<i32>(
        CmpOp::I32GtS,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s_imm16_lhs),
    );
    test_for::<u32>(
        CmpOp::I32GtU,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u_imm16_lhs),
    );
    test_for::<i32>(
        CmpOp::I32GeS,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s_imm16_lhs),
    );
    test_for::<u32>(
        CmpOp::I32GeU,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u_imm16_lhs),
    );
    test_for::<i64>(CmpOp::I64And, Instruction::branch_i64_and_imm16);
    test_for::<i64>(CmpOp::I64Or, Instruction::branch_i64_or_imm16);
    test_for::<i64>(CmpOp::I64Xor, Instruction::branch_i64_xor_imm16);
    test_for::<i64>(CmpOp::I64Eq, Instruction::branch_i64_eq_imm16);
    test_for::<i64>(CmpOp::I64Ne, Instruction::branch_i64_ne_imm16);
    test_for::<i64>(CmpOp::I64LtS, Instruction::branch_i64_lt_s_imm16_rhs);
    test_for::<u64>(CmpOp::I64LtU, Instruction::branch_i64_lt_u_imm16_rhs);
    test_for::<i64>(CmpOp::I64LeS, Instruction::branch_i64_le_s_imm16_rhs);
    test_for::<u64>(CmpOp::I64LeU, Instruction::branch_i64_le_u_imm16_rhs);
    test_for::<i64>(
        CmpOp::I64GtS,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s_imm16_lhs),
    );
    test_for::<u64>(
        CmpOp::I64GtU,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u_imm16_lhs),
    );
    test_for::<i64>(
        CmpOp::I64GeS,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s_imm16_lhs),
    );
    test_for::<u64>(
        CmpOp::I64GeU,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u_imm16_lhs),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm_lhs() {
    fn test_for<T>(op: CmpOp, expect_instr: fn(Reg, Const16<T>, BranchOffset16) -> Instruction)
    where
        T: WasmTy,
        Const16<T>: TryFrom<T> + Debug,
    {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (func (param {input_ty} {input_ty})
                    (loop $continue
                        {input_ty}.const 1
                        local.get 0
                        {input_ty}.{op_str}
                        {result_ty}.const 0
                        {result_ty}.ne
                        br_if $continue
                    )
                )
            )",
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expect_instr(
                    Reg::from(0),
                    <Const16<T>>::try_from(T::from(1)).ok().unwrap(),
                    BranchOffset16::from(0),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for::<i32>(CmpOp::I32And, Instruction::branch_i32_and_imm16);
    test_for::<i32>(CmpOp::I32Or, Instruction::branch_i32_or_imm16);
    test_for::<i32>(CmpOp::I32Xor, Instruction::branch_i32_xor_imm16);
    test_for::<i32>(CmpOp::I32Eq, Instruction::branch_i32_eq_imm16);
    test_for::<i32>(CmpOp::I32Ne, Instruction::branch_i32_ne_imm16);
    test_for::<i32>(
        CmpOp::I32LtS,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s_imm16_lhs),
    );
    test_for::<u32>(
        CmpOp::I32LtU,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u_imm16_lhs),
    );
    test_for::<i32>(
        CmpOp::I32LeS,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s_imm16_lhs),
    );
    test_for::<u32>(
        CmpOp::I32LeU,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u_imm16_lhs),
    );
    test_for::<i32>(CmpOp::I32GtS, Instruction::branch_i32_lt_s_imm16_rhs);
    test_for::<u32>(CmpOp::I32GtU, Instruction::branch_i32_lt_u_imm16_rhs);
    test_for::<i32>(CmpOp::I32GeS, Instruction::branch_i32_le_s_imm16_rhs);
    test_for::<u32>(CmpOp::I32GeU, Instruction::branch_i32_le_u_imm16_rhs);

    test_for::<i64>(CmpOp::I64And, Instruction::branch_i64_and_imm16);
    test_for::<i64>(CmpOp::I64Or, Instruction::branch_i64_or_imm16);
    test_for::<i64>(CmpOp::I64Xor, Instruction::branch_i64_xor_imm16);
    test_for::<i64>(CmpOp::I64Eq, Instruction::branch_i64_eq_imm16);
    test_for::<i64>(CmpOp::I64Ne, Instruction::branch_i64_ne_imm16);
    test_for::<i64>(
        CmpOp::I64LtS,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s_imm16_lhs),
    );
    test_for::<u64>(
        CmpOp::I64LtU,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u_imm16_lhs),
    );
    test_for::<i64>(
        CmpOp::I64LeS,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s_imm16_lhs),
    );
    test_for::<u64>(
        CmpOp::I64LeU,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u_imm16_lhs),
    );
    test_for::<i64>(CmpOp::I64GtS, Instruction::branch_i64_lt_s_imm16_rhs);
    test_for::<u64>(CmpOp::I64GtU, Instruction::branch_i64_lt_u_imm16_rhs);
    test_for::<i64>(CmpOp::I64GeS, Instruction::branch_i64_le_s_imm16_rhs);
    test_for::<u64>(CmpOp::I64GeU, Instruction::branch_i64_le_u_imm16_rhs);
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward() {
    fn test_for(op: CmpOp, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (func (param {input_ty} {input_ty})
                    (block $exit
                        local.get 0
                        local.get 1
                        {input_ty}.{op_str}
                        {result_ty}.const 0
                        {result_ty}.ne
                        br_if $exit
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

    test_for(CmpOp::I32And, Instruction::branch_i32_and);
    test_for(CmpOp::I32Or, Instruction::branch_i32_or);
    test_for(CmpOp::I32Xor, Instruction::branch_i32_xor);
    test_for(CmpOp::I32Eq, Instruction::branch_i32_eq);
    test_for(CmpOp::I32Ne, Instruction::branch_i32_ne);
    test_for(CmpOp::I32LtS, Instruction::branch_i32_lt_s);
    test_for(CmpOp::I32LtU, Instruction::branch_i32_lt_u);
    test_for(CmpOp::I32LeS, Instruction::branch_i32_le_s);
    test_for(CmpOp::I32LeU, Instruction::branch_i32_le_u);
    test_for(
        CmpOp::I32GtS,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        CmpOp::I32GtU,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(
        CmpOp::I32GeS,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        CmpOp::I32GeU,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );

    test_for(CmpOp::I64And, Instruction::branch_i64_and);
    test_for(CmpOp::I64Or, Instruction::branch_i64_or);
    test_for(CmpOp::I64Xor, Instruction::branch_i64_xor);
    test_for(CmpOp::I64Eq, Instruction::branch_i64_eq);
    test_for(CmpOp::I64Ne, Instruction::branch_i64_ne);
    test_for(CmpOp::I64LtS, Instruction::branch_i64_lt_s);
    test_for(CmpOp::I64LtU, Instruction::branch_i64_lt_u);
    test_for(CmpOp::I64LeS, Instruction::branch_i64_le_s);
    test_for(CmpOp::I64LeU, Instruction::branch_i64_le_u);
    test_for(
        CmpOp::I64GtS,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        CmpOp::I64GtU,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(
        CmpOp::I64GeS,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        CmpOp::I64GeU,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );

    test_for(CmpOp::F32Eq, Instruction::branch_f32_eq);
    test_for(CmpOp::F32Ne, Instruction::branch_f32_ne);
    test_for(CmpOp::F32Lt, Instruction::branch_f32_lt);
    test_for(CmpOp::F32Le, Instruction::branch_f32_le);
    test_for(CmpOp::F32Gt, swap_cmp_br_ops!(Instruction::branch_f32_lt));
    test_for(CmpOp::F32Ge, swap_cmp_br_ops!(Instruction::branch_f32_le));

    test_for(CmpOp::F64Eq, Instruction::branch_f64_eq);
    test_for(CmpOp::F64Ne, Instruction::branch_f64_ne);
    test_for(CmpOp::F64Lt, Instruction::branch_f64_lt);
    test_for(CmpOp::F64Le, Instruction::branch_f64_le);
    test_for(CmpOp::F64Gt, swap_cmp_br_ops!(Instruction::branch_f64_lt));
    test_for(CmpOp::F64Ge, swap_cmp_br_ops!(Instruction::branch_f64_le));
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward_nop_copy() {
    fn test_for(op: CmpOp, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (global $g (mut {input_ty}) ({input_ty}.const 0))
                (func (param {input_ty} {input_ty}) (result {input_ty})
                    global.get $g
                    (block $exit (param {input_ty}) (result {input_ty})
                        local.get 0
                        local.get 1
                        {input_ty}.{op_str}
                        {result_ty}.const 0
                        {result_ty}.ne
                        br_if $exit
                        drop
                        local.get 0
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

    test_for(CmpOp::I32And, Instruction::branch_i32_and);
    test_for(CmpOp::I32Or, Instruction::branch_i32_or);
    test_for(CmpOp::I32Xor, Instruction::branch_i32_xor);
    test_for(CmpOp::I32Eq, Instruction::branch_i32_eq);
    test_for(CmpOp::I32Ne, Instruction::branch_i32_ne);
    test_for(CmpOp::I32LtS, Instruction::branch_i32_lt_s);
    test_for(CmpOp::I32LtU, Instruction::branch_i32_lt_u);
    test_for(CmpOp::I32LeS, Instruction::branch_i32_le_s);
    test_for(CmpOp::I32LeU, Instruction::branch_i32_le_u);
    test_for(
        CmpOp::I32GtS,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        CmpOp::I32GtU,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(
        CmpOp::I32GeS,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        CmpOp::I32GeU,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );

    test_for(CmpOp::I64And, Instruction::branch_i64_and);
    test_for(CmpOp::I64Or, Instruction::branch_i64_or);
    test_for(CmpOp::I64Xor, Instruction::branch_i64_xor);
    test_for(CmpOp::I64Eq, Instruction::branch_i64_eq);
    test_for(CmpOp::I64Ne, Instruction::branch_i64_ne);
    test_for(CmpOp::I64LtS, Instruction::branch_i64_lt_s);
    test_for(CmpOp::I64LtU, Instruction::branch_i64_lt_u);
    test_for(CmpOp::I64LeS, Instruction::branch_i64_le_s);
    test_for(CmpOp::I64LeU, Instruction::branch_i64_le_u);
    test_for(
        CmpOp::I64GtS,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        CmpOp::I64GtU,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(
        CmpOp::I64GeS,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        CmpOp::I64GeU,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );

    test_for(CmpOp::F32Eq, Instruction::branch_f32_eq);
    test_for(CmpOp::F32Ne, Instruction::branch_f32_ne);
    test_for(CmpOp::F32Lt, Instruction::branch_f32_lt);
    test_for(CmpOp::F32Le, Instruction::branch_f32_le);
    test_for(CmpOp::F32Gt, swap_cmp_br_ops!(Instruction::branch_f32_lt));
    test_for(CmpOp::F32Ge, swap_cmp_br_ops!(Instruction::branch_f32_le));

    test_for(CmpOp::F64Eq, Instruction::branch_f64_eq);
    test_for(CmpOp::F64Ne, Instruction::branch_f64_ne);
    test_for(CmpOp::F64Lt, Instruction::branch_f64_lt);
    test_for(CmpOp::F64Le, Instruction::branch_f64_le);
    test_for(CmpOp::F64Gt, swap_cmp_br_ops!(Instruction::branch_f64_lt));
    test_for(CmpOp::F64Ge, swap_cmp_br_ops!(Instruction::branch_f64_le));
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward_multi_value() {
    fn test_for(op: CmpOp, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (func (param {input_ty} {input_ty}) (result {input_ty})
                    (block (result {input_ty})
                        local.get 0 ;; returned from block if `local.get 0 != 0`
                        local.get 0
                        local.get 1
                        {input_ty}.{op_str}
                        {result_ty}.const 0
                        {result_ty}.ne
                        br_if 0
                        drop
                        local.get 1 ;; returned from block if `local.get 0 == 0`
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

    test_for(CmpOp::I32And, Instruction::branch_i32_nand);
    test_for(CmpOp::I32Or, Instruction::branch_i32_nor);
    test_for(CmpOp::I32Xor, Instruction::branch_i32_xnor);
    test_for(CmpOp::I32Eq, Instruction::branch_i32_ne);
    test_for(CmpOp::I32Ne, Instruction::branch_i32_eq);
    test_for(
        CmpOp::I32LtS,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        CmpOp::I32LtU,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );
    test_for(
        CmpOp::I32LeS,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        CmpOp::I32LeU,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(CmpOp::I32GtS, Instruction::branch_i32_le_s);
    test_for(CmpOp::I32GtU, Instruction::branch_i32_le_u);
    test_for(CmpOp::I32GeS, Instruction::branch_i32_lt_s);
    test_for(CmpOp::I32GeU, Instruction::branch_i32_lt_u);

    test_for(CmpOp::I64And, Instruction::branch_i64_nand);
    test_for(CmpOp::I64Or, Instruction::branch_i64_nor);
    test_for(CmpOp::I64Xor, Instruction::branch_i64_xnor);
    test_for(CmpOp::I64Eq, Instruction::branch_i64_ne);
    test_for(CmpOp::I64Ne, Instruction::branch_i64_eq);
    test_for(
        CmpOp::I64LtS,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        CmpOp::I64LtU,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );
    test_for(
        CmpOp::I64LeS,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        CmpOp::I64LeU,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(CmpOp::I64GtS, Instruction::branch_i64_le_s);
    test_for(CmpOp::I64GtU, Instruction::branch_i64_le_u);
    test_for(CmpOp::I64GeS, Instruction::branch_i64_lt_s);
    test_for(CmpOp::I64GeU, Instruction::branch_i64_lt_u);

    test_for(CmpOp::F32Eq, Instruction::branch_f32_ne);
    test_for(CmpOp::F32Ne, Instruction::branch_f32_eq);
    test_for(CmpOp::F32Lt, Instruction::branch_f32_not_lt);
    test_for(CmpOp::F32Le, Instruction::branch_f32_not_le);
    test_for(
        CmpOp::F32Gt,
        swap_cmp_br_ops!(Instruction::branch_f32_not_lt),
    );
    test_for(
        CmpOp::F32Ge,
        swap_cmp_br_ops!(Instruction::branch_f32_not_le),
    );

    test_for(CmpOp::F64Eq, Instruction::branch_f64_ne);
    test_for(CmpOp::F64Ne, Instruction::branch_f64_eq);
    test_for(CmpOp::F64Lt, Instruction::branch_f64_not_lt);
    test_for(CmpOp::F64Le, Instruction::branch_f64_not_le);
    test_for(
        CmpOp::F64Gt,
        swap_cmp_br_ops!(Instruction::branch_f64_not_lt),
    );
    test_for(
        CmpOp::F64Ge,
        swap_cmp_br_ops!(Instruction::branch_f64_not_le),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward() {
    fn test_for(op: CmpOp, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (func (param {input_ty} {input_ty})
                    (if
                        ({result_ty}.ne
                            ({result_ty}.const 0)
                            ({input_ty}.{op_str}
                                (local.get 0)
                                (local.get 1)
                            )
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

    test_for(CmpOp::I32And, Instruction::branch_i32_nand);
    test_for(CmpOp::I32Or, Instruction::branch_i32_nor);
    test_for(CmpOp::I32Xor, Instruction::branch_i32_xnor);
    test_for(CmpOp::I32Eq, Instruction::branch_i32_ne);
    test_for(CmpOp::I32Ne, Instruction::branch_i32_eq);
    test_for(
        CmpOp::I32LtS,
        swap_cmp_br_ops!(Instruction::branch_i32_le_s),
    );
    test_for(
        CmpOp::I32LtU,
        swap_cmp_br_ops!(Instruction::branch_i32_le_u),
    );
    test_for(
        CmpOp::I32LeS,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_s),
    );
    test_for(
        CmpOp::I32LeU,
        swap_cmp_br_ops!(Instruction::branch_i32_lt_u),
    );
    test_for(CmpOp::I32GtS, Instruction::branch_i32_le_s);
    test_for(CmpOp::I32GtU, Instruction::branch_i32_le_u);
    test_for(CmpOp::I32GeS, Instruction::branch_i32_lt_s);
    test_for(CmpOp::I32GeU, Instruction::branch_i32_lt_u);

    test_for(CmpOp::I64And, Instruction::branch_i64_nand);
    test_for(CmpOp::I64Or, Instruction::branch_i64_nor);
    test_for(CmpOp::I64Xor, Instruction::branch_i64_xnor);
    test_for(CmpOp::I64Eq, Instruction::branch_i64_ne);
    test_for(CmpOp::I64Ne, Instruction::branch_i64_eq);
    test_for(
        CmpOp::I64LtS,
        swap_cmp_br_ops!(Instruction::branch_i64_le_s),
    );
    test_for(
        CmpOp::I64LtU,
        swap_cmp_br_ops!(Instruction::branch_i64_le_u),
    );
    test_for(
        CmpOp::I64LeS,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_s),
    );
    test_for(
        CmpOp::I64LeU,
        swap_cmp_br_ops!(Instruction::branch_i64_lt_u),
    );
    test_for(CmpOp::I64GtS, Instruction::branch_i64_le_s);
    test_for(CmpOp::I64GtU, Instruction::branch_i64_le_u);
    test_for(CmpOp::I64GeS, Instruction::branch_i64_lt_s);
    test_for(CmpOp::I64GeU, Instruction::branch_i64_lt_u);

    test_for(CmpOp::F32Eq, Instruction::branch_f32_ne);
    test_for(CmpOp::F32Ne, Instruction::branch_f32_eq);
    test_for(CmpOp::F32Lt, Instruction::branch_f32_not_lt);
    test_for(CmpOp::F32Le, Instruction::branch_f32_not_le);
    test_for(
        CmpOp::F32Gt,
        swap_cmp_br_ops!(Instruction::branch_f32_not_lt),
    );
    test_for(
        CmpOp::F32Ge,
        swap_cmp_br_ops!(Instruction::branch_f32_not_le),
    );

    test_for(CmpOp::F64Eq, Instruction::branch_f64_ne);
    test_for(CmpOp::F64Ne, Instruction::branch_f64_eq);
    test_for(CmpOp::F64Lt, Instruction::branch_f64_not_lt);
    test_for(CmpOp::F64Le, Instruction::branch_f64_not_le);
    test_for(
        CmpOp::F64Gt,
        swap_cmp_br_ops!(Instruction::branch_f64_not_lt),
    );
    test_for(
        CmpOp::F64Ge,
        swap_cmp_br_ops!(Instruction::branch_f64_not_le),
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_eqz_fuse() {
    fn test_for(op: CmpOp, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (func (param {input_ty} {input_ty})
                    (block
                        (local.get 0)
                        (local.get 1)
                        ({input_ty}.{op_str})
                        ({result_ty}.eqz)
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

    test_for(CmpOp::I32Eq, Instruction::branch_i32_ne);
    test_for(CmpOp::I32Ne, Instruction::branch_i32_eq);
    test_for(CmpOp::I32And, Instruction::branch_i32_nand);
    test_for(CmpOp::I32Or, Instruction::branch_i32_nor);
    test_for(CmpOp::I32Xor, Instruction::branch_i32_xnor);

    test_for(CmpOp::I64Eq, Instruction::branch_i64_ne);
    test_for(CmpOp::I64Ne, Instruction::branch_i64_eq);
    test_for(CmpOp::I64And, Instruction::branch_i64_nand);
    test_for(CmpOp::I64Or, Instruction::branch_i64_nor);
    test_for(CmpOp::I64Xor, Instruction::branch_i64_xnor);
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_eqz_fuse() {
    fn test_for(op: CmpOp, expect_instr: fn(Reg, Reg, BranchOffset16) -> Instruction) {
        let input_ty = op.param_ty();
        let result_ty = op.result_ty();
        let input_ty = DisplayValueType::from(input_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let op_str = op.op_str();
        let wasm = format!(
            r"
            (module
                (func (param {input_ty} {input_ty})
                    (if
                        ({result_ty}.eqz
                            ({input_ty}.{op_str}
                                (local.get 0)
                                (local.get 1)
                            )
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

    test_for(CmpOp::I32Eq, Instruction::branch_i32_eq);
    test_for(CmpOp::I32Ne, Instruction::branch_i32_ne);
    test_for(CmpOp::I32And, Instruction::branch_i32_and);
    test_for(CmpOp::I32Or, Instruction::branch_i32_or);
    test_for(CmpOp::I32Xor, Instruction::branch_i32_xor);

    test_for(CmpOp::I64Eq, Instruction::branch_i64_eq);
    test_for(CmpOp::I64Ne, Instruction::branch_i64_ne);
    test_for(CmpOp::I64And, Instruction::branch_i64_and);
    test_for(CmpOp::I64Or, Instruction::branch_i64_or);
    test_for(CmpOp::I64Xor, Instruction::branch_i64_xor);
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
