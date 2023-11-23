use core::mem;

use wasmi_core::UntypedValue;

use crate::{
    engine::{
        bytecode::{FuncIdx, SignatureIdx},
        regmach::{
            bytecode::{
                BinAssignInstr,
                BinAssignInstrImm,
                BinInstr,
                BinInstrImm,
                Const16,
                Const32,
                Instruction,
                LoadAtInstr,
                LoadInstr,
                LoadOffset16Instr,
                Register,
                RegisterSpan,
            },
            code_map::CompiledFuncEntity,
            translator::stack::ValueStack,
        },
        CompiledFunc,
        TranslationError,
    },
    module::ModuleResources,
};

macro_rules! relink_binop {
    ($this:expr, $instr:ident, $new_result:ident, $old_result:ident, $make_instr:expr) => {
        match relink_binop($instr, $new_result, $old_result, $make_instr)? {
            RelinkResult::Unchanged => Ok(false),
            RelinkResult::Relinked => Ok(true),
            RelinkResult::Exchanged(new_instr) => relink_exchange($this, new_instr),
        }
    };
}

macro_rules! relink_binop_imm16 {
    ($ty:ty, $this:expr, $instr:ident, $new_result:ident, $old_result:ident, $make_instr:expr) => {
        match relink_binop_imm16::<$ty>($instr, $new_result, $old_result, $make_instr)? {
            RelinkResult::Unchanged => Ok(false),
            RelinkResult::Relinked => Ok(true),
            RelinkResult::Exchanged(new_instr) => relink_exchange($this, new_instr),
        }
    };
}

macro_rules! relink_binop_assign {
    ($this:expr, $instr:ident, $new_result:ident, $old_result:ident, $make_instr:expr) => {
        match relink_binop_assign($instr, $new_result, $old_result, $make_instr) {
            None => Ok(false),
            Some(new_instr) => relink_exchange($this, new_instr),
        }
    };
}

macro_rules! relink_binop_assign_imm {
    ($ty:ty, $this:expr, $instr:ident, $stack:ident, $new_result:ident, $old_result:ident, $make_instr:expr, $make_instr_imm:expr) => {
        match relink_binop_assign_imm::<$ty>(
            $instr,
            $stack,
            $new_result,
            $old_result,
            $make_instr,
            $make_instr_imm,
        )? {
            None => Ok(false),
            Some(new_instr) => relink_exchange($this, new_instr),
        }
    };
}

macro_rules! relink_binop_assign_fimm {
    ($ty:ty, $this:expr, $instr:ident, $stack:ident, $new_result:ident, $old_result:ident, $make_instr:expr) => {
        match relink_binop_assign_fimm::<$ty>(
            $instr,
            $stack,
            $new_result,
            $old_result,
            $make_instr,
        )? {
            None => Ok(false),
            Some(new_instr) => relink_exchange($this, new_instr),
        }
    };
}

