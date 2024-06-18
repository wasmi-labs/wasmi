use crate::{
    engine::{
        bytecode::{
            BinInstr,
            BinInstrImm,
            FuncIdx,
            Instruction,
            LoadAtInstr,
            LoadInstr,
            LoadOffset16Instr,
            Register,
            RegisterSpan,
            SignatureIdx,
            UnaryInstr,
        },
        CompiledFunc,
    },
    module::ModuleHeader,
    Engine,
    Error,
    FuncType,
};

impl Instruction {
    #[rustfmt::skip]
    pub fn relink_result(
        &mut self,
        module: &ModuleHeader,
        new_result: Register,
        old_result: Register,
    ) -> Result<bool, Error> {
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
            | I::BranchCmpFallback { .. }
            | I::BranchI32And(_)
            | I::BranchI32AndImm(_)
            | I::BranchI32Or(_)
            | I::BranchI32OrImm(_)
            | I::BranchI32Xor(_)
            | I::BranchI32XorImm(_)
            | I::BranchI32AndEqz(_)
            | I::BranchI32AndEqzImm(_)
            | I::BranchI32OrEqz(_)
            | I::BranchI32OrEqzImm(_)
            | I::BranchI32XorEqz(_)
            | I::BranchI32XorEqzImm(_)
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
                relink_call_internal(results, *func, module, new_result, old_result)
            }
            I::CallImported0 { results, func } | I::CallImported { results, func } => {
                relink_call_imported(results, *func, module, new_result, old_result)
            }
            I::CallIndirect0 { results, func_type } | I::CallIndirect { results, func_type } => {
                relink_call_indirect(results, *func_type, module, new_result, old_result)
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
            I::I32AddImmIntoGlobal0 { .. } => Ok(false),
            I::I32AddImmFromGlobal0 { result, .. } => relink_simple(result, new_result, old_result),
            I::I32AddImmInoutGlobal0 { result, .. } => relink_simple(result, new_result, old_result),
            I::I32Load(instr) |
            I::I64Load(instr) |
            I::F32Load(instr) |
            I::F64Load(instr) |
            I::I32Load8s(instr) |
            I::I32Load8u(instr) |
            I::I32Load16s(instr) |
            I::I32Load16u(instr) |
            I::I64Load8s(instr) |
            I::I64Load8u(instr) |
            I::I64Load16s(instr) |
            I::I64Load16u(instr) |
            I::I64Load32s(instr) |
            I::I64Load32u(instr) => relink_simple(instr, new_result, old_result),
            I::I32LoadAt(instr) |
            I::I64LoadAt(instr) |
            I::F32LoadAt(instr) |
            I::F64LoadAt(instr) |
            I::I32Load8sAt(instr) |
            I::I32Load8uAt(instr) |
            I::I32Load16sAt(instr) |
            I::I32Load16uAt(instr) |
            I::I64Load8sAt(instr) |
            I::I64Load8uAt(instr) |
            I::I64Load16sAt(instr) |
            I::I64Load16uAt(instr) |
            I::I64Load32sAt(instr) |
            I::I64Load32uAt(instr) => relink_simple(instr, new_result, old_result),
            I::I32LoadOffset16(instr) |
            I::I64LoadOffset16(instr) |
            I::F32LoadOffset16(instr) |
            I::F64LoadOffset16(instr) |
            I::I32Load8sOffset16(instr) |
            I::I32Load8uOffset16(instr) |
            I::I32Load16sOffset16(instr) |
            I::I32Load16uOffset16(instr) |
            I::I64Load8sOffset16(instr) |
            I::I64Load8uOffset16(instr) |
            I::I64Load16sOffset16(instr) |
            I::I64Load16uOffset16(instr) |
            I::I64Load32sOffset16(instr) |
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
            I::I32Eq(instr) |
            I::I64Eq(instr) |
            I::I32Ne(instr) |
            I::I64Ne(instr) |
            I::I32LtS(instr) |
            I::I64LtS(instr) |
            I::I32LtU(instr) |
            I::I64LtU(instr) |
            I::I32LeS(instr) |
            I::I64LeS(instr) |
            I::I32LeU(instr) |
            I::I64LeU(instr) |
            I::I32GtS(instr) |
            I::I64GtS(instr) |
            I::I32GtU(instr) |
            I::I64GtU(instr) |
            I::I32GeS(instr) |
            I::I64GeS(instr) |
            I::I32GeU(instr) |
            I::I64GeU(instr) |
            I::F32Eq(instr) |
            I::F32Ne(instr) |
            I::F32Lt(instr) |
            I::F32Le(instr) |
            I::F32Gt(instr) |
            I::F32Ge(instr) |
            I::F64Eq(instr) |
            I::F64Ne(instr) |
            I::F64Lt(instr) |
            I::F64Le(instr) |
            I::F64Gt(instr) |
            I::F64Ge(instr) => relink_simple(instr, new_result, old_result),
            I::I32EqImm16(instr) |
            I::I32NeImm16(instr) |
            I::I32LtSImm16(instr) |
            I::I32LeSImm16(instr) |
            I::I32GtSImm16(instr) |
            I::I32GeSImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I32LtUImm16(instr) |
            I::I32LeUImm16(instr) |
            I::I32GtUImm16(instr) |
            I::I32GeUImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I64EqImm16(instr) |
            I::I64NeImm16(instr) |
            I::I64LtSImm16(instr) |
            I::I64LeSImm16(instr) |
            I::I64GtSImm16(instr) |
            I::I64GeSImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I64LtUImm16(instr) |
            I::I64LeUImm16(instr) |
            I::I64GtUImm16(instr) |
            I::I64GeUImm16(instr) => relink_simple(instr, new_result, old_result),

            I::I32Clz(instr) |
            I::I32Ctz(instr) |
            I::I32Popcnt(instr) |
            I::I64Clz(instr) |
            I::I64Ctz(instr) |
            I::I64Popcnt(instr) => relink_simple(instr, new_result, old_result),

            I::I32Add(instr) |
            I::I32Sub(instr) |
            I::I32Mul(instr) |
            I::I32DivS(instr) |
            I::I32DivU(instr) |
            I::I32RemS(instr) |
            I::I32RemU(instr) |
            I::I32And(instr) |
            I::I32AndEqz(instr) |
            I::I32Or(instr) |
            I::I32OrEqz(instr) |
            I::I32Xor(instr) |
            I::I32XorEqz(instr) |
            I::I32Shl(instr) |
            I::I32ShrS(instr) |
            I::I32ShrU(instr) |
            I::I32Rotl(instr) |
            I::I32Rotr(instr) |
            I::I64Add(instr) |
            I::I64Sub(instr) |
            I::I64Mul(instr) |
            I::I64DivS(instr) |
            I::I64DivU(instr) |
            I::I64RemS(instr) |
            I::I64RemU(instr) |
            I::I64And(instr) |
            I::I64Or(instr) |
            I::I64Xor(instr) |
            I::I64Shl(instr) |
            I::I64ShrS(instr) |
            I::I64ShrU(instr) |
            I::I64Rotl(instr) |
            I::I64Rotr(instr) => relink_simple(instr, new_result, old_result),

            I::F32Abs(instr) |
            I::F32Neg(instr) |
            I::F32Ceil(instr) |
            I::F32Floor(instr) |
            I::F32Trunc(instr) |
            I::F32Nearest(instr) |
            I::F32Sqrt(instr) |
            I::F64Abs(instr) |
            I::F64Neg(instr) |
            I::F64Ceil(instr) |
            I::F64Floor(instr) |
            I::F64Trunc(instr) |
            I::F64Nearest(instr) |
            I::F64Sqrt(instr) => relink_simple(instr, new_result, old_result),

            I::F32Add(instr) |
            I::F32Sub(instr) |
            I::F32Mul(instr) |
            I::F32Div(instr) |
            I::F32Min(instr) |
            I::F32Max(instr) |
            I::F32Copysign(instr) => relink_simple(instr, new_result, old_result),
            I::F64Add(instr) |
            I::F64Sub(instr) |
            I::F64Mul(instr) |
            I::F64Div(instr) |
            I::F64Min(instr) |
            I::F64Max(instr) |
            I::F64Copysign(instr) => relink_simple(instr, new_result, old_result),

            I::F32CopysignImm(instr) |
            I::F64CopysignImm(instr) => relink_simple(instr, new_result, old_result),

            I::I32AddImm16(instr) |
            I::I32SubImm16Rev(instr) |
            I::I32MulImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I32DivSImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I32DivSImm16Rev(instr) => relink_simple(instr, new_result, old_result),
            I::I32RemSImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I32RemSImm16Rev(instr) |
            I::I32AndEqzImm16(instr) |
            I::I32AndImm16(instr) |
            I::I32OrEqzImm16(instr) |
            I::I32OrImm16(instr) |
            I::I32XorEqzImm16(instr) |
            I::I32XorImm16(instr) |
            I::I32ShlImm(instr) |
            I::I32ShlImm16Rev(instr) |
            I::I32ShrSImm(instr) |
            I::I32ShrSImm16Rev(instr) |
            I::I32ShrUImm(instr) |
            I::I32ShrUImm16Rev(instr) |
            I::I32RotlImm(instr) |
            I::I32RotlImm16Rev(instr) |
            I::I32RotrImm(instr) |
            I::I32RotrImm16Rev(instr) => relink_simple(instr, new_result, old_result),
            I::I32DivUImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I32DivUImm16Rev(instr) => relink_simple(instr, new_result, old_result),
            I::I32RemUImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I32RemUImm16Rev(instr) => relink_simple(instr, new_result, old_result),

            I::I64AddImm16(instr) |
            I::I64SubImm16Rev(instr) |
            I::I64MulImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I64DivSImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I64DivSImm16Rev(instr) => relink_simple(instr, new_result, old_result),
            I::I64RemSImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I64RemSImm16Rev(instr) |
            I::I64AndImm16(instr) |
            I::I64OrImm16(instr) |
            I::I64XorImm16(instr) |
            I::I64ShlImm(instr) |
            I::I64ShlImm16Rev(instr) |
            I::I64ShrSImm(instr) |
            I::I64ShrSImm16Rev(instr) |
            I::I64ShrUImm(instr) |
            I::I64ShrUImm16Rev(instr) |
            I::I64RotlImm(instr) |
            I::I64RotlImm16Rev(instr) |
            I::I64RotrImm(instr) |
            I::I64RotrImm16Rev(instr) => relink_simple(instr, new_result, old_result),
            I::I64DivUImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I64DivUImm16Rev(instr) => relink_simple(instr, new_result, old_result),
            I::I64RemUImm16(instr) => relink_simple(instr, new_result, old_result),
            I::I64RemUImm16Rev(instr) => relink_simple(instr, new_result, old_result),

            I::I32WrapI64(instr) |
            I::I64ExtendI32S(instr) |
            I::I64ExtendI32U(instr) |
            I::I32TruncF32S(instr) |
            I::I32TruncF32U(instr) |
            I::I32TruncF64S(instr) |
            I::I32TruncF64U(instr) |
            I::I64TruncF32S(instr) |
            I::I64TruncF32U(instr) |
            I::I64TruncF64S(instr) |
            I::I64TruncF64U(instr) |
            I::I32TruncSatF32S(instr) |
            I::I32TruncSatF32U(instr) |
            I::I32TruncSatF64S(instr) |
            I::I32TruncSatF64U(instr) |
            I::I64TruncSatF32S(instr) |
            I::I64TruncSatF32U(instr) |
            I::I64TruncSatF64S(instr) |
            I::I64TruncSatF64U(instr) |
            I::I32Extend8S(instr) |
            I::I32Extend16S(instr) |
            I::I64Extend8S(instr) |
            I::I64Extend16S(instr) |
            I::I64Extend32S(instr) |
            I::F32DemoteF64(instr) |
            I::F64PromoteF32(instr) |
            I::F32ConvertI32S(instr) |
            I::F32ConvertI32U(instr) |
            I::F32ConvertI64S(instr) |
            I::F32ConvertI64U(instr) |
            I::F64ConvertI32S(instr) |
            I::F64ConvertI32U(instr) |
            I::F64ConvertI64S(instr) |
            I::F64ConvertI64U(instr) => relink_simple(instr, new_result, old_result),
        }
    }
}

