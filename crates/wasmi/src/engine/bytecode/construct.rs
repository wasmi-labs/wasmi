use super::{
    utils::{BranchOffset16, Sign},
    AnyConst32,
    BoundedRegSpan,
    BranchOffset,
    Const16,
    Const32,
    Data,
    Elem,
    EngineFunc,
    FixedRegSpan,
    Func,
    FuncType,
    Global,
    Instruction,
    Reg,
    RegSpan,
    Table,
};
use crate::core::TrapCode;
use core::num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64};

impl Instruction {
    /// Creates a new [`Instruction::Trap`] from the given [`TrapCode`].
    pub fn trap(trap_code: TrapCode) -> Self {
        Self::Trap { trap_code }
    }

    /// Creates a new [`Instruction::Const32`] from the given `value`.
    pub fn const32(value: impl Into<AnyConst32>) -> Self {
        Self::Const32 {
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::I64Const32`] from the given `value`.
    pub fn i64const32(value: impl Into<Const32<i64>>) -> Self {
        Self::I64Const32 {
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::F64Const32`] from the given `value`.
    pub fn f64const32(value: impl Into<Const32<f64>>) -> Self {
        Self::F64Const32 {
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnReg`] from the given [`Reg`] index.
    pub fn return_reg(index: impl Into<Reg>) -> Self {
        Self::ReturnReg {
            value: index.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnReg2`] for the given [`Reg`] indices.
    pub fn return_reg2(reg0: impl Into<Reg>, reg1: impl Into<Reg>) -> Self {
        Self::ReturnReg2 {
            values: [reg0.into(), reg1.into()],
        }
    }

    /// Creates a new [`Instruction::ReturnReg3`] for the given [`Reg`] indices.
    pub fn return_reg3(reg0: impl Into<Reg>, reg1: impl Into<Reg>, reg2: impl Into<Reg>) -> Self {
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
    pub fn return_span(values: BoundedRegSpan) -> Self {
        Self::ReturnSpan { values }
    }

    /// Creates a new [`Instruction::ReturnMany`] for the given [`Reg`] indices.
    pub fn return_many(reg0: impl Into<Reg>, reg1: impl Into<Reg>, reg2: impl Into<Reg>) -> Self {
        Self::ReturnMany {
            values: [reg0.into(), reg1.into(), reg2.into()],
        }
    }

    /// Creates a new [`Instruction::ReturnNez`] for the given `condition`.
    pub fn return_nez(condition: impl Into<Reg>) -> Self {
        Self::ReturnNez {
            condition: condition.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezReg`] for the given `condition` and `value`.
    pub fn return_nez_reg(condition: impl Into<Reg>, value: impl Into<Reg>) -> Self {
        Self::ReturnNezReg {
            condition: condition.into(),
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezReg2`] for the given `condition` and `value`.
    pub fn return_nez_reg2(
        condition: impl Into<Reg>,
        value0: impl Into<Reg>,
        value1: impl Into<Reg>,
    ) -> Self {
        Self::ReturnNezReg2 {
            condition: condition.into(),
            values: [value0.into(), value1.into()],
        }
    }

    /// Creates a new [`Instruction::ReturnNezImm32`] for the given `condition` and `value`.
    pub fn return_nez_imm32(condition: Reg, value: impl Into<AnyConst32>) -> Self {
        Self::ReturnNezImm32 {
            condition,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezI64Imm32`] for the given `condition` and `value`.
    pub fn return_nez_i64imm32(condition: Reg, value: impl Into<Const32<i64>>) -> Self {
        Self::ReturnNezI64Imm32 {
            condition,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezI64Imm32`] for the given `condition` and `value`.
    pub fn return_nez_f64imm32(condition: Reg, value: impl Into<Const32<f64>>) -> Self {
        Self::ReturnNezF64Imm32 {
            condition,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnNezMany`] for the given `condition` and `values`.
    pub fn return_nez_span(condition: Reg, values: BoundedRegSpan) -> Self {
        Self::ReturnNezSpan { condition, values }
    }

    /// Creates a new [`Instruction::ReturnNezMany`] for the given `condition` and `value`.
    pub fn return_nez_many(
        condition: impl Into<Reg>,
        head0: impl Into<Reg>,
        head1: impl Into<Reg>,
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

    /// Convenience constructor to create a new [`Instruction::BranchI32EqImm`] with a zero immediate value.
    pub fn branch_i32_eqz(condition: Reg, offset: BranchOffset16) -> Self {
        Self::branch_i32_eq_imm(condition, 0_i16, offset)
    }

    /// Convenience constructor to create a new [`Instruction::BranchI32NeImm`] with a zero immediate value.
    pub fn branch_i32_nez(condition: Reg, offset: BranchOffset16) -> Self {
        Self::branch_i32_ne_imm(condition, 0_i16, offset)
    }

    /// Convenience constructor to create a new [`Instruction::BranchI64EqImm`] with a zero immediate value.
    pub fn branch_i64_eqz(condition: Reg, offset: BranchOffset16) -> Self {
        Self::branch_i64_eq_imm(condition, 0_i16, offset)
    }

    /// Convenience constructor to create a new [`Instruction::BranchI64NeImm`] with a zero immediate value.
    pub fn branch_i64_nez(condition: Reg, offset: BranchOffset16) -> Self {
        Self::branch_i64_ne_imm(condition, 0_i16, offset)
    }

    /// Creates a new [`Instruction::BranchTable0`] for the given `index` and `len_targets`.
    pub fn branch_table_0(index: impl Into<Reg>, len_targets: u32) -> Self {
        Self::BranchTable0 {
            index: index.into(),
            len_targets,
        }
    }

    /// Creates a new [`Instruction::BranchTable1`] for the given `index` and `len_targets`.
    pub fn branch_table_1(index: impl Into<Reg>, len_targets: u32) -> Self {
        Self::BranchTable1 {
            index: index.into(),
            len_targets,
        }
    }

    /// Creates a new [`Instruction::BranchTable2`] for the given `index` and `len_targets`.
    pub fn branch_table_2(index: impl Into<Reg>, len_targets: u32) -> Self {
        Self::BranchTable2 {
            index: index.into(),
            len_targets,
        }
    }

    /// Creates a new [`Instruction::BranchTable3`] for the given `index` and `len_targets`.
    pub fn branch_table_3(index: impl Into<Reg>, len_targets: u32) -> Self {
        Self::BranchTable3 {
            index: index.into(),
            len_targets,
        }
    }

    /// Creates a new [`Instruction::BranchTableSpan`] for the given `index` and `len_targets`.
    pub fn branch_table_span(index: impl Into<Reg>, len_targets: u32) -> Self {
        Self::BranchTableSpan {
            index: index.into(),
            len_targets,
        }
    }

    /// Creates a new [`Instruction::BranchTableMany`] for the given `index` and `len_targets`.
    pub fn branch_table_many(index: impl Into<Reg>, len_targets: u32) -> Self {
        Self::BranchTableMany {
            index: index.into(),
            len_targets,
        }
    }

    /// Creates a new [`Instruction::BranchTableTarget`] for the given `index` and `len_targets`.
    pub fn branch_table_target(results: RegSpan, offset: BranchOffset) -> Self {
        Self::BranchTableTarget { results, offset }
    }

    /// Creates a new [`Instruction::BranchTableTargetNonOverlapping`] for the given `index` and `len_targets`.
    pub fn branch_table_target_non_overlapping(results: RegSpan, offset: BranchOffset) -> Self {
        Self::BranchTableTargetNonOverlapping { results, offset }
    }

    /// Creates a new [`Instruction::Copy`].
    pub fn copy(result: impl Into<Reg>, value: impl Into<Reg>) -> Self {
        Self::Copy {
            result: result.into(),
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::Copy2`].
    pub fn copy2(results: RegSpan, value0: impl Into<Reg>, value1: impl Into<Reg>) -> Self {
        Self::Copy2 {
            results: <FixedRegSpan<2>>::new(results).unwrap(),
            values: [value0.into(), value1.into()],
        }
    }

    /// Creates a new [`Instruction::CopyImm32`].
    pub fn copy_imm32(result: Reg, value: impl Into<AnyConst32>) -> Self {
        Self::CopyImm32 {
            result,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::CopyI64Imm32`].
    pub fn copy_i64imm32(result: Reg, value: impl Into<Const32<i64>>) -> Self {
        Self::CopyI64Imm32 {
            result,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::CopyF64Imm32`].
    pub fn copy_f64imm32(result: Reg, value: impl Into<Const32<f64>>) -> Self {
        Self::CopyF64Imm32 {
            result,
            value: value.into(),
        }
    }

    /// Creates a new [`Instruction::CopySpan`] copying multiple consecutive values.
    pub fn copy_span(results: RegSpan, values: RegSpan, len: u16) -> Self {
        debug_assert!(RegSpan::has_overlapping_copies(results, values, len));
        Self::CopySpan {
            results,
            values,
            len,
        }
    }

    /// Creates a new [`Instruction::CopySpanNonOverlapping`] copying multiple consecutive values.
    pub fn copy_span_non_overlapping(results: RegSpan, values: RegSpan, len: u16) -> Self {
        debug_assert!(!RegSpan::has_overlapping_copies(results, values, len));
        Self::CopySpanNonOverlapping {
            results,
            values,
            len,
        }
    }

    /// Creates a new [`Instruction::CopyMany`].
    pub fn copy_many(results: RegSpan, head0: impl Into<Reg>, head1: impl Into<Reg>) -> Self {
        Self::CopyMany {
            results,
            values: [head0.into(), head1.into()],
        }
    }

    /// Creates a new [`Instruction::CopyManyNonOverlapping`].
    pub fn copy_many_non_overlapping(
        results: RegSpan,
        head0: impl Into<Reg>,
        head1: impl Into<Reg>,
    ) -> Self {
        Self::CopyManyNonOverlapping {
            results,
            values: [head0.into(), head1.into()],
        }
    }

    /// Creates a new [`Instruction::GlobalGet`].
    pub fn global_get(result: Reg, global: Global) -> Self {
        Self::GlobalGet { result, global }
    }

    /// Creates a new [`Instruction::GlobalSet`].
    pub fn global_set(global: Global, input: Reg) -> Self {
        Self::GlobalSet { global, input }
    }

    /// Creates a new [`Instruction::GlobalSetI32Imm16`].
    pub fn global_set_i32imm16(global: Global, input: impl Into<Const16<i32>>) -> Self {
        Self::GlobalSetI32Imm16 {
            global,
            input: input.into(),
        }
    }

    /// Creates a new [`Instruction::GlobalSetI64Imm16`].
    pub fn global_set_i64imm16(global: Global, input: impl Into<Const16<i64>>) -> Self {
        Self::GlobalSetI64Imm16 {
            global,
            input: input.into(),
        }
    }

    /// Creates a new [`Instruction::F32CopysignImm`] instruction.
    pub fn f32_copysign_imm(result: Reg, lhs: Reg, rhs: Sign) -> Self {
        Self::F32CopysignImm { result, lhs, rhs }
    }

    /// Creates a new [`Instruction::F64CopysignImm`] instruction.
    pub fn f64_copysign_imm(result: Reg, lhs: Reg, rhs: Sign) -> Self {
        Self::F64CopysignImm { result, lhs, rhs }
    }

    /// Creates a new [`Instruction::RegisterAndImm32`].
    pub fn register_and_imm32(reg: impl Into<Reg>, imm: impl Into<AnyConst32>) -> Self {
        Self::RegisterAndImm32 {
            reg: reg.into(),
            imm: imm.into(),
        }
    }

    /// Creates a new [`Instruction::Select`].
    pub fn select(result: impl Into<Reg>, lhs: impl Into<Reg>) -> Self {
        Self::Select {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectImm32Rhs`].
    pub fn select_imm32_rhs(result: impl Into<Reg>, lhs: impl Into<Reg>) -> Self {
        Self::SelectImm32Rhs {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectImm32Lhs`].
    pub fn select_imm32_lhs(result: impl Into<Reg>, lhs: impl Into<AnyConst32>) -> Self {
        Self::SelectImm32Lhs {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectImm32`].
    pub fn select_imm32(result: impl Into<Reg>, lhs: impl Into<AnyConst32>) -> Self {
        Self::SelectImm32 {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectI64Imm32Rhs`].
    pub fn select_i64imm32_rhs(result: impl Into<Reg>, lhs: impl Into<Reg>) -> Self {
        Self::SelectI64Imm32Rhs {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectI64Imm32Lhs`].
    pub fn select_i64imm32_lhs(result: impl Into<Reg>, lhs: impl Into<Const32<i64>>) -> Self {
        Self::SelectI64Imm32Lhs {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectI64Imm32`].
    pub fn select_i64imm32(result: impl Into<Reg>, lhs: impl Into<Const32<i64>>) -> Self {
        Self::SelectI64Imm32 {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectF64Imm32Rhs`].
    pub fn select_f64imm32_rhs(result: impl Into<Reg>, lhs: impl Into<Reg>) -> Self {
        Self::SelectF64Imm32Rhs {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectF64Imm32Lhs`].
    pub fn select_f64imm32_lhs(result: impl Into<Reg>, lhs: impl Into<Const32<f64>>) -> Self {
        Self::SelectF64Imm32Lhs {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::SelectF64Imm32`].
    pub fn select_f64imm32(result: impl Into<Reg>, lhs: impl Into<Const32<f64>>) -> Self {
        Self::SelectF64Imm32 {
            result: result.into(),
            lhs: lhs.into(),
        }
    }

    /// Creates a new [`Instruction::RefFunc`] with the given `result` and `func`.
    pub fn ref_func(result: Reg, func: impl Into<Func>) -> Self {
        Self::RefFunc {
            result,
            func: func.into(),
        }
    }

    /// Creates a new [`Instruction::DataIndex`] from the given `index`.
    pub fn data_index(index: impl Into<Data>) -> Self {
        Self::DataIndex {
            index: index.into(),
        }
    }

    /// Creates a new [`Instruction::ElemIndex`] from the given `index`.
    pub fn elem_index(index: impl Into<Elem>) -> Self {
        Self::ElemIndex {
            index: index.into(),
        }
    }

    /// Creates a new [`Instruction::TableIndex`] from the given `index`.
    pub fn table_index(index: impl Into<Table>) -> Self {
        Self::TableIndex {
            index: index.into(),
        }
    }

    /// Creates a new [`Instruction::TableGet`] with the given `result` and `index`.
    pub fn table_get(result: Reg, index: Reg) -> Self {
        Self::TableGet { result, index }
    }

    /// Creates a new [`Instruction::TableGetImm`] with the given `result` and `index`.
    pub fn table_get_imm(result: Reg, index: u32) -> Self {
        Self::TableGetImm { result, index }
    }

    /// Creates a new [`Instruction::TableSize`] with the given `result` and `table`.
    pub fn table_size(result: Reg, table: impl Into<Table>) -> Self {
        Self::TableSize {
            result,
            table: table.into(),
        }
    }

    /// Creates a new [`Instruction::TableSet`] with the given `index` and `value`.
    pub fn table_set(index: Reg, value: Reg) -> Self {
        Self::TableSet { index, value }
    }

    /// Creates a new [`Instruction::TableSetAt`] with the given `index` and `value`.
    pub fn table_set_at(index: u32, value: Reg) -> Self {
        Self::TableSetAt { index, value }
    }

    /// Creates a new [`Instruction::TableCopy`] with the given `dst`, `src` and `len`.
    pub fn table_copy(dst: Reg, src: Reg, len: Reg) -> Self {
        Self::TableCopy { dst, src, len }
    }

    /// Creates a new [`Instruction::TableCopyTo`] with the given `dst`, `src` and `len`.
    pub fn table_copy_to(dst: impl Into<Const16<u32>>, src: Reg, len: Reg) -> Self {
        Self::TableCopyTo {
            dst: dst.into(),
            src,
            len,
        }
    }

    /// Creates a new [`Instruction::TableCopyFrom`] with the given `dst`, `src` and `len`.
    pub fn table_copy_from(dst: Reg, src: impl Into<Const16<u32>>, len: Reg) -> Self {
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
        len: Reg,
    ) -> Self {
        Self::TableCopyFromTo {
            dst: dst.into(),
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::TableCopyExact`] with the given `dst`, `src` and `len`.
    pub fn table_copy_exact(dst: Reg, src: Reg, len: impl Into<Const16<u32>>) -> Self {
        Self::TableCopyExact {
            dst,
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableCopyToExact`] with the given `dst`, `src` and `len`.
    pub fn table_copy_to_exact(
        dst: impl Into<Const16<u32>>,
        src: Reg,
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
        dst: Reg,
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
    pub fn table_init(dst: Reg, src: Reg, len: Reg) -> Self {
        Self::TableInit { dst, src, len }
    }

    /// Creates a new [`Instruction::TableInitTo`] with the given `dst`, `src` and `len`.
    pub fn table_init_to(dst: impl Into<Const16<u32>>, src: Reg, len: Reg) -> Self {
        Self::TableInitTo {
            dst: dst.into(),
            src,
            len,
        }
    }

    /// Creates a new [`Instruction::TableInitFrom`] with the given `dst`, `src` and `len`.
    pub fn table_init_from(dst: Reg, src: impl Into<Const16<u32>>, len: Reg) -> Self {
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
        len: Reg,
    ) -> Self {
        Self::TableInitFromTo {
            dst: dst.into(),
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::TableInitExact`] with the given `dst`, `src` and `len`.
    pub fn table_init_exact(dst: Reg, src: Reg, len: impl Into<Const16<u32>>) -> Self {
        Self::TableInitExact {
            dst,
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::TableInitToExact`] with the given `dst`, `src` and `len`.
    pub fn table_init_to_exact(
        dst: impl Into<Const16<u32>>,
        src: Reg,
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
        dst: Reg,
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
    pub fn table_fill(dst: Reg, len: Reg, value: Reg) -> Self {
        Self::TableFill { dst, len, value }
    }

    /// Creates a new [`Instruction::TableFillAt`] with the given `dst`, `len` and `value`.
    pub fn table_fill_at(dst: impl Into<Const16<u32>>, len: Reg, value: Reg) -> Self {
        Self::TableFillAt {
            dst: dst.into(),
            len,
            value,
        }
    }

    /// Creates a new [`Instruction::TableFillExact`] with the given `dst`, `len` and `value`.
    pub fn table_fill_exact(dst: Reg, len: impl Into<Const16<u32>>, value: Reg) -> Self {
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
        value: Reg,
    ) -> Self {
        Self::TableFillAtExact {
            dst: dst.into(),
            len: len.into(),
            value,
        }
    }

    /// Creates a new [`Instruction::TableGrow`] with the given `result`, `delta` and `value`.
    pub fn table_grow(result: Reg, delta: Reg, value: Reg) -> Self {
        Self::TableGrow {
            result,
            delta,
            value,
        }
    }

    /// Creates a new [`Instruction::TableGrowImm`] with the given `result`, `delta` and `value`.
    pub fn table_grow_imm(result: Reg, delta: impl Into<Const16<u32>>, value: Reg) -> Self {
        Self::TableGrowImm {
            result,
            delta: delta.into(),
            value,
        }
    }

    /// Creates a new [`Instruction::MemorySize`] with the given `result`.
    pub fn memory_size(result: Reg) -> Self {
        Self::MemorySize { result }
    }

    /// Creates a new [`Instruction::MemoryGrow`] with the given `result`, `delta`.
    pub fn memory_grow(result: Reg, delta: Reg) -> Self {
        Self::MemoryGrow { result, delta }
    }

    /// Creates a new [`Instruction::MemoryGrowBy`] with the given `result`, `delta` and `value`.
    pub fn memory_grow_by(result: Reg, delta: impl Into<Const16<u32>>) -> Self {
        Self::MemoryGrowBy {
            result,
            delta: delta.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryCopy`] with the given `dst`, `src` and `len`.
    pub fn memory_copy(dst: Reg, src: Reg, len: Reg) -> Self {
        Self::MemoryCopy { dst, src, len }
    }

    /// Creates a new [`Instruction::MemoryCopyTo`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_to(dst: impl Into<Const16<u32>>, src: Reg, len: Reg) -> Self {
        Self::MemoryCopyTo {
            dst: dst.into(),
            src,
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryCopyFrom`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_from(dst: Reg, src: impl Into<Const16<u32>>, len: Reg) -> Self {
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
        len: Reg,
    ) -> Self {
        Self::MemoryCopyFromTo {
            dst: dst.into(),
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryCopyExact`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_exact(dst: Reg, src: Reg, len: impl Into<Const16<u32>>) -> Self {
        Self::MemoryCopyExact {
            dst,
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryCopyToExact`] with the given `dst`, `src` and `len`.
    pub fn memory_copy_to_exact(
        dst: impl Into<Const16<u32>>,
        src: Reg,
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
        dst: Reg,
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
    pub fn memory_init(dst: Reg, src: Reg, len: Reg) -> Self {
        Self::MemoryInit { dst, src, len }
    }

    /// Creates a new [`Instruction::MemoryInitTo`] with the given `dst`, `src` and `len`.
    pub fn memory_init_to(dst: impl Into<Const16<u32>>, src: Reg, len: Reg) -> Self {
        Self::MemoryInitTo {
            dst: dst.into(),
            src,
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryInitFrom`] with the given `dst`, `src` and `len`.
    pub fn memory_init_from(dst: Reg, src: impl Into<Const16<u32>>, len: Reg) -> Self {
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
        len: Reg,
    ) -> Self {
        Self::MemoryInitFromTo {
            dst: dst.into(),
            src: src.into(),
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryInitExact`] with the given `dst`, `src` and `len`.
    pub fn memory_init_exact(dst: Reg, src: Reg, len: impl Into<Const16<u32>>) -> Self {
        Self::MemoryInitExact {
            dst,
            src,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryInitToExact`] with the given `dst`, `src` and `len`.
    pub fn memory_init_to_exact(
        dst: impl Into<Const16<u32>>,
        src: Reg,
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
        dst: Reg,
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
    pub fn memory_fill(dst: Reg, value: Reg, len: Reg) -> Self {
        Self::MemoryFill { dst, value, len }
    }

    /// Creates a new [`Instruction::MemoryFillAt`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_at(dst: impl Into<Const16<u32>>, value: Reg, len: Reg) -> Self {
        Self::MemoryFillAt {
            dst: dst.into(),
            value,
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryFillImm`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_imm(dst: Reg, value: u8, len: Reg) -> Self {
        Self::MemoryFillImm { dst, value, len }
    }

    /// Creates a new [`Instruction::MemoryFillExact`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_exact(dst: Reg, value: Reg, len: impl Into<Const16<u32>>) -> Self {
        Self::MemoryFillExact {
            dst,
            value,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryFillAtImm`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_at_imm(dst: impl Into<Const16<u32>>, value: u8, len: Reg) -> Self {
        Self::MemoryFillAtImm {
            dst: dst.into(),
            value,
            len,
        }
    }

    /// Creates a new [`Instruction::MemoryFillAtExact`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_at_exact(
        dst: impl Into<Const16<u32>>,
        value: Reg,
        len: impl Into<Const16<u32>>,
    ) -> Self {
        Self::MemoryFillAtExact {
            dst: dst.into(),
            value,
            len: len.into(),
        }
    }

    /// Creates a new [`Instruction::MemoryFillImmExact`] with the given `dst`, `value` and `len`.
    pub fn memory_fill_imm_exact(dst: Reg, value: u8, len: impl Into<Const16<u32>>) -> Self {
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
    pub fn register(reg: impl Into<Reg>) -> Self {
        Self::Register { reg: reg.into() }
    }

    /// Creates a new [`Instruction::Register2`] instruction parameter.
    pub fn register2(reg0: impl Into<Reg>, reg1: impl Into<Reg>) -> Self {
        Self::Register2 {
            regs: [reg0.into(), reg1.into()],
        }
    }

    /// Creates a new [`Instruction::Register3`] instruction parameter.
    pub fn register3(reg0: impl Into<Reg>, reg1: impl Into<Reg>, reg2: impl Into<Reg>) -> Self {
        Self::Register3 {
            regs: [reg0.into(), reg1.into(), reg2.into()],
        }
    }

    /// Creates a new [`Instruction::RegisterList`] instruction parameter.
    pub fn register_list(reg0: impl Into<Reg>, reg1: impl Into<Reg>, reg2: impl Into<Reg>) -> Self {
        Self::RegisterList {
            regs: [reg0.into(), reg1.into(), reg2.into()],
        }
    }

    /// Creates a new [`Instruction::RegisterSpan`].
    pub fn register_span(span: BoundedRegSpan) -> Self {
        Self::RegisterSpan { span }
    }

    /// Creates a new [`Instruction::CallIndirectParams`] for the given `index` and `table`.
    pub fn call_indirect_params(index: Reg, table: impl Into<Table>) -> Self {
        Self::CallIndirectParams {
            index,
            table: table.into(),
        }
    }

    /// Creates a new [`Instruction::CallIndirectParamsImm16`] for the given `index` and `table`.
    pub fn call_indirect_params_imm16(
        index: impl Into<Const16<u32>>,
        table: impl Into<Table>,
    ) -> Self {
        Self::CallIndirectParamsImm16 {
            index: index.into(),
            table: table.into(),
        }
    }

    /// Creates a new [`Instruction::CallInternal0`] for the given `func`.
    pub fn return_call_internal_0(func: EngineFunc) -> Self {
        Self::ReturnCallInternal0 { func }
    }

    /// Creates a new [`Instruction::CallInternal`] for the given `func`.
    pub fn return_call_internal(func: EngineFunc) -> Self {
        Self::ReturnCallInternal { func }
    }

    /// Creates a new [`Instruction::ReturnCallImported0`] for the given `func`.
    pub fn return_call_imported_0(func: impl Into<Func>) -> Self {
        Self::ReturnCallImported0 { func: func.into() }
    }

    /// Creates a new [`Instruction::ReturnCallImported`] for the given `func`.
    pub fn return_call_imported(func: impl Into<Func>) -> Self {
        Self::ReturnCallImported { func: func.into() }
    }

    /// Creates a new [`Instruction::ReturnCallIndirect0`] for the given `func`.
    pub fn return_call_indirect_0(func_type: impl Into<FuncType>) -> Self {
        Self::ReturnCallIndirect0 {
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnCallIndirect0Imm16`] for the given `func`.
    pub fn return_call_indirect_0_imm16(func_type: impl Into<FuncType>) -> Self {
        Self::ReturnCallIndirect0Imm16 {
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnCallIndirect`] for the given `func`.
    pub fn return_call_indirect(func_type: impl Into<FuncType>) -> Self {
        Self::ReturnCallIndirect {
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::ReturnCallIndirectImm16`] for the given `func`.
    pub fn return_call_indirect_imm16(func_type: impl Into<FuncType>) -> Self {
        Self::ReturnCallIndirectImm16 {
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::CallInternal0`] for the given `func`.
    pub fn call_internal_0(results: RegSpan, func: EngineFunc) -> Self {
        Self::CallInternal0 { results, func }
    }

    /// Creates a new [`Instruction::CallInternal`] for the given `func`.
    pub fn call_internal(results: RegSpan, func: EngineFunc) -> Self {
        Self::CallInternal { results, func }
    }

    /// Creates a new [`Instruction::CallImported0`] for the given `func`.
    pub fn call_imported_0(results: RegSpan, func: impl Into<Func>) -> Self {
        Self::CallImported0 {
            results,
            func: func.into(),
        }
    }

    /// Creates a new [`Instruction::CallImported`] for the given `func`.
    pub fn call_imported(results: RegSpan, func: impl Into<Func>) -> Self {
        Self::CallImported {
            results,
            func: func.into(),
        }
    }

    /// Creates a new [`Instruction::CallIndirect0`] for the given `func`.
    pub fn call_indirect_0(results: RegSpan, func_type: impl Into<FuncType>) -> Self {
        Self::CallIndirect0 {
            results,
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::CallIndirect0Imm16`] for the given `func`.
    pub fn call_indirect_0_imm16(results: RegSpan, func_type: impl Into<FuncType>) -> Self {
        Self::CallIndirect0Imm16 {
            results,
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::CallIndirect`] for the given `func`.
    pub fn call_indirect(results: RegSpan, func_type: impl Into<FuncType>) -> Self {
        Self::CallIndirect {
            results,
            func_type: func_type.into(),
        }
    }

    /// Creates a new [`Instruction::CallIndirectImm16`] for the given `func`.
    pub fn call_indirect_imm16(results: RegSpan, func_type: impl Into<FuncType>) -> Self {
        Self::CallIndirectImm16 {
            results,
            func_type: func_type.into(),
        }
    }
}

macro_rules! constructor_for_binary_instrs_v2 {
    (
        $(
            fn $fn_name:ident($($mode:tt)?) -> Self::$op_code:ident;
        )* $(,)?
    ) => {
        impl Instruction {
            $(
                constructor_for_binary_instrs_v2! {
                    @impl fn $fn_name($($mode)?) -> Self::$op_code
                }
            )*
        }
    };
    ( @impl fn $fn_name:ident() -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: Reg, rhs: Reg) -> Self {
            Self::$op_code { result, lhs, rhs }
        }
    };
    ( @impl fn $fn_name:ident({i32.binary_imm<i16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: Reg, rhs: impl Into<Const16<i32>>) -> Self {
            Self::$op_code { result, lhs, rhs: rhs.into() }
        }
    };
    ( @impl fn $fn_name:ident({i32.binary_imm<u16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: Reg, rhs: impl Into<Const16<u32>>) -> Self {
            Self::$op_code { result, lhs, rhs: rhs.into() }
        }
    };
    ( @impl fn $fn_name:ident({i64.binary_imm<i16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: Reg, rhs: impl Into<Const16<i64>>) -> Self {
            Self::$op_code { result, lhs, rhs: rhs.into() }
        }
    };
    ( @impl fn $fn_name:ident({i64.binary_imm<u16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: Reg, rhs: impl Into<Const16<u64>>) -> Self {
            Self::$op_code { result, lhs, rhs: rhs.into() }
        }
    };
    ( @impl fn $fn_name:ident({i32.binary_imm_rev<i16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: impl Into<Const16<i32>>, rhs: Reg) -> Self {
            Self::$op_code { result, lhs: lhs.into(), rhs }
        }
    };
    ( @impl fn $fn_name:ident({i32.binary_imm_rev<u16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: impl Into<Const16<u32>>, rhs: Reg) -> Self {
            Self::$op_code { result, lhs: lhs.into(), rhs }
        }
    };
    ( @impl fn $fn_name:ident({i64.binary_imm_rev<i16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: impl Into<Const16<i64>>, rhs: Reg) -> Self {
            Self::$op_code { result, lhs: lhs.into(), rhs }
        }
    };
    ( @impl fn $fn_name:ident({i64.binary_imm_rev<u16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, lhs: Const16<u64>, rhs: Reg) -> Self {
            Self::$op_code { result, lhs: lhs.into(), rhs }
        }
    };
}
constructor_for_binary_instrs_v2! {
    // Float Arithmetic

    fn f32_add() -> Self::F32Add;
    fn f64_add() -> Self::F64Add;
    fn f32_sub() -> Self::F32Sub;
    fn f64_sub() -> Self::F64Sub;
    fn f32_mul() -> Self::F32Mul;
    fn f64_mul() -> Self::F64Mul;
    fn f32_div() -> Self::F32Div;
    fn f64_div() -> Self::F64Div;
    fn f32_min() -> Self::F32Min;
    fn f64_min() -> Self::F64Min;
    fn f32_max() -> Self::F32Max;
    fn f64_max() -> Self::F64Max;
    fn f32_copysign() -> Self::F32Copysign;
    fn f64_copysign() -> Self::F64Copysign;

    // Integer Comparison

    fn i32_eq() -> Self::I32Eq;
    fn i32_eq_imm16({i32.binary_imm<i16>}) -> Self::I32EqImm16;

    fn i64_eq() -> Self::I64Eq;
    fn i64_eq_imm16({i64.binary_imm<i16>}) -> Self::I64EqImm16;

    fn i32_ne() -> Self::I32Ne;
    fn i32_ne_imm16({i32.binary_imm<i16>}) -> Self::I32NeImm16;

    fn i64_ne() -> Self::I64Ne;
    fn i64_ne_imm16({i64.binary_imm<i16>}) -> Self::I64NeImm16;

    fn i32_lt_s() -> Self::I32LtS;
    fn i32_lt_s_imm16({i32.binary_imm<i16>}) -> Self::I32LtSImm16;

    fn i64_lt_s() -> Self::I64LtS;
    fn i64_lt_s_imm16({i64.binary_imm<i16>}) -> Self::I64LtSImm16;

    fn i32_lt_u() -> Self::I32LtU;
    fn i32_lt_u_imm16({i32.binary_imm<u16>}) -> Self::I32LtUImm16;

    fn i64_lt_u() -> Self::I64LtU;
    fn i64_lt_u_imm16({i64.binary_imm<u16>}) -> Self::I64LtUImm16;

    fn i32_le_s() -> Self::I32LeS;
    fn i32_le_s_imm16({i32.binary_imm<i16>}) -> Self::I32LeSImm16;

    fn i64_le_s() -> Self::I64LeS;
    fn i64_le_s_imm16({i64.binary_imm<i16>}) -> Self::I64LeSImm16;

    fn i32_le_u() -> Self::I32LeU;
    fn i32_le_u_imm16({i32.binary_imm<u16>}) -> Self::I32LeUImm16;

    fn i64_le_u() -> Self::I64LeU;
    fn i64_le_u_imm16({i64.binary_imm<u16>}) -> Self::I64LeUImm16;

    fn i32_gt_s() -> Self::I32GtS;
    fn i32_gt_s_imm16({i32.binary_imm<i16>}) -> Self::I32GtSImm16;

    fn i64_gt_s() -> Self::I64GtS;
    fn i64_gt_s_imm16({i64.binary_imm<i16>}) -> Self::I64GtSImm16;

    fn i32_gt_u() -> Self::I32GtU;
    fn i32_gt_u_imm16({i32.binary_imm<u16>}) -> Self::I32GtUImm16;

    fn i64_gt_u() -> Self::I64GtU;
    fn i64_gt_u_imm16({i64.binary_imm<u16>}) -> Self::I64GtUImm16;

    fn i32_ge_s() -> Self::I32GeS;
    fn i32_ge_s_imm16({i32.binary_imm<i16>}) -> Self::I32GeSImm16;

    fn i64_ge_s() -> Self::I64GeS;
    fn i64_ge_s_imm16({i64.binary_imm<i16>}) -> Self::I64GeSImm16;

    fn i32_ge_u() -> Self::I32GeU;
    fn i32_ge_u_imm16({i32.binary_imm<u16>}) -> Self::I32GeUImm16;

    fn i64_ge_u() -> Self::I64GeU;
    fn i64_ge_u_imm16({i64.binary_imm<u16>}) -> Self::I64GeUImm16;

    // Float Comparison

    fn f32_eq() -> Self::F32Eq;
    fn f64_eq() -> Self::F64Eq;
    fn f32_ne() -> Self::F32Ne;
    fn f64_ne() -> Self::F64Ne;
    fn f32_lt() -> Self::F32Lt;
    fn f64_lt() -> Self::F64Lt;
    fn f32_le() -> Self::F32Le;
    fn f64_le() -> Self::F64Le;
    fn f32_gt() -> Self::F32Gt;
    fn f64_gt() -> Self::F64Gt;
    fn f32_ge() -> Self::F32Ge;
    fn f64_ge() -> Self::F64Ge;

    // Integer Arithmetic

    fn i32_add() -> Self::I32Add;
    fn i32_add_imm16({i32.binary_imm<i16>}) -> Self::I32AddImm16;

    fn i64_add() -> Self::I64Add;
    fn i64_add_imm16({i64.binary_imm<i16>}) -> Self::I64AddImm16;

    fn i32_sub() -> Self::I32Sub;
    fn i32_sub_imm16_rev({i32.binary_imm_rev<i16>}) -> Self::I32SubImm16Rev;

    fn i64_sub() -> Self::I64Sub;
    fn i64_sub_imm16_rev({i64.binary_imm_rev<i16>}) -> Self::I64SubImm16Rev;

    fn i32_mul() -> Self::I32Mul;
    fn i32_mul_imm16({i32.binary_imm<i16>}) -> Self::I32MulImm16;

    fn i64_mul() -> Self::I64Mul;
    fn i64_mul_imm16({i64.binary_imm<i16>}) -> Self::I64MulImm16;

    // Integer Division & Remainder

    fn i32_div_u() -> Self::I32DivU;
    fn i32_div_u_imm16_rev({i32.binary_imm_rev<u16>}) -> Self::I32DivUImm16Rev;

    fn i64_div_u() -> Self::I64DivU;
    fn i64_div_u_imm16_rev({i64.binary_imm_rev<u16>}) -> Self::I64DivUImm16Rev;

    fn i32_div_s() -> Self::I32DivS;
    fn i32_div_s_imm16_rev({i32.binary_imm_rev<i16>}) -> Self::I32DivSImm16Rev;

    fn i64_div_s() -> Self::I64DivS;
    fn i64_div_s_imm16_rev({i64.binary_imm_rev<i16>}) -> Self::I64DivSImm16Rev;

    fn i32_rem_u() -> Self::I32RemU;
    fn i32_rem_u_imm16_rev({i32.binary_imm_rev<u16>}) -> Self::I32RemUImm16Rev;

    fn i64_rem_u() -> Self::I64RemU;
    fn i64_rem_u_imm16_rev({i64.binary_imm_rev<u16>}) -> Self::I64RemUImm16Rev;

    fn i32_rem_s() -> Self::I32RemS;
    fn i32_rem_s_imm16_rev({i32.binary_imm_rev<i16>}) -> Self::I32RemSImm16Rev;

    fn i64_rem_s() -> Self::I64RemS;
    fn i64_rem_s_imm16_rev({i64.binary_imm_rev<i16>}) -> Self::I64RemSImm16Rev;

    // Integer Bitwise Logic

    fn i32_and() -> Self::I32And;
    fn i32_and_eqz() -> Self::I32AndEqz;
    fn i32_and_eqz_imm16({i32.binary_imm<i16>}) -> Self::I32AndEqzImm16;
    fn i32_and_imm16({i32.binary_imm<i16>}) -> Self::I32AndImm16;

    fn i64_and() -> Self::I64And;
    fn i64_and_imm16({i64.binary_imm<i16>}) -> Self::I64AndImm16;

    fn i32_or() -> Self::I32Or;
    fn i32_or_eqz() -> Self::I32OrEqz;
    fn i32_or_eqz_imm16({i32.binary_imm<i16>}) -> Self::I32OrEqzImm16;
    fn i32_or_imm16({i32.binary_imm<i16>}) -> Self::I32OrImm16;

    fn i64_or() -> Self::I64Or;
    fn i64_or_imm16({i64.binary_imm<i16>}) -> Self::I64OrImm16;

    fn i32_xor() -> Self::I32Xor;
    fn i32_xor_eqz() -> Self::I32XorEqz;
    fn i32_xor_eqz_imm16({i32.binary_imm<i16>}) -> Self::I32XorEqzImm16;
    fn i32_xor_imm16({i32.binary_imm<i16>}) -> Self::I32XorImm16;

    fn i64_xor() -> Self::I64Xor;
    fn i64_xor_imm16({i64.binary_imm<i16>}) -> Self::I64XorImm16;

    // Integer Shift & Rotate

    fn i32_shl() -> Self::I32Shl;
    fn i32_shl_imm({i32.binary_imm<i16>}) -> Self::I32ShlImm;
    fn i32_shl_imm16_rev({i32.binary_imm_rev<i16>}) -> Self::I32ShlImm16Rev;

    fn i64_shl() -> Self::I64Shl;
    fn i64_shl_imm({i64.binary_imm<i16>}) -> Self::I64ShlImm;
    fn i64_shl_imm16_rev({i64.binary_imm_rev<i16>}) -> Self::I64ShlImm16Rev;

    fn i32_shr_u() -> Self::I32ShrU;
    fn i32_shr_u_imm({i32.binary_imm<i16>}) -> Self::I32ShrUImm;
    fn i32_shr_u_imm16_rev({i32.binary_imm_rev<i16>}) -> Self::I32ShrUImm16Rev;

    fn i64_shr_u() -> Self::I64ShrU;
    fn i64_shr_u_imm({i64.binary_imm<i16>}) -> Self::I64ShrUImm;
    fn i64_shr_u_imm16_rev({i64.binary_imm_rev<i16>}) -> Self::I64ShrUImm16Rev;

    fn i32_shr_s() -> Self::I32ShrS;
    fn i32_shr_s_imm({i32.binary_imm<i16>}) -> Self::I32ShrSImm;
    fn i32_shr_s_imm16_rev({i32.binary_imm_rev<i16>}) -> Self::I32ShrSImm16Rev;

    fn i64_shr_s() -> Self::I64ShrS;
    fn i64_shr_s_imm({i64.binary_imm<i16>}) -> Self::I64ShrSImm;
    fn i64_shr_s_imm16_rev({i64.binary_imm_rev<i16>}) -> Self::I64ShrSImm16Rev;

    fn i32_rotl() -> Self::I32Rotl;
    fn i32_rotl_imm({i32.binary_imm<i16>}) -> Self::I32RotlImm;
    fn i32_rotl_imm16_rev({i32.binary_imm_rev<i16>}) -> Self::I32RotlImm16Rev;

    fn i64_rotl() -> Self::I64Rotl;
    fn i64_rotl_imm({i64.binary_imm<i16>}) -> Self::I64RotlImm;
    fn i64_rotl_imm16_rev({i64.binary_imm_rev<i16>}) -> Self::I64RotlImm16Rev;

    fn i32_rotr() -> Self::I32Rotr;
    fn i32_rotr_imm({i32.binary_imm<i16>}) -> Self::I32RotrImm;
    fn i32_rotr_imm16_rev({i32.binary_imm_rev<i16>}) -> Self::I32RotrImm16Rev;

    fn i64_rotr() -> Self::I64Rotr;
    fn i64_rotr_imm({i64.binary_imm<i16>}) -> Self::I64RotrImm;
    fn i64_rotr_imm16_rev({i64.binary_imm_rev<i16>}) -> Self::I64RotrImm16Rev;
}

macro_rules! constructor_for_unary_instrs {
    ( $( fn $constructor_name:ident() -> Self::$instr_name:ident; )* ) => {
        impl Instruction {
            $(
                #[doc = concat!("Creates a new [`Instruction::", stringify!($instr_name), "`].")]
                pub fn $constructor_name(result: Reg, input: Reg) -> Self {
                    Self::$instr_name { result, input }
                }
            )*
        }
    }
}
constructor_for_unary_instrs! {
    // Integer Unary

    fn i32_clz() -> Self::I32Clz;
    fn i32_ctz() -> Self::I32Ctz;
    fn i32_popcnt() -> Self::I32Popcnt;

    fn i64_clz() -> Self::I64Clz;
    fn i64_ctz() -> Self::I64Ctz;
    fn i64_popcnt() -> Self::I64Popcnt;

    // Float Unary

    fn f32_abs() -> Self::F32Abs;
    fn f32_neg() -> Self::F32Neg;
    fn f32_ceil() -> Self::F32Ceil;
    fn f32_floor() -> Self::F32Floor;
    fn f32_trunc() -> Self::F32Trunc;
    fn f32_nearest() -> Self::F32Nearest;
    fn f32_sqrt() -> Self::F32Sqrt;

    fn f64_abs() -> Self::F64Abs;
    fn f64_neg() -> Self::F64Neg;
    fn f64_ceil() -> Self::F64Ceil;
    fn f64_floor() -> Self::F64Floor;
    fn f64_trunc() -> Self::F64Trunc;
    fn f64_nearest() -> Self::F64Nearest;
    fn f64_sqrt() -> Self::F64Sqrt;

    // Conversion

    fn i32_extend8_s() -> Self::I32Extend8S;
    fn i32_extend16_s() -> Self::I32Extend16S;
    fn i64_extend8_s() -> Self::I64Extend8S;
    fn i64_extend16_s() -> Self::I64Extend16S;
    fn i64_extend32_s() -> Self::I64Extend32S;

    fn i32_wrap_i64() -> Self::I32WrapI64;

    fn f32_demote_f64() -> Self::F32DemoteF64;
    fn f64_promote_f32() -> Self::F64PromoteF32;

    fn i32_trunc_f32_s() -> Self::I32TruncF32S;
    fn i32_trunc_f32_u() -> Self::I32TruncF32U;
    fn i32_trunc_f64_s() -> Self::I32TruncF64S;
    fn i32_trunc_f64_u() -> Self::I32TruncF64U;

    fn i64_trunc_f32_s() -> Self::I64TruncF32S;
    fn i64_trunc_f32_u() -> Self::I64TruncF32U;
    fn i64_trunc_f64_s() -> Self::I64TruncF64S;
    fn i64_trunc_f64_u() -> Self::I64TruncF64U;

    fn i32_trunc_sat_f32_s() -> Self::I32TruncSatF32S;
    fn i32_trunc_sat_f32_u() -> Self::I32TruncSatF32U;
    fn i32_trunc_sat_f64_s() -> Self::I32TruncSatF64S;
    fn i32_trunc_sat_f64_u() -> Self::I32TruncSatF64U;

    fn i64_trunc_sat_f32_s() -> Self::I64TruncSatF32S;
    fn i64_trunc_sat_f32_u() -> Self::I64TruncSatF32U;
    fn i64_trunc_sat_f64_s() -> Self::I64TruncSatF64S;
    fn i64_trunc_sat_f64_u() -> Self::I64TruncSatF64U;

    fn f32_convert_i32_s() -> Self::F32ConvertI32S;
    fn f32_convert_i32_u() -> Self::F32ConvertI32U;
    fn f32_convert_i64_s() -> Self::F32ConvertI64S;
    fn f32_convert_i64_u() -> Self::F32ConvertI64U;

    fn f64_convert_i32_s() -> Self::F64ConvertI32S;
    fn f64_convert_i32_u() -> Self::F64ConvertI32U;
    fn f64_convert_i64_s() -> Self::F64ConvertI64S;
    fn f64_convert_i64_u() -> Self::F64ConvertI64U;
}

macro_rules! constructor_for_load_instrs {
    (
        $(
            fn $fn_name:ident($($mode:ident)?) -> Self::$op_code:ident;
        )* $(,)?
    ) => {
        impl Instruction {
            $(
                constructor_for_load_instrs! {
                    @impl fn $fn_name($($mode)?) -> Self::$op_code
                }
            )*
        }
    };
    ( @impl fn $fn_name:ident() -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, ptr: Reg) -> Self {
            Self::$op_code { result, ptr }
        }
    };
    ( @impl fn $fn_name:ident(at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, address: u32) -> Self {
            Self::$op_code { result, address: u32::from(address) }
        }
    };
    ( @impl fn $fn_name:ident(offset16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(result: Reg, ptr: Reg, offset: Const16<u32>) -> Self {
            Self::$op_code { result, ptr, offset }
        }
    };
}
constructor_for_load_instrs! {
    fn i32_load() -> Self::I32Load;
    fn i32_load_at(at) -> Self::I32LoadAt;
    fn i32_load_offset16(offset16) -> Self::I32LoadOffset16;

    fn i32_load8_s() -> Self::I32Load8s;
    fn i32_load8_s_at(at) -> Self::I32Load8sAt;
    fn i32_load8_s_offset16(offset16) -> Self::I32Load8sOffset16;

    fn i32_load8_u() -> Self::I32Load8u;
    fn i32_load8_u_at(at) -> Self::I32Load8uAt;
    fn i32_load8_u_offset16(offset16) -> Self::I32Load8uOffset16;

    fn i32_load16_s() -> Self::I32Load16s;
    fn i32_load16_s_at(at) -> Self::I32Load16sAt;
    fn i32_load16_s_offset16(offset16) -> Self::I32Load16sOffset16;

    fn i32_load16_u() -> Self::I32Load16u;
    fn i32_load16_u_at(at) -> Self::I32Load16uAt;
    fn i32_load16_u_offset16(offset16) -> Self::I32Load16uOffset16;

    fn i64_load() -> Self::I64Load;
    fn i64_load_at(at) -> Self::I64LoadAt;
    fn i64_load_offset16(offset16) -> Self::I64LoadOffset16;

    fn i64_load8_s() -> Self::I64Load8s;
    fn i64_load8_s_at(at) -> Self::I64Load8sAt;
    fn i64_load8_s_offset16(offset16) -> Self::I64Load8sOffset16;

    fn i64_load8_u() -> Self::I64Load8u;
    fn i64_load8_u_at(at) -> Self::I64Load8uAt;
    fn i64_load8_u_offset16(offset16) -> Self::I64Load8uOffset16;

    fn i64_load16_s() -> Self::I64Load16s;
    fn i64_load16_s_at(at) -> Self::I64Load16sAt;
    fn i64_load16_s_offset16(offset16) -> Self::I64Load16sOffset16;

    fn i64_load16_u() -> Self::I64Load16u;
    fn i64_load16_u_at(at) -> Self::I64Load16uAt;
    fn i64_load16_u_offset16(offset16) -> Self::I64Load16uOffset16;

    fn i64_load32_s() -> Self::I64Load32s;
    fn i64_load32_s_at(at) -> Self::I64Load32sAt;
    fn i64_load32_s_offset16(offset16) -> Self::I64Load32sOffset16;

    fn i64_load32_u() -> Self::I64Load32u;
    fn i64_load32_u_at(at) -> Self::I64Load32uAt;
    fn i64_load32_u_offset16(offset16) -> Self::I64Load32uOffset16;

    fn f32_load() -> Self::F32Load;
    fn f32_load_at(at) -> Self::F32LoadAt;
    fn f32_load_offset16(offset16) -> Self::F32LoadOffset16;

    fn f64_load() -> Self::F64Load;
    fn f64_load_at(at) -> Self::F64LoadAt;
    fn f64_load_offset16(offset16) -> Self::F64LoadOffset16;
}

macro_rules! constructor_for_store_instrs {
    (
        $( fn $fn_name:ident($($mode:tt)?) -> Self::$op_code:ident; )* $(,)?
    ) => {
        impl Instruction {
            $(
                constructor_for_store_instrs! {
                    @impl fn $fn_name($($mode)?) -> Self::$op_code
                }
            )*
        }
    };
    ( @impl fn $fn_name:ident() -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Reg, offset: u32) -> Self {
            Self::$op_code { ptr, offset: u32::from(offset) }
        }
    };
    ( @impl fn $fn_name:ident(at) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: u32, value: Reg) -> Self {
            Self::$op_code { address: u32::from(address), value }
        }
    };
    ( @impl fn $fn_name:ident(offset16) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Reg, offset: u16, value: Reg) -> Self {
            Self::$op_code { ptr, offset: offset.into(), value }
        }
    };
    ( @impl fn $fn_name:ident({offset16_imm<i8>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Reg, offset: u16, value: i8) -> Self {
            Self::$op_code { ptr, offset: offset.into(), value: value.into() }
        }
    };
    ( @impl fn $fn_name:ident({offset16_imm<i16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(ptr: Reg, offset: u16, value: i16) -> Self {
            Self::$op_code { ptr, offset: offset.into(), value: value.into() }
        }
    };
    ( @impl fn $fn_name:ident({at_imm<i8>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: u32, value: i8) -> Self {
            Self::$op_code { address: u32::from(address), value: value.into() }
        }
    };
    ( @impl fn $fn_name:ident({at_imm<i16>}) -> Self::$op_code:ident ) => {
        #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
        pub fn $fn_name(address: u32, value: i16) -> Self {
            Self::$op_code { address: u32::from(address), value: value.into() }
        }
    };
}
constructor_for_store_instrs! {
    fn i32_store() -> Self::I32Store;
    fn i32_store_offset16(offset16) -> Self::I32StoreOffset16;
    fn i32_store_offset16_imm16({offset16_imm<i16>}) -> Self::I32StoreOffset16Imm16;
    fn i32_store_at(at) -> Self::I32StoreAt;
    fn i32_store_at_imm16({at_imm<i16>}) -> Self::I32StoreAtImm16;

    fn i32_store8() -> Self::I32Store8;
    fn i32_store8_offset16(offset16) -> Self::I32Store8Offset16;
    fn i32_store8_offset16_imm({offset16_imm<i8>}) -> Self::I32Store8Offset16Imm;
    fn i32_store8_at(at) -> Self::I32Store8At;
    fn i32_store8_at_imm({at_imm<i8>}) -> Self::I32Store8AtImm;

    fn i32_store16() -> Self::I32Store16;
    fn i32_store16_offset16(offset16) -> Self::I32Store16Offset16;
    fn i32_store16_offset16_imm({offset16_imm<i16>}) -> Self::I32Store16Offset16Imm;
    fn i32_store16_at(at) -> Self::I32Store16At;
    fn i32_store16_at_imm({at_imm<i16>}) -> Self::I32Store16AtImm;

    fn i64_store() -> Self::I64Store;
    fn i64_store_offset16(offset16) -> Self::I64StoreOffset16;
    fn i64_store_offset16_imm16({offset16_imm<i16>}) -> Self::I64StoreOffset16Imm16;
    fn i64_store_at(at) -> Self::I64StoreAt;
    fn i64_store_at_imm16({at_imm<i16>}) -> Self::I64StoreAtImm16;

    fn i64_store8() -> Self::I64Store8;
    fn i64_store8_offset16(offset16) -> Self::I64Store8Offset16;
    fn i64_store8_offset16_imm({offset16_imm<i8>}) -> Self::I64Store8Offset16Imm;
    fn i64_store8_at(at) -> Self::I64Store8At;
    fn i64_store8_at_imm({at_imm<i8>}) -> Self::I64Store8AtImm;

    fn i64_store16() -> Self::I64Store16;
    fn i64_store16_offset16(offset16) -> Self::I64Store16Offset16;
    fn i64_store16_offset16_imm({offset16_imm<i16>}) -> Self::I64Store16Offset16Imm;
    fn i64_store16_at(at) -> Self::I64Store16At;
    fn i64_store16_at_imm({at_imm<i16>}) -> Self::I64Store16AtImm;

    fn i64_store32() -> Self::I64Store32;
    fn i64_store32_offset16(offset16) -> Self::I64Store32Offset16;
    fn i64_store32_offset16_imm16({offset16_imm<i16>}) -> Self::I64Store32Offset16Imm16;
    fn i64_store32_at(at) -> Self::I64Store32At;
    fn i64_store32_at_imm16({at_imm<i16>}) -> Self::I64Store32AtImm16;

    fn f32_store() -> Self::F32Store;
    fn f32_store_offset16(offset16) -> Self::F32StoreOffset16;
    fn f32_store_at(at) -> Self::F32StoreAt;

    fn f64_store() -> Self::F64Store;
    fn f64_store_offset16(offset16) -> Self::F64StoreOffset16;
    fn f64_store_at(at) -> Self::F64StoreAt;
}

impl Instruction {
    /// Creates a new [`Instruction::BranchCmpFallback`].
    pub fn branch_cmp_fallback(lhs: Reg, rhs: Reg, params: Reg) -> Self {
        Self::BranchCmpFallback { lhs, rhs, params }
    }
}

macro_rules! constructor_for_branch_cmp_instrs {
    ( $( fn $name:ident() -> Self::$op_code:ident; )* ) => {
        impl Instruction {
            $(
                #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
                pub fn $name(lhs: impl Into<Reg>, rhs: impl Into<Reg>, offset: BranchOffset16) -> Self {
                    Self::$op_code { lhs: lhs.into(), rhs: rhs.into(), offset }
                }
            )*
        }
    }
}
constructor_for_branch_cmp_instrs! {
    fn branch_i32_and() -> Self::BranchI32And;
    fn branch_i32_or() -> Self::BranchI32Or;
    fn branch_i32_xor() -> Self::BranchI32Xor;
    fn branch_i32_and_eqz() -> Self::BranchI32AndEqz;
    fn branch_i32_or_eqz() -> Self::BranchI32OrEqz;
    fn branch_i32_xor_eqz() -> Self::BranchI32XorEqz;
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

macro_rules! constructor_for_branch_cmp_imm_instrs {
    ( $( fn $name:ident($ty:ty) -> Self::$op_code:ident; )* ) => {
        impl Instruction {
            $(
                #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
                pub fn $name(lhs: impl Into<Reg>, rhs: impl Into<Const16<$ty>>, offset: BranchOffset16) -> Self {
                    Self::$op_code { lhs: lhs.into(), rhs: rhs.into(), offset }
                }
            )*
        }
    }
}
constructor_for_branch_cmp_imm_instrs! {
    fn branch_i32_and_imm(i32) -> Self::BranchI32AndImm;
    fn branch_i32_or_imm(i32) -> Self::BranchI32OrImm;
    fn branch_i32_xor_imm(i32) -> Self::BranchI32XorImm;
    fn branch_i32_and_eqz_imm(i32) -> Self::BranchI32AndEqzImm;
    fn branch_i32_or_eqz_imm(i32) -> Self::BranchI32OrEqzImm;
    fn branch_i32_xor_eqz_imm(i32) -> Self::BranchI32XorEqzImm;
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

macro_rules! constructor_for_divrem_imm_instrs {
    ( $( fn $name:ident($ty:ty) -> Self::$op_code:ident; )* ) => {
        impl Instruction {
            $(
                #[doc = concat!("Creates a new [`Instruction::", stringify!($op_code), "`].")]
                pub fn $name(result: Reg, lhs: Reg, rhs: impl Into<Const16<$ty>>) -> Self {
                    Self::$op_code { result, lhs, rhs: rhs.into() }
                }
            )*
        }
    }
}
constructor_for_divrem_imm_instrs! {
    fn i32_div_s_imm16(NonZeroI32) -> Self::I32DivSImm16;
    fn i32_div_u_imm16(NonZeroU32) -> Self::I32DivUImm16;
    fn i32_rem_s_imm16(NonZeroI32) -> Self::I32RemSImm16;
    fn i32_rem_u_imm16(NonZeroU32) -> Self::I32RemUImm16;

    fn i64_div_s_imm16(NonZeroI64) -> Self::I64DivSImm16;
    fn i64_div_u_imm16(NonZeroU64) -> Self::I64DivUImm16;
    fn i64_rem_s_imm16(NonZeroI64) -> Self::I64RemSImm16;
    fn i64_rem_u_imm16(NonZeroU64) -> Self::I64RemUImm16;
}