impl Instruction {
    #[rustfmt::skip]
    pub fn relink_result(
        &mut self,
        stack: &mut ValueStack,
        res: &ModuleResources,
        new_result: Register,
        old_result: Register,
    ) -> Result<bool, TranslationError> {
        use Instruction as I;
        match self {
            I::TableIdx(_)
            | I::DataSegmentIdx(_)
            | I::ElementSegmentIdx(_)
            | I::Const32(_)
            | I::I64Const32(_)
            | I::F64Const32(_)
            | I::Register(_)
            | I::Register2(_)
            | I::Register3(_)
            | I::RegisterList(_)
            | I::CallIndirectParams(_)
            | I::CallIndirectParamsImm16(_)
            | I::Trap(_)
            | I::ConsumeFuel(_)
            | I::Return
            | I::ReturnReg { .. }
            | I::ReturnReg2 { .. }
            | I::ReturnReg3 { .. }
            | I::ReturnImm32 { .. }
            | I::ReturnI64Imm32 { .. }
            | I::ReturnF64Imm32 { .. }
            | I::ReturnSpan { .. }
            | I::ReturnMany { .. }
            | I::ReturnNez { .. }
            | I::ReturnNezReg { .. }
            | I::ReturnNezReg2 { .. }
            | I::ReturnNezImm32 { .. }
            | I::ReturnNezI64Imm32 { .. }
            | I::ReturnNezF64Imm32 { .. }
            | I::ReturnNezSpan { .. }
            | I::ReturnNezMany { .. }
            | I::Branch { .. }
            | I::BranchEqz { .. }
            | I::BranchNez { .. }
            | I::BranchTable { .. }
            | I::BranchI32Eq(_)
            | I::BranchI32EqImm(_)
            | I::BranchI32Ne(_)
            | I::BranchI32NeImm(_)
            | I::BranchI32LtS(_)
            | I::BranchI32LtSImm(_)
            | I::BranchI32LtU(_)
            | I::BranchI32LtUImm(_)
            | I::BranchI32LeS(_)
            | I::BranchI32LeSImm(_)
            | I::BranchI32LeU(_)
            | I::BranchI32LeUImm(_)
            | I::BranchI32GtS(_)
            | I::BranchI32GtSImm(_)
            | I::BranchI32GtU(_)
            | I::BranchI32GtUImm(_)
            | I::BranchI32GeS(_)
            | I::BranchI32GeSImm(_)
            | I::BranchI32GeU(_)
            | I::BranchI32GeUImm(_)
            | I::BranchI64Eq(_)
            | I::BranchI64EqImm(_)
            | I::BranchI64Ne(_)
            | I::BranchI64NeImm(_)
            | I::BranchI64LtS(_)
            | I::BranchI64LtSImm(_)
            | I::BranchI64LtU(_)
            | I::BranchI64LtUImm(_)
            | I::BranchI64LeS(_)
            | I::BranchI64LeSImm(_)
            | I::BranchI64LeU(_)
            | I::BranchI64LeUImm(_)
            | I::BranchI64GtS(_)
            | I::BranchI64GtSImm(_)
            | I::BranchI64GtU(_)
            | I::BranchI64GtUImm(_)
            | I::BranchI64GeS(_)
            | I::BranchI64GeSImm(_)
            | I::BranchI64GeU(_)
            | I::BranchI64GeUImm(_)
            | I::BranchF32Eq(_)
            | I::BranchF32Ne(_)
            | I::BranchF32Lt(_)
            | I::BranchF32Le(_)
            | I::BranchF32Gt(_)
            | I::BranchF32Ge(_)
            | I::BranchF64Eq(_)
            | I::BranchF64Ne(_)
            | I::BranchF64Lt(_)
            | I::BranchF64Le(_)
            | I::BranchF64Gt(_)
            | I::BranchF64Ge(_) => Ok(false),
            I::Copy { result, .. }
            | I::CopyImm32 { result, .. }
            | I::CopyI64Imm32 { result, .. }
            | I::CopyF64Imm32 { result, .. } => relink_simple(result, new_result, old_result),
            I::CopySpan { .. }
            | I::CopySpanNonOverlapping { .. }
            | I::Copy2 { .. }
            | I::CopyMany { .. }
            | I::CopyManyNonOverlapping { .. }
            | I::ReturnCallInternal0 { .. }
            | I::ReturnCallInternal { .. }
            | I::ReturnCallImported0 { .. }
            | I::ReturnCallImported { .. }
            | I::ReturnCallIndirect0 { .. }
            | I::ReturnCallIndirect { .. } => Ok(false),
            I::CallInternal0 { results, func } | I::CallInternal { results, func } => {
                relink_call_internal(results, *func, res, new_result, old_result)
            }
            I::CallImported0 { results, func } | I::CallImported { results, func } => {
                relink_call_imported(results, *func, res, new_result, old_result)
            }
            I::CallIndirect0 { results, func_type } | I::CallIndirect { results, func_type } => {
                relink_call_indirect(results, *func_type, res, new_result, old_result)
            }
            I::Select { result, .. }
            | I::SelectRev { result, .. }
            | I::SelectImm32 {
                result_or_condition: result,
                ..
            }
            | I::SelectI64Imm32 {
                result_or_condition: result,
                ..
            }
            | I::SelectF64Imm32 {
                result_or_condition: result,
                ..
            } => {
                // Note: the `result_or_condition` necessarily points to the actual `result`
                //       register since we make sure elsewhere that only the correct instruction
                //       word is given to this method.
                relink_simple(result, new_result, old_result)
            }
            I::RefFunc { result, .. }
            | I::TableGet { result, .. }
            | I::TableGetImm { result, .. }
            | I::TableSize { result, .. } => relink_simple(result, new_result, old_result),
            I::TableSet { .. }
            | I::TableSetAt { .. }
            | I::TableCopy { .. }
            | I::TableCopyTo { .. }
            | I::TableCopyFrom { .. }
            | I::TableCopyFromTo { .. }
            | I::TableCopyExact { .. }
            | I::TableCopyToExact { .. }
            | I::TableCopyFromExact { .. }
            | I::TableCopyFromToExact { .. }
            | I::TableInit { .. }
            | I::TableInitTo { .. }
            | I::TableInitFrom { .. }
            | I::TableInitFromTo { .. }
            | I::TableInitExact { .. }
            | I::TableInitToExact { .. }
            | I::TableInitFromExact { .. }
            | I::TableInitFromToExact { .. }
            | I::TableFill { .. }
            | I::TableFillAt { .. }
            | I::TableFillExact { .. }
            | I::TableFillAtExact { .. } => Ok(false),
            I::TableGrow { result, .. } | I::TableGrowImm { result, .. } => {
                relink_simple(result, new_result, old_result)
            }
            I::ElemDrop(_) | I::DataDrop(_) => Ok(false),
            I::MemorySize { result }
            | I::MemoryGrow { result, .. }
            | I::MemoryGrowBy { result, .. } => relink_simple(result, new_result, old_result),
            I::MemoryCopy { .. }
            | I::MemoryCopyTo { .. }
            | I::MemoryCopyFrom { .. }
            | I::MemoryCopyFromTo { .. }
            | I::MemoryCopyExact { .. }
            | I::MemoryCopyToExact { .. }
            | I::MemoryCopyFromExact { .. }
            | I::MemoryCopyFromToExact { .. }
            | I::MemoryFill { .. }
            | I::MemoryFillAt { .. }
            | I::MemoryFillImm { .. }
            | I::MemoryFillExact { .. }
            | I::MemoryFillAtImm { .. }
            | I::MemoryFillAtExact { .. }
            | I::MemoryFillImmExact { .. }
            | I::MemoryFillAtImmExact { .. }
            | I::MemoryInit { .. }
            | I::MemoryInitTo { .. }
            | I::MemoryInitFrom { .. }
            | I::MemoryInitFromTo { .. }
            | I::MemoryInitExact { .. }
            | I::MemoryInitToExact { .. }
            | I::MemoryInitFromExact { .. }
            | I::MemoryInitFromToExact { .. } => Ok(false),
            I::GlobalGet { result, .. } => relink_simple(result, new_result, old_result),
            I::GlobalSet { .. } | I::GlobalSetI32Imm16 { .. } | I::GlobalSetI64Imm16 { .. } => {
                Ok(false)
            }
            I::I32Load(instr) => relink_simple(instr, new_result, old_result),
            I::I32LoadAt(instr) => relink_simple(instr, new_result, old_result),
            I::I32LoadOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load(instr) => relink_simple(instr, new_result, old_result),
            I::I64LoadAt(instr) => relink_simple(instr, new_result, old_result),
            I::I64LoadOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::F32Load(instr) => relink_simple(instr, new_result, old_result),
            I::F32LoadAt(instr) => relink_simple(instr, new_result, old_result),
            I::F32LoadOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::F64Load(instr) => relink_simple(instr, new_result, old_result),
            I::F64LoadAt(instr) => relink_simple(instr, new_result, old_result),
            I::F64LoadOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load8s(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load8sAt(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load8sOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load8u(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load8uAt(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load8uOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load16s(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load16sAt(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load16sOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load16u(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load16uAt(instr) => relink_simple(instr, new_result, old_result),
            I::I32Load16uOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load8s(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load8sAt(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load8sOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load8u(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load8uAt(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load8uOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load16s(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load16sAt(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load16sOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load16u(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load16uAt(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load16uOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load32s(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load32sAt(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load32sOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load32u(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load32uAt(instr) => relink_simple(instr, new_result, old_result),
            I::I64Load32uOffset16(instr) => relink_simple(instr, new_result, old_result),
            I::I32Store(_)
            | I::I32StoreOffset16(_)
            | I::I32StoreOffset16Imm16(_)
            | I::I32StoreAt(_)
            | I::I32StoreAtImm16(_)
            | I::I32Store8(_)
            | I::I32Store8Offset16(_)
            | I::I32Store8Offset16Imm(_)
            | I::I32Store8At(_)
            | I::I32Store8AtImm(_)
            | I::I32Store16(_)
            | I::I32Store16Offset16(_)
            | I::I32Store16Offset16Imm(_)
            | I::I32Store16At(_)
            | I::I32Store16AtImm(_)
            | I::I64Store(_)
            | I::I64StoreOffset16(_)
            | I::I64StoreOffset16Imm16(_)
            | I::I64StoreAt(_)
            | I::I64StoreAtImm16(_)
            | I::I64Store8(_)
            | I::I64Store8Offset16(_)
            | I::I64Store8Offset16Imm(_)
            | I::I64Store8At(_)
            | I::I64Store8AtImm(_)
            | I::I64Store16(_)
            | I::I64Store16Offset16(_)
            | I::I64Store16Offset16Imm(_)
            | I::I64Store16At(_)
            | I::I64Store16AtImm(_)
            | I::I64Store32(_)
            | I::I64Store32Offset16(_)
            | I::I64Store32Offset16Imm16(_)
            | I::I64Store32At(_)
            | I::I64Store32AtImm16(_)
            | I::F32Store(_)
            | I::F32StoreOffset16(_)
            | I::F32StoreAt(_)
            | I::F64Store(_)
            | I::F64StoreOffset16(_)
            | I::F64StoreAt(_) => Ok(false),
            I::I32Eq(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_eq_assign),
            I::I64Eq(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_eq_assign),
            I::I32Ne(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_ne_assign),
            I::I64Ne(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_ne_assign),
            I::I32LtS(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_lt_s_assign),
            I::I64LtS(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_lt_s_assign),
            I::I32LtU(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_lt_u_assign),
            I::I64LtU(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_lt_u_assign),
            I::I32LeS(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_le_s_assign),
            I::I64LeS(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_le_s_assign),
            I::I32LeU(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_le_u_assign),
            I::I64LeU(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_le_u_assign),
            I::I32GtS(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_gt_s_assign),
            I::I64GtS(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_gt_s_assign),
            I::I32GtU(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_gt_u_assign),
            I::I64GtU(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_gt_u_assign),
            I::I32GeS(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_ge_s_assign),
            I::I64GeS(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_ge_s_assign),
            I::I32GeU(instr) => relink_binop!(self, instr, new_result, old_result, I::i32_ge_u_assign),
            I::I64GeU(instr) => relink_binop!(self, instr, new_result, old_result, I::i64_ge_u_assign),
            I::F32Eq(instr) => relink_binop!(self, instr, new_result, old_result, I::f32_eq_assign),
            I::F32Ne(instr) => relink_binop!(self, instr, new_result, old_result, I::f32_ne_assign),
            I::F32Lt(instr) => relink_binop!(self, instr, new_result, old_result, I::f32_lt_assign),
            I::F32Le(instr) => relink_binop!(self, instr, new_result, old_result, I::f32_le_assign),
            I::F32Gt(instr) => relink_binop!(self, instr, new_result, old_result, I::f32_gt_assign),
            I::F32Ge(instr) => relink_binop!(self, instr, new_result, old_result, I::f32_ge_assign),
            I::F64Eq(instr) => relink_binop!(self, instr, new_result, old_result, I::f64_eq_assign),
            I::F64Ne(instr) => relink_binop!(self, instr, new_result, old_result, I::f64_ne_assign),
            I::F64Lt(instr) => relink_binop!(self, instr, new_result, old_result, I::f64_lt_assign),
            I::F64Le(instr) => relink_binop!(self, instr, new_result, old_result, I::f64_le_assign),
            I::F64Gt(instr) => relink_binop!(self, instr, new_result, old_result, I::f64_gt_assign),
            I::F64Ge(instr) => relink_binop!(self, instr, new_result, old_result, I::f64_ge_assign),
            I::I32EqImm16(instr) => relink_binop_imm16!(i32, self, instr, new_result, old_result, I::i32_eq_assign_imm),
            I::I32NeImm16(instr) => relink_binop_imm16!(i32, self, instr, new_result, old_result, I::i32_ne_assign_imm),
            I::I32LtSImm16(instr) => relink_binop_imm16!(i32, self, instr, new_result, old_result, I::i32_lt_s_assign_imm),
            I::I32LtUImm16(instr) => relink_binop_imm16!(u32, self, instr, new_result, old_result, I::i32_lt_u_assign_imm),
            I::I32LeSImm16(instr) => relink_binop_imm16!(i32, self, instr, new_result, old_result, I::i32_le_s_assign_imm),
            I::I32LeUImm16(instr) => relink_binop_imm16!(u32, self, instr, new_result, old_result, I::i32_le_u_assign_imm),
            I::I32GtSImm16(instr) => relink_binop_imm16!(i32, self, instr, new_result, old_result, I::i32_gt_s_assign_imm),
            I::I32GtUImm16(instr) => relink_binop_imm16!(u32, self, instr, new_result, old_result, I::i32_gt_u_assign_imm),
            I::I32GeSImm16(instr) => relink_binop_imm16!(i32, self, instr, new_result, old_result, I::i32_ge_s_assign_imm),
            I::I32GeUImm16(instr) => relink_binop_imm16!(u32, self, instr, new_result, old_result, I::i32_ge_u_assign_imm),
            I::I64EqImm16(instr) => relink_binop_imm16!(i64, self, instr, new_result, old_result, I::i64_eq_assign_imm32),
            I::I64NeImm16(instr) => relink_binop_imm16!(i64, self, instr, new_result, old_result, I::i64_ne_assign_imm32),
            I::I64LtSImm16(instr) => relink_binop_imm16!(i64, self, instr, new_result, old_result, I::i64_lt_s_assign_imm32),
            I::I64LtUImm16(instr) => relink_binop_imm16!(u64, self, instr, new_result, old_result, I::i64_lt_u_assign_imm32),
            I::I64LeSImm16(instr) => relink_binop_imm16!(i64, self, instr, new_result, old_result, I::i64_le_s_assign_imm32),
            I::I64LeUImm16(instr) => relink_binop_imm16!(u64, self, instr, new_result, old_result, I::i64_le_u_assign_imm32),
            I::I64GtSImm16(instr) => relink_binop_imm16!(i64, self, instr, new_result, old_result, I::i64_gt_s_assign_imm32),
            I::I64GtUImm16(instr) => relink_binop_imm16!(u64, self, instr, new_result, old_result, I::i64_gt_u_assign_imm32),
            I::I64GeSImm16(instr) => relink_binop_imm16!(i64, self, instr, new_result, old_result, I::i64_ge_s_assign_imm32),
            I::I64GeUImm16(instr) => relink_binop_imm16!(u64, self, instr, new_result, old_result, I::i64_ge_u_assign_imm32),
            I::I32EqAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_eq),
            I::I32NeAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_ne),
            I::I32LtSAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_lt_s),
            I::I32LtUAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_lt_u),
            I::I32LeSAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_le_s),
            I::I32LeUAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_le_u),
            I::I32GtSAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_gt_s),
            I::I32GtUAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_gt_u),
            I::I32GeSAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_ge_s),
            I::I32GeUAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i32_ge_u),
            I::I64EqAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_eq),
            I::I64NeAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_ne),
            I::I64LtSAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_lt_s),
            I::I64LtUAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_lt_u),
            I::I64LeSAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_le_s),
            I::I64LeUAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_le_u),
            I::I64GtSAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_gt_s),
            I::I64GtUAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_gt_u),
            I::I64GeSAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_ge_s),
            I::I64GeUAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::i64_ge_u),
            I::F32EqAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f32_eq),
            I::F32NeAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f32_ne),
            I::F32LtAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f32_lt),
            I::F32LeAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f32_le),
            I::F32GtAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f32_gt),
            I::F32GeAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f32_ge),
            I::F64EqAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f64_eq),
            I::F64NeAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f64_ne),
            I::F64LtAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f64_lt),
            I::F64LeAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f64_le),
            I::F64GtAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f64_gt),
            I::F64GeAssign(instr) => relink_binop_assign!(self, instr, new_result, old_result, I::f64_ge),
            I::I32EqAssignImm(instr) => relink_binop_assign_imm!(i32, self, instr, stack, new_result, old_result, I::i32_eq, I::i32_eq_imm16),
            I::I32NeAssignImm(instr) => relink_binop_assign_imm!(i32, self, instr, stack, new_result, old_result, I::i32_ne, I::i32_ne_imm16),
            I::I32LtSAssignImm(instr) => relink_binop_assign_imm!(i32, self, instr, stack, new_result, old_result, I::i32_lt_s, I::i32_lt_s_imm16),
            I::I32LtUAssignImm(instr) => relink_binop_assign_imm!(u32, self, instr, stack, new_result, old_result, I::i32_lt_u, I::i32_lt_u_imm16),
            I::I32LeSAssignImm(instr) => relink_binop_assign_imm!(i32, self, instr, stack, new_result, old_result, I::i32_le_s, I::i32_le_s_imm16),
            I::I32LeUAssignImm(instr) => relink_binop_assign_imm!(u32, self, instr, stack, new_result, old_result, I::i32_le_u, I::i32_le_u_imm16),
            I::I32GtSAssignImm(instr) => relink_binop_assign_imm!(i32, self, instr, stack, new_result, old_result, I::i32_gt_s, I::i32_gt_s_imm16),
            I::I32GtUAssignImm(instr) => relink_binop_assign_imm!(u32, self, instr, stack, new_result, old_result, I::i32_gt_u, I::i32_gt_u_imm16),
            I::I32GeSAssignImm(instr) => relink_binop_assign_imm!(i32, self, instr, stack, new_result, old_result, I::i32_ge_s, I::i32_ge_s_imm16),
            I::I32GeUAssignImm(instr) => relink_binop_assign_imm!(u32, self, instr, stack, new_result, old_result, I::i32_ge_u, I::i32_ge_u_imm16),
            I::I64EqAssignImm32(instr) => relink_binop_assign_imm!(i64, self, instr, stack, new_result, old_result, I::i64_eq, I::i64_eq_imm16),
            I::I64NeAssignImm32(instr) => relink_binop_assign_imm!(i64, self, instr, stack, new_result, old_result, I::i64_ne, I::i64_ne_imm16),
            I::I64LtSAssignImm32(instr) => relink_binop_assign_imm!(i64, self, instr, stack, new_result, old_result, I::i64_lt_s, I::i64_lt_s_imm16),
            I::I64LtUAssignImm32(instr) => relink_binop_assign_imm!(u64, self, instr, stack, new_result, old_result, I::i64_lt_u, I::i64_lt_u_imm16),
            I::I64LeSAssignImm32(instr) => relink_binop_assign_imm!(i64, self, instr, stack, new_result, old_result, I::i64_le_s, I::i64_le_s_imm16),
            I::I64LeUAssignImm32(instr) => relink_binop_assign_imm!(u64, self, instr, stack, new_result, old_result, I::i64_le_u, I::i64_le_u_imm16),
            I::I64GtSAssignImm32(instr) => relink_binop_assign_imm!(i64, self, instr, stack, new_result, old_result, I::i64_gt_s, I::i64_gt_s_imm16),
            I::I64GtUAssignImm32(instr) => relink_binop_assign_imm!(u64, self, instr, stack, new_result, old_result, I::i64_gt_u, I::i64_gt_u_imm16),
            I::I64GeSAssignImm32(instr) => relink_binop_assign_imm!(i64, self, instr, stack, new_result, old_result, I::i64_ge_s, I::i64_ge_s_imm16),
            I::I64GeUAssignImm32(instr) => relink_binop_assign_imm!(u64, self, instr, stack, new_result, old_result, I::i64_ge_u, I::i64_ge_u_imm16),
            I::F32EqAssignImm(instr) => relink_binop_assign_fimm!(f32, self, instr, stack, new_result, old_result, I::f32_eq),
            I::F32NeAssignImm(instr) => relink_binop_assign_fimm!(f32, self, instr, stack, new_result, old_result, I::f32_ne),
            I::F32LtAssignImm(instr) => relink_binop_assign_fimm!(f32, self, instr, stack, new_result, old_result, I::f32_lt),
            I::F32LeAssignImm(instr) => relink_binop_assign_fimm!(f32, self, instr, stack, new_result, old_result, I::f32_le),
            I::F32GtAssignImm(instr) => relink_binop_assign_fimm!(f32, self, instr, stack, new_result, old_result, I::f32_gt),
            I::F32GeAssignImm(instr) => relink_binop_assign_fimm!(f32, self, instr, stack, new_result, old_result, I::f32_ge),
            I::F64EqAssignImm32(instr) => relink_binop_assign_fimm!(f64, self, instr, stack, new_result, old_result, I::f64_eq),
            I::F64NeAssignImm32(instr) => relink_binop_assign_fimm!(f64, self, instr, stack, new_result, old_result, I::f64_ne),
            I::F64LtAssignImm32(instr) => relink_binop_assign_fimm!(f64, self, instr, stack, new_result, old_result, I::f64_lt),
            I::F64LeAssignImm32(instr) => relink_binop_assign_fimm!(f64, self, instr, stack, new_result, old_result, I::f64_le),
            I::F64GtAssignImm32(instr) => relink_binop_assign_fimm!(f64, self, instr, stack, new_result, old_result, I::f64_gt),
            I::F64GeAssignImm32(instr) => relink_binop_assign_fimm!(f64, self, instr, stack, new_result, old_result, I::f64_ge),
            _ => todo!(),
        }
    }
}

