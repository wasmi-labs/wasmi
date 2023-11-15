use crate::engine::regmach::bytecode::{
    BinInstr,
    BinInstrImm16,
    Const16,
    CopysignImmInstr,
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

macro_rules! visit_registers {
    ( $f:expr, $($field:expr),* $(,)? ) => {{
        $(
            $f($field)
        );*
    }};
}

/// Trait implemented by types that allow to visit their [`Register`] fields.
pub trait VisitInputRegisters {
    /// Calls `f` on all input [`Register`].
    fn visit_input_registers(&mut self, f: impl FnMut(&mut Register));
}

impl VisitInputRegisters for Instruction {
    #[rustfmt::skip]
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        match self {
            Instruction::TableIdx(_) |
            Instruction::DataSegmentIdx(_) |
            Instruction::ElementSegmentIdx(_) |
            Instruction::Const32(_) |
            Instruction::I64Const32(_) |
            Instruction::F64Const32(_) => {},
            Instruction::Register(register) => f(register),
            Instruction::Register2(registers) => registers.visit_input_registers(f),
            Instruction::Register3(registers) |
            Instruction::RegisterList(registers) => registers.visit_input_registers(f),
            Instruction::Trap(_) |
            Instruction::ConsumeFuel(_) |
            Instruction::Return => {},
            Instruction::ReturnReg { value } => f(value),
            Instruction::ReturnReg2 { values } => values.visit_input_registers(f),
            Instruction::ReturnReg3 { values } => values.visit_input_registers(f),
            Instruction::ReturnImm32 { .. } |
            Instruction::ReturnI64Imm32 { .. } |
            Instruction::ReturnF64Imm32 { .. } => {},
            Instruction::ReturnSpan { values } => {
                values.visit_input_registers(f);
            }
            Instruction::ReturnMany { values } => {
                values.visit_input_registers(f);
            }
            Instruction::ReturnNez { condition } => f(condition),
            Instruction::ReturnNezReg { condition, value } => visit_registers!(f, condition, value),
            Instruction::ReturnNezReg2 { condition, values } => {
                f(condition);
                values.visit_input_registers(f);
            }
            Instruction::ReturnNezImm32 { condition, .. } => f(condition),
            Instruction::ReturnNezI64Imm32 { condition, .. } => f(condition),
            Instruction::ReturnNezF64Imm32 { condition, .. } => f(condition),
            Instruction::ReturnNezSpan { condition, values } => {
                f(condition);
                values.visit_input_registers(f);
            }
            Instruction::ReturnNezMany { condition, values } => {
                f(condition);
                values.visit_input_registers(f);
            }
            Instruction::Branch { .. } => {},
            Instruction::BranchEqz { condition, .. } |
            Instruction::BranchNez { condition, .. } => f(condition),
            Instruction::BranchTable { index, .. } => f(index),
            Instruction::Copy { result, value } => {
                // Note: for copy instruction unlike all other instructions
                //       we need to also visit the result register since
                //       encoding of `local.set` or `local.tee` might actually
                //       result in a `copy` instruction with a `result` register
                //       allocated in the storage-space.
                visit_registers!(f, result, value)
            }
            Instruction::Copy2 { results: _, values } => {
                // Note: no need to visit `result` as in `Instruction::Copy` since
                //       this is mainly about updating registers in the reserve space
                //       due to optimizations of `local.set`.
                values.visit_input_registers(f);
            }
            Instruction::CopyImm32 { result: _, value: _ } |
            Instruction::CopyI64Imm32 { result: _, value: _ } |
            Instruction::CopyF64Imm32 { result: _, value: _ } => {},
            Instruction::CopySpan { results: _, values, len: _ } => {
                values.visit_input_registers(f);
            }
            Instruction::CopySpanNonOverlapping { results: _, values, len: _ } => {
                values.visit_input_registers(f);
            }
            Instruction::CopyMany { results: _, values } => {
                values.visit_input_registers(f);
            }
            Instruction::CallIndirectParams(params) => f(&mut params.index),
            Instruction::CallIndirectParamsImm16(_) => {},
            Instruction::ReturnCallInternal0 { .. } |
            Instruction::ReturnCallInternal { .. } |
            Instruction::ReturnCallImported0 { .. } |
            Instruction::ReturnCallImported { .. } |
            Instruction::ReturnCallIndirect0 { .. } |
            Instruction::ReturnCallIndirect { .. } => {},
            Instruction::CallInternal0 { .. } |
            Instruction::CallInternal { .. } |
            Instruction::CallImported0 { .. } |
            Instruction::CallImported { .. } |
            Instruction::CallIndirect0 { .. } |
            Instruction::CallIndirect { .. } => {},
            Instruction::Select { condition, lhs, .. } => visit_registers!(f, condition, lhs),
            Instruction::SelectRev { condition, rhs, .. } => visit_registers!(f, condition, rhs),
            Instruction::SelectImm32 { result_or_condition, .. } |
            Instruction::SelectI64Imm32 { result_or_condition, .. } |
            Instruction::SelectF64Imm32 { result_or_condition, .. } => f(result_or_condition),
            Instruction::RefFunc { .. } |
            Instruction::TableGet { .. } |
            Instruction::TableGetImm { .. } |
            Instruction::TableSize { .. } => {},
            Instruction::TableSet { index, value } => visit_registers!(f, index, value),
            Instruction::TableSetAt { value, .. } => f(value),
            Instruction::TableCopy { dst, src, len } => visit_registers!(f, dst, src, len),
            Instruction::TableCopyTo { dst: _, src, len } => visit_registers!(f, src, len),
            Instruction::TableCopyFrom { dst, src: _, len } => visit_registers!(f, dst, len),
            Instruction::TableCopyFromTo { dst: _, src: _, len } => f(len),
            Instruction::TableCopyExact { dst, src, len: _ } => visit_registers!(f, dst, src),
            Instruction::TableCopyToExact { dst: _, src, len: _ } => f(src),
            Instruction::TableCopyFromExact { dst, src: _, len: _ } => f(dst),
            Instruction::TableCopyFromToExact { dst: _, src: _, len: _ } => {},
            Instruction::TableInit { dst, src, len } => visit_registers!(f, dst, src, len),
            Instruction::TableInitTo { dst: _, src, len } => visit_registers!(f, src, len),
            Instruction::TableInitFrom { dst, src: _, len } => visit_registers!(f, dst, len),
            Instruction::TableInitFromTo { dst: _, src: _, len } => f(len),
            Instruction::TableInitExact { dst, src, len: _ } => visit_registers!(f, dst, src),
            Instruction::TableInitToExact { dst: _, src, len: _ } => f(src),
            Instruction::TableInitFromExact { dst, src: _, len: _ } => f(dst),
            Instruction::TableInitFromToExact { dst: _, src: _, len: _ } => {},
            Instruction::TableFill { dst, len, value } => visit_registers!(f, dst, len, value),
            Instruction::TableFillAt { dst: _, len, value } => visit_registers!(f, len, value),
            Instruction::TableFillExact { dst, len: _, value } => visit_registers!(f, dst, value),
            Instruction::TableFillAtExact { dst: _, len: _, value } => f(value),
            Instruction::TableGrow { result: _, delta, value } => visit_registers!(f, delta, value),
            Instruction::TableGrowImm { result: _, delta: _, value } => f(value),
            Instruction::ElemDrop(_) => {}
            Instruction::DataDrop(_) => {}
            Instruction::MemorySize { result: _ } => {},
            Instruction::MemoryGrow { result: _, delta } => f(delta),
            Instruction::MemoryGrowBy { result: _, delta: _ } => {},
            Instruction::MemoryCopy { dst, src, len } => visit_registers!(f, dst, src, len),
            Instruction::MemoryCopyTo { dst: _, src, len } => visit_registers!(f, src, len),
            Instruction::MemoryCopyFrom { dst, src: _, len } => visit_registers!(f, dst, len),
            Instruction::MemoryCopyFromTo { dst: _, src: _, len } => f(len),
            Instruction::MemoryCopyExact { dst, src, len: _ } => visit_registers!(f, dst, src),
            Instruction::MemoryCopyToExact { dst: _, src, len: _ } => f(src),
            Instruction::MemoryCopyFromExact { dst, src: _, len: _ } => f(dst),
            Instruction::MemoryCopyFromToExact { dst: _, src: _, len: _ } => {},
            Instruction::MemoryFill { dst, value, len } => visit_registers!(f, dst, value, len),
            Instruction::MemoryFillAt { dst: _, value, len } => visit_registers!(f, value, len),
            Instruction::MemoryFillImm { dst, value: _, len } => visit_registers!(f, dst, len),
            Instruction::MemoryFillExact { dst, value, len: _ } => visit_registers!(f, dst, value),
            Instruction::MemoryFillAtImm { dst: _, value: _, len } => f(len),
            Instruction::MemoryFillAtExact { dst: _, value, len: _ } => f(value),
            Instruction::MemoryFillImmExact { dst, value: _, len: _ } => f(dst),
            Instruction::MemoryFillAtImmExact { dst: _, value: _, len: _ } => {},
            Instruction::MemoryInit { dst, src, len } => visit_registers!(f, dst, src, len),
            Instruction::MemoryInitTo { dst: _, src, len } => visit_registers!(f, src, len),
            Instruction::MemoryInitFrom { dst, src: _, len } => visit_registers!(f, dst, len),
            Instruction::MemoryInitFromTo { dst: _, src: _, len } => f(len),
            Instruction::MemoryInitExact { dst, src, len: _ } => visit_registers!(f, dst, src),
            Instruction::MemoryInitToExact { dst: _, src, len: _ } => f(src),
            Instruction::MemoryInitFromExact { dst, src: _, len: _ } => f(dst),
            Instruction::MemoryInitFromToExact { dst: _, src: _, len: _ } => {},
            Instruction::GlobalGet { result: _, global: _ } => {},
            Instruction::GlobalSet { global: _, input } => f(input),
            Instruction::GlobalSetI32Imm16 { global: _, input: _ } |
            Instruction::GlobalSetI64Imm16 { global: _, input: _ } => {},
            Instruction::I32Load(instr) => instr.visit_input_registers(f),
            Instruction::I32LoadAt(instr) => instr.visit_input_registers(f),
            Instruction::I32LoadOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Load(instr) => instr.visit_input_registers(f),
            Instruction::I64LoadAt(instr) => instr.visit_input_registers(f),
            Instruction::I64LoadOffset16(instr) => instr.visit_input_registers(f),
            Instruction::F32Load(instr) => instr.visit_input_registers(f),
            Instruction::F32LoadAt(instr) => instr.visit_input_registers(f),
            Instruction::F32LoadOffset16(instr) => instr.visit_input_registers(f),
            Instruction::F64Load(instr) => instr.visit_input_registers(f),
            Instruction::F64LoadAt(instr) => instr.visit_input_registers(f),
            Instruction::F64LoadOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I32Load8s(instr) => instr.visit_input_registers(f),
            Instruction::I32Load8sAt(instr) => instr.visit_input_registers(f),
            Instruction::I32Load8sOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I32Load8u(instr) => instr.visit_input_registers(f),
            Instruction::I32Load8uAt(instr) => instr.visit_input_registers(f),
            Instruction::I32Load8uOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I32Load16s(instr) => instr.visit_input_registers(f),
            Instruction::I32Load16sAt(instr) => instr.visit_input_registers(f),
            Instruction::I32Load16sOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I32Load16u(instr) => instr.visit_input_registers(f),
            Instruction::I32Load16uAt(instr) => instr.visit_input_registers(f),
            Instruction::I32Load16uOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Load8s(instr) => instr.visit_input_registers(f),
            Instruction::I64Load8sAt(instr) => instr.visit_input_registers(f),
            Instruction::I64Load8sOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Load8u(instr) => instr.visit_input_registers(f),
            Instruction::I64Load8uAt(instr) => instr.visit_input_registers(f),
            Instruction::I64Load8uOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Load16s(instr) => instr.visit_input_registers(f),
            Instruction::I64Load16sAt(instr) => instr.visit_input_registers(f),
            Instruction::I64Load16sOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Load16u(instr) => instr.visit_input_registers(f),
            Instruction::I64Load16uAt(instr) => instr.visit_input_registers(f),
            Instruction::I64Load16uOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Load32s(instr) => instr.visit_input_registers(f),
            Instruction::I64Load32sAt(instr) => instr.visit_input_registers(f),
            Instruction::I64Load32sOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Load32u(instr) => instr.visit_input_registers(f),
            Instruction::I64Load32uAt(instr) => instr.visit_input_registers(f),
            Instruction::I64Load32uOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I32Store(instr) => instr.visit_input_registers(f),
            Instruction::I32StoreOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I32StoreOffset16Imm16(instr) => instr.visit_input_registers(f),
            Instruction::I32StoreAt(instr) => instr.visit_input_registers(f),
            Instruction::I32StoreAtImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32Store8(instr) => instr.visit_input_registers(f),
            Instruction::I32Store8Offset16(instr) => instr.visit_input_registers(f),
            Instruction::I32Store8Offset16Imm(instr) => instr.visit_input_registers(f),
            Instruction::I32Store8At(instr) => instr.visit_input_registers(f),
            Instruction::I32Store8AtImm(instr) => instr.visit_input_registers(f),
            Instruction::I32Store16(instr) => instr.visit_input_registers(f),
            Instruction::I32Store16Offset16(instr) => instr.visit_input_registers(f),
            Instruction::I32Store16Offset16Imm(instr) => instr.visit_input_registers(f),
            Instruction::I32Store16At(instr) => instr.visit_input_registers(f),
            Instruction::I32Store16AtImm(instr) => instr.visit_input_registers(f),
            Instruction::I64Store(instr) => instr.visit_input_registers(f),
            Instruction::I64StoreOffset16(instr) => instr.visit_input_registers(f),
            Instruction::I64StoreOffset16Imm16(instr) => instr.visit_input_registers(f),
            Instruction::I64StoreAt(instr) => instr.visit_input_registers(f),
            Instruction::I64StoreAtImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64Store8(instr) => instr.visit_input_registers(f),
            Instruction::I64Store8Offset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Store8Offset16Imm(instr) => instr.visit_input_registers(f),
            Instruction::I64Store8At(instr) => instr.visit_input_registers(f),
            Instruction::I64Store8AtImm(instr) => instr.visit_input_registers(f),
            Instruction::I64Store16(instr) => instr.visit_input_registers(f),
            Instruction::I64Store16Offset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Store16Offset16Imm(instr) => instr.visit_input_registers(f),
            Instruction::I64Store16At(instr) => instr.visit_input_registers(f),
            Instruction::I64Store16AtImm(instr) => instr.visit_input_registers(f),
            Instruction::I64Store32(instr) => instr.visit_input_registers(f),
            Instruction::I64Store32Offset16(instr) => instr.visit_input_registers(f),
            Instruction::I64Store32Offset16Imm16(instr) => instr.visit_input_registers(f),
            Instruction::I64Store32At(instr) => instr.visit_input_registers(f),
            Instruction::I64Store32AtImm16(instr) => instr.visit_input_registers(f),
            Instruction::F32Store(instr) => instr.visit_input_registers(f),
            Instruction::F32StoreOffset16(instr) => instr.visit_input_registers(f),
            Instruction::F32StoreAt(instr) => instr.visit_input_registers(f),
            Instruction::F64Store(instr) => instr.visit_input_registers(f),
            Instruction::F64StoreOffset16(instr) => instr.visit_input_registers(f),
            Instruction::F64StoreAt(instr) => instr.visit_input_registers(f),
            Instruction::I32Eq(instr) => instr.visit_input_registers(f),
            Instruction::I32EqImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64Eq(instr) => instr.visit_input_registers(f),
            Instruction::I64EqImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32Ne(instr) => instr.visit_input_registers(f),
            Instruction::I32NeImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64Ne(instr) => instr.visit_input_registers(f),
            Instruction::I64NeImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32LtS(instr) => instr.visit_input_registers(f),
            Instruction::I32LtU(instr) => instr.visit_input_registers(f),
            Instruction::I32LtSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32LtUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64LtS(instr) => instr.visit_input_registers(f),
            Instruction::I64LtU(instr) => instr.visit_input_registers(f),
            Instruction::I64LtSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64LtUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32GtS(instr) => instr.visit_input_registers(f),
            Instruction::I32GtU(instr) => instr.visit_input_registers(f),
            Instruction::I32GtSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32GtUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64GtS(instr) => instr.visit_input_registers(f),
            Instruction::I64GtU(instr) => instr.visit_input_registers(f),
            Instruction::I64GtSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64GtUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32LeS(instr) => instr.visit_input_registers(f),
            Instruction::I32LeU(instr) => instr.visit_input_registers(f),
            Instruction::I32LeSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32LeUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64LeS(instr) => instr.visit_input_registers(f),
            Instruction::I64LeU(instr) => instr.visit_input_registers(f),
            Instruction::I64LeSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64LeUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32GeS(instr) => instr.visit_input_registers(f),
            Instruction::I32GeU(instr) => instr.visit_input_registers(f),
            Instruction::I32GeSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32GeUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64GeS(instr) => instr.visit_input_registers(f),
            Instruction::I64GeU(instr) => instr.visit_input_registers(f),
            Instruction::I64GeSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64GeUImm16(instr) => instr.visit_input_registers(f),
            Instruction::F32Eq(instr) => instr.visit_input_registers(f),
            Instruction::F64Eq(instr) => instr.visit_input_registers(f),
            Instruction::F32Ne(instr) => instr.visit_input_registers(f),
            Instruction::F64Ne(instr) => instr.visit_input_registers(f),
            Instruction::F32Lt(instr) => instr.visit_input_registers(f),
            Instruction::F64Lt(instr) => instr.visit_input_registers(f),
            Instruction::F32Le(instr) => instr.visit_input_registers(f),
            Instruction::F64Le(instr) => instr.visit_input_registers(f),
            Instruction::F32Gt(instr) => instr.visit_input_registers(f),
            Instruction::F64Gt(instr) => instr.visit_input_registers(f),
            Instruction::F32Ge(instr) => instr.visit_input_registers(f),
            Instruction::F64Ge(instr) => instr.visit_input_registers(f),
            Instruction::I32Clz(instr) => instr.visit_input_registers(f),
            Instruction::I64Clz(instr) => instr.visit_input_registers(f),
            Instruction::I32Ctz(instr) => instr.visit_input_registers(f),
            Instruction::I64Ctz(instr) => instr.visit_input_registers(f),
            Instruction::I32Popcnt(instr) => instr.visit_input_registers(f),
            Instruction::I64Popcnt(instr) => instr.visit_input_registers(f),
            Instruction::I32Add(instr) => instr.visit_input_registers(f),
            Instruction::I64Add(instr) => instr.visit_input_registers(f),
            Instruction::I32AddImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64AddImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32Sub(instr) => instr.visit_input_registers(f),
            Instruction::I64Sub(instr) => instr.visit_input_registers(f),
            Instruction::I32SubImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64SubImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32SubImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64SubImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32Mul(instr) => instr.visit_input_registers(f),
            Instruction::I64Mul(instr) => instr.visit_input_registers(f),
            Instruction::I32MulImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64MulImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32DivS(instr) => instr.visit_input_registers(f),
            Instruction::I64DivS(instr) => instr.visit_input_registers(f),
            Instruction::I32DivSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64DivSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32DivSImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64DivSImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32DivU(instr) => instr.visit_input_registers(f),
            Instruction::I64DivU(instr) => instr.visit_input_registers(f),
            Instruction::I32DivUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64DivUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32DivUImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64DivUImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32RemS(instr) => instr.visit_input_registers(f),
            Instruction::I64RemS(instr) => instr.visit_input_registers(f),
            Instruction::I32RemSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64RemSImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32RemSImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64RemSImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32RemU(instr) => instr.visit_input_registers(f),
            Instruction::I64RemU(instr) => instr.visit_input_registers(f),
            Instruction::I32RemUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64RemUImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32RemUImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64RemUImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32And(instr) => instr.visit_input_registers(f),
            Instruction::I64And(instr) => instr.visit_input_registers(f),
            Instruction::I32AndImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64AndImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32Or(instr) => instr.visit_input_registers(f),
            Instruction::I64Or(instr) => instr.visit_input_registers(f),
            Instruction::I32OrImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64OrImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32Xor(instr) => instr.visit_input_registers(f),
            Instruction::I64Xor(instr) => instr.visit_input_registers(f),
            Instruction::I32XorImm16(instr) => instr.visit_input_registers(f),
            Instruction::I64XorImm16(instr) => instr.visit_input_registers(f),
            Instruction::I32Shl(instr) => instr.visit_input_registers(f),
            Instruction::I64Shl(instr) => instr.visit_input_registers(f),
            Instruction::I32ShlImm(instr) => instr.visit_input_registers(f),
            Instruction::I64ShlImm(instr) => instr.visit_input_registers(f),
            Instruction::I32ShlImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64ShlImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32ShrU(instr) => instr.visit_input_registers(f),
            Instruction::I64ShrU(instr) => instr.visit_input_registers(f),
            Instruction::I32ShrUImm(instr) => instr.visit_input_registers(f),
            Instruction::I64ShrUImm(instr) => instr.visit_input_registers(f),
            Instruction::I32ShrUImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64ShrUImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32ShrS(instr) => instr.visit_input_registers(f),
            Instruction::I64ShrS(instr) => instr.visit_input_registers(f),
            Instruction::I32ShrSImm(instr) => instr.visit_input_registers(f),
            Instruction::I64ShrSImm(instr) => instr.visit_input_registers(f),
            Instruction::I32ShrSImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64ShrSImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32Rotl(instr) => instr.visit_input_registers(f),
            Instruction::I64Rotl(instr) => instr.visit_input_registers(f),
            Instruction::I32RotlImm(instr) => instr.visit_input_registers(f),
            Instruction::I64RotlImm(instr) => instr.visit_input_registers(f),
            Instruction::I32RotlImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64RotlImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I32Rotr(instr) => instr.visit_input_registers(f),
            Instruction::I64Rotr(instr) => instr.visit_input_registers(f),
            Instruction::I32RotrImm(instr) => instr.visit_input_registers(f),
            Instruction::I64RotrImm(instr) => instr.visit_input_registers(f),
            Instruction::I32RotrImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::I64RotrImm16Rev(instr) => instr.visit_input_registers(f),
            Instruction::F32Abs(instr) => instr.visit_input_registers(f),
            Instruction::F64Abs(instr) => instr.visit_input_registers(f),
            Instruction::F32Neg(instr) => instr.visit_input_registers(f),
            Instruction::F64Neg(instr) => instr.visit_input_registers(f),
            Instruction::F32Ceil(instr) => instr.visit_input_registers(f),
            Instruction::F64Ceil(instr) => instr.visit_input_registers(f),
            Instruction::F32Floor(instr) => instr.visit_input_registers(f),
            Instruction::F64Floor(instr) => instr.visit_input_registers(f),
            Instruction::F32Trunc(instr) => instr.visit_input_registers(f),
            Instruction::F64Trunc(instr) => instr.visit_input_registers(f),
            Instruction::F32Nearest(instr) => instr.visit_input_registers(f),
            Instruction::F64Nearest(instr) => instr.visit_input_registers(f),
            Instruction::F32Sqrt(instr) => instr.visit_input_registers(f),
            Instruction::F64Sqrt(instr) => instr.visit_input_registers(f),
            Instruction::F32Add(instr) => instr.visit_input_registers(f),
            Instruction::F64Add(instr) => instr.visit_input_registers(f),
            Instruction::F32Sub(instr) => instr.visit_input_registers(f),
            Instruction::F64Sub(instr) => instr.visit_input_registers(f),
            Instruction::F32Mul(instr) => instr.visit_input_registers(f),
            Instruction::F64Mul(instr) => instr.visit_input_registers(f),
            Instruction::F32Div(instr) => instr.visit_input_registers(f),
            Instruction::F64Div(instr) => instr.visit_input_registers(f),
            Instruction::F32Min(instr) => instr.visit_input_registers(f),
            Instruction::F64Min(instr) => instr.visit_input_registers(f),
            Instruction::F32Max(instr) => instr.visit_input_registers(f),
            Instruction::F64Max(instr) => instr.visit_input_registers(f),
            Instruction::F32Copysign(instr) => instr.visit_input_registers(f),
            Instruction::F64Copysign(instr) => instr.visit_input_registers(f),
            Instruction::F32CopysignImm(instr) => instr.visit_input_registers(f),
            Instruction::F64CopysignImm(instr) => instr.visit_input_registers(f),
            Instruction::I32WrapI64(instr) => instr.visit_input_registers(f),
            Instruction::I64ExtendI32S(instr) => instr.visit_input_registers(f),
            Instruction::I64ExtendI32U(instr) => instr.visit_input_registers(f),
            Instruction::I32TruncF32S(instr) => instr.visit_input_registers(f),
            Instruction::I32TruncF32U(instr) => instr.visit_input_registers(f),
            Instruction::I32TruncF64S(instr) => instr.visit_input_registers(f),
            Instruction::I32TruncF64U(instr) => instr.visit_input_registers(f),
            Instruction::I64TruncF32S(instr) => instr.visit_input_registers(f),
            Instruction::I64TruncF32U(instr) => instr.visit_input_registers(f),
            Instruction::I64TruncF64S(instr) => instr.visit_input_registers(f),
            Instruction::I64TruncF64U(instr) => instr.visit_input_registers(f),
            Instruction::I32TruncSatF32S(instr) => instr.visit_input_registers(f),
            Instruction::I32TruncSatF32U(instr) => instr.visit_input_registers(f),
            Instruction::I32TruncSatF64S(instr) => instr.visit_input_registers(f),
            Instruction::I32TruncSatF64U(instr) => instr.visit_input_registers(f),
            Instruction::I64TruncSatF32S(instr) => instr.visit_input_registers(f),
            Instruction::I64TruncSatF32U(instr) => instr.visit_input_registers(f),
            Instruction::I64TruncSatF64S(instr) => instr.visit_input_registers(f),
            Instruction::I64TruncSatF64U(instr) => instr.visit_input_registers(f),
            Instruction::I32Extend8S(instr) => instr.visit_input_registers(f),
            Instruction::I32Extend16S(instr) => instr.visit_input_registers(f),
            Instruction::I64Extend8S(instr) => instr.visit_input_registers(f),
            Instruction::I64Extend16S(instr) => instr.visit_input_registers(f),
            Instruction::I64Extend32S(instr) => instr.visit_input_registers(f),
            Instruction::F32DemoteF64(instr) => instr.visit_input_registers(f),
            Instruction::F64PromoteF32(instr) => instr.visit_input_registers(f),
            Instruction::F32ConvertI32S(instr) => instr.visit_input_registers(f),
            Instruction::F32ConvertI32U(instr) => instr.visit_input_registers(f),
            Instruction::F32ConvertI64S(instr) => instr.visit_input_registers(f),
            Instruction::F32ConvertI64U(instr) => instr.visit_input_registers(f),
            Instruction::F64ConvertI32S(instr) => instr.visit_input_registers(f),
            Instruction::F64ConvertI32U(instr) => instr.visit_input_registers(f),
            Instruction::F64ConvertI64S(instr) => instr.visit_input_registers(f),
            Instruction::F64ConvertI64U(instr) => instr.visit_input_registers(f),
        }
    }
}

