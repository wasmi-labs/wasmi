use super::{Instruction, Reg, RegSpan, RegSpanIter};

impl Instruction {
    /// Visit [`Reg`]s of `self` via the `visitor`.
    pub fn visit_regs<V: VisitRegs>(&mut self, visitor: &mut V) {
        HostVisitor::host_visitor(self, visitor)
    }
}

/// Implemented by [`Reg`] visitors to visit [`Reg`]s of an [`Instruction`] via [`Instruction::visit_regs`].
pub trait VisitRegs {
    /// Visits a [`Reg`] storing the result of an [`Instruction`].
    fn visit_result_reg(&mut self, reg: &mut Reg);
    /// Visits a [`RegSpan`] storing the results of an [`Instruction`].
    fn visit_result_regs(&mut self, reg: &mut RegSpan, len: Option<u16>);
    /// Visits a [`Reg`] storing an input of an [`Instruction`].
    fn visit_input_reg(&mut self, reg: &mut Reg);
    /// Visits a [`RegSpan`] storing inputs of an [`Instruction`].
    fn visit_input_regs(&mut self, regs: &mut RegSpan, len: Option<u16>);
}

/// Internal trait used to dispatch to a [`VisitRegs`] visitor.
trait HostVisitor {
    /// Host the [`VisitRegs`] visitor in the appropriate way.
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V);
}

impl HostVisitor for &'_ mut Reg {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_input_reg(self);
    }
}

impl<const N: usize> HostVisitor for &'_ mut [Reg; N] {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        for reg in self {
            visitor.visit_input_reg(reg);
        }
    }
}

impl HostVisitor for &'_ mut RegSpan {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_input_regs(self, None);
    }
}

impl HostVisitor for &'_ mut RegSpanIter {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        let len = self.len_as_u16();
        let mut span = self.span();
        visitor.visit_input_regs(&mut span, Some(len));
        *self = span.iter(len);
    }
}

/// Type-wrapper to signal that the wrapped [`Reg`], [`RegSpan`] (etc.) is a result.
pub struct Res<T>(T);

impl HostVisitor for Res<&'_ mut Reg> {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_reg(self.0);
    }
}

impl HostVisitor for Res<&'_ mut RegSpan> {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        visitor.visit_result_regs(self.0, None);
    }
}

macro_rules! host_visitor {
    ( $visitor:expr => $($r:expr),* $(,)? ) => {{
        $( HostVisitor::host_visitor($r, $visitor) );*
    }};
}