fn relink_exchange(
    instr: &mut Instruction,
    new_instr: Instruction,
) -> Result<bool, TranslationError> {
    _ = mem::replace(instr, new_instr);
    Ok(true)
}

fn relink_simple<T>(
    result: &mut T,
    new_result: Register,
    old_result: Register,
) -> Result<bool, TranslationError>
where
    T: ResultMut,
{
    let result = result.result_mut();
    if *result != old_result {
        // Note: This is a safeguard to prevent miscompilations.
        return Ok(false);
    }
    debug_assert_ne!(*result, new_result);
    *result = new_result;
    Ok(true)
}

fn relink_call_internal(
    results: &mut RegisterSpan,
    func: CompiledFunc,
    res: &ModuleResources,
    new_result: Register,
    old_result: Register,
) -> Result<bool, TranslationError> {
    let len_results = res
        .engine()
        .resolve_func_2(func, CompiledFuncEntity::len_results);
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
}

fn relink_call_imported(
    results: &mut RegisterSpan,
    func: FuncIdx,
    res: &ModuleResources,
    new_result: Register,
    old_result: Register,
) -> Result<bool, TranslationError> {
    let func_idx = func.to_u32().into();
    let func_type = res.get_type_of_func(func_idx);
    let len_results = res
        .engine()
        .resolve_func_type(func_type, |func_type| func_type.results().len());
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
}