fn relink_simple<T>(
    result: &mut T,
    new_result: Register,
    old_result: Register,
) -> Result<bool, Error>
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

fn get_engine(module: &ModuleHeader) -> Engine {
    module.engine().upgrade().unwrap_or_else(|| {
        panic!(
            "engine for result relinking does not exist: {:?}",
            module.engine()
        )
    })
}

fn relink_call_internal(
    results: &mut RegisterSpan,
    func: CompiledFunc,
    module: &ModuleHeader,
    new_result: Register,
    old_result: Register,
) -> Result<bool, Error> {
    let Some(module_func) = module.get_func_index(func) else {
        panic!("missing module func for compiled func: {func:?}")
    };
    let engine = get_engine(module);
    let func_type = module.get_type_of_func(module_func);
    let len_results = engine.resolve_func_type(func_type, FuncType::len_results);
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
}

fn relink_call_imported(
    results: &mut RegisterSpan,
    func: FuncIdx,
    module: &ModuleHeader,
    new_result: Register,
    old_result: Register,
) -> Result<bool, Error> {
    let engine = get_engine(module);
    let func_idx = u32::from(func).into();
    let func_type = module.get_type_of_func(func_idx);
    let len_results = engine.resolve_func_type(func_type, |func_type| func_type.results().len());
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
}

fn relink_call_indirect(
    results: &mut RegisterSpan,
    func_type: SignatureIdx,
    module: &ModuleHeader,
    new_result: Register,
    old_result: Register,
) -> Result<bool, Error> {
    let engine = get_engine(module);
    let func_type_idx = u32::from(func_type).into();
    let func_type = module.get_func_type(func_type_idx);
    let len_results = engine.resolve_func_type(func_type, |func_type| func_type.results().len());
    if len_results != 1 {
        return Ok(false);
    }
    relink_simple(results.head_mut(), new_result, old_result)
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

impl ResultMut for UnaryInstr {
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