impl HostVisitor for &'_ mut Instruction {
    fn host_visitor<V: VisitRegs>(self, visitor: &mut V) {
        use Instruction as Instr;
        match self {
            Instr::Trap { .. } |
            Instr::ConsumeFuel { .. } |
            Instr::Return => {}
            Instr::ReturnReg { value } => host_visitor!(visitor => value),
            Instr::ReturnReg2 { values } => host_visitor!(visitor => values),
            Instr::ReturnReg3 { values } => host_visitor!(visitor => values),
            Instr::ReturnImm32 { .. } |
            Instr::ReturnI64Imm32 { .. } |
            Instr::ReturnF64Imm32 { .. } => {}
            Instr::ReturnSpan { values } => host_visitor!(visitor => values),
            Instr::ReturnMany { values } => host_visitor!(visitor => values),
            Instr::ReturnNez { condition } => host_visitor!(visitor => condition),
            Instr::ReturnNezReg { condition, value } => host_visitor!(visitor => condition, value),
            Instr::ReturnNezReg2 { condition, values } => host_visitor!(visitor => condition, values),
            Instr::ReturnNezImm32 { condition, .. } |
            Instr::ReturnNezI64Imm32 { condition, .. } |
            Instr::ReturnNezF64Imm32 { condition, .. } => host_visitor!(visitor => condition),
            Instr::ReturnNezSpan { condition, values } => host_visitor!(visitor => condition, values),
            Instr::ReturnNezMany { condition, values } => host_visitor!(visitor => condition, values),
            Instr::Branch { .. } => {}
            Instr::BranchCmpFallback { lhs, rhs, params } => host_visitor!(visitor => lhs, rhs, params),
            Instr::BranchI32And { lhs, rhs, .. } |
            Instr::BranchI32Or { lhs, rhs, .. } |
            Instr::BranchI32Xor { lhs, rhs, .. } |
            Instr::BranchI32AndEqz { lhs, rhs, .. } |
            Instr::BranchI32OrEqz { lhs, rhs, .. } |
            Instr::BranchI32XorEqz { lhs, rhs, .. } |
            Instr::BranchI32Eq { lhs, rhs, .. } |
            Instr::BranchI32Ne { lhs, rhs, .. } |
            Instr::BranchI32LtS { lhs, rhs, .. } |
            Instr::BranchI32LtU { lhs, rhs, .. } |
            Instr::BranchI32LeS { lhs, rhs, .. } |
            Instr::BranchI32LeU { lhs, rhs, .. } |
            Instr::BranchI32GtS { lhs, rhs, .. } |
            Instr::BranchI32GtU { lhs, rhs, .. } |
            Instr::BranchI32GeS { lhs, rhs, .. } |
            Instr::BranchI32GeU { lhs, rhs, .. } |
            Instr::BranchI64Eq { lhs, rhs, .. } |
            Instr::BranchI64Ne { lhs, rhs, .. } |
            Instr::BranchI64LtS { lhs, rhs, .. } |
            Instr::BranchI64LtU { lhs, rhs, .. } |
            Instr::BranchI64LeS { lhs, rhs, .. } |
            Instr::BranchI64LeU { lhs, rhs, .. } |
            Instr::BranchI64GtS { lhs, rhs, .. } |
            Instr::BranchI64GtU { lhs, rhs, .. } |
            Instr::BranchI64GeS { lhs, rhs, .. } |
            Instr::BranchI64GeU { lhs, rhs, .. } |
            Instr::BranchF32Eq { lhs, rhs, .. } |
            Instr::BranchF32Ne { lhs, rhs, .. } |
            Instr::BranchF32Lt { lhs, rhs, .. } |
            Instr::BranchF32Le { lhs, rhs, .. } |
            Instr::BranchF32Gt { lhs, rhs, .. } |
            Instr::BranchF32Ge { lhs, rhs, .. } |
            Instr::BranchF64Eq { lhs, rhs, .. } |
            Instr::BranchF64Ne { lhs, rhs, .. } |
            Instr::BranchF64Lt { lhs, rhs, .. } |
            Instr::BranchF64Le { lhs, rhs, .. } |
            Instr::BranchF64Gt { lhs, rhs, .. } |
            Instr::BranchF64Ge { lhs, rhs, .. } => host_visitor!(visitor => lhs, rhs),
            Instr::BranchI32AndImm { lhs, .. } |
            Instr::BranchI32OrImm { lhs, .. } |
            Instr::BranchI32XorImm { lhs, .. } |
            Instr::BranchI32AndEqzImm { lhs, .. } |
            Instr::BranchI32OrEqzImm { lhs, .. } |
            Instr::BranchI32XorEqzImm { lhs, .. } |
            Instr::BranchI32EqImm { lhs, .. } |
            Instr::BranchI32NeImm { lhs, .. } |
            Instr::BranchI32LtSImm { lhs, .. } |
            Instr::BranchI32LtUImm { lhs, .. } |
            Instr::BranchI32LeSImm { lhs, .. } |
            Instr::BranchI32LeUImm { lhs, .. } |
            Instr::BranchI32GtSImm { lhs, .. } |
            Instr::BranchI32GtUImm { lhs, .. } |
            Instr::BranchI32GeSImm { lhs, .. } |
            Instr::BranchI32GeUImm { lhs, .. } |
            Instr::BranchI64EqImm { lhs, .. } |
            Instr::BranchI64NeImm { lhs, .. } |
            Instr::BranchI64LtSImm { lhs, .. } |
            Instr::BranchI64LtUImm { lhs, .. } |
            Instr::BranchI64LeSImm { lhs, .. } |
            Instr::BranchI64LeUImm { lhs, .. } |
            Instr::BranchI64GtSImm { lhs, .. } |
            Instr::BranchI64GtUImm { lhs, .. } |
            Instr::BranchI64GeSImm { lhs, .. } |
            Instr::BranchI64GeUImm { lhs, .. } => host_visitor!(visitor => lhs),
            Instr::BranchTable0 { index, .. } |
            Instr::BranchTable1 { index, .. } |
            Instr::BranchTable2 { index, .. } |
            Instr::BranchTable3 { index, .. } |
            Instr::BranchTableSpan { index, .. } |
            Instr::BranchTableMany { index, .. } => host_visitor!(visitor => index),
            Instr::Copy { result, value } => host_visitor!(visitor => Res(result), value),
            Instr::Copy2 { results, values } => host_visitor!(visitor => Res(results), values),
            Instr::CopyImm32 { result, .. } |
            Instr::CopyI64Imm32 { result, .. } |
            Instr::CopyF64Imm32 { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::CopySpan { results, values, .. } |
            Instr::CopySpanNonOverlapping { results, values, .. } => host_visitor!(visitor => Res(results), values),
            Instr::CopyMany { results, values } |
            Instr::CopyManyNonOverlapping { results, values } => host_visitor!(visitor => Res(results), values),
            Instr::ReturnCallInternal0 { .. } |
            Instr::ReturnCallInternal { .. } |
            Instr::ReturnCallImported0 { .. } |
            Instr::ReturnCallImported { .. } |
            Instr::ReturnCallIndirect0 { .. } |
            Instr::ReturnCallIndirect0Imm16 { .. } |
            Instr::ReturnCallIndirect { .. } |
            Instr::ReturnCallIndirectImm16 { .. } => {}
            Instr::CallInternal0 { results, .. } |
            Instr::CallInternal { results, .. } |
            Instr::CallImported0 { results, .. } |
            Instr::CallImported { results, .. } |
            Instr::CallIndirect0 { results, .. } |
            Instr::CallIndirect0Imm16 { results, .. } |
            Instr::CallIndirect { results, .. } |
            Instr::CallIndirectImm16 { results, .. } => host_visitor!(visitor => Res(results)),
            Instr::Select { result, lhs } |
            Instr::SelectImm32Rhs { result, lhs } => host_visitor!(visitor => Res(result), lhs),
            Instr::SelectImm32Lhs { result, .. } |
            Instr::SelectImm32 { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::SelectI64Imm32Rhs { result, lhs } => host_visitor!(visitor => Res(result), lhs),
            Instr::SelectI64Imm32Lhs { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::SelectI64Imm32 { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::SelectF64Imm32Rhs { result, lhs } => host_visitor!(visitor => Res(result), lhs),
            Instr::SelectF64Imm32Lhs { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::SelectF64Imm32 { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::RefFunc { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::GlobalGet { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::GlobalSet { input, .. } => host_visitor!(visitor => input),
            Instr::GlobalSetI32Imm16 { .. } |
            Instr::GlobalSetI64Imm16 { .. } => {}
            Instr::I32Load { result, ptr } |
            Instr::I64Load { result, ptr } |
            Instr::F32Load { result, ptr } |
            Instr::F64Load { result, ptr } |
            Instr::I32Load8s { result, ptr } |
            Instr::I32Load8u { result, ptr } |
            Instr::I32Load16s { result, ptr } |
            Instr::I32Load16u { result, ptr } |
            Instr::I64Load8s { result, ptr } |
            Instr::I64Load8u { result, ptr } |
            Instr::I64Load16s { result, ptr } |
            Instr::I64Load16u { result, ptr } |
            Instr::I64Load32s { result, ptr } |
            Instr::I64Load32u { result, ptr } => host_visitor!(visitor => Res(result), ptr),
            Instr::I32LoadAt { result, .. } |
            Instr::I64LoadAt { result, .. } |
            Instr::F32LoadAt { result, .. } |
            Instr::F64LoadAt { result, .. } |
            Instr::I32Load8sAt { result, .. } |
            Instr::I32Load8uAt { result, .. } |
            Instr::I32Load16sAt { result, .. } |
            Instr::I32Load16uAt { result, .. } |
            Instr::I64Load8sAt { result, .. } |
            Instr::I64Load8uAt { result, .. } |
            Instr::I64Load16sAt { result, .. } |
            Instr::I64Load16uAt { result, .. } |
            Instr::I64Load32sAt { result, .. } |
            Instr::I64Load32uAt { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::I32LoadOffset16 { result, ptr, .. } |
            Instr::I64LoadOffset16 { result, ptr, .. } |
            Instr::F32LoadOffset16 { result, ptr, .. } |
            Instr::F64LoadOffset16 { result, ptr, .. } |
            Instr::I32Load8sOffset16 { result, ptr, .. } |
            Instr::I32Load8uOffset16 { result, ptr, .. } |
            Instr::I32Load16sOffset16 { result, ptr, .. } |
            Instr::I32Load16uOffset16 { result, ptr, .. } |
            Instr::I64Load8sOffset16 { result, ptr, .. } |
            Instr::I64Load8uOffset16 { result, ptr, .. } |
            Instr::I64Load16sOffset16 { result, ptr, .. } |
            Instr::I64Load16uOffset16 { result, ptr, .. } |
            Instr::I64Load32sOffset16 { result, ptr, .. } |
            Instr::I64Load32uOffset16 { result, ptr, .. } => host_visitor!(visitor => Res(result), ptr),
            Instr::I32Store { ptr, .. } |
            Instr::I32Store8 { ptr, .. } |
            Instr::I32Store16 { ptr, .. } |
            Instr::I64Store { ptr, .. } |
            Instr::I64Store8 { ptr, .. } |
            Instr::I64Store16 { ptr, .. } |
            Instr::I64Store32 { ptr, .. } |
            Instr::F32Store { ptr, .. } |
            Instr::F64Store { ptr, .. } => host_visitor!(visitor => ptr),
            Instr::I32StoreOffset16 { ptr, value, .. } |
            Instr::I32Store8Offset16 { ptr, value, .. } |
            Instr::I32Store16Offset16 { ptr, value, .. } |
            Instr::I64StoreOffset16 { ptr, value, .. } |
            Instr::I64Store8Offset16 { ptr, value, .. } |
            Instr::I64Store16Offset16 { ptr, value, .. } |
            Instr::I64Store32Offset16 { ptr, value, .. } |
            Instr::F32StoreOffset16 { ptr, value, .. } |
            Instr::F64StoreOffset16 { ptr, value, .. } => host_visitor!(visitor => ptr, value),
            Instr::I32StoreAt { value, .. } |
            Instr::I32Store8At { value, .. } |
            Instr::I32Store16At { value, .. } |
            Instr::I64StoreAt { value, .. } |
            Instr::I64Store8At { value, .. } |
            Instr::I64Store16At { value, .. } |
            Instr::I64Store32At { value, .. } |
            Instr::F32StoreAt { value, .. } |
            Instr::F64StoreAt { value, .. } => host_visitor!(visitor => value),
            Instr::I32StoreOffset16Imm16 { ptr, .. } |
            Instr::I32Store8Offset16Imm { ptr, .. } |
            Instr::I32Store16Offset16Imm { ptr, .. } |
            Instr::I64StoreOffset16Imm16 { ptr, .. } |
            Instr::I64Store8Offset16Imm { ptr, .. } |
            Instr::I64Store16Offset16Imm { ptr, .. } |
            Instr::I64Store32Offset16Imm16 { ptr, .. } => host_visitor!(visitor => ptr),
            Instr::I32StoreAtImm16 { .. } |
            Instr::I32Store8AtImm { .. } |
            Instr::I32Store16AtImm { .. } |
            Instr::I64StoreAtImm16 { .. } |
            Instr::I64Store8AtImm { .. } |
            Instr::I64Store16AtImm { .. } |
            Instr::I64Store32AtImm16 { .. } => {}
            Instr::I32Eq { result, lhs, rhs } |
            Instr::I32Ne { result, lhs, rhs } |
            Instr::I32LtS { result, lhs, rhs } |
            Instr::I32LtU { result, lhs, rhs } |
            Instr::I32GtS { result, lhs, rhs } |
            Instr::I32GtU { result, lhs, rhs } |
            Instr::I32LeS { result, lhs, rhs } |
            Instr::I32LeU { result, lhs, rhs } |
            Instr::I32GeS { result, lhs, rhs } |
            Instr::I32GeU { result, lhs, rhs } |
            Instr::I64Eq { result, lhs, rhs } |
            Instr::I64Ne { result, lhs, rhs } |
            Instr::I64LtS { result, lhs, rhs } |
            Instr::I64LtU { result, lhs, rhs } |
            Instr::I64GtS { result, lhs, rhs } |
            Instr::I64GtU { result, lhs, rhs } |
            Instr::I64LeS { result, lhs, rhs } |
            Instr::I64LeU { result, lhs, rhs } |
            Instr::I64GeS { result, lhs, rhs } |
            Instr::I64GeU { result, lhs, rhs } |
            Instr::F32Eq { result, lhs, rhs } |
            Instr::F32Ne { result, lhs, rhs } |
            Instr::F32Lt { result, lhs, rhs } |
            Instr::F32Le { result, lhs, rhs } |
            Instr::F32Gt { result, lhs, rhs } |
            Instr::F32Ge { result, lhs, rhs } |
            Instr::F64Eq { result, lhs, rhs } |
            Instr::F64Ne { result, lhs, rhs } |
            Instr::F64Lt { result, lhs, rhs } |
            Instr::F64Le { result, lhs, rhs } |
            Instr::F64Gt { result, lhs, rhs } |
            Instr::F64Ge { result, lhs, rhs } => host_visitor!(visitor => Res(result), lhs, rhs),
            Instr::I32EqImm16 { result, lhs, .. } |
            Instr::I32NeImm16 { result, lhs, .. } |
            Instr::I32LtSImm16 { result, lhs, .. } |
            Instr::I32LtUImm16 { result, lhs, .. } |
            Instr::I32GtSImm16 { result, lhs, .. } |
            Instr::I32GtUImm16 { result, lhs, .. } |
            Instr::I32LeSImm16 { result, lhs, .. } |
            Instr::I32LeUImm16 { result, lhs, .. } |
            Instr::I32GeSImm16 { result, lhs, .. } |
            Instr::I32GeUImm16 { result, lhs, .. } |
            Instr::I64EqImm16 { result, lhs, .. } |
            Instr::I64NeImm16 { result, lhs, .. } |
            Instr::I64LtSImm16 { result, lhs, .. } |
            Instr::I64LtUImm16 { result, lhs, .. } |
            Instr::I64GtSImm16 { result, lhs, .. } |
            Instr::I64GtUImm16 { result, lhs, .. } |
            Instr::I64LeSImm16 { result, lhs, .. } |
            Instr::I64LeUImm16 { result, lhs, .. } |
            Instr::I64GeSImm16 { result, lhs, .. } |
            Instr::I64GeUImm16 { result, lhs, .. } => host_visitor!(visitor => Res(result), lhs),
            Instr::I32Clz { result, input } |
            Instr::I32Ctz { result, input } |
            Instr::I32Popcnt { result, input } => host_visitor!(visitor => Res(result), input),
            Instr::I32Add { result, lhs, rhs } |
            Instr::I32Sub { result, lhs, rhs } |
            Instr::I32Mul { result, lhs, rhs } |
            Instr::I32DivS { result, lhs, rhs } |
            Instr::I32DivU { result, lhs, rhs } |
            Instr::I32RemS { result, lhs, rhs } |
            Instr::I32RemU { result, lhs, rhs } |
            Instr::I32And { result, lhs, rhs } |
            Instr::I32AndEqz { result, lhs, rhs } |
            Instr::I32Or { result, lhs, rhs } |
            Instr::I32OrEqz { result, lhs, rhs } |
            Instr::I32Xor { result, lhs, rhs } |
            Instr::I32XorEqz { result, lhs, rhs } |
            Instr::I32Shl { result, lhs, rhs } |
            Instr::I32ShrU { result, lhs, rhs } |
            Instr::I32ShrS { result, lhs, rhs } |
            Instr::I32Rotl { result, lhs, rhs } |
            Instr::I32Rotr { result, lhs, rhs } => host_visitor!(visitor => Res(result), lhs, rhs),
            Instr::I32AddImm16 { result, lhs, .. } |
            Instr::I32MulImm16 { result, lhs, .. } |
            Instr::I32DivSImm16 { result, lhs, .. } |
            Instr::I32DivUImm16 { result, lhs, .. } |
            Instr::I32RemSImm16 { result, lhs, .. } |
            Instr::I32RemUImm16 { result, lhs, .. } |
            Instr::I32AndEqzImm16 { result, lhs, .. } |
            Instr::I32AndImm16 { result, lhs, .. } |
            Instr::I32OrEqzImm16 { result, lhs, .. } |
            Instr::I32OrImm16 { result, lhs, .. } |
            Instr::I32XorEqzImm16 { result, lhs, .. } |
            Instr::I32XorImm16 { result, lhs, .. } |
            Instr::I32ShlImm { result, lhs, .. } |
            Instr::I32ShrUImm { result, lhs, .. } |
            Instr::I32ShrSImm { result, lhs, .. } |
            Instr::I32RotlImm { result, lhs, .. } |
            Instr::I32RotrImm { result, lhs, .. } => host_visitor!(visitor => Res(result), lhs),
            Instr::I32SubImm16Rev { result, rhs, .. } |
            Instr::I32DivSImm16Rev { result, rhs, .. } |
            Instr::I32DivUImm16Rev { result, rhs, .. } |
            Instr::I32RemSImm16Rev { result, rhs, .. } |
            Instr::I32RemUImm16Rev { result, rhs, .. } |
            Instr::I32ShlImm16Rev { result, rhs, .. } |
            Instr::I32ShrUImm16Rev { result, rhs, .. } |
            Instr::I32ShrSImm16Rev { result, rhs, .. } |
            Instr::I32RotlImm16Rev { result, rhs, .. } |
            Instr::I32RotrImm16Rev { result, rhs, .. } => host_visitor!(visitor => Res(result), rhs),
            Instr::I64Clz { result, input } |
            Instr::I64Ctz { result, input } |
            Instr::I64Popcnt { result, input } => host_visitor!(visitor => Res(result), input),
            Instr::I64Add { result, lhs, rhs } |
            Instr::I64Sub { result, lhs, rhs } |
            Instr::I64Mul { result, lhs, rhs } |
            Instr::I64DivS { result, lhs, rhs } |
            Instr::I64DivU { result, lhs, rhs } |
            Instr::I64RemS { result, lhs, rhs } |
            Instr::I64RemU { result, lhs, rhs } |
            Instr::I64And { result, lhs, rhs } |
            Instr::I64Or { result, lhs, rhs } |
            Instr::I64Xor { result, lhs, rhs } |
            Instr::I64Shl { result, lhs, rhs } |
            Instr::I64ShrU { result, lhs, rhs } |
            Instr::I64ShrS { result, lhs, rhs } |
            Instr::I64Rotl { result, lhs, rhs } |
            Instr::I64Rotr { result, lhs, rhs } => host_visitor!(visitor => Res(result), lhs, rhs),
            Instr::I64AddImm16 { result, lhs, .. } |
            Instr::I64MulImm16 { result, lhs, .. } |
            Instr::I64DivSImm16 { result, lhs, .. } |
            Instr::I64DivUImm16 { result, lhs, .. } |
            Instr::I64RemSImm16 { result, lhs, .. } |
            Instr::I64RemUImm16 { result, lhs, .. } |
            Instr::I64AndImm16 { result, lhs, .. } |
            Instr::I64OrImm16 { result, lhs, .. } |
            Instr::I64XorImm16 { result, lhs, .. } |
            Instr::I64ShlImm { result, lhs, .. } |
            Instr::I64ShrUImm { result, lhs, .. } |
            Instr::I64ShrSImm { result, lhs, .. } |
            Instr::I64RotlImm { result, lhs, .. } |
            Instr::I64RotrImm { result, lhs, .. } => host_visitor!(visitor => Res(result), lhs),
            Instr::I64SubImm16Rev { result, rhs, .. } |
            Instr::I64DivSImm16Rev { result, rhs, .. } |
            Instr::I64DivUImm16Rev { result, rhs, .. } |
            Instr::I64RemSImm16Rev { result, rhs, .. } |
            Instr::I64RemUImm16Rev { result, rhs, .. } |
            Instr::I64ShlImm16Rev { result, rhs, .. } |
            Instr::I64ShrUImm16Rev { result, rhs, .. } |
            Instr::I64ShrSImm16Rev { result, rhs, .. } |
            Instr::I64RotlImm16Rev { result, rhs, .. } |
            Instr::I64RotrImm16Rev { result, rhs, .. } => host_visitor!(visitor => Res(result), rhs),
            Instr::I32WrapI64 { result, input } |
            Instr::I32Extend8S { result, input } |
            Instr::I32Extend16S { result, input } |
            Instr::I64Extend8S { result, input } |
            Instr::I64Extend16S { result, input } |
            Instr::I64Extend32S { result, input } |
            Instr::F32Abs { result, input } |
            Instr::F32Neg { result, input } |
            Instr::F32Ceil { result, input } |
            Instr::F32Floor { result, input } |
            Instr::F32Trunc { result, input } |
            Instr::F32Nearest { result, input } |
            Instr::F32Sqrt { result, input } => host_visitor!(visitor => Res(result), input),
            Instr::F32Add { result, lhs, rhs } |
            Instr::F32Sub { result, lhs, rhs } |
            Instr::F32Mul { result, lhs, rhs } |
            Instr::F32Div { result, lhs, rhs } |
            Instr::F32Min { result, lhs, rhs } |
            Instr::F32Max { result, lhs, rhs } |
            Instr::F32Copysign { result, lhs, rhs } => host_visitor!(visitor => Res(result), lhs, rhs),
            Instr::F32CopysignImm { result, lhs, .. } => host_visitor!(visitor => Res(result), lhs),
            Instr::F64Abs { result, input } |
            Instr::F64Neg { result, input } |
            Instr::F64Ceil { result, input } |
            Instr::F64Floor { result, input } |
            Instr::F64Trunc { result, input } |
            Instr::F64Nearest { result, input } |
            Instr::F64Sqrt { result, input } => host_visitor!(visitor => Res(result), input),
            Instr::F64Add { result, lhs, rhs } |
            Instr::F64Sub { result, lhs, rhs } |
            Instr::F64Mul { result, lhs, rhs } |
            Instr::F64Div { result, lhs, rhs } |
            Instr::F64Min { result, lhs, rhs } |
            Instr::F64Max { result, lhs, rhs } |
            Instr::F64Copysign { result, lhs, rhs } => host_visitor!(visitor => Res(result), lhs, rhs),
            Instr::F64CopysignImm { result, lhs, .. } => host_visitor!(visitor => Res(result), lhs),
            Instr::I32TruncF32S { result, input } |
            Instr::I32TruncF32U { result, input } |
            Instr::I32TruncF64S { result, input } |
            Instr::I32TruncF64U { result, input } |
            Instr::I64TruncF32S { result, input } |
            Instr::I64TruncF32U { result, input } |
            Instr::I64TruncF64S { result, input } |
            Instr::I64TruncF64U { result, input } |
            Instr::I32TruncSatF32S { result, input } |
            Instr::I32TruncSatF32U { result, input } |
            Instr::I32TruncSatF64S { result, input } |
            Instr::I32TruncSatF64U { result, input } |
            Instr::I64TruncSatF32S { result, input } |
            Instr::I64TruncSatF32U { result, input } |
            Instr::I64TruncSatF64S { result, input } |
            Instr::I64TruncSatF64U { result, input } |
            Instr::F32DemoteF64 { result, input } |
            Instr::F64PromoteF32 { result, input } |
            Instr::F32ConvertI32S { result, input } |
            Instr::F32ConvertI32U { result, input } |
            Instr::F32ConvertI64S { result, input } |
            Instr::F32ConvertI64U { result, input } |
            Instr::F64ConvertI32S { result, input } |
            Instr::F64ConvertI32U { result, input } |
            Instr::F64ConvertI64S { result, input } |
            Instr::F64ConvertI64U { result, input } => host_visitor!(visitor => Res(result), input),
            Instr::TableGet { result, index } => host_visitor!(visitor => Res(result), index),
            Instr::TableGetImm { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::TableSize { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::TableSet { index, value } => host_visitor!(visitor => index, value),
            Instr::TableSetAt { value, .. } => host_visitor!(visitor => value),
            Instr::TableCopy { dst, src, len } => host_visitor!(visitor => dst, src, len),
            Instr::TableCopyTo { src, len, .. } => host_visitor!(visitor => src, len),
            Instr::TableCopyFrom { dst, len, .. } => host_visitor!(visitor => dst, len),
            Instr::TableCopyFromTo { len, .. } => host_visitor!(visitor => len),
            Instr::TableCopyExact { dst, src, .. } => host_visitor!(visitor => dst, src),
            Instr::TableCopyToExact { src, .. } => host_visitor!(visitor => src),
            Instr::TableCopyFromExact { dst, .. } => host_visitor!(visitor => dst),
            Instr::TableCopyFromToExact { .. } => {}
            Instr::TableInit { dst, src, len } => host_visitor!(visitor => dst, src, len),
            Instr::TableInitTo { src, len, .. } => host_visitor!(visitor => src, len),
            Instr::TableInitFrom { dst, len, .. } => host_visitor!(visitor => dst, len),
            Instr::TableInitFromTo { len, .. } => host_visitor!(visitor => len),
            Instr::TableInitExact { dst, src, .. } => host_visitor!(visitor => dst, src),
            Instr::TableInitToExact { src, .. } => host_visitor!(visitor => src),
            Instr::TableInitFromExact { dst, .. } => host_visitor!(visitor => dst),
            Instr::TableInitFromToExact { .. } => {}
            Instr::TableFill { dst, len, value } => host_visitor!(visitor => dst, len, value),
            Instr::TableFillAt { len, value, .. } => host_visitor!(visitor => len, value),
            Instr::TableFillExact { dst, value, .. } => host_visitor!(visitor => dst, value),
            Instr::TableFillAtExact { value, .. } => host_visitor!(visitor => value),
            Instr::TableGrow { result, delta, value } => host_visitor!(visitor => Res(result), delta, value),
            Instr::TableGrowImm { result, value, .. } => host_visitor!(visitor => Res(result), value),
            Instr::ElemDrop(_) |
            Instr::DataDrop(_) => {}
            Instr::MemorySize { result } => host_visitor!(visitor => Res(result)),
            Instr::MemoryGrow { result, delta } => host_visitor!(visitor => Res(result), delta),
            Instr::MemoryGrowBy { result, .. } => host_visitor!(visitor => Res(result)),
            Instr::MemoryCopy { dst, src, len } => host_visitor!(visitor => dst, src, len),
            Instr::MemoryCopyTo { src, len, .. } => host_visitor!(visitor => src, len),
            Instr::MemoryCopyFrom { dst, len, .. } => host_visitor!(visitor => dst, len),
            Instr::MemoryCopyFromTo { len, .. } => host_visitor!(visitor => len),
            Instr::MemoryCopyExact { dst, src, .. } => host_visitor!(visitor => dst, src),
            Instr::MemoryCopyToExact { src, .. } => host_visitor!(visitor => src),
            Instr::MemoryCopyFromExact { dst, .. } => host_visitor!(visitor => dst),
            Instr::MemoryCopyFromToExact { .. } => {}
            Instr::MemoryFill { dst, value, len } => host_visitor!(visitor => dst, value, len),
            Instr::MemoryFillAt { value, len, .. } => host_visitor!(visitor => value, len),
            Instr::MemoryFillImm { dst, len, .. } => host_visitor!(visitor => dst, len),
            Instr::MemoryFillExact { dst, value, .. } => host_visitor!(visitor => dst, value),
            Instr::MemoryFillAtImm { len, .. } => host_visitor!(visitor => len),
            Instr::MemoryFillAtExact { value, .. } => host_visitor!(visitor => value),
            Instr::MemoryFillImmExact { dst, .. } => host_visitor!(visitor => dst),
            Instr::MemoryFillAtImmExact { .. } => {}
            Instr::MemoryInit { dst, src, len } => host_visitor!(visitor => dst, src, len),
            Instr::MemoryInitTo { src, len, .. } => host_visitor!(visitor => src, len),
            Instr::MemoryInitFrom { dst, len, .. } => host_visitor!(visitor => dst, len),
            Instr::MemoryInitFromTo { len, .. } => host_visitor!(visitor => len),
            Instr::MemoryInitExact { dst, src, .. } => host_visitor!(visitor => dst, src),
            Instr::MemoryInitToExact { src, .. } => host_visitor!(visitor => src),
            Instr::MemoryInitFromExact { dst, .. } => host_visitor!(visitor => dst),
            Instr::MemoryInitFromToExact { .. } => {}
            Instr::TableIndex { .. } |
            Instr::DataIndex { .. } |
            Instr::ElemIndex { .. } |
            Instr::Const32 { .. } |
            Instr::I64Const32 { .. } |
            Instr::F64Const32 { .. } => {}
            Instr::BranchTableTarget { results, .. } |
            Instr::BranchTableTargetNonOverlapping { results, .. } => host_visitor!(visitor => results),
            Instr::RegisterAndImm32 { reg, .. } => host_visitor!(visitor => reg),
            Instr::RegisterSpan { span } => host_visitor!(visitor => span),
            Instr::Register { reg } => host_visitor!(visitor => reg),
            Instr::Register2 { regs } => host_visitor!(visitor => regs),
            Instr::Register3 { regs } => host_visitor!(visitor => regs),
            Instr::RegisterList { regs } => host_visitor!(visitor => regs),
            Instr::CallIndirectParams { index, .. } => host_visitor!(visitor => index),
            Instr::CallIndirectParamsImm16 { .. } => {}
        }
    }
}
