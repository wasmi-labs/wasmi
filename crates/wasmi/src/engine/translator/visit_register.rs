use crate::engine::bytecode::{Instruction, Reg, RegSpan, RegSpanIter};

macro_rules! visit_registers {
    ( $f:expr, $($field:expr),* $(,)? ) => {{
        $(
            $f($field)
        );*
    }};
}

/// Trait implemented by types that allow to visit their [`Reg`] fields.
pub trait VisitInputRegisters {
    /// Calls `f` on all input [`Reg`].
    fn visit_input_registers(&mut self, f: impl FnMut(&mut Reg));
}

impl VisitInputRegisters for Instruction {
    #[rustfmt::skip]
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Reg)) {
        match self {
            Instruction::TableIndex { .. } |
            Instruction::DataIndex { .. } |
            Instruction::ElemIndex { .. } |
            Instruction::Const32 { .. } |
            Instruction::I64Const32 { .. } |
            Instruction::F64Const32 { .. } => {},
            Instruction::RegisterAndImm32 { reg, .. } => f(reg),
            Instruction::Register { reg } => f(reg),
            Instruction::Register2 { regs } => regs.visit_input_registers(f),
            Instruction::Register3 { regs } |
            Instruction::RegisterList { regs } => regs.visit_input_registers(f),
            Instruction::RegisterSpan { span } => span.visit_input_registers(f),
            Instruction::BranchTableTarget { .. } |
            Instruction::BranchTableTargetNonOverlapping { .. } => {},
            Instruction::Trap { .. } |
            Instruction::ConsumeFuel { .. } |
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
            Instruction::BranchTable0 { index, .. } |
            Instruction::BranchTable1 { index, .. } |
            Instruction::BranchTable2 { index, .. } |
            Instruction::BranchTable3 { index, .. } |
            Instruction::BranchTableSpan { index, .. } |
            Instruction::BranchTableMany { index, .. } => f(index),

            Instruction::BranchCmpFallback { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32And { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32AndImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32Or { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32OrImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32Xor { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32XorImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32AndEqz { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32AndEqzImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32OrEqz { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32OrEqzImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32XorEqz { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32XorEqzImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32Eq { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32EqImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32Ne { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32NeImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32LtS { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32LtSImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32LtU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32LtUImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32LeS { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32LeSImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32LeU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32LeUImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32GtS { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32GtSImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32GtU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32GtUImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32GeS { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32GeSImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI32GeU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI32GeUImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64Eq { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64EqImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64Ne { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64NeImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64LtS { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64LtSImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64LtU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64LtUImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64LeS { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64LeSImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64LeU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64LeUImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64GtS { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64GtSImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64GtU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64GtUImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64GeS { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64GeSImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchI64GeU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::BranchI64GeUImm { lhs, .. } => visit_registers!(f, lhs),
            Instruction::BranchF32Eq { lhs, rhs, .. } |
            Instruction::BranchF32Ne { lhs, rhs, .. } |
            Instruction::BranchF32Lt { lhs, rhs, .. } |
            Instruction::BranchF32Le { lhs, rhs, .. } |
            Instruction::BranchF32Gt { lhs, rhs, .. } |
            Instruction::BranchF32Ge { lhs, rhs, .. } |
            Instruction::BranchF64Eq { lhs, rhs, .. } |
            Instruction::BranchF64Ne { lhs, rhs, .. } |
            Instruction::BranchF64Lt { lhs, rhs, .. } |
            Instruction::BranchF64Le { lhs, rhs, .. } |
            Instruction::BranchF64Gt { lhs, rhs, .. } |
            Instruction::BranchF64Ge { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),

            Instruction::Copy { result, value } => {
                // Note: for copy instructions unlike all other instructions
                //       we need to also visit the result register since
                //       encoding of `local.set` or `local.tee` might actually
                //       result in a `copy` instruction with a `result` register
                //       allocated in the storage-space.
                visit_registers!(f, result, value)
            }
            Instruction::Copy2 { results, values } => {
                // Note: we need to visit the results of the `Copy2` instruction
                //       as well since it might have been generated while preserving locals
                //       on the compilation stack when entering a control flow `block`
                //       or `if`.
                f(results.head_mut());
                values.visit_input_registers(f);
            }
            Instruction::CopyImm32 { result: _, value: _ } |
            Instruction::CopyI64Imm32 { result: _, value: _ } |
            Instruction::CopyF64Imm32 { result: _, value: _ } => {},
            Instruction::CopySpan { results: _, values, len: _ } => {
                values.visit_input_registers(f);
            }
            Instruction::CopySpanNonOverlapping { results, values, len: _ } => {
                // Note: we need to visit the results of the `CopySpanNonOverlapping` instruction
                //       as well since it might have been generated while preserving locals
                //       on the compilation stack when entering a control flow `block`
                //       or `if`.
                f(results.head_mut());
                values.visit_input_registers(f);
            }
            Instruction::CopyMany { results: _, values } => {
                values.visit_input_registers(f);
            }
            Instruction::CopyManyNonOverlapping { results, values } => {
                // Note: we need to visit the results of the `CopyManyNonOverlapping` instruction
                //       as well since it might have been generated while preserving locals
                //       on the compilation stack when entering a control flow `block`
                //       or `if`.
                f(results.head_mut());
                values.visit_input_registers(f);
            }
            Instruction::CallIndirectParams { index, .. } => f(index),
            Instruction::CallIndirectParamsImm16 { .. } => {},
            Instruction::ReturnCallInternal0 { .. } |
            Instruction::ReturnCallInternal { .. } |
            Instruction::ReturnCallImported0 { .. } |
            Instruction::ReturnCallImported { .. } |
            Instruction::ReturnCallIndirect0 { .. } |
            Instruction::ReturnCallIndirect0Imm16 { .. } |
            Instruction::ReturnCallIndirect { .. } |
            Instruction::ReturnCallIndirectImm16 { .. } => {},
            Instruction::CallInternal0 { .. } |
            Instruction::CallInternal { .. } |
            Instruction::CallImported0 { .. } |
            Instruction::CallImported { .. } |
            Instruction::CallIndirect0 { .. } |
            Instruction::CallIndirect0Imm16 { .. } |
            Instruction::CallIndirect { .. } |
            Instruction::CallIndirectImm16 { .. } => {},
            Instruction::Select { lhs, .. } => f(lhs),
            Instruction::SelectImm32Rhs { lhs, .. } => f(lhs),
            Instruction::SelectImm32Lhs { .. } |
            Instruction::SelectImm32 { .. } => {},
            Instruction::SelectI64Imm32Rhs { lhs, .. } => f(lhs),
            Instruction::SelectI64Imm32Lhs { .. } |
            Instruction::SelectI64Imm32 { .. } => {},
            Instruction::SelectF64Imm32Rhs { lhs, .. } => f(lhs),
            Instruction::SelectF64Imm32Lhs { .. } |
            Instruction::SelectF64Imm32 { .. } => {},
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
            Instruction::I32Load { ptr, .. } |
            Instruction::I64Load { ptr, .. } |
            Instruction::F32Load { ptr, .. } |
            Instruction::F64Load { ptr, .. } |
            Instruction::I32Load8s { ptr, .. } |
            Instruction::I32Load8u { ptr, .. } |
            Instruction::I32Load16s { ptr, .. } |
            Instruction::I32Load16u { ptr, .. } |
            Instruction::I64Load8s { ptr, .. } |
            Instruction::I64Load8u { ptr, .. } |
            Instruction::I64Load16s { ptr, .. } |
            Instruction::I64Load16u { ptr, .. } |
            Instruction::I64Load32s { ptr, .. } |
            Instruction::I64Load32u { ptr, .. } |
            Instruction::I32LoadOffset16 { ptr, .. } |
            Instruction::I64LoadOffset16 { ptr, .. } |
            Instruction::F32LoadOffset16 { ptr, .. } |
            Instruction::F64LoadOffset16 { ptr, .. } |
            Instruction::I32Load8sOffset16 { ptr, .. } |
            Instruction::I32Load8uOffset16 { ptr, .. } |
            Instruction::I32Load16sOffset16 { ptr, .. } |
            Instruction::I32Load16uOffset16 { ptr, .. } |
            Instruction::I64Load8sOffset16 { ptr, .. } |
            Instruction::I64Load8uOffset16 { ptr, .. } |
            Instruction::I64Load16sOffset16 { ptr, .. } |
            Instruction::I64Load16uOffset16 { ptr, .. } |
            Instruction::I64Load32sOffset16 { ptr, .. } |
            Instruction::I64Load32uOffset16 { ptr, .. } => f(ptr),
            Instruction::I32LoadAt { .. } => {},
            Instruction::I64LoadAt { .. } => {},
            Instruction::F32LoadAt { .. } => {},
            Instruction::F64LoadAt { .. } => {},
            Instruction::I32Load8sAt { .. } => {},
            Instruction::I32Load8uAt { .. } => {},
            Instruction::I32Load16sAt { .. } => {},
            Instruction::I32Load16uAt { .. } => {},
            Instruction::I64Load8sAt { .. } => {},
            Instruction::I64Load8uAt { .. } => {},
            Instruction::I64Load16sAt { .. } => {},
            Instruction::I64Load16uAt { .. } => {},
            Instruction::I64Load32sAt { .. } => {},
            Instruction::I64Load32uAt { .. } => {},
            Instruction::I32Store { ptr, .. } |
            Instruction::I32StoreOffset16Imm16 { ptr, .. } |
            Instruction::I32Store8 { ptr, .. } |
            Instruction::I32Store8Offset16Imm { ptr, .. } |
            Instruction::I32Store16 { ptr, .. } |
            Instruction::I32Store16Offset16Imm { ptr, .. } |
            Instruction::I64Store { ptr, .. } |
            Instruction::I64StoreOffset16Imm16 { ptr, .. } |
            Instruction::I64Store8 { ptr, .. } |
            Instruction::I64Store8Offset16Imm { ptr, .. } |
            Instruction::I64Store16 { ptr, .. } |
            Instruction::I64Store16Offset16Imm { ptr, .. } |
            Instruction::I64Store32 { ptr, .. } |
            Instruction::I64Store32Offset16Imm16 { ptr, .. } |
            Instruction::F32Store { ptr, .. } |
            Instruction::F64Store { ptr, .. } => f(ptr),
            Instruction::F32StoreAt { value, .. } |
            Instruction::F64StoreAt { value, .. } |
            Instruction::I32StoreAt { value, .. } |
            Instruction::I32Store8At { value, .. } |
            Instruction::I32Store16At { value, .. } |
            Instruction::I64StoreAt { value, .. } |
            Instruction::I64Store8At { value, .. } |
            Instruction::I64Store16At { value, .. } |
            Instruction::I64Store32At { value, .. } => f(value),
            Instruction::I32StoreAtImm16 { .. } |
            Instruction::I32Store8AtImm { .. } |
            Instruction::I32Store16AtImm { .. } |
            Instruction::I64StoreAtImm16 { .. } |
            Instruction::I64Store8AtImm { .. } |
            Instruction::I64Store16AtImm { .. } |
            Instruction::I64Store32AtImm16 { .. } => {}
            Instruction::I32StoreOffset16 { ptr, value, ..} |
            Instruction::I32Store8Offset16 { ptr, value, ..} |
            Instruction::I32Store16Offset16 { ptr, value, ..} |
            Instruction::I64StoreOffset16 { ptr, value, ..} |
            Instruction::I64Store8Offset16 { ptr, value, ..} |
            Instruction::I64Store16Offset16 { ptr, value, ..} |
            Instruction::I64Store32Offset16 { ptr, value, ..} |
            Instruction::F32StoreOffset16 { ptr, value, .. } |
            Instruction::F64StoreOffset16 { ptr, value, .. } => visit_registers!(f, ptr, value),
            Instruction::I32Eq { lhs, rhs, .. } |
            Instruction::I64Eq { lhs, rhs, .. } |
            Instruction::I32Ne { lhs, rhs, .. } |
            Instruction::I64Ne { lhs, rhs, .. } |
            Instruction::I32LtS { lhs, rhs, .. } |
            Instruction::I32LtU { lhs, rhs, .. } |
            Instruction::I64LtS { lhs, rhs, .. } |
            Instruction::I64LtU { lhs, rhs, .. } |
            Instruction::I32GtS { lhs, rhs, .. } |
            Instruction::I32GtU { lhs, rhs, .. } |
            Instruction::I64GtS { lhs, rhs, .. } |
            Instruction::I64GtU { lhs, rhs, .. } |
            Instruction::I32LeS { lhs, rhs, .. } |
            Instruction::I32LeU { lhs, rhs, .. } |
            Instruction::I64LeS { lhs, rhs, .. } |
            Instruction::I64LeU { lhs, rhs, .. } |
            Instruction::I32GeS { lhs, rhs, .. } |
            Instruction::I32GeU { lhs, rhs, .. } |
            Instruction::I64GeS { lhs, rhs, .. } |
            Instruction::I64GeU { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::I32EqImm16 { lhs, .. } |
            Instruction::I64EqImm16 { lhs, .. } |
            Instruction::I32NeImm16 { lhs, .. } |
            Instruction::I64NeImm16 { lhs, .. } |
            Instruction::I32LtSImm16 { lhs, .. } |
            Instruction::I32LtUImm16 { lhs, .. } |
            Instruction::I64LtSImm16 { lhs, .. } |
            Instruction::I64LtUImm16 { lhs, .. } |
            Instruction::I32GtSImm16 { lhs, .. } |
            Instruction::I32GtUImm16 { lhs, .. } |
            Instruction::I64GtSImm16 { lhs, .. } |
            Instruction::I64GtUImm16 { lhs, .. } |
            Instruction::I32LeSImm16 { lhs, .. } |
            Instruction::I32LeUImm16 { lhs, .. } |
            Instruction::I64LeSImm16 { lhs, .. } |
            Instruction::I64LeUImm16 { lhs, .. } |
            Instruction::I32GeSImm16 { lhs, .. } |
            Instruction::I32GeUImm16 { lhs, .. } |
            Instruction::I64GeSImm16 { lhs, .. } |
            Instruction::I64GeUImm16 { lhs, .. } => f(lhs),
            Instruction::F32Eq { lhs, rhs, .. } |
            Instruction::F64Eq { lhs, rhs, .. } |
            Instruction::F32Ne { lhs, rhs, .. } |
            Instruction::F64Ne { lhs, rhs, .. } |
            Instruction::F32Lt { lhs, rhs, .. } |
            Instruction::F64Lt { lhs, rhs, .. } |
            Instruction::F32Le { lhs, rhs, .. } |
            Instruction::F64Le { lhs, rhs, .. } |
            Instruction::F32Gt { lhs, rhs, .. } |
            Instruction::F64Gt { lhs, rhs, .. } |
            Instruction::F32Ge { lhs, rhs, .. } |
            Instruction::F64Ge { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::I32Clz { input, .. } |
            Instruction::I64Clz { input, .. } |
            Instruction::I32Ctz { input, .. } |
            Instruction::I64Ctz { input, .. } |
            Instruction::I32Popcnt { input, .. } |
            Instruction::I64Popcnt { input, .. } => f(input),
            Instruction::I32Add { lhs, rhs, .. } |
            Instruction::I64Add { lhs, rhs, .. } |
            Instruction::I32Sub { lhs, rhs, .. } |
            Instruction::I64Sub { lhs, rhs, .. } |
            Instruction::I32Mul { lhs, rhs, .. } |
            Instruction::I64Mul { lhs, rhs, .. } |
            Instruction::I32DivS { lhs, rhs, .. } |
            Instruction::I64DivS { lhs, rhs, .. } |
            Instruction::I32DivU { lhs, rhs, .. } |
            Instruction::I64DivU { lhs, rhs, .. } |
            Instruction::I32RemS { lhs, rhs, .. } |
            Instruction::I64RemS { lhs, rhs, .. } |
            Instruction::I32RemU { lhs, rhs, .. } |
            Instruction::I64RemU { lhs, rhs, .. } |
            Instruction::I32And { lhs, rhs, .. } |
            Instruction::I32AndEqz { lhs, rhs, .. } |
            Instruction::I64And { lhs, rhs, .. } |
            Instruction::I32Or { lhs, rhs, .. } |
            Instruction::I32OrEqz { lhs, rhs, .. } |
            Instruction::I64Or { lhs, rhs, .. } |
            Instruction::I32Xor { lhs, rhs, .. } |
            Instruction::I32XorEqz { lhs, rhs, .. } |
            Instruction::I64Xor { lhs, rhs, .. } |
            Instruction::I32Shl { lhs, rhs, .. } |
            Instruction::I64Shl { lhs, rhs, .. } |
            Instruction::I32ShrU { lhs, rhs, .. } |
            Instruction::I64ShrU { lhs, rhs, .. } |
            Instruction::I32ShrS { lhs, rhs, .. } |
            Instruction::I64ShrS { lhs, rhs, .. } |
            Instruction::I32Rotl { lhs, rhs, .. } |
            Instruction::I64Rotl { lhs, rhs, .. } |
            Instruction::I32Rotr { lhs, rhs, .. } |
            Instruction::I64Rotr { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::I32AddImm16 { lhs, .. } |
            Instruction::I64AddImm16 { lhs, .. } |
            Instruction::I32MulImm16 { lhs, .. } |
            Instruction::I64MulImm16 { lhs, .. } |
            Instruction::I32DivSImm16 { lhs, .. } |
            Instruction::I64DivSImm16 { lhs, .. } |
            Instruction::I32DivUImm16 { lhs, .. } |
            Instruction::I64DivUImm16 { lhs, .. } |
            Instruction::I32RemSImm16 { lhs, .. } |
            Instruction::I64RemSImm16 { lhs, .. } |
            Instruction::I32RemUImm16 { lhs, .. } |
            Instruction::I64RemUImm16 { lhs, .. } |
            Instruction::I32AndEqzImm16 { lhs, .. } |
            Instruction::I32AndImm16 { lhs, .. } |
            Instruction::I64AndImm16 { lhs, .. } |
            Instruction::I32OrEqzImm16 { lhs, .. } |
            Instruction::I32OrImm16 { lhs, .. } |
            Instruction::I64OrImm16 { lhs, .. } |
            Instruction::I32XorEqzImm16 { lhs, .. } |
            Instruction::I32XorImm16 { lhs, .. } |
            Instruction::I64XorImm16 { lhs, .. } |
            Instruction::I32ShlImm { lhs, .. } |
            Instruction::I64ShlImm { lhs, .. } |
            Instruction::I32ShrUImm { lhs, .. } |
            Instruction::I64ShrUImm { lhs, .. } |
            Instruction::I32ShrSImm { lhs, .. } |
            Instruction::I64ShrSImm { lhs, .. } |
            Instruction::I32RotlImm { lhs, .. } |
            Instruction::I64RotlImm { lhs, .. } |
            Instruction::I32RotrImm { lhs, .. } |
            Instruction::I64RotrImm { lhs, .. } => f(lhs),
            Instruction::I32SubImm16Rev { rhs, .. } |
            Instruction::I64SubImm16Rev { rhs, .. } |
            Instruction::I32DivSImm16Rev { rhs, .. } |
            Instruction::I64DivSImm16Rev { rhs, .. } |
            Instruction::I32DivUImm16Rev { rhs, .. } |
            Instruction::I64DivUImm16Rev { rhs, .. } |
            Instruction::I32RemSImm16Rev { rhs, .. } |
            Instruction::I64RemSImm16Rev { rhs, .. } |
            Instruction::I32RemUImm16Rev { rhs, .. } |
            Instruction::I64RemUImm16Rev { rhs, .. } |
            Instruction::I32ShlImm16Rev { rhs, .. } |
            Instruction::I64ShlImm16Rev { rhs, .. } |
            Instruction::I32ShrUImm16Rev { rhs, .. } |
            Instruction::I64ShrUImm16Rev { rhs, .. } |
            Instruction::I32ShrSImm16Rev { rhs, .. } |
            Instruction::I64ShrSImm16Rev { rhs, .. } |
            Instruction::I32RotlImm16Rev { rhs, .. } |
            Instruction::I64RotlImm16Rev { rhs, .. } |
            Instruction::I32RotrImm16Rev { rhs, .. } |
            Instruction::I64RotrImm16Rev { rhs, .. } => f(rhs),
            Instruction::F32Abs { input, .. } |
            Instruction::F64Abs { input, .. } |
            Instruction::F32Neg { input, .. } |
            Instruction::F64Neg { input, .. } |
            Instruction::F32Ceil { input, .. } |
            Instruction::F64Ceil { input, .. } |
            Instruction::F32Floor { input, .. } |
            Instruction::F64Floor { input, .. } |
            Instruction::F32Trunc { input, .. } |
            Instruction::F64Trunc { input, .. } |
            Instruction::F32Nearest { input, .. } |
            Instruction::F64Nearest { input, .. } |
            Instruction::F32Sqrt { input, .. } |
            Instruction::F64Sqrt { input, .. } => f(input),
            Instruction::F32Add { lhs, rhs, .. } |
            Instruction::F64Add { lhs, rhs, .. } |
            Instruction::F32Sub { lhs, rhs, .. } |
            Instruction::F64Sub { lhs, rhs, .. } |
            Instruction::F32Mul { lhs, rhs, .. } |
            Instruction::F64Mul { lhs, rhs, .. } |
            Instruction::F32Div { lhs, rhs, .. } |
            Instruction::F64Div { lhs, rhs, .. } |
            Instruction::F32Min { lhs, rhs, .. } |
            Instruction::F64Min { lhs, rhs, .. } |
            Instruction::F32Max { lhs, rhs, .. } |
            Instruction::F64Max { lhs, rhs, .. } |
            Instruction::F32Copysign { lhs, rhs, .. } |
            Instruction::F64Copysign { lhs, rhs, .. } => visit_registers!(f, lhs, rhs),
            Instruction::F32CopysignImm { lhs, .. } |
            Instruction::F64CopysignImm { lhs, .. } => f(lhs),
            Instruction::I32WrapI64 { input, .. } |
            Instruction::I32TruncF32S { input, .. } |
            Instruction::I32TruncF32U { input, .. } |
            Instruction::I32TruncF64S { input, .. } |
            Instruction::I32TruncF64U { input, .. } |
            Instruction::I64TruncF32S { input, .. } |
            Instruction::I64TruncF32U { input, .. } |
            Instruction::I64TruncF64S { input, .. } |
            Instruction::I64TruncF64U { input, .. } |
            Instruction::I32TruncSatF32S { input, .. } |
            Instruction::I32TruncSatF32U { input, .. } |
            Instruction::I32TruncSatF64S { input, .. } |
            Instruction::I32TruncSatF64U { input, .. } |
            Instruction::I64TruncSatF32S { input, .. } |
            Instruction::I64TruncSatF32U { input, .. } |
            Instruction::I64TruncSatF64S { input, .. } |
            Instruction::I64TruncSatF64U { input, .. } |
            Instruction::I32Extend8S { input, .. } |
            Instruction::I32Extend16S { input, .. } |
            Instruction::I64Extend8S { input, .. } |
            Instruction::I64Extend16S { input, .. } |
            Instruction::I64Extend32S { input, .. } |
            Instruction::F32DemoteF64 { input, .. } |
            Instruction::F64PromoteF32 { input, .. } |
            Instruction::F32ConvertI32S { input, .. } |
            Instruction::F32ConvertI32U { input, .. } |
            Instruction::F32ConvertI64S { input, .. } |
            Instruction::F32ConvertI64U { input, .. } |
            Instruction::F64ConvertI32S { input, .. } |
            Instruction::F64ConvertI32U { input, .. } |
            Instruction::F64ConvertI64S { input, .. } |
            Instruction::F64ConvertI64U { input, .. } => f(input),
        }
    }
}

impl<const N: usize> VisitInputRegisters for [Reg; N] {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Reg)) {
        for register in self {
            f(register);
        }
    }
}

impl VisitInputRegisters for RegSpan {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Reg)) {
        f(self.head_mut())
    }
}

impl VisitInputRegisters for RegSpanIter {
    fn visit_input_registers(&mut self, mut f: impl FnMut(&mut Reg)) {
        let len = self.len_as_u16();
        let mut span = self.span();
        f(span.head_mut());
        *self = span.iter_u16(len);
    }
}