impl<const N: usize> VisitInputRegisters for [Register; N] {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        for register in self {
            f(register);
        }
    }
}

impl LoadInstr {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.ptr);
    }
}

impl LoadAtInstr {
    fn visit_input_registers(&mut self, _f: impl FnMut(&mut Register)) {
        // Nothing to do.
    }
}

impl LoadOffset16Instr {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.ptr)
    }
}

impl VisitInputRegisters for StoreInstr {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.ptr);
    }
}

impl VisitInputRegisters for StoreAtInstr<Register> {
    fn visit_input_registers(&mut self, _f: impl FnMut(&mut Register)) {
        // Nothing to do.
    }
}

impl VisitInputRegisters for StoreAtInstr<i8> {
    fn visit_input_registers(&mut self, _f: impl FnMut(&mut Register)) {
        // Nothing to do.
    }
}

impl VisitInputRegisters for StoreAtInstr<i16> {
    fn visit_input_registers(&mut self, _f: impl FnMut(&mut Register)) {
        // Nothing to do.
    }
}

impl VisitInputRegisters for StoreAtInstr<Const16<i32>> {
    fn visit_input_registers(&mut self, _f: impl FnMut(&mut Register)) {
        // Nothing to do.
    }
}

impl VisitInputRegisters for StoreAtInstr<Const16<i64>> {
    fn visit_input_registers(&mut self, _f: impl FnMut(&mut Register)) {
        // Nothing to do.
    }
}