fn relink_call_indirect(
    results: &mut RegisterSpan,
    func_type: SignatureIdx,
    res: &ModuleResources,
    new_result: Register,
    old_result: Register,
) -> Result<bool, TranslationError> {
    let func_type_idx = func_type.to_u32().into();
    let func_type = res.get_func_type(func_type_idx);
    let len_results = res
        .engine()
        .resolve_func_type(func_type, |func_type| func_type.results().len());
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
}

#[derive(Debug, Copy, Clone)]
enum RelinkResult {
    Unchanged,
    Relinked,
    Exchanged(Instruction),
}

fn relink_binop(
    instr: &mut BinInstr,
    new_result: Register,
    old_result: Register,
    make_bin: fn(result: Register, rhs: Register) -> Instruction,
) -> Result<RelinkResult, TranslationError> {
    if !relink_simple(instr, new_result, old_result)? {
        return Ok(RelinkResult::Unchanged);
    }
    if instr.result == instr.lhs {
        let new_instr = make_bin(new_result, instr.rhs);
        return Ok(RelinkResult::Exchanged(new_instr));
    }
    Ok(RelinkResult::Relinked)
}

fn relink_binop_imm16<T>(
    instr: &mut BinInstrImm<Const16<T>>,
    new_result: Register,
    old_result: Register,
    make_bin: fn(result: Register, rhs: Const32<T>) -> Instruction,
) -> Result<RelinkResult, TranslationError>
where
    Const16<T>: Into<Const32<T>>,
{
    if !relink_simple(instr, new_result, old_result)? {
        return Ok(RelinkResult::Unchanged);
    }
    if instr.result == instr.reg_in {
        let new_instr = make_bin(new_result, instr.imm_in.into());
        return Ok(RelinkResult::Exchanged(new_instr));
    }
    Ok(RelinkResult::Relinked)
}

