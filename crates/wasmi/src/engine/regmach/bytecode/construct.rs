use super::{
    utils::{BranchOffset16, CopysignImmInstr, Sign},
    AnyConst32,
    BinInstr,
    BinInstrImm16,
    BranchBinOpInstr,
    BranchBinOpInstrImm,
    CallIndirectParams,
    Const16,
    Const32,
    Instruction,
    LoadAtInstr,
    LoadInstr,
    LoadOffset16Instr,
    Register,
    RegisterSpan,
    RegisterSpanIter,
    StoreAtInstr,
    StoreInstr,
    StoreOffset16Instr,
    UnaryInstr,
};
use crate::engine::{
    bytecode::{BranchOffset, DataSegmentIdx, ElementSegmentIdx, FuncIdx, SignatureIdx, TableIdx},
    regmach::bytecode,
    CompiledFunc,
};

macro_rules! constructor_for {
    (
        $(
            fn $fn_name:ident($mode:ident) -> Self::$op_code:ident;
        )* $(,)?
    ) => {
        $( constructor_for! { @impl fn $fn_name($mode) -> Self::$op_code } )*
    };
    ( @impl fn $fn_name:ident(unary) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, input: Register) -> Self {
            Self::$op_code(UnaryInstr::new(result, input))
        }
    };
    ( @impl fn $fn_name:ident(binary) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: Register) -> Self {
            Self::$op_code(BinInstr::new(result, lhs, rhs))
        }
    };
    ( @impl fn $fn_name:ident(binary_imm) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register) -> Self {
            Self::$op_code(UnaryInstr::new(result, lhs))
        }
    };
    ( @impl fn $fn_name:ident(binary_i32imm16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: impl Into<Const16<i32>>) -> Self {
            Self::$op_code(BinInstrImm16::new(result, lhs, rhs.into()))
        }
    };
    ( @impl fn $fn_name:ident(binary_u32imm16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: impl Into<Const16<u32>>) -> Self {
            Self::$op_code(BinInstrImm16::new(result, lhs, rhs.into()))
        }
    };
    ( @impl fn $fn_name:ident(binary_i64imm16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: impl Into<Const16<i64>>) -> Self {
            Self::$op_code(BinInstrImm16::new(result, lhs, rhs.into()))
        }
    };
    ( @impl fn $fn_name:ident(binary_u64imm16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Register, rhs: impl Into<Const16<u64>>) -> Self {
            Self::$op_code(BinInstrImm16::new(result, lhs, rhs.into()))
        }
    };
    ( @impl fn $fn_name:ident(binary_i32imm16_rev) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: impl Into<Const16<i32>>, rhs: Register) -> Self {
            Self::$op_code(BinInstrImm16::new(result, rhs, lhs.into()))
        }
    };
    ( @impl fn $fn_name:ident(binary_u32imm16_rev) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: impl Into<Const16<u32>>, rhs: Register) -> Self {
            Self::$op_code(BinInstrImm16::new(result, rhs, lhs.into()))
        }
    };
    ( @impl fn $fn_name:ident(binary_i64imm16_rev) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: impl Into<Const16<i64>>, rhs: Register) -> Self {
            Self::$op_code(BinInstrImm16::new(result, rhs, lhs.into()))
        }
    };
    ( @impl fn $fn_name:ident(binary_u64imm16_rev) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, lhs: Const16<u64>, rhs: Register) -> Self {
            Self::$op_code(BinInstrImm16::new(result, rhs, lhs))
        }
    };
    ( @impl fn $fn_name:ident(load) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, ptr: Register) -> Self {
            Self::$op_code(LoadInstr::new(result, ptr))
        }
    };
    ( @impl fn $fn_name:ident(load_at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, address: Const32<u32>) -> Self {
            Self::$op_code(LoadAtInstr::new(result, address))
        }
    };
    ( @impl fn $fn_name:ident(load_offset16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Register, ptr: Register, offset: Const16<u32>) -> Self {
            Self::$op_code(LoadOffset16Instr::new(result, ptr, offset))
        }
    };
    ( @impl fn $fn_name:ident(store) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Register, offset: Const32<u32>) -> Self {
            Self::$op_code(StoreInstr::new(ptr, offset))
        }
    };
    ( @impl fn $fn_name:ident(store_at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: Const32<u32>, value: Register) -> Self {
            Self::$op_code(StoreAtInstr::new(address, value))
        }
    };
    ( @impl fn $fn_name:ident(store_offset16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Register, offset: u16, value: Register) -> Self {
            Self::$op_code(StoreOffset16Instr::new(ptr, offset.into(), value))
        }
    };
    ( @impl fn $fn_name:ident(store_offset16_imm8) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Register, offset: u16, value: i8) -> Self {
            Self::$op_code(StoreOffset16Instr::new(ptr, offset.into(), value.into()))
        }
    };
    ( @impl fn $fn_name:ident(store_offset16_imm16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Register, offset: u16, value: i16) -> Self {
            Self::$op_code(StoreOffset16Instr::new(ptr, offset.into(), value.into()))
        }
    };
    ( @impl fn $fn_name:ident(store_at_imm8) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: Const32<u32>, value: i8) -> Self {
            Self::$op_code(StoreAtInstr::new(address, value.into()))
        }
    };
    ( @impl fn $fn_name:ident(store_at_imm16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: Const32<u32>, value: i16) -> Self {
            Self::$op_code(StoreAtInstr::new(address, value.into()))
        }
    };
}

macro_rules! constructor_for_branch_binop {
    ( $( fn $name:ident() -> Self::$op_code:ident; )* ) => {
        impl Instruction {
            $(
                #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
                pub fn $name(lhs: Register, rhs: Register, offset: BranchOffset16) -> Self {
                    Self::$op_code(BranchBinOpInstr::new(lhs, rhs, offset))
                }
            )*
        }
    }
}
constructor_for_branch_binop! {
    fn branch_i32_eq() -> Self::BranchI32Eq;
    fn branch_i32_ne() -> Self::BranchI32Ne;
    fn branch_i32_lt_s() -> Self::BranchI32LtS;
    fn branch_i32_lt_u() -> Self::BranchI32LtU;
    fn branch_i32_le_s() -> Self::BranchI32LeS;
    fn branch_i32_le_u() -> Self::BranchI32LeU;
    fn branch_i32_gt_s() -> Self::BranchI32GtS;
    fn branch_i32_gt_u() -> Self::BranchI32GtU;
    fn branch_i32_ge_s() -> Self::BranchI32GeS;
    fn branch_i32_ge_u() -> Self::BranchI32GeU;

    fn branch_i64_eq() -> Self::BranchI64Eq;
    fn branch_i64_ne() -> Self::BranchI64Ne;
    fn branch_i64_lt_s() -> Self::BranchI64LtS;
    fn branch_i64_lt_u() -> Self::BranchI64LtU;
    fn branch_i64_le_s() -> Self::BranchI64LeS;
    fn branch_i64_le_u() -> Self::BranchI64LeU;
    fn branch_i64_gt_s() -> Self::BranchI64GtS;
    fn branch_i64_gt_u() -> Self::BranchI64GtU;
    fn branch_i64_ge_s() -> Self::BranchI64GeS;
    fn branch_i64_ge_u() -> Self::BranchI64GeU;

    fn branch_f32_eq() -> Self::BranchF32Eq;
    fn branch_f32_ne() -> Self::BranchF32Ne;
    fn branch_f32_lt() -> Self::BranchF32Lt;
    fn branch_f32_le() -> Self::BranchF32Le;
    fn branch_f32_gt() -> Self::BranchF32Gt;
    fn branch_f32_ge() -> Self::BranchF32Ge;

    fn branch_f64_eq() -> Self::BranchF64Eq;
    fn branch_f64_ne() -> Self::BranchF64Ne;
    fn branch_f64_lt() -> Self::BranchF64Lt;
    fn branch_f64_le() -> Self::BranchF64Le;
    fn branch_f64_gt() -> Self::BranchF64Gt;
    fn branch_f64_ge() -> Self::BranchF64Ge;
}

