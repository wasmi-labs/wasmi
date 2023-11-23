//! Implements methods on [`Instruction`] to query the single result [`Register`].
//!
//! This is used for an optimization with `local.set` and `local.tee` translation
//! to replace the result of the previous [`Instruction`] instead of emitting a
//! copy instruction to model the `local.set` or `local.tee` translation.

use crate::{
    engine::{
        bytecode::{FuncIdx, SignatureIdx},
        regmach::{
            bytecode::{
                BinAssignInstr,
                BinAssignInstrImm,
                BinInstr,
                BinInstrImm,
                Instruction,
                LoadAtInstr,
                LoadInstr,
                LoadOffset16Instr,
                Register,
                RegisterSpan,
                UnaryInstr,
            },
            code_map::CompiledFuncEntity,
        },
        CompiledFunc,
    },
    module::ModuleResources,
};

impl Instruction {
    /// Returns the single `result` [`Register`] of the [`Instruction`] if any.
    ///
    /// # Note
    ///
    /// - A call [`Instruction`] variants will return `Some` if they return a single value.
    /// - A non-call [`Instruction`] with multiple `result` [`Register`] return `None`.
    #[rustfmt::skip]
    pub fn result_mut(&mut self, res: &ModuleResources) -> Option<&mut Register> {
        match self {
            Instruction::TableIdx(_) |
            Instruction::DataSegmentIdx(_) |
            Instruction::ElementSegmentIdx(_) |
            Instruction::Const32(_) |
            Instruction::I64Const32(_) |
            Instruction::F64Const32(_) |
            Instruction::Register(_) |
            Instruction::Register2(_) |
            Instruction::Register3(_) |
            Instruction::RegisterList(_) |
            Instruction::Trap(_) |
            Instruction::ConsumeFuel(_) |
            Instruction::Return |
            Instruction::ReturnReg { .. } |
            Instruction::ReturnReg2 { .. } |
            Instruction::ReturnReg3 { .. } |
            Instruction::ReturnImm32 { .. } |
            Instruction::ReturnI64Imm32 { .. } |
            Instruction::ReturnF64Imm32 { .. } |
            Instruction::ReturnSpan { .. } |
            Instruction::ReturnMany { .. } |
            Instruction::ReturnNez { .. } |
            Instruction::ReturnNezReg { .. } |
            Instruction::ReturnNezReg2 { .. } |
            Instruction::ReturnNezImm32 { .. } |
            Instruction::ReturnNezI64Imm32 { .. } |
            Instruction::ReturnNezF64Imm32 { .. } |
            Instruction::ReturnNezSpan { .. } |
            Instruction::ReturnNezMany { .. } |
            Instruction::Branch { .. } |
            Instruction::BranchEqz { .. } |
            Instruction::BranchNez { .. } |
            Instruction::BranchTable { .. } => None,
            Instruction::BranchI32Eq(_) |
            Instruction::BranchI32EqImm(_) |
            Instruction::BranchI32Ne(_) |
            Instruction::BranchI32NeImm(_) |
            Instruction::BranchI32LtS(_) |
            Instruction::BranchI32LtSImm(_) |
            Instruction::BranchI32LtU(_) |
            Instruction::BranchI32LtUImm(_) |
            Instruction::BranchI32LeS(_) |
            Instruction::BranchI32LeSImm(_) |
            Instruction::BranchI32LeU(_) |
            Instruction::BranchI32LeUImm(_) |
            Instruction::BranchI32GtS(_) |
            Instruction::BranchI32GtSImm(_) |
            Instruction::BranchI32GtU(_) |
            Instruction::BranchI32GtUImm(_) |
            Instruction::BranchI32GeS(_) |
            Instruction::BranchI32GeSImm(_) |
            Instruction::BranchI32GeU(_) |
            Instruction::BranchI32GeUImm(_) |
            Instruction::BranchI64Eq(_) |
            Instruction::BranchI64EqImm(_) |
            Instruction::BranchI64Ne(_) |
            Instruction::BranchI64NeImm(_) |
            Instruction::BranchI64LtS(_) |
            Instruction::BranchI64LtSImm(_) |
            Instruction::BranchI64LtU(_) |
            Instruction::BranchI64LtUImm(_) |
            Instruction::BranchI64LeS(_) |
            Instruction::BranchI64LeSImm(_) |
            Instruction::BranchI64LeU(_) |
            Instruction::BranchI64LeUImm(_) |
            Instruction::BranchI64GtS(_) |
            Instruction::BranchI64GtSImm(_) |
            Instruction::BranchI64GtU(_) |
            Instruction::BranchI64GtUImm(_) |
            Instruction::BranchI64GeS(_) |
            Instruction::BranchI64GeSImm(_) |
            Instruction::BranchI64GeU(_) |
            Instruction::BranchI64GeUImm(_) |
            Instruction::BranchF32Eq(_) |
            Instruction::BranchF32Ne(_) |
            Instruction::BranchF32Lt(_) |
            Instruction::BranchF32Le(_) |
            Instruction::BranchF32Gt(_) |
            Instruction::BranchF32Ge(_) |
            Instruction::BranchF64Eq(_) |
            Instruction::BranchF64Ne(_) |
            Instruction::BranchF64Lt(_) |
            Instruction::BranchF64Le(_) |
            Instruction::BranchF64Gt(_) |
            Instruction::BranchF64Ge(_) => None,
            Instruction::Copy { result, .. } |
            Instruction::CopyImm32 { result, .. } |
            Instruction::CopyI64Imm32 { result, .. } |
            Instruction::CopyF64Imm32 { result, .. } => Some(result),
            Instruction::CopySpan { .. } |
            Instruction::CopySpanNonOverlapping { .. } |
            Instruction::Copy2 { .. } |
            Instruction::CopyMany { .. } => None,
            Instruction::CopyManyNonOverlapping { .. } => None,
            Instruction::CallIndirectParams(_) |
            Instruction::CallIndirectParamsImm16(_) |
            Instruction::ReturnCallInternal0 { .. } |
            Instruction::ReturnCallInternal { .. } |
            Instruction::ReturnCallImported0 { .. } |
            Instruction::ReturnCallImported { .. } |
            Instruction::ReturnCallIndirect0 { .. } |
            Instruction::ReturnCallIndirect { .. } => None,
            Instruction::CallInternal0 { results, func } |
            Instruction::CallInternal { results, func } => call_internal_result_mut(results, *func, res),
            Instruction::CallImported0 { results, func } |
            Instruction::CallImported { results, func } => call_imported_result_mut(results, *func, res),
            Instruction::CallIndirect0 { results, func_type } |
            Instruction::CallIndirect { results, func_type } => call_indirect_result_mut(results, *func_type, res),
            Instruction::Select { result, .. } |
            Instruction::SelectRev { result, .. } => Some(result),
            Instruction::SelectImm32 { result_or_condition, .. } |
            Instruction::SelectI64Imm32 { result_or_condition, .. } |
            Instruction::SelectF64Imm32 { result_or_condition, .. } => {
                // Note: the `result_or_condition` necessarily points to the actual `result`
                //       register since we make sure elsewhere that only the correct instruction
                //       word is given to this method.
                Some(result_or_condition)
            },
            Instruction::RefFunc { result, .. } |
            Instruction::TableGet { result, .. } |
            Instruction::TableGetImm { result, .. } |
            Instruction::TableSize { result, .. } => Some(result),
            Instruction::TableSet { .. } |
            Instruction::TableSetAt { .. } => None,
            Instruction::TableCopy { .. } |
            Instruction::TableCopyTo { .. } |
            Instruction::TableCopyFrom { .. } |
            Instruction::TableCopyFromTo { .. } |
            Instruction::TableCopyExact { .. } |
            Instruction::TableCopyToExact { .. } |
            Instruction::TableCopyFromExact { .. } |
            Instruction::TableCopyFromToExact { .. } => None,
            Instruction::TableInit { .. } |
            Instruction::TableInitTo { .. } |
            Instruction::TableInitFrom { .. } |
            Instruction::TableInitFromTo { .. } |
            Instruction::TableInitExact { .. } |
            Instruction::TableInitToExact { .. } |
            Instruction::TableInitFromExact { .. } |
            Instruction::TableInitFromToExact { .. } => None,
            Instruction::TableFill { .. } |
            Instruction::TableFillAt { .. } |
            Instruction::TableFillExact { .. } |
            Instruction::TableFillAtExact { .. } => None,
            Instruction::TableGrow { result, .. } |
            Instruction::TableGrowImm { result, .. } => Some(result),
            Instruction::ElemDrop(_) => None,
            Instruction::DataDrop(_) => None,
            Instruction::MemorySize { result } |
            Instruction::MemoryGrow { result, .. } |
            Instruction::MemoryGrowBy { result, .. } => Some(result),
            Instruction::MemoryCopy { .. } |
            Instruction::MemoryCopyTo { .. } |
            Instruction::MemoryCopyFrom { .. } |
            Instruction::MemoryCopyFromTo { .. } |
            Instruction::MemoryCopyExact { .. } |
            Instruction::MemoryCopyToExact { .. } |
            Instruction::MemoryCopyFromExact { .. } |
            Instruction::MemoryCopyFromToExact { .. } => None,
            Instruction::MemoryFill { .. } |
            Instruction::MemoryFillAt { .. } |
            Instruction::MemoryFillImm { .. } |
            Instruction::MemoryFillExact { .. } |
            Instruction::MemoryFillAtImm { .. } |
            Instruction::MemoryFillAtExact { .. } |
            Instruction::MemoryFillImmExact { .. } |
            Instruction::MemoryFillAtImmExact { .. } => None,
            Instruction::MemoryInit { .. } |
            Instruction::MemoryInitTo { .. } |
            Instruction::MemoryInitFrom { .. } |
            Instruction::MemoryInitFromTo { .. } |
            Instruction::MemoryInitExact { .. } |
            Instruction::MemoryInitToExact { .. } |
            Instruction::MemoryInitFromExact { .. } |
            Instruction::MemoryInitFromToExact { .. } => None,
            Instruction::GlobalGet { result, .. } => Some(result),
            Instruction::GlobalSet { .. } |
            Instruction::GlobalSetI32Imm16 { .. } |
            Instruction::GlobalSetI64Imm16 { .. } => None,
            Instruction::I32Load(instr) => instr.result_mut(),
            Instruction::I32LoadAt(instr) => instr.result_mut(),
            Instruction::I32LoadOffset16(instr) => instr.result_mut(),
            Instruction::I64Load(instr) => instr.result_mut(),
            Instruction::I64LoadAt(instr) => instr.result_mut(),
            Instruction::I64LoadOffset16(instr) => instr.result_mut(),
            Instruction::F32Load(instr) => instr.result_mut(),
            Instruction::F32LoadAt(instr) => instr.result_mut(),
            Instruction::F32LoadOffset16(instr) => instr.result_mut(),
            Instruction::F64Load(instr) => instr.result_mut(),
            Instruction::F64LoadAt(instr) => instr.result_mut(),
            Instruction::F64LoadOffset16(instr) => instr.result_mut(),
            Instruction::I32Load8s(instr) => instr.result_mut(),
            Instruction::I32Load8sAt(instr) => instr.result_mut(),
            Instruction::I32Load8sOffset16(instr) => instr.result_mut(),
            Instruction::I32Load8u(instr) => instr.result_mut(),
            Instruction::I32Load8uAt(instr) => instr.result_mut(),
            Instruction::I32Load8uOffset16(instr) => instr.result_mut(),
            Instruction::I32Load16s(instr) => instr.result_mut(),
            Instruction::I32Load16sAt(instr) => instr.result_mut(),
            Instruction::I32Load16sOffset16(instr) => instr.result_mut(),
            Instruction::I32Load16u(instr) => instr.result_mut(),
            Instruction::I32Load16uAt(instr) => instr.result_mut(),
            Instruction::I32Load16uOffset16(instr) => instr.result_mut(),
            Instruction::I64Load8s(instr) => instr.result_mut(),
            Instruction::I64Load8sAt(instr) => instr.result_mut(),
            Instruction::I64Load8sOffset16(instr) => instr.result_mut(),
            Instruction::I64Load8u(instr) => instr.result_mut(),
            Instruction::I64Load8uAt(instr) => instr.result_mut(),
            Instruction::I64Load8uOffset16(instr) => instr.result_mut(),
            Instruction::I64Load16s(instr) => instr.result_mut(),
            Instruction::I64Load16sAt(instr) => instr.result_mut(),
            Instruction::I64Load16sOffset16(instr) => instr.result_mut(),
            Instruction::I64Load16u(instr) => instr.result_mut(),
            Instruction::I64Load16uAt(instr) => instr.result_mut(),
            Instruction::I64Load16uOffset16(instr) => instr.result_mut(),
            Instruction::I64Load32s(instr) => instr.result_mut(),
            Instruction::I64Load32sAt(instr) => instr.result_mut(),
            Instruction::I64Load32sOffset16(instr) => instr.result_mut(),
            Instruction::I64Load32u(instr) => instr.result_mut(),
            Instruction::I64Load32uAt(instr) => instr.result_mut(),
            Instruction::I64Load32uOffset16(instr) => instr.result_mut(),
            Instruction::I32Store(_) |
            Instruction::I32StoreOffset16(_) |
            Instruction::I32StoreOffset16Imm16(_) |
            Instruction::I32StoreAt(_) |
            Instruction::I32StoreAtImm16(_) |
            Instruction::I32Store8(_) |
            Instruction::I32Store8Offset16(_) |
            Instruction::I32Store8Offset16Imm(_) |
            Instruction::I32Store8At(_) |
            Instruction::I32Store8AtImm(_) |
            Instruction::I32Store16(_) |
            Instruction::I32Store16Offset16(_) |
            Instruction::I32Store16Offset16Imm(_) |
            Instruction::I32Store16At(_) |
            Instruction::I32Store16AtImm(_) |
            Instruction::I64Store(_) |
            Instruction::I64StoreOffset16(_) |
            Instruction::I64StoreOffset16Imm16(_) |
            Instruction::I64StoreAt(_) |
            Instruction::I64StoreAtImm16(_) |
            Instruction::I64Store8(_) |
            Instruction::I64Store8Offset16(_) |
            Instruction::I64Store8Offset16Imm(_) |
            Instruction::I64Store8At(_) |
            Instruction::I64Store8AtImm(_) |
            Instruction::I64Store16(_) |
            Instruction::I64Store16Offset16(_) |
            Instruction::I64Store16Offset16Imm(_) |
            Instruction::I64Store16At(_) |
            Instruction::I64Store16AtImm(_) |
            Instruction::I64Store32(_) |
            Instruction::I64Store32Offset16(_) |
            Instruction::I64Store32Offset16Imm16(_) |
            Instruction::I64Store32At(_) |
            Instruction::I64Store32AtImm16(_) |
            Instruction::F32Store(_) |
            Instruction::F32StoreOffset16(_) |
            Instruction::F32StoreAt(_) |
            Instruction::F64Store(_) |
            Instruction::F64StoreOffset16(_) |
            Instruction::F64StoreAt(_) => None,
            Instruction::I32Eq(instr) => instr.result_mut(),
            Instruction::I32EqImm16(instr) => instr.result_mut(),
            Instruction::I64Eq(instr) => instr.result_mut(),
            Instruction::I64EqImm16(instr) => instr.result_mut(),
            Instruction::I32Ne(instr) => instr.result_mut(),
            Instruction::I32NeImm16(instr) => instr.result_mut(),
            Instruction::I64Ne(instr) => instr.result_mut(),
            Instruction::I64NeImm16(instr) => instr.result_mut(),
            Instruction::I32LtS(instr) |
            Instruction::I32LtU(instr) => instr.result_mut(),
            Instruction::I32LtSImm16(instr) => instr.result_mut(),
            Instruction::I32LtUImm16(instr) => instr.result_mut(),
            Instruction::I64LtS(instr) |
            Instruction::I64LtU(instr) => instr.result_mut(),
            Instruction::I64LtSImm16(instr) => instr.result_mut(),
            Instruction::I64LtUImm16(instr) => instr.result_mut(),
            Instruction::I32GtS(instr) |
            Instruction::I32GtU(instr) => instr.result_mut(),
            Instruction::I32GtSImm16(instr) => instr.result_mut(),
            Instruction::I32GtUImm16(instr) => instr.result_mut(),
            Instruction::I64GtS(instr) |
            Instruction::I64GtU(instr) => instr.result_mut(),
            Instruction::I64GtSImm16(instr) => instr.result_mut(),
            Instruction::I64GtUImm16(instr) => instr.result_mut(),
            Instruction::I32LeS(instr) |
            Instruction::I32LeU(instr) => instr.result_mut(),
            Instruction::I32LeSImm16(instr) => instr.result_mut(),
            Instruction::I32LeUImm16(instr) => instr.result_mut(),
            Instruction::I64LeS(instr) |
            Instruction::I64LeU(instr) => instr.result_mut(),
            Instruction::I64LeSImm16(instr) => instr.result_mut(),
            Instruction::I64LeUImm16(instr) => instr.result_mut(),
            Instruction::I32GeS(instr) |
            Instruction::I32GeU(instr) => instr.result_mut(),
            Instruction::I32GeSImm16(instr) => instr.result_mut(),
            Instruction::I32GeUImm16(instr) => instr.result_mut(),
            Instruction::I64GeS(instr) |
            Instruction::I64GeU(instr) => instr.result_mut(),
            Instruction::I64GeSImm16(instr) => instr.result_mut(),
            Instruction::I64GeUImm16(instr) => instr.result_mut(),
            Instruction::F32Eq(instr) |
            Instruction::F64Eq(instr) |
            Instruction::F32Ne(instr) |
            Instruction::F64Ne(instr) |
            Instruction::F32Lt(instr) |
            Instruction::F64Lt(instr) |
            Instruction::F32Le(instr) |
            Instruction::F64Le(instr) |
            Instruction::F32Gt(instr) |
            Instruction::F64Gt(instr) |
            Instruction::F32Ge(instr) |
            Instruction::F64Ge(instr) => instr.result_mut(),
            Instruction::I32Clz(instr) |
            Instruction::I64Clz(instr) |
            Instruction::I32Ctz(instr) |
            Instruction::I64Ctz(instr) |
            Instruction::I32Popcnt(instr) |
            Instruction::I64Popcnt(instr) => instr.result_mut(),
            Instruction::I32Add(instr) |
            Instruction::I64Add(instr) => instr.result_mut(),
            Instruction::I32AddImm16(instr) => instr.result_mut(),
            Instruction::I64AddImm16(instr) => instr.result_mut(),
            Instruction::I32Sub(instr) |
            Instruction::I64Sub(instr) => instr.result_mut(),
            Instruction::I32SubImm16(instr) => instr.result_mut(),
            Instruction::I64SubImm16(instr) => instr.result_mut(),
            Instruction::I32SubImm16Rev(instr) => instr.result_mut(),
            Instruction::I64SubImm16Rev(instr) => instr.result_mut(),
            Instruction::I32Mul(instr) |
            Instruction::I64Mul(instr) => instr.result_mut(),
            Instruction::I32MulImm16(instr) => instr.result_mut(),
            Instruction::I64MulImm16(instr) => instr.result_mut(),
            Instruction::I32DivS(instr) |
            Instruction::I64DivS(instr) => instr.result_mut(),
            Instruction::I32DivSImm16(instr) => instr.result_mut(),
            Instruction::I64DivSImm16(instr) => instr.result_mut(),
            Instruction::I32DivSImm16Rev(instr) => instr.result_mut(),
            Instruction::I64DivSImm16Rev(instr) => instr.result_mut(),
            Instruction::I32DivU(instr) |
            Instruction::I64DivU(instr) => instr.result_mut(),
            Instruction::I32DivUImm16(instr) => instr.result_mut(),
            Instruction::I64DivUImm16(instr) => instr.result_mut(),
            Instruction::I32DivUImm16Rev(instr) => instr.result_mut(),
            Instruction::I64DivUImm16Rev(instr) => instr.result_mut(),
            Instruction::I32RemS(instr) |
            Instruction::I64RemS(instr) => instr.result_mut(),
            Instruction::I32RemSImm16(instr) => instr.result_mut(),
            Instruction::I64RemSImm16(instr) => instr.result_mut(),
            Instruction::I32RemSImm16Rev(instr) => instr.result_mut(),
            Instruction::I64RemSImm16Rev(instr) => instr.result_mut(),
            Instruction::I32RemU(instr) |
            Instruction::I64RemU(instr) => instr.result_mut(),
            Instruction::I32RemUImm16(instr) => instr.result_mut(),
            Instruction::I64RemUImm16(instr) => instr.result_mut(),
            Instruction::I32RemUImm16Rev(instr) => instr.result_mut(),
            Instruction::I64RemUImm16Rev(instr) => instr.result_mut(),
            Instruction::I32And(instr) |
            Instruction::I64And(instr) => instr.result_mut(),
            Instruction::I32AndImm16(instr) => instr.result_mut(),
            Instruction::I64AndImm16(instr) => instr.result_mut(),
            Instruction::I32Or(instr) |
            Instruction::I64Or(instr) => instr.result_mut(),
            Instruction::I32OrImm16(instr) => instr.result_mut(),
            Instruction::I64OrImm16(instr) => instr.result_mut(),
            Instruction::I32Xor(instr) |
            Instruction::I64Xor(instr) => instr.result_mut(),
            Instruction::I32XorImm16(instr) => instr.result_mut(),
            Instruction::I64XorImm16(instr) => instr.result_mut(),
            Instruction::I32Shl(instr) |
            Instruction::I64Shl(instr) => instr.result_mut(),
            Instruction::I32ShlImm(instr) => instr.result_mut(),
            Instruction::I64ShlImm(instr) => instr.result_mut(),
            Instruction::I32ShlImm16Rev(instr) => instr.result_mut(),
            Instruction::I64ShlImm16Rev(instr) => instr.result_mut(),
            Instruction::I32ShrU(instr) |
            Instruction::I64ShrU(instr) => instr.result_mut(),
            Instruction::I32ShrUImm(instr) => instr.result_mut(),
            Instruction::I64ShrUImm(instr) => instr.result_mut(),
            Instruction::I32ShrUImm16Rev(instr) => instr.result_mut(),
            Instruction::I64ShrUImm16Rev(instr) => instr.result_mut(),
            Instruction::I32ShrS(instr) |
            Instruction::I64ShrS(instr) => instr.result_mut(),
            Instruction::I32ShrSImm(instr) => instr.result_mut(),
            Instruction::I64ShrSImm(instr) => instr.result_mut(),
            Instruction::I32ShrSImm16Rev(instr) => instr.result_mut(),
            Instruction::I64ShrSImm16Rev(instr) => instr.result_mut(),
            Instruction::I32Rotl(instr) |
            Instruction::I64Rotl(instr) => instr.result_mut(),
            Instruction::I32RotlImm(instr) => instr.result_mut(),
            Instruction::I64RotlImm(instr) => instr.result_mut(),
            Instruction::I32RotlImm16Rev(instr) => instr.result_mut(),
            Instruction::I64RotlImm16Rev(instr) => instr.result_mut(),
            Instruction::I32Rotr(instr) |
            Instruction::I64Rotr(instr) => instr.result_mut(),
            Instruction::I32RotrImm(instr) => instr.result_mut(),
            Instruction::I64RotrImm(instr) => instr.result_mut(),
            Instruction::I32RotrImm16Rev(instr) => instr.result_mut(),
            Instruction::I64RotrImm16Rev(instr) => instr.result_mut(),
            Instruction::F32Abs(instr) |
            Instruction::F64Abs(instr) |
            Instruction::F32Neg(instr) |
            Instruction::F64Neg(instr) |
            Instruction::F32Ceil(instr) |
            Instruction::F64Ceil(instr) |
            Instruction::F32Floor(instr) |
            Instruction::F64Floor(instr) |
            Instruction::F32Trunc(instr) |
            Instruction::F64Trunc(instr) |
            Instruction::F32Nearest(instr) |
            Instruction::F64Nearest(instr) |
            Instruction::F32Sqrt(instr) |
            Instruction::F64Sqrt(instr) => instr.result_mut(),
            Instruction::F32Add(instr) |
            Instruction::F64Add(instr) |
            Instruction::F32Sub(instr) |
            Instruction::F64Sub(instr) |
            Instruction::F32Mul(instr) |
            Instruction::F64Mul(instr) |
            Instruction::F32Div(instr) |
            Instruction::F64Div(instr) |
            Instruction::F32Min(instr) |
            Instruction::F64Min(instr) |
            Instruction::F32Max(instr) |
            Instruction::F64Max(instr) |
            Instruction::F32Copysign(instr) |
            Instruction::F64Copysign(instr) => instr.result_mut(),
            Instruction::F32CopysignImm(instr) |
            Instruction::F64CopysignImm(instr) => instr.result_mut(),
            Instruction::I32WrapI64(instr) |
            Instruction::I64ExtendI32S(instr) |
            Instruction::I64ExtendI32U(instr) |
            Instruction::I32TruncF32S(instr) |
            Instruction::I32TruncF32U(instr) |
            Instruction::I32TruncF64S(instr) |
            Instruction::I32TruncF64U(instr) |
            Instruction::I64TruncF32S(instr) |
            Instruction::I64TruncF32U(instr) |
            Instruction::I64TruncF64S(instr) |
            Instruction::I64TruncF64U(instr) |
            Instruction::I32TruncSatF32S(instr) |
            Instruction::I32TruncSatF32U(instr) |
            Instruction::I32TruncSatF64S(instr) |
            Instruction::I32TruncSatF64U(instr) |
            Instruction::I64TruncSatF32S(instr) |
            Instruction::I64TruncSatF32U(instr) |
            Instruction::I64TruncSatF64S(instr) |
            Instruction::I64TruncSatF64U(instr) |
            Instruction::I32Extend8S(instr) |
            Instruction::I32Extend16S(instr) |
            Instruction::I64Extend8S(instr) |
            Instruction::I64Extend16S(instr) |
            Instruction::I64Extend32S(instr) |
            Instruction::F32DemoteF64(instr) |
            Instruction::F64PromoteF32(instr) |
            Instruction::F32ConvertI32S(instr) |
            Instruction::F32ConvertI32U(instr) |
            Instruction::F32ConvertI64S(instr) |
            Instruction::F32ConvertI64U(instr) |
            Instruction::F64ConvertI32S(instr) |
            Instruction::F64ConvertI32U(instr) |
            Instruction::F64ConvertI64S(instr) |
            Instruction::F64ConvertI64U(instr) => instr.result_mut(),
            Instruction::I32EqAssign(instr) |
            Instruction::I32NeAssign(instr) |
            Instruction::I32LtSAssign(instr) |
            Instruction::I32LtUAssign(instr) |
            Instruction::I32GtSAssign(instr) |
            Instruction::I32GtUAssign(instr) |
            Instruction::I32LeSAssign(instr) |
            Instruction::I32LeUAssign(instr) |
            Instruction::I32GeSAssign(instr) |
            Instruction::I32GeUAssign(instr) |
            Instruction::I64EqAssign(instr) |
            Instruction::I64NeAssign(instr) |
            Instruction::I64LtSAssign(instr) |
            Instruction::I64LtUAssign(instr) |
            Instruction::I64GtSAssign(instr) |
            Instruction::I64GtUAssign(instr) |
            Instruction::I64LeSAssign(instr) |
            Instruction::I64LeUAssign(instr) |
            Instruction::I64GeSAssign(instr) |
            Instruction::I64GeUAssign(instr) |
            Instruction::F32EqAssign(instr) |
            Instruction::F32NeAssign(instr) |
            Instruction::F32LtAssign(instr) |
            Instruction::F32LeAssign(instr) |
            Instruction::F32GtAssign(instr) |
            Instruction::F32GeAssign(instr) |
            Instruction::F64EqAssign(instr) |
            Instruction::F64NeAssign(instr) |
            Instruction::F64LtAssign(instr) |
            Instruction::F64LeAssign(instr) |
            Instruction::F64GtAssign(instr) |
            Instruction::F64GeAssign(instr) |
            Instruction::I32AddAssign(instr) |
            Instruction::I32SubAssign(instr) |
            Instruction::I32MulAssign(instr) |
            Instruction::I32DivSAssign(instr) |
            Instruction::I32DivUAssign(instr) |
            Instruction::I32RemSAssign(instr) |
            Instruction::I32RemUAssign(instr) |
            Instruction::I32AndAssign(instr) |
            Instruction::I32OrAssign(instr) |
            Instruction::I32XorAssign(instr) |
            Instruction::I32ShlAssign(instr) |
            Instruction::I32ShrUAssign(instr) |
            Instruction::I32ShrSAssign(instr) |
            Instruction::I32RotlAssign(instr) |
            Instruction::I32RotrAssign(instr) |
            Instruction::I64AddAssign(instr) |
            Instruction::I64SubAssign(instr) |
            Instruction::I64MulAssign(instr) |
            Instruction::I64DivSAssign(instr) |
            Instruction::I64DivUAssign(instr) |
            Instruction::I64RemSAssign(instr) |
            Instruction::I64RemUAssign(instr) |
            Instruction::I64AndAssign(instr) |
            Instruction::I64OrAssign(instr) |
            Instruction::I64XorAssign(instr) |
            Instruction::I64ShlAssign(instr) |
            Instruction::I64ShrUAssign(instr) |
            Instruction::I64ShrSAssign(instr) |
            Instruction::I64RotlAssign(instr) |
            Instruction::I64RotrAssign(instr) |
            Instruction::F32AddAssign(instr) |
            Instruction::F32SubAssign(instr) |
            Instruction::F32MulAssign(instr) |
            Instruction::F32DivAssign(instr) |
            Instruction::F32MinAssign(instr) |
            Instruction::F32MaxAssign(instr) |
            Instruction::F32CopysignAssign(instr) |
            Instruction::F64AddAssign(instr) |
            Instruction::F64SubAssign(instr) |
            Instruction::F64MulAssign(instr) |
            Instruction::F64DivAssign(instr) |
            Instruction::F64MinAssign(instr) |
            Instruction::F64MaxAssign(instr) |
            Instruction::F64CopysignAssign(instr) => instr.result_mut(),
            Instruction::I32EqAssignImm(instr) |
            Instruction::I32NeAssignImm(instr) |
            Instruction::I32LtSAssignImm(instr) |
            Instruction::I32GtSAssignImm(instr) |
            Instruction::I32LeSAssignImm(instr) |
            Instruction::I32GeSAssignImm(instr) => instr.result_mut(),
            Instruction::I32LtUAssignImm(instr) |
            Instruction::I32GtUAssignImm(instr) |
            Instruction::I32LeUAssignImm(instr) |
            Instruction::I32GeUAssignImm(instr) => instr.result_mut(),
            Instruction::I64EqAssignImm32(instr) |
            Instruction::I64NeAssignImm32(instr) |
            Instruction::I64LtSAssignImm32(instr) |
            Instruction::I64LeSAssignImm32(instr) |
            Instruction::I64GtSAssignImm32(instr) |
            Instruction::I64GeSAssignImm32(instr) => instr.result_mut(),
            Instruction::I64LtUAssignImm32(instr) |
            Instruction::I64GtUAssignImm32(instr) |
            Instruction::I64LeUAssignImm32(instr) |
            Instruction::I64GeUAssignImm32(instr) => instr.result_mut(),
            Instruction::F32EqAssignImm(instr) |
            Instruction::F32NeAssignImm(instr) |
            Instruction::F32LtAssignImm(instr) |
            Instruction::F32LeAssignImm(instr) |
            Instruction::F32GtAssignImm(instr) |
            Instruction::F32GeAssignImm(instr) => instr.result_mut(),
            Instruction::F64EqAssignImm32(instr) |
            Instruction::F64NeAssignImm32(instr) |
            Instruction::F64LtAssignImm32(instr) |
            Instruction::F64LeAssignImm32(instr) |
            Instruction::F64GtAssignImm32(instr) |
            Instruction::F64GeAssignImm32(instr) => instr.result_mut(),
            Instruction::I32AddAssignImm(instr) |
            Instruction::I32SubAssignImm(instr) |
            Instruction::I32MulAssignImm(instr) |
            Instruction::I32DivSAssignImm(instr) |
            Instruction::I32RemSAssignImm(instr) |
            Instruction::I32AndAssignImm(instr) |
            Instruction::I32OrAssignImm(instr) |
            Instruction::I32XorAssignImm(instr) |
            Instruction::I32ShlAssignImm(instr) |
            Instruction::I32ShrSAssignImm(instr) |
            Instruction::I32ShrUAssignImm(instr) |
            Instruction::I32RotlAssignImm(instr) |
            Instruction::I32RotrAssignImm(instr) => instr.result_mut(),
            Instruction::I32DivUAssignImm(instr) |
            Instruction::I32RemUAssignImm(instr) => instr.result_mut(),
            Instruction::I64AddAssignImm32(instr) |
            Instruction::I64SubAssignImm32(instr) |
            Instruction::I64MulAssignImm32(instr) |
            Instruction::I64DivSAssignImm32(instr) |
            Instruction::I64RemSAssignImm32(instr) |
            Instruction::I64AndAssignImm32(instr) |
            Instruction::I64OrAssignImm32(instr) |
            Instruction::I64XorAssignImm32(instr) |
            Instruction::I64ShlAssignImm32(instr) |
            Instruction::I64ShrSAssignImm32(instr) |
            Instruction::I64ShrUAssignImm32(instr) |
            Instruction::I64RotlAssignImm32(instr) |
            Instruction::I64RotrAssignImm32(instr) => instr.result_mut(),
            Instruction::I64DivUAssignImm32(instr) |
            Instruction::I64RemUAssignImm32(instr) => instr.result_mut(),
            Instruction::F32AddAssignImm(instr) |
            Instruction::F32SubAssignImm(instr) |
            Instruction::F32MulAssignImm(instr) |
            Instruction::F32DivAssignImm(instr) |
            Instruction::F32MinAssignImm(instr) |
            Instruction::F32MaxAssignImm(instr) => instr.result_mut(),
            Instruction::F32CopysignAssignImm(instr) => instr.result_mut(),
            Instruction::F64AddAssignImm32(instr) |
            Instruction::F64SubAssignImm32(instr) |
            Instruction::F64MulAssignImm32(instr) |
            Instruction::F64DivAssignImm32(instr) |
            Instruction::F64MinAssignImm32(instr) |
            Instruction::F64MaxAssignImm32(instr) => instr.result_mut(),
            Instruction::F64CopysignAssignImm(instr) => instr.result_mut(),
        }
    }
}