fn relink_binop_assign(
    instr: &BinAssignInstr,
    new_result: Register,
    old_result: Register,
    make_bin: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
) -> Option<Instruction> {
    if instr.inout != old_result {
        // Note: This is a safeguard to prevent bugs.
        return None;
    }
    debug_assert_ne!(instr.inout, new_result);
    Some(make_bin(new_result, instr.inout, instr.rhs))
}

fn relink_binop_assign_imm<T>(
    instr: &BinAssignInstrImm<Const32<T>>,
    stack: &mut ValueStack,
    new_result: Register,
    old_result: Register,
    make_bin: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
    make_bin_imm: fn(result: Register, lhs: Register, rhs: Const16<T>) -> Instruction,
) -> Result<Option<Instruction>, TranslationError>
where
    T: Copy + From<Const32<T>> + Into<UntypedValue>,
    Const16<T>: TryFrom<T>,
{
    if instr.inout != old_result {
        // Note: This is a safeguard to prevent bugs.
        return Ok(None);
    }
    debug_assert_ne!(instr.inout, new_result);
    let rhs = T::from(instr.rhs);
    let new_instr = match <Const16<T>>::try_from(rhs) {
        Ok(rhs) => make_bin_imm(new_result, instr.inout, rhs),
        Err(_) => {
            let rhs = stack.alloc_const(rhs)?;
            make_bin(new_result, instr.inout, rhs)
        }
    };
    Ok(Some(new_instr))
}