macro_rules! constructor_for_branch_binop_imm {
    ( $( fn $name:ident($ty:ty) -> Self::$op_code:ident; )* ) => {
        impl Instruction {
            $(
                #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
                pub fn $name(lhs: Register, rhs: Const16<$ty>, offset: BranchOffset16) -> Self {
                    Self::$op_code(BranchBinOpInstrImm::new(lhs, rhs, offset))
                }
            )*
        }
    }
}
constructor_for_branch_binop_imm! {
    fn branch_i32_eq_imm(i32) -> Self::BranchI32EqImm;
    fn branch_i32_ne_imm(i32) -> Self::BranchI32NeImm;
    fn branch_i32_lt_s_imm(i32) -> Self::BranchI32LtSImm;
    fn branch_i32_lt_u_imm(u32) -> Self::BranchI32LtUImm;
    fn branch_i32_le_s_imm(i32) -> Self::BranchI32LeSImm;
    fn branch_i32_le_u_imm(u32) -> Self::BranchI32LeUImm;
    fn branch_i32_gt_s_imm(i32) -> Self::BranchI32GtSImm;
    fn branch_i32_gt_u_imm(u32) -> Self::BranchI32GtUImm;
    fn branch_i32_ge_s_imm(i32) -> Self::BranchI32GeSImm;
    fn branch_i32_ge_u_imm(u32) -> Self::BranchI32GeUImm;

    fn branch_i64_eq_imm(i64) -> Self::BranchI64EqImm;
    fn branch_i64_ne_imm(i64) -> Self::BranchI64NeImm;
    fn branch_i64_lt_s_imm(i64) -> Self::BranchI64LtSImm;
    fn branch_i64_lt_u_imm(u64) -> Self::BranchI64LtUImm;
    fn branch_i64_le_s_imm(i64) -> Self::BranchI64LeSImm;
    fn branch_i64_le_u_imm(u64) -> Self::BranchI64LeUImm;
    fn branch_i64_gt_s_imm(i64) -> Self::BranchI64GtSImm;
    fn branch_i64_gt_u_imm(u64) -> Self::BranchI64GtUImm;
    fn branch_i64_ge_s_imm(i64) -> Self::BranchI64GeSImm;
    fn branch_i64_ge_u_imm(u64) -> Self::BranchI64GeUImm;
}

impl Instruction {
    /// Creates a new [`Instruction::Const32`] from the given `value`.
    pub fn const32(value: impl Into<AnyConst32>) -> Self {
        Self::Const32(value.into())
    }

    /// Creates a new [`Instruction::I64Const32`] from the given `value`.
    pub fn i64const32(value: impl Into<Const32<i64>>) -> Self {
        Self::I64Const32(value.into())
    }

    /// Creates a new [`Instruction::F64Const32`] from the given `value`.
    pub fn f64const32(value: impl Into<Const32<f64>>) -> Self {
        Self::F64Const32(value.into())
    }