/// Returns the result [`Register`] of `func` if `func` returns a single value.
///
/// Otherwise returns `None`.
fn call_internal_result_mut<'a>(
    results: &'a mut RegisterSpan,
    func: CompiledFunc,
    res: &ModuleResources,
) -> Option<&'a mut Register> {
    let len_results = res
        .engine()
        .resolve_func_2(func, CompiledFuncEntity::len_results);
    if len_results == 1 {
        return Some(results.head_mut());
    }
    None
}

/// Returns the result [`Register`] of `func` if `func` returns a single value.
///
/// Otherwise returns `None`.
fn call_imported_result_mut<'a>(
    results: &'a mut RegisterSpan,
    func: FuncIdx,
    res: &ModuleResources,
) -> Option<&'a mut Register> {
    let func_idx = func.to_u32().into();
    let func_type = res.get_type_of_func(func_idx);
    let len_results = res
        .engine()
        .resolve_func_type(func_type, |func_type| func_type.results().len());
    if len_results == 1 {
        return Some(results.head_mut());
    }
    None
}

/// Returns the result [`Register`] of `func` if `func_type` returns a single value.
///
/// Otherwise returns `None`.
fn call_indirect_result_mut<'a>(
    results: &'a mut RegisterSpan,
    func_type: SignatureIdx,
    res: &ModuleResources,
) -> Option<&'a mut Register> {
    let func_type_idx = func_type.to_u32().into();
    let func_type = res.get_func_type(func_type_idx);
    let len_results = res
        .engine()
        .resolve_func_type(func_type, |func_type| func_type.results().len());
    if len_results == 1 {
        return Some(results.head_mut());
    }
    None
}

