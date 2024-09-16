use crate::{
    engine::{
        bytecode::{index, Instruction, Reg, RegSpan},
        EngineFunc,
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
        new_result: Reg,
        old_result: Reg,
    ) -> Result<bool, Error> {
        use Instruction as I;
        match self {
            I::TableIndex { .. } |
            I::DataIndex { .. } |
            I::ElemIndex { .. } |
            I::Const32 { .. } |
            I::I64Const32 { .. } |
            I::F64Const32 { .. } |
            I::RegisterAndImm32 { .. } |
            I::Register { .. } |
            I::Register2 { .. } |
            I::Register3 { .. } |
            I::RegisterSpan { .. } |
            I::RegisterList { .. } |
            I::BranchTableTarget { .. } |
            I::BranchTableTargetNonOverlapping { .. } |
            I::CallIndirectParams { .. } |
            I::CallIndirectParamsImm16 { .. } |

            I::Trap { .. } |
            I::ConsumeFuel { .. } |

            I::Return |
            I::ReturnReg { .. } |
            I::ReturnReg2 { .. } |
            I::ReturnReg3 { .. } |
            I::ReturnImm32 { .. } |
            I::ReturnI64Imm32 { .. } |
            I::ReturnF64Imm32 { .. } |
            I::ReturnSpan { .. } |
            I::ReturnMany { .. } |
            I::ReturnNez { .. } |
            I::ReturnNezReg { .. } |
            I::ReturnNezReg2 { .. } |
            I::ReturnNezImm32 { .. } |
            I::ReturnNezI64Imm32 { .. } |
            I::ReturnNezF64Imm32 { .. } |
            I::ReturnNezSpan { .. } |
            I::ReturnNezMany { .. } |

            I::Branch { .. } |
            I::BranchCmpFallback { .. } |
            I::BranchI32And { .. } |
            I::BranchI32AndImm { .. } |
            I::BranchI32Or { .. } |
            I::BranchI32OrImm { .. } |
            I::BranchI32Xor { .. } |
            I::BranchI32XorImm { .. } |
            I::BranchI32AndEqz { .. } |
            I::BranchI32AndEqzImm { .. } |
            I::BranchI32OrEqz { .. } |
            I::BranchI32OrEqzImm { .. } |
            I::BranchI32XorEqz { .. } |
            I::BranchI32XorEqzImm { .. } |
            I::BranchTable0 { .. } |
            I::BranchTable1 { .. } |
            I::BranchTable2 { .. } |
            I::BranchTable3 { .. } |
            I::BranchTableSpan { .. } |
            I::BranchTableMany { .. } |
            I::BranchI32Eq { .. } |
            I::BranchI32EqImm { .. } |
            I::BranchI32Ne { .. } |
            I::BranchI32NeImm { .. } |
            I::BranchI32LtS { .. } |
            I::BranchI32LtSImm { .. } |
            I::BranchI32LtU { .. } |
            I::BranchI32LtUImm { .. } |
            I::BranchI32LeS { .. } |
            I::BranchI32LeSImm { .. } |
            I::BranchI32LeU { .. } |
            I::BranchI32LeUImm { .. } |
            I::BranchI32GtS { .. } |
            I::BranchI32GtSImm { .. } |
            I::BranchI32GtU { .. } |
            I::BranchI32GtUImm { .. } |
            I::BranchI32GeS { .. } |
            I::BranchI32GeSImm { .. } |
            I::BranchI32GeU { .. } |
            I::BranchI32GeUImm { .. } |
            I::BranchI64Eq { .. } |
            I::BranchI64EqImm { .. } |
            I::BranchI64Ne { .. } |
            I::BranchI64NeImm { .. } |
            I::BranchI64LtS { .. } |
            I::BranchI64LtSImm { .. } |
            I::BranchI64LtU { .. } |
            I::BranchI64LtUImm { .. } |
            I::BranchI64LeS { .. } |
            I::BranchI64LeSImm { .. } |
            I::BranchI64LeU { .. } |
            I::BranchI64LeUImm { .. } |
            I::BranchI64GtS { .. } |
            I::BranchI64GtSImm { .. } |
            I::BranchI64GtU { .. } |
            I::BranchI64GtUImm { .. } |
            I::BranchI64GeS { .. } |
            I::BranchI64GeSImm { .. } |
            I::BranchI64GeU { .. } |
            I::BranchI64GeUImm { .. } |
            I::BranchF32Eq { .. } |
            I::BranchF32Ne { .. } |
            I::BranchF32Lt { .. } |
            I::BranchF32Le { .. } |
            I::BranchF32Gt { .. } |
            I::BranchF32Ge { .. } |
            I::BranchF64Eq { .. } |
            I::BranchF64Ne { .. } |
            I::BranchF64Lt { .. } |
            I::BranchF64Le { .. } |
            I::BranchF64Gt { .. } |
            I::BranchF64Ge { .. } => Ok(false),

            I::Copy { result, .. } |
            I::CopyImm32 { result, .. } |
            I::CopyI64Imm32 { result, .. } |
            I::CopyF64Imm32 { result, .. } => relink_simple(result, new_result, old_result),
            I::CopySpan { .. } |
            I::CopySpanNonOverlapping { .. } |
            I::Copy2 { .. } |
            I::CopyMany { .. } |
            I::CopyManyNonOverlapping { .. } |

            I::ReturnCallInternal0 { .. } |
            I::ReturnCallInternal { .. } |
            I::ReturnCallImported0 { .. } |
            I::ReturnCallImported { .. } |
            I::ReturnCallIndirect0 { .. } |
            I::ReturnCallIndirect0Imm16 { .. } |
            I::ReturnCallIndirect { .. } |
            I::ReturnCallIndirectImm16 { .. } => Ok(false),

            I::CallInternal0 { results, func } |
            I::CallInternal { results, func } => {
                relink_call_internal(results, *func, module, new_result, old_result)
            }
            I::CallImported0 { results, func } |
            I::CallImported { results, func } => {
                relink_call_imported(results, *func, module, new_result, old_result)
            }
            I::CallIndirect0 { results, func_type } |
            I::CallIndirect0Imm16 { results, func_type } |
            I::CallIndirect { results, func_type } |
            I::CallIndirectImm16 { results, func_type } => {
                relink_call_indirect(results, *func_type, module, new_result, old_result)
            }

            I::Select { result, .. } |
            I::SelectImm32Rhs { result, .. } |
            I::SelectImm32Lhs { result, .. } |
            I::SelectImm32 { result, .. } |
            I::SelectI64Imm32Rhs { result, .. } |
            I::SelectI64Imm32Lhs { result, .. } |
            I::SelectI64Imm32 { result, .. } |
            I::SelectF64Imm32Rhs { result, .. } |
            I::SelectF64Imm32Lhs { result, .. } |
            I::SelectF64Imm32 { result, .. } |

            I::RefFunc { result, .. } |
            I::TableGet { result, .. } |
            I::TableGetImm { result, .. } |
            I::TableSize { result, .. } => relink_simple(result, new_result, old_result),
            I::TableSet { .. } |
            I::TableSetAt { .. } |
            I::TableCopy { .. } |
            I::TableCopyTo { .. } |
            I::TableCopyFrom { .. } |
            I::TableCopyFromTo { .. } |
            I::TableCopyExact { .. } |
            I::TableCopyToExact { .. } |
            I::TableCopyFromExact { .. } |
            I::TableCopyFromToExact { .. } |
            I::TableInit { .. } |
            I::TableInitTo { .. } |
            I::TableInitFrom { .. } |
            I::TableInitFromTo { .. } |
            I::TableInitExact { .. } |
            I::TableInitToExact { .. } |
            I::TableInitFromExact { .. } |
            I::TableInitFromToExact { .. } |
            I::TableFill { .. } |
            I::TableFillAt { .. } |
            I::TableFillExact { .. } |
            I::TableFillAtExact { .. } => Ok(false),
            I::TableGrow { result, .. } |
            I::TableGrowImm { result, .. } => {
                relink_simple(result, new_result, old_result)
            }

            I::ElemDrop(_) |
            I::DataDrop(_) => Ok(false),

            I::MemorySize { result } |
            I::MemoryGrow { result, .. } |
            I::MemoryGrowBy { result, .. } => relink_simple(result, new_result, old_result),
            I::MemoryCopy { .. } |
            I::MemoryCopyTo { .. } |
            I::MemoryCopyFrom { .. } |
            I::MemoryCopyFromTo { .. } |
            I::MemoryCopyExact { .. } |
            I::MemoryCopyToExact { .. } |
            I::MemoryCopyFromExact { .. } |
            I::MemoryCopyFromToExact { .. } |
            I::MemoryFill { .. } |
            I::MemoryFillAt { .. } |
            I::MemoryFillImm { .. } |
            I::MemoryFillExact { .. } |
            I::MemoryFillAtImm { .. } |
            I::MemoryFillAtExact { .. } |
            I::MemoryFillImmExact { .. } |
            I::MemoryFillAtImmExact { .. } |
            I::MemoryInit { .. } |
            I::MemoryInitTo { .. } |
            I::MemoryInitFrom { .. } |
            I::MemoryInitFromTo { .. } |
            I::MemoryInitExact { .. } |
            I::MemoryInitToExact { .. } |
            I::MemoryInitFromExact { .. } |
            I::MemoryInitFromToExact { .. } => Ok(false),

            I::GlobalGet { result, .. } => relink_simple(result, new_result, old_result),
            I::GlobalSet { .. } |
            I::GlobalSetI32Imm16 { .. } |
            I::GlobalSetI64Imm16 { .. } => Ok(false),

            I::I32Load { result, .. } |
            I::I64Load { result, .. } |
            I::F32Load { result, .. } |
            I::F64Load { result, .. } |
            I::I32Load8s { result, .. } |
            I::I32Load8u { result, .. } |
            I::I32Load16s { result, .. } |
            I::I32Load16u { result, .. } |
            I::I64Load8s { result, .. } |
            I::I64Load8u { result, .. } |
            I::I64Load16s { result, .. } |
            I::I64Load16u { result, .. } |
            I::I64Load32s { result, .. } |
            I::I64Load32u { result, .. } |
            I::I32LoadAt { result, .. } |
            I::I64LoadAt { result, .. } |
            I::F32LoadAt { result, .. } |
            I::F64LoadAt { result, .. } |
            I::I32Load8sAt { result, .. } |
            I::I32Load8uAt { result, .. } |
            I::I32Load16sAt { result, .. } |
            I::I32Load16uAt { result, .. } |
            I::I64Load8sAt { result, .. } |
            I::I64Load8uAt { result, .. } |
            I::I64Load16sAt { result, .. } |
            I::I64Load16uAt { result, .. } |
            I::I64Load32sAt { result, .. } |
            I::I64Load32uAt { result, .. } |
            I::I32LoadOffset16 { result, .. } |
            I::I64LoadOffset16 { result, .. } |
            I::F32LoadOffset16 { result, .. } |
            I::F64LoadOffset16 { result, .. } |
            I::I32Load8sOffset16 { result, .. } |
            I::I32Load8uOffset16 { result, .. } |
            I::I32Load16sOffset16 { result, .. } |
            I::I32Load16uOffset16 { result, .. } |
            I::I64Load8sOffset16 { result, .. } |
            I::I64Load8uOffset16 { result, .. } |
            I::I64Load16sOffset16 { result, .. } |
            I::I64Load16uOffset16 { result, .. } |
            I::I64Load32sOffset16 { result, .. } |
            I::I64Load32uOffset16 { result, .. } => relink_simple(result, new_result, old_result),

            I::I32Store { .. } |
            I::I32StoreOffset16 { .. } |
            I::I32StoreOffset16Imm16 { .. } |
            I::I32StoreAt { .. } |
            I::I32StoreAtImm16 { .. } |
            I::I32Store8 { .. } |
            I::I32Store8Offset16 { .. } |
            I::I32Store8Offset16Imm { .. } |
            I::I32Store8At { .. } |
            I::I32Store8AtImm { .. } |
            I::I32Store16 { .. } |
            I::I32Store16Offset16 { .. } |
            I::I32Store16Offset16Imm { .. } |
            I::I32Store16At { .. } |
            I::I32Store16AtImm { .. } |
            I::I64Store { .. } |
            I::I64StoreOffset16 { .. } |
            I::I64StoreOffset16Imm16 { .. } |
            I::I64StoreAt { .. } |
            I::I64StoreAtImm16 { .. } |
            I::I64Store8 { .. } |
            I::I64Store8Offset16 { .. } |
            I::I64Store8Offset16Imm { .. } |
            I::I64Store8At { .. } |
            I::I64Store8AtImm { .. } |
            I::I64Store16 { .. } |
            I::I64Store16Offset16 { .. } |
            I::I64Store16Offset16Imm { .. } |
            I::I64Store16At { .. } |
            I::I64Store16AtImm { .. } |
            I::I64Store32 { .. } |
            I::I64Store32Offset16 { .. } |
            I::I64Store32Offset16Imm16 { .. } |
            I::I64Store32At { .. } |
            I::I64Store32AtImm16 { .. } |
            I::F32Store { .. } |
            I::F32StoreOffset16 { .. } |
            I::F32StoreAt { .. } |
            I::F64Store { .. } |
            I::F64StoreOffset16 { .. } |
            I::F64StoreAt { .. } => Ok(false),

            I::I32Eq { result, .. } |
            I::I64Eq { result, .. } |
            I::I32Ne { result, .. } |
            I::I64Ne { result, .. } |
            I::I32LtS { result, .. } |
            I::I64LtS { result, .. } |
            I::I32LtU { result, .. } |
            I::I64LtU { result, .. } |
            I::I32LeS { result, .. } |
            I::I64LeS { result, .. } |
            I::I32LeU { result, .. } |
            I::I64LeU { result, .. } |
            I::I32GtS { result, .. } |
            I::I64GtS { result, .. } |
            I::I32GtU { result, .. } |
            I::I64GtU { result, .. } |
            I::I32GeS { result, .. } |
            I::I64GeS { result, .. } |
            I::I32GeU { result, .. } |
            I::I64GeU { result, .. } |
            I::F32Eq { result, .. } |
            I::F32Ne { result, .. } |
            I::F32Lt { result, .. } |
            I::F32Le { result, .. } |
            I::F32Gt { result, .. } |
            I::F32Ge { result, .. } |
            I::F64Eq { result, .. } |
            I::F64Ne { result, .. } |
            I::F64Lt { result, .. } |
            I::F64Le { result, .. } |
            I::F64Gt { result, .. } |
            I::F64Ge { result, .. } |
            I::I32EqImm16 { result, .. } |
            I::I32NeImm16 { result, .. } |
            I::I32LtSImm16 { result, .. } |
            I::I32LeSImm16 { result, .. } |
            I::I32GtSImm16 { result, .. } |
            I::I32GeSImm16 { result, .. } |
            I::I32LtUImm16 { result, .. } |
            I::I32LeUImm16 { result, .. } |
            I::I32GtUImm16 { result, .. } |
            I::I32GeUImm16 { result, .. } |
            I::I64EqImm16 { result, .. } |
            I::I64NeImm16 { result, .. } |
            I::I64LtSImm16 { result, .. } |
            I::I64LeSImm16 { result, .. } |
            I::I64GtSImm16 { result, .. } |
            I::I64GeSImm16 { result, .. } |
            I::I64LtUImm16 { result, .. } |
            I::I64LeUImm16 { result, .. } |
            I::I64GtUImm16 { result, .. } |
            I::I64GeUImm16 { result, .. } |

            I::I32Clz { result, .. } |
            I::I32Ctz { result, .. } |
            I::I32Popcnt { result, .. } |
            I::I64Clz { result, .. } |
            I::I64Ctz { result, .. } |
            I::I64Popcnt { result, .. } |

            I::I32Add { result, ..} |
            I::I32Sub { result, ..} |
            I::I32Mul { result, ..} |
            I::I32DivS { result, ..} |
            I::I32DivU { result, ..} |
            I::I32RemS { result, ..} |
            I::I32RemU { result, ..} |
            I::I32And { result, ..} |
            I::I32AndEqz { result, ..} |
            I::I32Or { result, ..} |
            I::I32OrEqz { result, ..} |
            I::I32Xor { result, ..} |
            I::I32XorEqz { result, ..} |
            I::I32Shl { result, ..} |
            I::I32ShrS { result, ..} |
            I::I32ShrU { result, ..} |
            I::I32Rotl { result, ..} |
            I::I32Rotr { result, ..} |
            I::I64Add { result, ..} |
            I::I64Sub { result, ..} |
            I::I64Mul { result, ..} |
            I::I64DivS { result, ..} |
            I::I64DivU { result, ..} |
            I::I64RemS { result, ..} |
            I::I64RemU { result, ..} |
            I::I64And { result, ..} |
            I::I64Or { result, ..} |
            I::I64Xor { result, ..} |
            I::I64Shl { result, ..} |
            I::I64ShrS { result, ..} |
            I::I64ShrU { result, ..} |
            I::I64Rotl { result, ..} |
            I::I64Rotr { result, ..} |

            I::F32Abs { result, .. } |
            I::F32Neg { result, .. } |
            I::F32Ceil { result, .. } |
            I::F32Floor { result, .. } |
            I::F32Trunc { result, .. } |
            I::F32Nearest { result, .. } |
            I::F32Sqrt { result, .. } |
            I::F64Abs { result, .. } |
            I::F64Neg { result, .. } |
            I::F64Ceil { result, .. } |
            I::F64Floor { result, .. } |
            I::F64Trunc { result, .. } |
            I::F64Nearest { result, .. } |
            I::F64Sqrt { result, .. } |

            I::F32Add { result, .. } |
            I::F32Sub { result, .. } |
            I::F32Mul { result, .. } |
            I::F32Div { result, .. } |
            I::F32Min { result, .. } |
            I::F32Max { result, .. } |
            I::F32Copysign { result, .. } |
            I::F64Add { result, .. } |
            I::F64Sub { result, .. } |
            I::F64Mul { result, .. } |
            I::F64Div { result, .. } |
            I::F64Min { result, .. } |
            I::F64Max { result, .. } |
            I::F64Copysign { result, .. } |
            I::F32CopysignImm { result, .. } |
            I::F64CopysignImm { result, .. } |

            I::I32AddImm16 { result, .. } |
            I::I32SubImm16Rev { result, .. } |
            I::I32MulImm16 { result, .. } |
            I::I32DivSImm16 { result, .. } |
            I::I32DivSImm16Rev { result, .. } |
            I::I32RemSImm16 { result, .. } |
            I::I32RemSImm16Rev { result, .. } |
            I::I32AndEqzImm16 { result, .. } |
            I::I32AndImm16 { result, .. } |
            I::I32OrEqzImm16 { result, .. } |
            I::I32OrImm16 { result, .. } |
            I::I32XorEqzImm16 { result, .. } |
            I::I32XorImm16 { result, .. } |
            I::I32ShlImm { result, .. } |
            I::I32ShlImm16Rev { result, .. } |
            I::I32ShrSImm { result, .. } |
            I::I32ShrSImm16Rev { result, .. } |
            I::I32ShrUImm { result, .. } |
            I::I32ShrUImm16Rev { result, .. } |
            I::I32RotlImm { result, .. } |
            I::I32RotlImm16Rev { result, .. } |
            I::I32RotrImm { result, .. } |
            I::I32RotrImm16Rev { result, .. } |
            I::I32DivUImm16 { result, .. } |
            I::I32DivUImm16Rev { result, .. } |
            I::I32RemUImm16 { result, .. } |
            I::I32RemUImm16Rev { result, .. } |

            I::I64AddImm16 { result, .. } |
            I::I64SubImm16Rev { result, .. } |
            I::I64MulImm16 { result, .. } |
            I::I64DivSImm16 { result, .. } |
            I::I64DivSImm16Rev { result, .. } |
            I::I64RemSImm16 { result, .. } |
            I::I64RemSImm16Rev { result, .. } |
            I::I64AndImm16 { result, .. } |
            I::I64OrImm16 { result, .. } |
            I::I64XorImm16 { result, .. } |
            I::I64ShlImm { result, .. } |
            I::I64ShlImm16Rev { result, .. } |
            I::I64ShrSImm { result, .. } |
            I::I64ShrSImm16Rev { result, .. } |
            I::I64ShrUImm { result, .. } |
            I::I64ShrUImm16Rev { result, .. } |
            I::I64RotlImm { result, .. } |
            I::I64RotlImm16Rev { result, .. } |
            I::I64RotrImm { result, .. } |
            I::I64RotrImm16Rev { result, .. } |
            I::I64DivUImm16 { result, .. } |
            I::I64DivUImm16Rev { result, .. } |
            I::I64RemUImm16 { result, .. } |
            I::I64RemUImm16Rev { result, .. } |

            I::I32WrapI64 { result, .. } |
            I::I32TruncF32S { result, .. } |
            I::I32TruncF32U { result, .. } |
            I::I32TruncF64S { result, .. } |
            I::I32TruncF64U { result, .. } |
            I::I64TruncF32S { result, .. } |
            I::I64TruncF32U { result, .. } |
            I::I64TruncF64S { result, .. } |
            I::I64TruncF64U { result, .. } |
            I::I32TruncSatF32S { result, .. } |
            I::I32TruncSatF32U { result, .. } |
            I::I32TruncSatF64S { result, .. } |
            I::I32TruncSatF64U { result, .. } |
            I::I64TruncSatF32S { result, .. } |
            I::I64TruncSatF32U { result, .. } |
            I::I64TruncSatF64S { result, .. } |
            I::I64TruncSatF64U { result, .. } |
            I::I32Extend8S { result, .. } |
            I::I32Extend16S { result, .. } |
            I::I64Extend8S { result, .. } |
            I::I64Extend16S { result, .. } |
            I::I64Extend32S { result, .. } |
            I::F32DemoteF64 { result, .. } |
            I::F64PromoteF32 { result, .. } |
            I::F32ConvertI32S { result, .. } |
            I::F32ConvertI32U { result, .. } |
            I::F32ConvertI64S { result, .. } |
            I::F32ConvertI64U { result, .. } |
            I::F64ConvertI32S { result, .. } |
            I::F64ConvertI32U { result, .. } |
            I::F64ConvertI64S { result, .. } |
            I::F64ConvertI64U { result, .. } => relink_simple(result, new_result, old_result),
        }
    }
}

fn relink_simple(result: &mut Reg, new_result: Reg, old_result: Reg) -> Result<bool, Error> {
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
    results: &mut RegSpan,
    func: EngineFunc,
    module: &ModuleHeader,
    new_result: Reg,
    old_result: Reg,
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
    results: &mut RegSpan,
    func: index::Func,
    module: &ModuleHeader,
    new_result: Reg,
    old_result: Reg,
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
    results: &mut RegSpan,
    func_type: index::FuncType,
    module: &ModuleHeader,
    new_result: Reg,
    old_result: Reg,
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