impl VisitInputRegisters for StoreOffset16Instr<Register> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        visit_registers!(f, &mut self.ptr, &mut self.value)
    }
}

impl VisitInputRegisters for StoreOffset16Instr<i8> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.ptr)
    }
}

impl VisitInputRegisters for StoreOffset16Instr<i16> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.ptr)
    }
}

impl VisitInputRegisters for StoreOffset16Instr<Const16<i32>> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.ptr)
    }
}

impl VisitInputRegisters for StoreOffset16Instr<Const16<i64>> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.ptr)
    }
}

impl VisitInputRegisters for UnaryInstr {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.input)
    }
}

impl VisitInputRegisters for BinInstr {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        visit_registers!(f, &mut self.lhs, &mut self.rhs)
    }
}

impl VisitInputRegisters for BinInstrImm16<i32> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.reg_in)
    }
}

impl VisitInputRegisters for BinInstrImm16<u32> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.reg_in)
    }
}

impl VisitInputRegisters for BinInstrImm16<i64> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.reg_in)
    }
}

impl VisitInputRegisters for BinInstrImm16<u64> {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.reg_in)
    }
}

impl VisitInputRegisters for CopysignImmInstr {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(&mut self.lhs)
    }
}

impl VisitInputRegisters for RegisterSpan {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        f(self.head_mut())
    }
}

impl VisitInputRegisters for RegisterSpanIter {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Register)) {
        let len = self.len_as_u16();
        let mut span = self.span();
        f(span.head_mut());
        *self = span.iter_u16(len);
    }
}