impl BinAssignInstr {
    /// Returns the single `result` [`Register`] of the [`BinAssignInstr`] if any.
    pub fn result_mut(&mut self) -> Option<&mut Register> {
        Some(&mut self.inout)
    }
}

impl<T> BinAssignInstrImm<T> {
    /// Returns the single `result` [`Register`] of the [`BinAssignInstrImm32`] if any.
    pub fn result_mut(&mut self) -> Option<&mut Register> {
        Some(&mut self.inout)
    }
}

impl LoadInstr {
    /// Returns the single `result` [`Register`] of the [`LoadInstr`] if any.
    pub fn result_mut(&mut self) -> Option<&mut Register> {
        Some(&mut self.result)
    }
}

impl LoadAtInstr {
    /// Returns the single `result` [`Register`] of the [`LoadAtInstr`] if any.
    pub fn result_mut(&mut self) -> Option<&mut Register> {
        Some(&mut self.result)
    }
}

impl LoadOffset16Instr {
    /// Returns the single `result` [`Register`] of the [`LoadOffset16Instr`] if any.
    pub fn result_mut(&mut self) -> Option<&mut Register> {
        Some(&mut self.result)
    }
}

impl BinInstr {
    /// Returns the single `result` [`Register`] of the [`BinInstr`] if any.
    pub fn result_mut(&mut self) -> Option<&mut Register> {
        Some(&mut self.result)
    }
}

impl<T> BinInstrImm<T> {
    /// Returns the single `result` [`Register`] of the [`BinInstrImm16`] if any.
    pub fn result_mut(&mut self) -> Option<&mut Register> {
        Some(&mut self.result)
    }
}

impl UnaryInstr {
    /// Returns the single `result` [`Register`] of the [`UnaryInstr`] if any.
    pub fn result_mut(&mut self) -> Option<&mut Register> {
        Some(&mut self.result)
    }
}