    /// Creates a new [`Instruction::ReturnReg`] from the given [`Register`] index.
    pub fn return_reg(index: impl Into<Register>) -> Self {
        Self::ReturnReg {
            value: index.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnReg2`] for the given [`Register`] indices.
    pub fn return_reg2(reg0: impl Into<Register>, reg1: impl Into<Register>) -> Self {
        Self::ReturnReg2 {
            values: [reg0.into(), reg1.into()],
        }
    }

    /// Creates a new [`Instruction::ReturnReg3`] for the given [`Register`] indices.
    pub fn return_reg3(
        reg0: impl Into<Register>,
        reg1: impl Into<Register>,
        reg2: impl Into<Register>,
    ) -> Self {
        Self::ReturnReg3 {
            values: [reg0.into(), reg1.into(), reg2.into()],
        }
    }

    /// Creates a new [`Instruction::ReturnImm32`] from the given `value`.
    pub fn return_imm32(value: impl Into<AnyConst32>) -> Self {
        Self::ReturnImm32 {
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnI64Imm32`] from the given `value`.
    pub fn return_i64imm32(value: impl Into<Const32<i64>>) -> Self {
        Self::ReturnI64Imm32 {
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnF64Imm32`] from the given `value`.
    pub fn return_f64imm32(value: impl Into<Const32<f64>>) -> Self {
        Self::ReturnF64Imm32 {
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnSpan`] from the given `values`.
    pub fn return_span(values: RegisterSpanIter) -> Self {
        Self::ReturnSpan { values }
    }

    /// Creates a new [`Instruction::ReturnMany`] for the given [`Register`] indices.
    pub fn return_many(
        reg0: impl Into<Register>,
        reg1: impl Into<Register>,
        reg2: impl Into<Register>,
    ) -> Self {
        Self::ReturnMany {
            values: [reg0.into(), reg1.into(), reg2.into()],
        }
    }

    /// Creates a new [`Instruction::ReturnNez`] for the given `condition`.
    pub fn return_nez(condition: impl Into<Register>) -> Self {
        Self::ReturnNez {
            condition: condition.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezReg`] for the given `condition` and `value`.
    pub fn return_nez_reg(condition: impl Into<Register>, value: impl Into<Register>) -> Self {
        Self::ReturnNezReg {
            condition: condition.into(),
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezReg2`] for the given `condition` and `value`.
    pub fn return_nez_reg2(
        condition: impl Into<Register>,
        value0: impl Into<Register>,
        value1: impl Into<Register>,
    ) -> Self {
        Self::ReturnNezReg2 {
            condition: condition.into(),
            values: [value0.into(), value1.into()],
        }
    }

    /// Creates a new [`Instruction::ReturnNezImm32`] for the given `condition` and `value`.
    pub fn return_nez_imm32(condition: Register, value: impl Into<AnyConst32>) -> Self {
        Self::ReturnNezImm32 {
            condition,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezI64Imm32`] for the given `condition` and `value`.
    pub fn return_nez_i64imm32(condition: Register, value: impl Into<Const32<i64>>) -> Self {
        Self::ReturnNezI64Imm32 {
            condition,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezI64Imm32`] for the given `condition` and `value`.
    pub fn return_nez_f64imm32(condition: Register, value: impl Into<Const32<f64>>) -> Self {
        Self::ReturnNezF64Imm32 {
            condition,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezMany`] for the given `condition` and `values`.
    pub fn return_nez_span(condition: Register, values: RegisterSpanIter) -> Self {
        Self::ReturnNezSpan { condition, values }
    }

    /// Creates a new [`Instruction::ReturnNezMany`] for the given `condition` and `value`.
    pub fn return_nez_many(
        condition: impl Into<Register>,
        head0: impl Into<Register>,
        head1: impl Into<Register>,
    ) -> Self {
        Self::ReturnNezMany {
            condition: condition.into(),
            values: [head0.into(), head1.into()],
        }
    }

    /// Creates a new [`Instruction::Branch`] for the given `offset`.
    pub fn branch(offset: BranchOffset) -> Self {
        Self::Branch { offset }
    }

    /// Creates a new [`Instruction::BranchEqz`] for the given `condition` and `offset`.
    pub fn branch_eqz(condition: Register, offset: BranchOffset) -> Self {
        Self::BranchEqz { condition, offset }
    }

    /// Creates a new [`Instruction::BranchNez`] for the given `condition` and `offset`.
    pub fn branch_nez(condition: Register, offset: BranchOffset) -> Self {
        Self::BranchNez { condition, offset }
    }

    /// Creates a new [`Instruction::BranchTable`] for the given `index` and `len_targets`.
    pub fn branch_table(index: Register, len_targets: impl Into<Const32<u32>>) -> Self {
        Self::BranchTable {
            index,
            len_targets: len_targets.into(),
        }
    }

    /// Creates a new [`Instruction::Copy`].
    pub fn copy(result: impl Into<Register>, value: impl Into<Register>) -> Self {
        Self::Copy {
            result: result.into(),
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::Copy2`].
    pub fn copy2(
        results: RegisterSpan,
        value0: impl Into<Register>,
        value1: impl Into<Register>,
    ) -> Self {
        Self::Copy2 {
            results,
            values: [value0.into(), value1.into()],
        }
    }

    /// Creates a new [`Instruction::CopyImm32`].
    pub fn copy_imm32(result: Register, value: impl Into<AnyConst32>) -> Self {
        Self::CopyImm32 {
            result,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::CopyI64Imm32`].
    pub fn copy_i64imm32(result: Register, value: impl Into<Const32<i64>>) -> Self {
        Self::CopyI64Imm32 {
            result,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::CopyF64Imm32`].
    pub fn copy_f64imm32(result: Register, value: impl Into<Const32<f64>>) -> Self {
        Self::CopyF64Imm32 {
            result,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::CopySpan`] copying multiple consecutive values.
    pub fn copy_span(results: RegisterSpan, values: RegisterSpan, len: u16) -> Self {
        debug_assert!(!results.iter_u16(len).is_overlapping(&values.iter_u16(len)));
        Self::CopySpan {
            results,
            values,
            len,
        }
    }

    /// Creates a new [`Instruction::CopySpanNonOverlapping`] copying multiple consecutive values.
    pub fn copy_span_non_overlapping(
        results: RegisterSpan,
        values: RegisterSpan,
        len: u16,
    ) -> Self {
        debug_assert!(!results.iter_u16(len).is_overlapping(&values.iter_u16(len)));
        Self::CopySpanNonOverlapping {
            results,
            values,
            len,
        }
    }

    /// Creates a new [`Instruction::CopyMany`].
    pub fn copy_many(
        results: RegisterSpan,
        head0: impl Into<Register>,
        head1: impl Into<Register>,
    ) -> Self {
        Self::CopyMany {
            results,
            values: [head0.into(), head1.into()],
        }
    }

    /// Creates a new [`Instruction::CopyManyNonOverlapping`].
    pub fn copy_many_non_overlapping(
        results: RegisterSpan,
        head0: impl Into<Register>,
        head1: impl Into<Register>,
    ) -> Self {
        Self::CopyManyNonOverlapping {
            results,
            values: [head0.into(), head1.into()],
        }
    }

    /// Creates a new [`Instruction::GlobalGet`].
    pub fn global_get(result: Register, global: bytecode::GlobalIdx) -> Self {
        Self::GlobalGet { result, global }
    }

    /// Creates a new [`Instruction::GlobalSet`].
    pub fn global_set(global: bytecode::GlobalIdx, input: Register) -> Self {
        Self::GlobalSet { global, input }
    }

    /// Creates a new [`Instruction::GlobalSetI32Imm16`].
    pub fn global_set_i32imm16(
        global: bytecode::GlobalIdx,
        input: impl Into<Const16<i32>>,
    ) -> Self {
        Self::GlobalSetI32Imm16 {
            global,
            input: input.into(),
        }
    }

    /// Creates a new [`Instruction::GlobalSetI64Imm16`].
    pub fn global_set_i64imm16(
        global: bytecode::GlobalIdx,
        input: impl Into<Const16<i64>>,
    ) -> Self {
        Self::GlobalSetI64Imm16 {
            global,
            input: input.into(),
        }
    }

    /// Creates a new [`Instruction::F32CopysignImm`] instruction.
    pub fn f32_copysign_imm(result: Register, lhs: Register, rhs: Sign) -> Self {
        Self::F32CopysignImm(CopysignImmInstr { result, lhs, rhs })
    }

    /// Creates a new [`Instruction::F64CopysignImm`] instruction.
    pub fn f64_copysign_imm(result: Register, lhs: Register, rhs: Sign) -> Self {
        Self::F64CopysignImm(CopysignImmInstr { result, lhs, rhs })
    }

    /// Creates a new [`Instruction::Select`].
    pub fn select(result: Register, condition: Register, lhs: Register) -> Self {
        Self::Select {
            result,
            condition,
            lhs,
        }
    }

    /// Creates a new [`Instruction::SelectRev`].
    pub fn select_rev(result: Register, condition: Register, rhs: Register) -> Self {
        Self::SelectRev {
            result,
            condition,
            rhs,
        }
    }

    /// Creates a new [`Instruction::SelectImm32`].
    pub fn select_imm32(result_or_condition: Register, lhs_or_rhs: impl Into<AnyConst32>) -> Self {
        Self::SelectImm32 {
            result_or_condition,
            lhs_or_rhs: lhs_or_rhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectI64Imm32`].
    pub fn select_i64imm32(
        result_or_condition: Register,
        lhs_or_rhs: impl Into<Const32<i64>>,
    ) -> Self {
        Self::SelectI64Imm32 {
            result_or_condition,
            lhs_or_rhs: lhs_or_rhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectF64Imm32`].
    pub fn select_f64imm32(
        result_or_condition: Register,
        lhs_or_rhs: impl Into<Const32<f64>>,
    ) -> Self {
        Self::SelectF64Imm32 {
            result_or_condition,
            lhs_or_rhs: lhs_or_rhs.into(),
        }
    }

    /// Creates a new [`Instruction::RefFunc`] with the given `result` and `func`.
    pub fn ref_func(result: Register, func: impl Into<FuncIdx>) -> Self {
        Self::RefFunc {
            result,
            func: func.into(),
        }
    }

    /// Creates a new [`Instruction::DataSegmentIdx`] from the given `index`.
    pub fn data_idx(index: impl Into<DataSegmentIdx>) -> Self {
        Self::DataSegmentIdx(index.into())
    }

    /// Creates a new [`Instruction::ElementSegmentIdx`] from the given `index`.
    pub fn elem_idx(index: impl Into<ElementSegmentIdx>) -> Self {
        Self::ElementSegmentIdx(index.into())
    }

    /// Creates a new [`Instruction::TableIdx`] from the given `index`.
    pub fn table_idx(index: impl Into<TableIdx>) -> Self {
        Self::TableIdx(index.into())
    }

    /// Creates a new [`Instruction::TableGet`] with the given `result` and `index`.
    pub fn table_get(result: Register, index: Register) -> Self {
        Self::TableGet { result, index }
    }

    /// Creates a new [`Instruction::TableGetImm`] with the given `result` and `index`.
    pub fn table_get_imm(result: Register, index: impl Into<Const32<u32>>) -> Self {
        Self::TableGetImm {
            result,
            index: index.into(),
        }
    }

    /// Creates a new [`Instruction::TableSize`] with the given `result` and `table`.
    pub fn table_size(result: Register, table: impl Into<TableIdx>) -> Self {
        Self::TableSize {
            result,
            table: table.into(),
        }
    }

    /// Creates a new [`Instruction::TableSet`] with the given `index` and `value`.
    pub fn table_set(index: Register, value: Register) -> Self {
        Self::TableSet { index, value }
    }

    /// Creates a new [`Instruction::TableSetAt`] with the given `index` and `value`.
    pub fn table_set_at(index: impl Into<Const32<u32>>, value: Register) -> Self {
        Self::TableSetAt {
            index: index.into(),
            value,
        }
    }

    /// Creates a new [`Instruction::TableCopy`] with the given `dst`, `src` and `len`.
    pub fn table_copy(dst: Register, src: Register, len: Register) -> Self {
        Self::TableCopy { dst, src, len }
    }

    /// Creates a new [`Instruction::TableCopyTo`] with the given `dst`, `src` and `len`.
    pub fn table_copy_to(dst: impl Into<Const16<u32>>, src: Register, len: Register) -> Self {
        Self::TableCopyTo {
            dst: dst.into(),
            src,
            len,
        }
    }

    /// Creates a new [`Instruction::TableCopyFrom`] with the given `dst`, `src` and `len`.
    pub fn table_copy_from(dst: Register, src: impl Into<Const16<u32>>, len: Register) -> Self {
        Self::TableCopyFrom {
            dst,
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::TableCopyFromTo`] with the given `dst`, `src` and `len`.
    pub fn table_copy_from_to(
        dst: impl Into<Const16<u32>>,
        src: impl Into<Const16<u32>>,
        len: Register,
    ) -> Self {
        Self::TableCopyFromTo {
            dst: dst.into(),
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::TableCopyExact`] with the given `dst`, `src` and `len`.
    pub fn table_copy_exact(dst: Register, src: Register, len: impl Into<Const16<u32>>) -> Self {
        Self::TableCopyExact {
            dst,
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableCopyToExact`] with the given `dst`, `src` and `len`.
    pub fn table_copy_to_exact(
        dst: impl Into<Const16<u32>>,
        src: Register,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::TableCopyToExact {
            dst: dst.into(),
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableCopyFromExact`] with the given `dst`, `src` and `len`.
    pub fn table_copy_from_exact(
        dst: Register,
        src: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::TableCopyFromExact {
            dst,
            src: src.into(),
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableCopyFromToExact`] with the given `dst`, `src` and `len`.
    pub fn table_copy_from_to_exact(
        dst: impl Into<Const16<u32>>,
        src: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::TableCopyFromToExact {
            dst: dst.into(),
            src: src.into(),
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableInit`] with the given `dst`, `src` and `len`.
    pub fn table_init(dst: Register, src: Register, len: Register) -> Self {
        Self::TableInit { dst, src, len }
    }

    /// Creates a new [`Instruction::TableInitTo`] with the given `dst`, `src` and `len`.
    pub fn table_init_to(dst: impl Into<Const16<u32>>, src: Register, len: Register) -> Self {
        Self::TableInitTo {
            dst: dst.into(),
            src,
            len,
        }
    }

    /// Creates a new [`Instruction::TableInitFrom`] with the given `dst`, `src` and `len`.
    pub fn table_init_from(dst: Register, src: impl Into<Const16<u32>>, len: Register) -> Self {
        Self::TableInitFrom {
            dst,
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::TableInitFromTo`] with the given `dst`, `src` and `len`.
    pub fn table_init_from_to(
        dst: impl Into<Const16<u32>>,
        src: impl Into<Const16<u32>>,
        len: Register,
    ) -> Self {
        Self::TableInitFromTo {
            dst: dst.into(),
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::TableInitExact`] with the given `dst`, `src` and `len`.
    pub fn table_init_exact(dst: Register, src: Register, len: impl Into<Const16<u32>>) -> Self {
        Self::TableInitExact {
            dst,
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableInitToExact`] with the given `dst`, `src` and `len`.
    pub fn table_init_to_exact(
        dst: impl Into<Const16<u32>>,
        src: Register,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::TableInitToExact {
            dst: dst.into(),
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableInitFromExact`] with the given `dst`, `src` and `len`.
    pub fn table_init_from_exact(
        dst: Register,
        src: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::TableInitFromExact {
            dst,
            src: src.into(),
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableInitFromToExact`] with the given `dst`, `src` and `len`.
    pub fn table_init_from_to_exact(
        dst: impl Into<Const16<u32>>,
        src: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::TableInitFromToExact {
            dst: dst.into(),
            src: src.into(),
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableFill`] with the given `dst`, `len` and `value`.
    pub fn table_fill(dst: Register, len: Register, value: Register) -> Self {
        Self::TableFill { dst, len, value }
    }

    /// Creates a new [`Instruction::TableFillAt`] with the given `dst`, `len` and `value`.
    pub fn table_fill_at(dst: impl Into<Const16<u32>>, len: Register, value: Register) -> Self {
        Self::TableFillAt {
            dst: dst.into(),
            len,
            value,
        }
    }

    /// Creates a new [`Instruction::TableFillExact`] with the given `dst`, `len` and `value`.
    pub fn table_fill_exact(dst: Register, len: impl Into<Const16<u32>>, value: Register) -> Self {
        Self::TableFillExact {
            dst,
            len: len.into(),
            value,
        }
    }

    /// Creates a new [`Instruction::TableFillAtExact`] with the given `dst`, `len` and `value`.
    pub fn table_fill_at_exact(
        dst: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
        value: Register,
    ) -> Self {
        Self::TableFillAtExact {
            dst: dst.into(),
            len: len.into(),
            value,
        }
    }

    /// Creates a new [`Instruction::TableGrow`] with the given `result`, `delta` and `value`.
    pub fn table_grow(result: Register, delta: Register, value: Register) -> Self {
        Self::TableGrow {
            result,
            delta,
            value,
        }
    }

    /// Creates a new [`Instruction::TableGrowImm`] with the given `result`, `delta` and `value`.
    pub fn table_grow_imm(
        result: Register,
        delta: impl Into<Const16<u32>>,
        value: Register,
    ) -> Self {
        Self::TableGrowImm {
            result,
            delta: delta.into(),
            value,
        }
    }

    /// Creates a new [`Instruction::MemorySize`] with the given `result`.
    pub fn memory_size(result: Register) -> Self {
        Self::MemorySize { result }
    }

    /// Creates a new [`Instruction::MemoryGrow`] with the given `result`, `delta`.
    pub fn memory_grow(result: Register, delta: Register) -> Self {
        Self::MemoryGrow { result, delta }
    }

    /// Creates a new [`Instruction::MemoryGrowBy`] with the given `result`, `delta` and `value`.
    pub fn memory_grow_by(result: Register, delta: impl Into<Const16<u32>>) -> Self {
        Self::MemoryGrowBy {
            result,
            delta: delta.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryCopy`] with the given `dst`, `src` and `len`.
    pub fn memory_copy(dst: Register, src: Register, len: Register) -> Self {
        Self::MemoryCopy { dst, src, len }
    }

    /// Creates a new [`Instruction::MemoryCopyTo`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_to(dst: impl Into<Const16<u32>>, src: Register, len: Register) -> Self {
        Self::MemoryCopyTo {
            dst: dst.into(),
            src,
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryCopyFrom`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_from(dst: Register, src: impl Into<Const16<u32>>, len: Register) -> Self {
        Self::MemoryCopyFrom {
            dst,
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryCopyFromTo`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_from_to(
        dst: impl Into<Const16<u32>>,
        src: impl Into<Const16<u32>>,
        len: Register,
    ) -> Self {
        Self::MemoryCopyFromTo {
            dst: dst.into(),
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryCopyExact`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_exact(dst: Register, src: Register, len: impl Into<Const16<u32>>) -> Self {
        Self::MemoryCopyExact {
            dst,
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryCopyToExact`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_to_exact(
        dst: impl Into<Const16<u32>>,
        src: Register,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryCopyToExact {
            dst: dst.into(),
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryCopyFromExact`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_from_exact(
        dst: Register,
        src: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryCopyFromExact {
            dst,
            src: src.into(),
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryCopyFromToExact`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_from_to_exact(
        dst: impl Into<Const16<u32>>,
        src: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryCopyFromToExact {
            dst: dst.into(),
            src: src.into(),
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryInit`] with the given `dst`, `src` and `len`.
    pub fn memory_init(dst: Register, src: Register, len: Register) -> Self {
        Self::MemoryInit { dst, src, len }
    }

    /// Creates a new [`Instruction::MemoryInitTo`] with the given `dst`, `src` and `len`.
    pub fn memory_init_to(dst: impl Into<Const16<u32>>, src: Register, len: Register) -> Self {
        Self::MemoryInitTo {
            dst: dst.into(),
            src,
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryInitFrom`] with the given `dst`, `src` and `len`.
    pub fn memory_init_from(dst: Register, src: impl Into<Const16<u32>>, len: Register) -> Self {
        Self::MemoryInitFrom {
            dst,
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryInitFromTo`] with the given `dst`, `src` and `len`.
    pub fn memory_init_from_to(
        dst: impl Into<Const16<u32>>,
        src: impl Into<Const16<u32>>,
        len: Register,
    ) -> Self {
        Self::MemoryInitFromTo {
            dst: dst.into(),
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryInitExact`] with the given `dst`, `src` and `len`.
    pub fn memory_init_exact(dst: Register, src: Register, len: impl Into<Const16<u32>>) -> Self {
        Self::MemoryInitExact {
            dst,
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryInitToExact`] with the given `dst`, `src` and `len`.
    pub fn memory_init_to_exact(
        dst: impl Into<Const16<u32>>,
        src: Register,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryInitToExact {
            dst: dst.into(),
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryInitFromExact`] with the given `dst`, `src` and `len`.
    pub fn memory_init_from_exact(
        dst: Register,
        src: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryInitFromExact {
            dst,
            src: src.into(),
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryInitFromToExact`] with the given `dst`, `src` and `len`.
    pub fn memory_init_from_to_exact(
        dst: impl Into<Const16<u32>>,
        src: impl Into<Const16<u32>>,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryInitFromToExact {
            dst: dst.into(),
            src: src.into(),
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryFill`] with the given `dst`, `value` and `len`.
    pub fn memory_fill(dst: Register, value: Register, len: Register) -> Self {
        Self::MemoryFill { dst, value, len }
    }

    /// Creates a new [`Instruction::MemoryFillAt`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_at(dst: impl Into<Const16<u32>>, value: Register, len: Register) -> Self {
        Self::MemoryFillAt {
            dst: dst.into(),
            value,
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryFillImm`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_imm(dst: Register, value: u8, len: Register) -> Self {
        Self::MemoryFillImm { dst, value, len }
    }

    /// Creates a new [`Instruction::MemoryFillExact`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_exact(dst: Register, value: Register, len: impl Into<Const16<u32>>) -> Self {
        Self::MemoryFillExact {
            dst,
            value,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryFillAtImm`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_at_imm(dst: impl Into<Const16<u32>>, value: u8, len: Register) -> Self {
        Self::MemoryFillAtImm {
            dst: dst.into(),
            value,
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryFillAtExact`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_at_exact(
        dst: impl Into<Const16<u32>>,
        value: Register,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryFillAtExact {
            dst: dst.into(),
            value,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryFillImmExact`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_imm_exact(dst: Register, value: u8, len: impl Into<Const16<u32>>) -> Self {
        Self::MemoryFillImmExact {
            dst,
            value,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryFillAtImmExact`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_at_imm_exact(
        dst: impl Into<Const16<u32>>,
        value: u8,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryFillAtImmExact {
            dst: dst.into(),
            value,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::Register`] instruction parameter.
    pub fn register(reg: impl Into<Register>) -> Self {
        Self::Register(reg.into())
    }

    /// Creates a new [`Instruction::Register2`] instruction parameter.
    pub fn register2(reg0: impl Into<Register>, reg1: impl Into<Register>) -> Self {
        Self::Register2([reg0.into(), reg1.into()])
    }

    /// Creates a new [`Instruction::Register3`] instruction parameter.
    pub fn register3(
        reg0: impl Into<Register>,
        reg1: impl Into<Register>,
        reg2: impl Into<Register>,
    ) -> Self {
        Self::Register3([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Instruction::RegisterList`] instruction parameter.
    pub fn register_list(
        reg0: impl Into<Register>,
        reg1: impl Into<Register>,
        reg2: impl Into<Register>,
    ) -> Self {
        Self::RegisterList([reg0.into(), reg1.into(), reg2.into()])
    }

    /// Creates a new [`Instruction::CallIndirectParams`] for the given `index` and `table`.
    pub fn call_indirect_params(index: Register, table: impl Into<TableIdx>) -> Self {
        Self::CallIndirectParams(CallIndirectParams {
            index,
            table: table.into(),
        })
    }

    /// Creates a new [`Instruction::CallIndirectParamsImm16`] for the given `index` and `table`.
    pub fn call_indirect_params_imm16(
        index: impl Into<Const16<u32>>,
        table: impl Into<TableIdx>,
    ) -> Self {
        Self::CallIndirectParamsImm16(CallIndirectParams {
            index: index.into(),
            table: table.into(),
        })
    }

    /// Creates a new [`Instruction::CallInternal0`] for the given `func`.
    pub fn return_call_internal_0(func: CompiledFunc) -> Self {
        Self::ReturnCallInternal0 { func }
    }

    /// Creates a new [`Instruction::CallInternal`] for the given `func`.
    pub fn return_call_internal(func: CompiledFunc) -> Self {
        Self::ReturnCallInternal { func }
    }

    /// Creates a new [`Instruction::ReturnCallImported0`] for the given `func`.
    pub fn return_call_imported_0(func: impl Into<FuncIdx>) -> Self {
        Self::ReturnCallImported0 { func: func.into() }
    }

    /// Creates a new [`Instruction::ReturnCallImported`] for the given `func`.
    pub fn return_call_imported(func: impl Into<FuncIdx>) -> Self {
        Self::ReturnCallImported { func: func.into() }
    }

    /// Creates a new [`Instruction::ReturnCallIndirect0`] for the given `func`.
    pub fn return_call_indirect_0(func_type: impl Into<SignatureIdx>) -> Self {
        Self::ReturnCallIndirect0 {
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnCallIndirect`] for the given `func`.
    pub fn return_call_indirect(func_type: impl Into<SignatureIdx>) -> Self {
        Self::ReturnCallIndirect {
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::CallInternal0`] for the given `func`.
    pub fn call_internal_0(results: RegisterSpan, func: CompiledFunc) -> Self {
        Self::CallInternal0 { results, func }
    }

    /// Creates a new [`Instruction::CallInternal`] for the given `func`.
    pub fn call_internal(results: RegisterSpan, func: CompiledFunc) -> Self {
        Self::CallInternal { results, func }
    }

    /// Creates a new [`Instruction::CallImported0`] for the given `func`.
    pub fn call_imported_0(results: RegisterSpan, func: impl Into<FuncIdx>) -> Self {
        Self::CallImported0 {
            results,
            func: func.into(),
        }
    }

    /// Creates a new [`Instruction::CallImported`] for the given `func`.
    pub fn call_imported(results: RegisterSpan, func: impl Into<FuncIdx>) -> Self {
        Self::CallImported {
            results,
            func: func.into(),
        }
    }

    /// Creates a new [`Instruction::CallIndirect0`] for the given `func`.
    pub fn call_indirect_0(results: RegisterSpan, func_type: impl Into<SignatureIdx>) -> Self {
        Self::CallIndirect0 {
            results,
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::CallIndirect`] for the given `func`.
    pub fn call_indirect(results: RegisterSpan, func_type: impl Into<SignatureIdx>) -> Self {
        Self::CallIndirect {
            results,
            func_type: func_type.into(),
        }
    }

    constructor_for! {
        // Load

        fn i32_load(load) -> Self::I32Load;
        fn i32_load_at(load_at) -> Self::I32LoadAt;
        fn i32_load_offset16(load_offset16) -> Self::I32LoadOffset16;

        fn i32_load8_s(load) -> Self::I32Load8s;
        fn i32_load8_s_at(load_at) -> Self::I32Load8sAt;
        fn i32_load8_s_offset16(load_offset16) -> Self::I32Load8sOffset16;

        fn i32_load8_u(load) -> Self::I32Load8u;
        fn i32_load8_u_at(load_at) -> Self::I32Load8uAt;
        fn i32_load8_u_offset16(load_offset16) -> Self::I32Load8uOffset16;

        fn i32_load16_s(load) -> Self::I32Load16s;
        fn i32_load16_s_at(load_at) -> Self::I32Load16sAt;
        fn i32_load16_s_offset16(load_offset16) -> Self::I32Load16sOffset16;

        fn i32_load16_u(load) -> Self::I32Load16u;
        fn i32_load16_u_at(load_at) -> Self::I32Load16uAt;
        fn i32_load16_u_offset16(load_offset16) -> Self::I32Load16uOffset16;

        fn i64_load(load) -> Self::I64Load;
        fn i64_load_at(load_at) -> Self::I64LoadAt;
        fn i64_load_offset16(load_offset16) -> Self::I64LoadOffset16;

        fn i64_load8_s(load) -> Self::I64Load8s;
        fn i64_load8_s_at(load_at) -> Self::I64Load8sAt;
        fn i64_load8_s_offset16(load_offset16) -> Self::I64Load8sOffset16;

        fn i64_load8_u(load) -> Self::I64Load8u;
        fn i64_load8_u_at(load_at) -> Self::I64Load8uAt;
        fn i64_load8_u_offset16(load_offset16) -> Self::I64Load8uOffset16;

        fn i64_load16_s(load) -> Self::I64Load16s;
        fn i64_load16_s_at(load_at) -> Self::I64Load16sAt;
        fn i64_load16_s_offset16(load_offset16) -> Self::I64Load16sOffset16;

        fn i64_load16_u(load) -> Self::I64Load16u;
        fn i64_load16_u_at(load_at) -> Self::I64Load16uAt;
        fn i64_load16_u_offset16(load_offset16) -> Self::I64Load16uOffset16;

        fn i64_load32_s(load) -> Self::I64Load32s;
        fn i64_load32_s_at(load_at) -> Self::I64Load32sAt;
        fn i64_load32_s_offset16(load_offset16) -> Self::I64Load32sOffset16;

        fn i64_load32_u(load) -> Self::I64Load32u;
        fn i64_load32_u_at(load_at) -> Self::I64Load32uAt;
        fn i64_load32_u_offset16(load_offset16) -> Self::I64Load32uOffset16;

        fn f32_load(load) -> Self::F32Load;
        fn f32_load_at(load_at) -> Self::F32LoadAt;
        fn f32_load_offset16(load_offset16) -> Self::F32LoadOffset16;

        fn f64_load(load) -> Self::F64Load;
        fn f64_load_at(load_at) -> Self::F64LoadAt;
        fn f64_load_offset16(load_offset16) -> Self::F64LoadOffset16;

        // Store

        fn i32_store(store) -> Self::I32Store;
        fn i32_store_offset16(store_offset16) -> Self::I32StoreOffset16;
        fn i32_store_offset16_imm16(store_offset16_imm16) -> Self::I32StoreOffset16Imm16;
        fn i32_store_at(store_at) -> Self::I32StoreAt;
        fn i32_store_at_imm16(store_at_imm16) -> Self::I32StoreAtImm16;

        fn i32_store8(store) -> Self::I32Store8;
        fn i32_store8_offset16(store_offset16) -> Self::I32Store8Offset16;
        fn i32_store8_offset16_imm(store_offset16_imm8) -> Self::I32Store8Offset16Imm;
        fn i32_store8_at(store_at) -> Self::I32Store8At;
        fn i32_store8_at_imm(store_at_imm8) -> Self::I32Store8AtImm;

        fn i32_store16(store) -> Self::I32Store16;
        fn i32_store16_offset16(store_offset16) -> Self::I32Store16Offset16;
        fn i32_store16_offset16_imm(store_offset16_imm16) -> Self::I32Store16Offset16Imm;
        fn i32_store16_at(store_at) -> Self::I32Store16At;
        fn i32_store16_at_imm(store_at_imm16) -> Self::I32Store16AtImm;

        fn i64_store(store) -> Self::I64Store;
        fn i64_store_offset16(store_offset16) -> Self::I64StoreOffset16;
        fn i64_store_offset16_imm16(store_offset16_imm16) -> Self::I64StoreOffset16Imm16;
        fn i64_store_at(store_at) -> Self::I64StoreAt;
        fn i64_store_at_imm16(store_at_imm16) -> Self::I64StoreAtImm16;

        fn i64_store8(store) -> Self::I64Store8;
        fn i64_store8_offset16(store_offset16) -> Self::I64Store8Offset16;
        fn i64_store8_offset16_imm(store_offset16_imm8) -> Self::I64Store8Offset16Imm;
        fn i64_store8_at(store_at) -> Self::I64Store8At;
        fn i64_store8_at_imm(store_at_imm8) -> Self::I64Store8AtImm;

        fn i64_store16(store) -> Self::I64Store16;
        fn i64_store16_offset16(store_offset16) -> Self::I64Store16Offset16;
        fn i64_store16_offset16_imm(store_offset16_imm16) -> Self::I64Store16Offset16Imm;
        fn i64_store16_at(store_at) -> Self::I64Store16At;
        fn i64_store16_at_imm(store_at_imm16) -> Self::I64Store16AtImm;

        fn i64_store32(store) -> Self::I64Store32;
        fn i64_store32_offset16(store_offset16) -> Self::I64Store32Offset16;
        fn i64_store32_offset16_imm16(store_offset16_imm16) -> Self::I64Store32Offset16Imm16;
        fn i64_store32_at(store_at) -> Self::I64Store32At;
        fn i64_store32_at_imm16(store_at_imm16) -> Self::I64Store32AtImm16;

        fn f32_store(store) -> Self::F32Store;
        fn f32_store_offset16(store_offset16) -> Self::F32StoreOffset16;
        fn f32_store_at(store_at) -> Self::F32StoreAt;

        fn f64_store(store) -> Self::F64Store;
        fn f64_store_offset16(store_offset16) -> Self::F64StoreOffset16;
        fn f64_store_at(store_at) -> Self::F64StoreAt;

        // Integer Unary

        fn i32_clz(unary) -> Self::I32Clz;
        fn i32_ctz(unary) -> Self::I32Ctz;
        fn i32_popcnt(unary) -> Self::I32Popcnt;

        fn i64_clz(unary) -> Self::I64Clz;
        fn i64_ctz(unary) -> Self::I64Ctz;
        fn i64_popcnt(unary) -> Self::I64Popcnt;

        // Float Unary

        fn f32_abs(unary) -> Self::F32Abs;
        fn f32_neg(unary) -> Self::F32Neg;
        fn f32_ceil(unary) -> Self::F32Ceil;
        fn f32_floor(unary) -> Self::F32Floor;
        fn f32_trunc(unary) -> Self::F32Trunc;
        fn f32_nearest(unary) -> Self::F32Nearest;
        fn f32_sqrt(unary) -> Self::F32Sqrt;

        fn f64_abs(unary) -> Self::F64Abs;
        fn f64_neg(unary) -> Self::F64Neg;
        fn f64_ceil(unary) -> Self::F64Ceil;
        fn f64_floor(unary) -> Self::F64Floor;
        fn f64_trunc(unary) -> Self::F64Trunc;
        fn f64_nearest(unary) -> Self::F64Nearest;
        fn f64_sqrt(unary) -> Self::F64Sqrt;

        // Float Arithmetic

        fn f32_add(binary) -> Self::F32Add;
        fn f64_add(binary) -> Self::F64Add;
        fn f32_sub(binary) -> Self::F32Sub;
        fn f64_sub(binary) -> Self::F64Sub;
        fn f32_mul(binary) -> Self::F32Mul;
        fn f64_mul(binary) -> Self::F64Mul;
        fn f32_div(binary) -> Self::F32Div;
        fn f64_div(binary) -> Self::F64Div;
        fn f32_min(binary) -> Self::F32Min;
        fn f64_min(binary) -> Self::F64Min;
        fn f32_max(binary) -> Self::F32Max;
        fn f64_max(binary) -> Self::F64Max;
        fn f32_copysign(binary) -> Self::F32Copysign;
        fn f64_copysign(binary) -> Self::F64Copysign;

        // Integer Comparison

        fn i32_eq(binary) -> Self::I32Eq;
        fn i32_eq_imm16(binary_i32imm16) -> Self::I32EqImm16;

        fn i64_eq(binary) -> Self::I64Eq;
        fn i64_eq_imm16(binary_i64imm16) -> Self::I64EqImm16;

        fn i32_ne(binary) -> Self::I32Ne;
        fn i32_ne_imm16(binary_i32imm16) -> Self::I32NeImm16;

        fn i64_ne(binary) -> Self::I64Ne;
        fn i64_ne_imm16(binary_i64imm16) -> Self::I64NeImm16;

        fn i32_lt_s(binary) -> Self::I32LtS;
        fn i32_lt_s_imm16(binary_i32imm16) -> Self::I32LtSImm16;

        fn i64_lt_s(binary) -> Self::I64LtS;
        fn i64_lt_s_imm16(binary_i64imm16) -> Self::I64LtSImm16;

        fn i32_lt_u(binary) -> Self::I32LtU;
        fn i32_lt_u_imm16(binary_u32imm16) -> Self::I32LtUImm16;

        fn i64_lt_u(binary) -> Self::I64LtU;
        fn i64_lt_u_imm16(binary_u64imm16) -> Self::I64LtUImm16;

        fn i32_le_s(binary) -> Self::I32LeS;
        fn i32_le_s_imm16(binary_i32imm16) -> Self::I32LeSImm16;

        fn i64_le_s(binary) -> Self::I64LeS;
        fn i64_le_s_imm16(binary_i64imm16) -> Self::I64LeSImm16;

        fn i32_le_u(binary) -> Self::I32LeU;
        fn i32_le_u_imm16(binary_u32imm16) -> Self::I32LeUImm16;

        fn i64_le_u(binary) -> Self::I64LeU;
        fn i64_le_u_imm16(binary_u64imm16) -> Self::I64LeUImm16;

        fn i32_gt_s(binary) -> Self::I32GtS;
        fn i32_gt_s_imm16(binary_i32imm16) -> Self::I32GtSImm16;

        fn i64_gt_s(binary) -> Self::I64GtS;
        fn i64_gt_s_imm16(binary_i64imm16) -> Self::I64GtSImm16;

        fn i32_gt_u(binary) -> Self::I32GtU;
        fn i32_gt_u_imm16(binary_u32imm16) -> Self::I32GtUImm16;

        fn i64_gt_u(binary) -> Self::I64GtU;
        fn i64_gt_u_imm16(binary_u64imm16) -> Self::I64GtUImm16;

        fn i32_ge_s(binary) -> Self::I32GeS;
        fn i32_ge_s_imm16(binary_i32imm16) -> Self::I32GeSImm16;

        fn i64_ge_s(binary) -> Self::I64GeS;
        fn i64_ge_s_imm16(binary_i64imm16) -> Self::I64GeSImm16;

        fn i32_ge_u(binary) -> Self::I32GeU;
        fn i32_ge_u_imm16(binary_u32imm16) -> Self::I32GeUImm16;

        fn i64_ge_u(binary) -> Self::I64GeU;
        fn i64_ge_u_imm16(binary_u64imm16) -> Self::I64GeUImm16;

        // Float Comparison

        fn f32_eq(binary) -> Self::F32Eq;
        fn f64_eq(binary) -> Self::F64Eq;
        fn f32_ne(binary) -> Self::F32Ne;
        fn f64_ne(binary) -> Self::F64Ne;
        fn f32_lt(binary) -> Self::F32Lt;
        fn f64_lt(binary) -> Self::F64Lt;
        fn f32_le(binary) -> Self::F32Le;
        fn f64_le(binary) -> Self::F64Le;
        fn f32_gt(binary) -> Self::F32Gt;
        fn f64_gt(binary) -> Self::F64Gt;
        fn f32_ge(binary) -> Self::F32Ge;
        fn f64_ge(binary) -> Self::F64Ge;

        // Integer Arithmetic

        fn i32_add(binary) -> Self::I32Add;
        fn i32_add_imm16(binary_i32imm16) -> Self::I32AddImm16;

        fn i64_add(binary) -> Self::I64Add;
        fn i64_add_imm16(binary_i64imm16) -> Self::I64AddImm16;

        fn i32_sub(binary) -> Self::I32Sub;
        fn i32_sub_imm16(binary_i32imm16) -> Self::I32SubImm16;
        fn i32_sub_imm16_rev(binary_i32imm16_rev) -> Self::I32SubImm16Rev;

        fn i64_sub(binary) -> Self::I64Sub;
        fn i64_sub_imm16(binary_i64imm16) -> Self::I64SubImm16;
        fn i64_sub_imm16_rev(binary_i64imm16_rev) -> Self::I64SubImm16Rev;

        fn i32_mul(binary) -> Self::I32Mul;
        fn i32_mul_imm16(binary_i32imm16) -> Self::I32MulImm16;

        fn i64_mul(binary) -> Self::I64Mul;
        fn i64_mul_imm16(binary_i64imm16) -> Self::I64MulImm16;

        // Integer Division & Remainder

        fn i32_div_u(binary) -> Self::I32DivU;
        fn i32_div_u_imm16(binary_u32imm16) -> Self::I32DivUImm16;
        fn i32_div_u_imm16_rev(binary_u32imm16_rev) -> Self::I32DivUImm16Rev;

        fn i64_div_u(binary) -> Self::I64DivU;
        fn i64_div_u_imm16(binary_u64imm16) -> Self::I64DivUImm16;
        fn i64_div_u_imm16_rev(binary_u64imm16_rev) -> Self::I64DivUImm16Rev;

        fn i32_div_s(binary) -> Self::I32DivS;
        fn i32_div_s_imm16(binary_i32imm16) -> Self::I32DivSImm16;
        fn i32_div_s_imm16_rev(binary_i32imm16_rev) -> Self::I32DivSImm16Rev;

        fn i64_div_s(binary) -> Self::I64DivS;
        fn i64_div_s_imm16(binary_i64imm16) -> Self::I64DivSImm16;
        fn i64_div_s_imm16_rev(binary_i64imm16_rev) -> Self::I64DivSImm16Rev;

        fn i32_rem_u(binary) -> Self::I32RemU;
        fn i32_rem_u_imm16(binary_u32imm16) -> Self::I32RemUImm16;
        fn i32_rem_u_imm16_rev(binary_u32imm16_rev) -> Self::I32RemUImm16Rev;

        fn i64_rem_u(binary) -> Self::I64RemU;
        fn i64_rem_u_imm16(binary_u64imm16) -> Self::I64RemUImm16;
        fn i64_rem_u_imm16_rev(binary_u64imm16_rev) -> Self::I64RemUImm16Rev;

        fn i32_rem_s(binary) -> Self::I32RemS;
        fn i32_rem_s_imm16(binary_i32imm16) -> Self::I32RemSImm16;
        fn i32_rem_s_imm16_rev(binary_i32imm16_rev) -> Self::I32RemSImm16Rev;

        fn i64_rem_s(binary) -> Self::I64RemS;
        fn i64_rem_s_imm16(binary_i64imm16) -> Self::I64RemSImm16;
        fn i64_rem_s_imm16_rev(binary_i64imm16_rev) -> Self::I64RemSImm16Rev;

        // Integer Bitwise Logic

        fn i32_and(binary) -> Self::I32And;
        fn i32_and_imm16(binary_i32imm16) -> Self::I32AndImm16;

        fn i64_and(binary) -> Self::I64And;
        fn i64_and_imm16(binary_i64imm16) -> Self::I64AndImm16;

        fn i32_or(binary) -> Self::I32Or;
        fn i32_or_imm16(binary_i32imm16) -> Self::I32OrImm16;

        fn i64_or(binary) -> Self::I64Or;
        fn i64_or_imm16(binary_i64imm16) -> Self::I64OrImm16;

        fn i32_xor(binary) -> Self::I32Xor;
        fn i32_xor_imm16(binary_i32imm16) -> Self::I32XorImm16;

        fn i64_xor(binary) -> Self::I64Xor;
        fn i64_xor_imm16(binary_i64imm16) -> Self::I64XorImm16;

        // Integer Shift & Rotate

        fn i32_shl(binary) -> Self::I32Shl;
        fn i32_shl_imm(binary_i32imm16) -> Self::I32ShlImm;
        fn i32_shl_imm16_rev(binary_i32imm16_rev) -> Self::I32ShlImm16Rev;

        fn i64_shl(binary) -> Self::I64Shl;
        fn i64_shl_imm(binary_i64imm16) -> Self::I64ShlImm;
        fn i64_shl_imm16_rev(binary_i64imm16_rev) -> Self::I64ShlImm16Rev;

        fn i32_shr_u(binary) -> Self::I32ShrU;
        fn i32_shr_u_imm(binary_i32imm16) -> Self::I32ShrUImm;
        fn i32_shr_u_imm16_rev(binary_i32imm16_rev) -> Self::I32ShrUImm16Rev;

        fn i64_shr_u(binary) -> Self::I64ShrU;
        fn i64_shr_u_imm(binary_i64imm16) -> Self::I64ShrUImm;
        fn i64_shr_u_imm16_rev(binary_i64imm16_rev) -> Self::I64ShrUImm16Rev;

        fn i32_shr_s(binary) -> Self::I32ShrS;
        fn i32_shr_s_imm(binary_i32imm16) -> Self::I32ShrSImm;
        fn i32_shr_s_imm16_rev(binary_i32imm16_rev) -> Self::I32ShrSImm16Rev;

        fn i64_shr_s(binary) -> Self::I64ShrS;
        fn i64_shr_s_imm(binary_i64imm16) -> Self::I64ShrSImm;
        fn i64_shr_s_imm16_rev(binary_i64imm16_rev) -> Self::I64ShrSImm16Rev;

        fn i32_rotl(binary) -> Self::I32Rotl;
        fn i32_rotl_imm(binary_i32imm16) -> Self::I32RotlImm;
        fn i32_rotl_imm16_rev(binary_i32imm16_rev) -> Self::I32RotlImm16Rev;

        fn i64_rotl(binary) -> Self::I64Rotl;
        fn i64_rotl_imm(binary_i64imm16) -> Self::I64RotlImm;
        fn i64_rotl_imm16_rev(binary_i64imm16_rev) -> Self::I64RotlImm16Rev;

        fn i32_rotr(binary) -> Self::I32Rotr;
        fn i32_rotr_imm(binary_i32imm16) -> Self::I32RotrImm;
        fn i32_rotr_imm16_rev(binary_i32imm16_rev) -> Self::I32RotrImm16Rev;

        fn i64_rotr(binary) -> Self::I64Rotr;
        fn i64_rotr_imm(binary_i64imm16) -> Self::I64RotrImm;
        fn i64_rotr_imm16_rev(binary_i64imm16_rev) -> Self::I64RotrImm16Rev;

        // Conversions

        fn i32_extend8_s(unary) -> Self::I32Extend8S;
        fn i32_extend16_s(unary) -> Self::I32Extend16S;
        fn i64_extend8_s(unary) -> Self::I64Extend8S;
        fn i64_extend16_s(unary) -> Self::I64Extend16S;
        fn i64_extend32_s(unary) -> Self::I64Extend32S;

        fn i32_wrap_i64(unary) -> Self::I32WrapI64;
        fn i64_extend_i32_s(unary) -> Self::I64ExtendI32S;
        fn i64_extend_i32_u(unary) -> Self::I64ExtendI32U;

        fn f32_demote_f64(unary) -> Self::F32DemoteF64;
        fn f64_promote_f32(unary) -> Self::F64PromoteF32;

        fn i32_trunc_f32_s(unary) -> Self::I32TruncF32S;
        fn i32_trunc_f32_u(unary) -> Self::I32TruncF32U;
        fn i32_trunc_f64_s(unary) -> Self::I32TruncF64S;
        fn i32_trunc_f64_u(unary) -> Self::I32TruncF64U;

        fn i64_trunc_f32_s(unary) -> Self::I64TruncF32S;
        fn i64_trunc_f32_u(unary) -> Self::I64TruncF32U;
        fn i64_trunc_f64_s(unary) -> Self::I64TruncF64S;
        fn i64_trunc_f64_u(unary) -> Self::I64TruncF64U;

        fn i32_trunc_sat_f32_s(unary) -> Self::I32TruncSatF32S;
        fn i32_trunc_sat_f32_u(unary) -> Self::I32TruncSatF32U;
        fn i32_trunc_sat_f64_s(unary) -> Self::I32TruncSatF64S;
        fn i32_trunc_sat_f64_u(unary) -> Self::I32TruncSatF64U;

        fn i64_trunc_sat_f32_s(unary) -> Self::I64TruncSatF32S;
        fn i64_trunc_sat_f32_u(unary) -> Self::I64TruncSatF32U;
        fn i64_trunc_sat_f64_s(unary) -> Self::I64TruncSatF64S;
        fn i64_trunc_sat_f64_u(unary) -> Self::I64TruncSatF64U;

        fn f32_convert_i32_s(unary) -> Self::F32ConvertI32S;
        fn f32_convert_i32_u(unary) -> Self::F32ConvertI32U;
        fn f32_convert_i64_s(unary) -> Self::F32ConvertI64S;
        fn f32_convert_i64_u(unary) -> Self::F32ConvertI64U;

        fn f64_convert_i32_s(unary) -> Self::F64ConvertI32S;
        fn f64_convert_i32_u(unary) -> Self::F64ConvertI32U;
        fn f64_convert_i64_s(unary) -> Self::F64ConvertI64S;
        fn f64_convert_i64_u(unary) -> Self::F64ConvertI64U;
    }
}
