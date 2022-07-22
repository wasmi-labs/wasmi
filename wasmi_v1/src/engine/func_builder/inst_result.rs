use super::{IrInstruction, IrRegister};

impl IrInstruction {
    /// Returns the single result [`IrRegister`] of the instruction if any.
    ///
    /// # Note
    ///
    /// For instructions with potentially multiple result [`IrRegister`] they only
    /// return `Some` if they actually have a single result and `None` otherwise.
    pub fn result_mut(&mut self) -> Option<&mut IrRegister> {
        match self {
            Self::Br { .. }
            | Self::BrMulti { .. }
            | Self::BrEqz { .. }
            | Self::BrNez { .. }
            | Self::BrNezSingle { .. }
            | Self::BrNezMulti { .. }
            | Self::ReturnNez { .. }
            | Self::ReturnNezImm { .. }
            | Self::ReturnNezMulti { .. }
            | Self::BrTable { .. }
            | Self::Trap { .. }
            | Self::Return { .. }
            | Self::ReturnImm { .. }
            | Self::ReturnMulti { .. } => None,
            Self::Call { results, .. } | Self::CallIndirect { results, .. } => results.single_mut(),
            Self::Copy { result, .. } => Some(result),
            Self::CopyImm { result, .. } => Some(result),
            Self::CopyMany { results, .. } => results.single_mut(),
            Self::Select { result, .. } | Self::GlobalGet { result, .. } => Some(result),
            Self::GlobalSet { .. } => None,
            Self::I32Load { result, .. }
            | Self::I64Load { result, .. }
            | Self::F32Load { result, .. }
            | Self::F64Load { result, .. }
            | Self::I32Load8S { result, .. }
            | Self::I32Load8U { result, .. }
            | Self::I32Load16S { result, .. }
            | Self::I32Load16U { result, .. }
            | Self::I64Load8S { result, .. }
            | Self::I64Load8U { result, .. }
            | Self::I64Load16S { result, .. }
            | Self::I64Load16U { result, .. }
            | Self::I64Load32S { result, .. }
            | Self::I64Load32U { result, .. } => Some(result),
            Self::I32Store { .. }
            | Self::I32StoreImm { .. }
            | Self::I64Store { .. }
            | Self::I64StoreImm { .. }
            | Self::F32Store { .. }
            | Self::F32StoreImm { .. }
            | Self::F64Store { .. }
            | Self::F64StoreImm { .. }
            | Self::I32Store8 { .. }
            | Self::I32Store8Imm { .. }
            | Self::I32Store16 { .. }
            | Self::I32Store16Imm { .. }
            | Self::I64Store8 { .. }
            | Self::I64Store8Imm { .. }
            | Self::I64Store16 { .. }
            | Self::I64Store16Imm { .. }
            | Self::I64Store32 { .. }
            | Self::I64Store32Imm { .. } => None,
            Self::MemorySize { result, .. }
            | Self::MemoryGrow { result, .. }
            | Self::I32Eq { result, .. }
            | Self::I32EqImm { result, .. }
            | Self::I32Ne { result, .. }
            | Self::I32NeImm { result, .. }
            | Self::I32LtS { result, .. }
            | Self::I32LtSImm { result, .. }
            | Self::I32LtU { result, .. }
            | Self::I32LtUImm { result, .. }
            | Self::I32GtS { result, .. }
            | Self::I32GtSImm { result, .. }
            | Self::I32GtU { result, .. }
            | Self::I32GtUImm { result, .. }
            | Self::I32LeS { result, .. }
            | Self::I32LeSImm { result, .. }
            | Self::I32LeU { result, .. }
            | Self::I32LeUImm { result, .. }
            | Self::I32GeS { result, .. }
            | Self::I32GeSImm { result, .. }
            | Self::I32GeU { result, .. }
            | Self::I32GeUImm { result, .. }
            | Self::I64Eq { result, .. }
            | Self::I64EqImm { result, .. }
            | Self::I64Ne { result, .. }
            | Self::I64NeImm { result, .. }
            | Self::I64LtS { result, .. }
            | Self::I64LtSImm { result, .. }
            | Self::I64LtU { result, .. }
            | Self::I64LtUImm { result, .. }
            | Self::I64GtS { result, .. }
            | Self::I64GtSImm { result, .. }
            | Self::I64GtU { result, .. }
            | Self::I64GtUImm { result, .. }
            | Self::I64LeS { result, .. }
            | Self::I64LeSImm { result, .. }
            | Self::I64LeU { result, .. }
            | Self::I64LeUImm { result, .. }
            | Self::I64GeS { result, .. }
            | Self::I64GeSImm { result, .. }
            | Self::I64GeU { result, .. }
            | Self::I64GeUImm { result, .. }
            | Self::F32Eq { result, .. }
            | Self::F32EqImm { result, .. }
            | Self::F32Ne { result, .. }
            | Self::F32NeImm { result, .. }
            | Self::F32Lt { result, .. }
            | Self::F32LtImm { result, .. }
            | Self::F32Gt { result, .. }
            | Self::F32GtImm { result, .. }
            | Self::F32Le { result, .. }
            | Self::F32LeImm { result, .. }
            | Self::F32Ge { result, .. }
            | Self::F32GeImm { result, .. }
            | Self::F64Eq { result, .. }
            | Self::F64EqImm { result, .. }
            | Self::F64Ne { result, .. }
            | Self::F64NeImm { result, .. }
            | Self::F64Lt { result, .. }
            | Self::F64LtImm { result, .. }
            | Self::F64Gt { result, .. }
            | Self::F64GtImm { result, .. }
            | Self::F64Le { result, .. }
            | Self::F64LeImm { result, .. }
            | Self::F64Ge { result, .. }
            | Self::F64GeImm { result, .. }
            | Self::I32Clz { result, .. }
            | Self::I32Ctz { result, .. }
            | Self::I32Popcnt { result, .. }
            | Self::I32Add { result, .. }
            | Self::I32AddImm { result, .. }
            | Self::I32Sub { result, .. }
            | Self::I32SubImm { result, .. }
            | Self::I32Mul { result, .. }
            | Self::I32MulImm { result, .. }
            | Self::I32DivS { result, .. }
            | Self::I32DivSImm { result, .. }
            | Self::I32DivU { result, .. }
            | Self::I32DivUImm { result, .. }
            | Self::I32RemS { result, .. }
            | Self::I32RemSImm { result, .. }
            | Self::I32RemU { result, .. }
            | Self::I32RemUImm { result, .. }
            | Self::I32And { result, .. }
            | Self::I32AndImm { result, .. }
            | Self::I32Or { result, .. }
            | Self::I32OrImm { result, .. }
            | Self::I32Xor { result, .. }
            | Self::I32XorImm { result, .. }
            | Self::I32Shl { result, .. }
            | Self::I32ShlImm { result, .. }
            | Self::I32ShrS { result, .. }
            | Self::I32ShrSImm { result, .. }
            | Self::I32ShrU { result, .. }
            | Self::I32ShrUImm { result, .. }
            | Self::I32Rotl { result, .. }
            | Self::I32RotlImm { result, .. }
            | Self::I32Rotr { result, .. }
            | Self::I32RotrImm { result, .. }
            | Self::I64Clz { result, .. }
            | Self::I64Ctz { result, .. }
            | Self::I64Popcnt { result, .. }
            | Self::I64Add { result, .. }
            | Self::I64AddImm { result, .. }
            | Self::I64Sub { result, .. }
            | Self::I64SubImm { result, .. }
            | Self::I64Mul { result, .. }
            | Self::I64MulImm { result, .. }
            | Self::I64DivS { result, .. }
            | Self::I64DivSImm { result, .. }
            | Self::I64DivU { result, .. }
            | Self::I64DivUImm { result, .. }
            | Self::I64RemS { result, .. }
            | Self::I64RemSImm { result, .. }
            | Self::I64RemU { result, .. }
            | Self::I64RemUImm { result, .. }
            | Self::I64And { result, .. }
            | Self::I64AndImm { result, .. }
            | Self::I64Or { result, .. }
            | Self::I64OrImm { result, .. }
            | Self::I64Xor { result, .. }
            | Self::I64XorImm { result, .. }
            | Self::I64Shl { result, .. }
            | Self::I64ShlImm { result, .. }
            | Self::I64ShrS { result, .. }
            | Self::I64ShrSImm { result, .. }
            | Self::I64ShrU { result, .. }
            | Self::I64ShrUImm { result, .. }
            | Self::I64Rotl { result, .. }
            | Self::I64RotlImm { result, .. }
            | Self::I64Rotr { result, .. }
            | Self::I64RotrImm { result, .. }
            | Self::F32Abs { result, .. }
            | Self::F32Neg { result, .. }
            | Self::F32Ceil { result, .. }
            | Self::F32Floor { result, .. }
            | Self::F32Trunc { result, .. }
            | Self::F32Nearest { result, .. }
            | Self::F32Sqrt { result, .. }
            | Self::F32Add { result, .. }
            | Self::F32AddImm { result, .. }
            | Self::F32Sub { result, .. }
            | Self::F32SubImm { result, .. }
            | Self::F32Mul { result, .. }
            | Self::F32MulImm { result, .. }
            | Self::F32Div { result, .. }
            | Self::F32DivImm { result, .. }
            | Self::F32Min { result, .. }
            | Self::F32MinImm { result, .. }
            | Self::F32Max { result, .. }
            | Self::F32MaxImm { result, .. }
            | Self::F32Copysign { result, .. }
            | Self::F32CopysignImm { result, .. }
            | Self::F64Abs { result, .. }
            | Self::F64Neg { result, .. }
            | Self::F64Ceil { result, .. }
            | Self::F64Floor { result, .. }
            | Self::F64Trunc { result, .. }
            | Self::F64Nearest { result, .. }
            | Self::F64Sqrt { result, .. }
            | Self::F64Add { result, .. }
            | Self::F64AddImm { result, .. }
            | Self::F64Sub { result, .. }
            | Self::F64SubImm { result, .. }
            | Self::F64Mul { result, .. }
            | Self::F64MulImm { result, .. }
            | Self::F64Div { result, .. }
            | Self::F64DivImm { result, .. }
            | Self::F64Min { result, .. }
            | Self::F64MinImm { result, .. }
            | Self::F64Max { result, .. }
            | Self::F64MaxImm { result, .. }
            | Self::F64Copysign { result, .. }
            | Self::F64CopysignImm { result, .. }
            | Self::I32WrapI64 { result, .. }
            | Self::I32TruncSF32 { result, .. }
            | Self::I32TruncUF32 { result, .. }
            | Self::I32TruncSF64 { result, .. }
            | Self::I32TruncUF64 { result, .. }
            | Self::I64ExtendSI32 { result, .. }
            | Self::I64ExtendUI32 { result, .. }
            | Self::I64TruncSF32 { result, .. }
            | Self::I64TruncUF32 { result, .. }
            | Self::I64TruncSF64 { result, .. }
            | Self::I64TruncUF64 { result, .. }
            | Self::F32ConvertSI32 { result, .. }
            | Self::F32ConvertUI32 { result, .. }
            | Self::F32ConvertSI64 { result, .. }
            | Self::F32ConvertUI64 { result, .. }
            | Self::F32DemoteF64 { result, .. }
            | Self::F64ConvertSI32 { result, .. }
            | Self::F64ConvertUI32 { result, .. }
            | Self::F64ConvertSI64 { result, .. }
            | Self::F64ConvertUI64 { result, .. }
            | Self::F64PromoteF32 { result, .. }
            | Self::I32Extend8S { result, .. }
            | Self::I32Extend16S { result, .. }
            | Self::I64Extend8S { result, .. }
            | Self::I64Extend16S { result, .. }
            | Self::I64Extend32S { result, .. }
            | Self::I32TruncSatF32S { result, .. }
            | Self::I32TruncSatF32U { result, .. }
            | Self::I32TruncSatF64S { result, .. }
            | Self::I32TruncSatF64U { result, .. }
            | Self::I64TruncSatF32S { result, .. }
            | Self::I64TruncSatF32U { result, .. }
            | Self::I64TruncSatF64S { result, .. }
            | Self::I64TruncSatF64U { result, .. } => Some(result),
        }
    }
}