fn relink_binop_assign_fimm<T>(
    instr: &BinAssignInstrImm<Const32<T>>,
    stack: &mut ValueStack,
    new_result: Register,
    old_result: Register,
    make_bin: fn(result: Register, lhs: Register, rhs: Register) -> Instruction,
) -> Result<Option<Instruction>, TranslationError>
where
    T: From<Const32<T>> + Into<UntypedValue>,
{
    if instr.inout != old_result {
        // Note: This is a safeguard to prevent bugs.
        return Ok(None);
    }
    debug_assert_ne!(instr.inout, new_result);
    let rhs = stack.alloc_const(T::from(instr.rhs))?;
    let new_instr = make_bin(new_result, instr.inout, rhs);
    Ok(Some(new_instr))
}

trait ResultMut {
    fn result_mut(&mut self) -> &mut Register;
}

impl ResultMut for Register {
    fn result_mut(&mut self) -> &mut Register {
        self
    }
}

impl ResultMut for LoadInstr {
    fn result_mut(&mut self) -> &mut Register {
        &mut self.result
    }
}

impl ResultMut for LoadAtInstr {
    fn result_mut(&mut self) -> &mut Register {
        &mut self.result
    }
}

impl ResultMut for LoadOffset16Instr {
    fn result_mut(&mut self) -> &mut Register {
        &mut self.result
    }
}

impl ResultMut for BinInstr {
    fn result_mut(&mut self) -> &mut Register {
        &mut self.result
    }
}

impl<T> ResultMut for BinInstrImm<T> {
    fn result_mut(&mut self) -> &mut Register {
        &mut self.result
    }
}
