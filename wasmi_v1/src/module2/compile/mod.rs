pub use self::block_type::BlockType;
use super::{utils::value_type_from_wasmparser, FuncIdx, ModuleResources};
use crate::{engine::FunctionBuilder, Engine, ModuleError};
use wasmparser::{FuncValidator, FunctionBody, Operator, TypeOrFuncType, ValidatorResources};

mod block_type;

/// Translates the Wasm bytecode into `wasmi` bytecode.
///
/// # Note
///
/// - Uses the given `engine` as target for the translation.
/// - Uses the given `parser` and `validator` for parsing and validation of
///   the incoming Wasm bytecode stream.
/// - Uses the given module resources `res` as shared immutable data of the
///   already parsed and validated module parts required for the translation.
///
/// # Errors
///
/// If the function body fails to validate.
pub fn translate<'parser>(
    engine: &Engine,
    func: FuncIdx,
    func_body: FunctionBody<'parser>,
    validator: FuncValidator<ValidatorResources>,
    res: ModuleResources<'parser>,
) -> Result<(), ModuleError> {
    FunctionTranslator::new(engine, func, func_body, validator, res).translate()
}

/// Translates Wasm bytecode into `wasmi` bytecode for a single Wasm function.
struct FunctionTranslator<'engine, 'parser> {
    /// The target `wasmi` engine for `wasmi` bytecode translation.
    engine: &'engine Engine,
    /// The index of the translated function.
    func: FuncIdx,
    /// The function body that shall be translated.
    func_body: FunctionBody<'parser>,
    /// The interface to incrementally build up the `wasmi` bytecode function.
    func_builder: FunctionBuilder<'engine, 'parser>,
    /// The Wasm validator.
    validator: FuncValidator<ValidatorResources>,
    /// The `wasmi` module resources.
    ///
    /// Provides immutable information about the translated Wasm module
    /// required for function translation to `wasmi` bytecode.
    res: ModuleResources<'parser>,
}

impl<'engine, 'parser> FunctionTranslator<'engine, 'parser> {
    /// Creates a new Wasm to `wasmi` bytecode function translator.
    fn new(
        engine: &'engine Engine,
        func: FuncIdx,
        func_body: FunctionBody<'parser>,
        validator: FuncValidator<ValidatorResources>,
        res: ModuleResources<'parser>,
    ) -> Self {
        let func_builder = FunctionBuilder::new(engine, func, res);
        Self {
            engine,
            func,
            func_body,
            func_builder,
            validator,
            res,
        }
    }

    /// Starts translation of the Wasm stream into `wasmi` bytecode.
    fn translate(&mut self) -> Result<(), ModuleError> {
        self.translate_locals()?;
        self.translate_operators()?;
        Ok(())
    }

    /// Translates local variables of the Wasm function.
    fn translate_locals(&mut self) -> Result<(), ModuleError> {
        let mut reader = self.func_body.get_locals_reader()?;
        let len_locals = reader.get_count();
        for _ in 0..len_locals {
            let offset = reader.original_position();
            let (amount, value_type) = reader.read()?;
            self.validator.define_locals(offset, amount, value_type)?;
            let value_type = value_type_from_wasmparser(&value_type)?;
            self.func_builder.translate_locals(amount, value_type)?;
        }
        Ok(())
    }

    /// Translates the Wasm operators of the Wasm function.
    fn translate_operators(&mut self) -> Result<(), ModuleError> {
        let mut reader = self.func_body.get_operators_reader()?;
        while !reader.eof() {
            let (operator, offset) = reader.read_with_offset()?;
            self.validator.op(offset, &operator)?;
            self.translate_operator(operator)?;
        }
        reader.ensure_end()?;
        Ok(())
    }

    /// Translate a Wasm `block` control flow operator.
    fn translate_block(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from(ty)?;
        self.func_builder.translate_block(block_type)?;
        Ok(())
    }

    /// Translate a Wasm `loop` control flow operator.
    fn translate_loop(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from(ty)?;
        self.func_builder.translate_loop(block_type)?;
        Ok(())
    }

    /// Translate a Wasm `if` control flow operator.
    fn translate_if(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from(ty)?;
        self.func_builder.translate_if(block_type)?;
        Ok(())
    }

    /// Translate a single Wasm operator of the Wasm function.
    fn translate_operator(&mut self, operator: Operator) -> Result<(), ModuleError> {
        let unsupported_error = || Err(ModuleError::unsupported(&operator));
        match operator {
            Operator::Unreachable => todo!(),
            Operator::Nop => todo!(),
            Operator::Block { ty } => self.translate_block(ty),
            Operator::Loop { ty } => self.translate_loop(ty),
            Operator::If { ty } => todo!(),
            Operator::Else => todo!(),
            Operator::Try { .. }
            | Operator::Catch { .. }
            | Operator::Throw { .. }
            | Operator::Rethrow { .. } => unsupported_error(),
            Operator::End => todo!(),
            Operator::Br { relative_depth } => todo!(),
            Operator::BrIf { relative_depth } => todo!(),
            Operator::BrTable { table } => todo!(),
            Operator::Return => todo!(),
            Operator::Call { function_index } => todo!(),
            Operator::CallIndirect { index, table_index } => todo!(),
            Operator::ReturnCall { .. }
            | Operator::ReturnCallIndirect { .. }
            | Operator::Delegate { .. }
            | Operator::CatchAll => unsupported_error(),
            Operator::Drop => todo!(),
            Operator::Select => todo!(),
            Operator::TypedSelect { ty } => unsupported_error(),
            Operator::LocalGet { local_index } => todo!(),
            Operator::LocalSet { local_index } => todo!(),
            Operator::LocalTee { local_index } => todo!(),
            Operator::GlobalGet { global_index } => todo!(),
            Operator::GlobalSet { global_index } => todo!(),
            Operator::I32Load { memarg } => todo!(),
            Operator::I64Load { memarg } => todo!(),
            Operator::F32Load { memarg } => todo!(),
            Operator::F64Load { memarg } => todo!(),
            Operator::I32Load8S { memarg } => todo!(),
            Operator::I32Load8U { memarg } => todo!(),
            Operator::I32Load16S { memarg } => todo!(),
            Operator::I32Load16U { memarg } => todo!(),
            Operator::I64Load8S { memarg } => todo!(),
            Operator::I64Load8U { memarg } => todo!(),
            Operator::I64Load16S { memarg } => todo!(),
            Operator::I64Load16U { memarg } => todo!(),
            Operator::I64Load32S { memarg } => todo!(),
            Operator::I64Load32U { memarg } => todo!(),
            Operator::I32Store { memarg } => todo!(),
            Operator::I64Store { memarg } => todo!(),
            Operator::F32Store { memarg } => todo!(),
            Operator::F64Store { memarg } => todo!(),
            Operator::I32Store8 { memarg } => todo!(),
            Operator::I32Store16 { memarg } => todo!(),
            Operator::I64Store8 { memarg } => todo!(),
            Operator::I64Store16 { memarg } => todo!(),
            Operator::I64Store32 { memarg } => todo!(),
            Operator::MemorySize { mem, mem_byte } => todo!(),
            Operator::MemoryGrow { mem, mem_byte } => todo!(),
            Operator::I32Const { value } => todo!(),
            Operator::I64Const { value } => todo!(),
            Operator::F32Const { value } => todo!(),
            Operator::F64Const { value } => todo!(),
            Operator::RefNull { .. } | Operator::RefIsNull | Operator::RefFunc { .. } => {
                unsupported_error()
            }
            Operator::I32Eqz => todo!(),
            Operator::I32Eq => todo!(),
            Operator::I32Ne => todo!(),
            Operator::I32LtS => todo!(),
            Operator::I32LtU => todo!(),
            Operator::I32GtS => todo!(),
            Operator::I32GtU => todo!(),
            Operator::I32LeS => todo!(),
            Operator::I32LeU => todo!(),
            Operator::I32GeS => todo!(),
            Operator::I32GeU => todo!(),
            Operator::I64Eqz => todo!(),
            Operator::I64Eq => todo!(),
            Operator::I64Ne => todo!(),
            Operator::I64LtS => todo!(),
            Operator::I64LtU => todo!(),
            Operator::I64GtS => todo!(),
            Operator::I64GtU => todo!(),
            Operator::I64LeS => todo!(),
            Operator::I64LeU => todo!(),
            Operator::I64GeS => todo!(),
            Operator::I64GeU => todo!(),
            Operator::F32Eq => todo!(),
            Operator::F32Ne => todo!(),
            Operator::F32Lt => todo!(),
            Operator::F32Gt => todo!(),
            Operator::F32Le => todo!(),
            Operator::F32Ge => todo!(),
            Operator::F64Eq => todo!(),
            Operator::F64Ne => todo!(),
            Operator::F64Lt => todo!(),
            Operator::F64Gt => todo!(),
            Operator::F64Le => todo!(),
            Operator::F64Ge => todo!(),
            Operator::I32Clz => todo!(),
            Operator::I32Ctz => todo!(),
            Operator::I32Popcnt => todo!(),
            Operator::I32Add => todo!(),
            Operator::I32Sub => todo!(),
            Operator::I32Mul => todo!(),
            Operator::I32DivS => todo!(),
            Operator::I32DivU => todo!(),
            Operator::I32RemS => todo!(),
            Operator::I32RemU => todo!(),
            Operator::I32And => todo!(),
            Operator::I32Or => todo!(),
            Operator::I32Xor => todo!(),
            Operator::I32Shl => todo!(),
            Operator::I32ShrS => todo!(),
            Operator::I32ShrU => todo!(),
            Operator::I32Rotl => todo!(),
            Operator::I32Rotr => todo!(),
            Operator::I64Clz => todo!(),
            Operator::I64Ctz => todo!(),
            Operator::I64Popcnt => todo!(),
            Operator::I64Add => todo!(),
            Operator::I64Sub => todo!(),
            Operator::I64Mul => todo!(),
            Operator::I64DivS => todo!(),
            Operator::I64DivU => todo!(),
            Operator::I64RemS => todo!(),
            Operator::I64RemU => todo!(),
            Operator::I64And => todo!(),
            Operator::I64Or => todo!(),
            Operator::I64Xor => todo!(),
            Operator::I64Shl => todo!(),
            Operator::I64ShrS => todo!(),
            Operator::I64ShrU => todo!(),
            Operator::I64Rotl => todo!(),
            Operator::I64Rotr => todo!(),
            Operator::F32Abs => todo!(),
            Operator::F32Neg => todo!(),
            Operator::F32Ceil => todo!(),
            Operator::F32Floor => todo!(),
            Operator::F32Trunc => todo!(),
            Operator::F32Nearest => todo!(),
            Operator::F32Sqrt => todo!(),
            Operator::F32Add => todo!(),
            Operator::F32Sub => todo!(),
            Operator::F32Mul => todo!(),
            Operator::F32Div => todo!(),
            Operator::F32Min => todo!(),
            Operator::F32Max => todo!(),
            Operator::F32Copysign => todo!(),
            Operator::F64Abs => todo!(),
            Operator::F64Neg => todo!(),
            Operator::F64Ceil => todo!(),
            Operator::F64Floor => todo!(),
            Operator::F64Trunc => todo!(),
            Operator::F64Nearest => todo!(),
            Operator::F64Sqrt => todo!(),
            Operator::F64Add => todo!(),
            Operator::F64Sub => todo!(),
            Operator::F64Mul => todo!(),
            Operator::F64Div => todo!(),
            Operator::F64Min => todo!(),
            Operator::F64Max => todo!(),
            Operator::F64Copysign => todo!(),
            Operator::I32WrapI64 => todo!(),
            Operator::I32TruncF32S => todo!(),
            Operator::I32TruncF32U => todo!(),
            Operator::I32TruncF64S => todo!(),
            Operator::I32TruncF64U => todo!(),
            Operator::I64ExtendI32S => todo!(),
            Operator::I64ExtendI32U => todo!(),
            Operator::I64TruncF32S => todo!(),
            Operator::I64TruncF32U => todo!(),
            Operator::I64TruncF64S => todo!(),
            Operator::I64TruncF64U => todo!(),
            Operator::F32ConvertI32S => todo!(),
            Operator::F32ConvertI32U => todo!(),
            Operator::F32ConvertI64S => todo!(),
            Operator::F32ConvertI64U => todo!(),
            Operator::F32DemoteF64 => todo!(),
            Operator::F64ConvertI32S => todo!(),
            Operator::F64ConvertI32U => todo!(),
            Operator::F64ConvertI64S => todo!(),
            Operator::F64ConvertI64U => todo!(),
            Operator::F64PromoteF32 => todo!(),
            Operator::I32ReinterpretF32 => todo!(),
            Operator::I64ReinterpretF64 => todo!(),
            Operator::F32ReinterpretI32 => todo!(),
            Operator::F64ReinterpretI64 => todo!(),
            Operator::I32Extend8S => todo!(),
            Operator::I32Extend16S => todo!(),
            Operator::I64Extend8S => todo!(),
            Operator::I64Extend16S => todo!(),
            Operator::I64Extend32S => todo!(),
            Operator::I32TruncSatF32S => todo!(),
            Operator::I32TruncSatF32U => todo!(),
            Operator::I32TruncSatF64S => todo!(),
            Operator::I32TruncSatF64U => todo!(),
            Operator::I64TruncSatF32S => todo!(),
            Operator::I64TruncSatF32U => todo!(),
            Operator::I64TruncSatF64S => todo!(),
            Operator::I64TruncSatF64U => todo!(),
            Operator::MemoryInit { .. }
            | Operator::DataDrop { .. }
            | Operator::MemoryCopy { .. }
            | Operator::MemoryFill { .. }
            | Operator::TableInit { .. }
            | Operator::ElemDrop { .. }
            | Operator::TableCopy { .. }
            | Operator::TableFill { .. }
            | Operator::TableGet { .. }
            | Operator::TableSet { .. }
            | Operator::TableGrow { .. }
            | Operator::TableSize { .. }
            | Operator::MemoryAtomicNotify { .. }
            | Operator::MemoryAtomicWait32 { .. }
            | Operator::MemoryAtomicWait64 { .. }
            | Operator::AtomicFence { .. }
            | Operator::I32AtomicLoad { .. }
            | Operator::I64AtomicLoad { .. }
            | Operator::I32AtomicLoad8U { .. }
            | Operator::I32AtomicLoad16U { .. }
            | Operator::I64AtomicLoad8U { .. }
            | Operator::I64AtomicLoad16U { .. }
            | Operator::I64AtomicLoad32U { .. }
            | Operator::I32AtomicStore { .. }
            | Operator::I64AtomicStore { .. }
            | Operator::I32AtomicStore8 { .. }
            | Operator::I32AtomicStore16 { .. }
            | Operator::I64AtomicStore8 { .. }
            | Operator::I64AtomicStore16 { .. }
            | Operator::I64AtomicStore32 { .. }
            | Operator::I32AtomicRmwAdd { .. }
            | Operator::I64AtomicRmwAdd { .. }
            | Operator::I32AtomicRmw8AddU { .. }
            | Operator::I32AtomicRmw16AddU { .. }
            | Operator::I64AtomicRmw8AddU { .. }
            | Operator::I64AtomicRmw16AddU { .. }
            | Operator::I64AtomicRmw32AddU { .. }
            | Operator::I32AtomicRmwSub { .. }
            | Operator::I64AtomicRmwSub { .. }
            | Operator::I32AtomicRmw8SubU { .. }
            | Operator::I32AtomicRmw16SubU { .. }
            | Operator::I64AtomicRmw8SubU { .. }
            | Operator::I64AtomicRmw16SubU { .. }
            | Operator::I64AtomicRmw32SubU { .. }
            | Operator::I32AtomicRmwAnd { .. }
            | Operator::I64AtomicRmwAnd { .. }
            | Operator::I32AtomicRmw8AndU { .. }
            | Operator::I32AtomicRmw16AndU { .. }
            | Operator::I64AtomicRmw8AndU { .. }
            | Operator::I64AtomicRmw16AndU { .. }
            | Operator::I64AtomicRmw32AndU { .. }
            | Operator::I32AtomicRmwOr { .. }
            | Operator::I64AtomicRmwOr { .. }
            | Operator::I32AtomicRmw8OrU { .. }
            | Operator::I32AtomicRmw16OrU { .. }
            | Operator::I64AtomicRmw8OrU { .. }
            | Operator::I64AtomicRmw16OrU { .. }
            | Operator::I64AtomicRmw32OrU { .. }
            | Operator::I32AtomicRmwXor { .. }
            | Operator::I64AtomicRmwXor { .. }
            | Operator::I32AtomicRmw8XorU { .. }
            | Operator::I32AtomicRmw16XorU { .. }
            | Operator::I64AtomicRmw8XorU { .. }
            | Operator::I64AtomicRmw16XorU { .. }
            | Operator::I64AtomicRmw32XorU { .. }
            | Operator::I32AtomicRmwXchg { .. }
            | Operator::I64AtomicRmwXchg { .. }
            | Operator::I32AtomicRmw8XchgU { .. }
            | Operator::I32AtomicRmw16XchgU { .. }
            | Operator::I64AtomicRmw8XchgU { .. }
            | Operator::I64AtomicRmw16XchgU { .. }
            | Operator::I64AtomicRmw32XchgU { .. }
            | Operator::I32AtomicRmwCmpxchg { .. }
            | Operator::I64AtomicRmwCmpxchg { .. }
            | Operator::I32AtomicRmw8CmpxchgU { .. }
            | Operator::I32AtomicRmw16CmpxchgU { .. }
            | Operator::I64AtomicRmw8CmpxchgU { .. }
            | Operator::I64AtomicRmw16CmpxchgU { .. }
            | Operator::I64AtomicRmw32CmpxchgU { .. }
            | Operator::V128Load { .. }
            | Operator::V128Load8x8S { .. }
            | Operator::V128Load8x8U { .. }
            | Operator::V128Load16x4S { .. }
            | Operator::V128Load16x4U { .. }
            | Operator::V128Load32x2S { .. }
            | Operator::V128Load32x2U { .. }
            | Operator::V128Load8Splat { .. }
            | Operator::V128Load16Splat { .. }
            | Operator::V128Load32Splat { .. }
            | Operator::V128Load64Splat { .. }
            | Operator::V128Load32Zero { .. }
            | Operator::V128Load64Zero { .. }
            | Operator::V128Store { .. }
            | Operator::V128Load8Lane { .. }
            | Operator::V128Load16Lane { .. }
            | Operator::V128Load32Lane { .. }
            | Operator::V128Load64Lane { .. }
            | Operator::V128Store8Lane { .. }
            | Operator::V128Store16Lane { .. }
            | Operator::V128Store32Lane { .. }
            | Operator::V128Store64Lane { .. }
            | Operator::V128Const { .. }
            | Operator::I8x16Shuffle { .. }
            | Operator::I8x16ExtractLaneS { .. }
            | Operator::I8x16ExtractLaneU { .. }
            | Operator::I8x16ReplaceLane { .. }
            | Operator::I16x8ExtractLaneS { .. }
            | Operator::I16x8ExtractLaneU { .. }
            | Operator::I16x8ReplaceLane { .. }
            | Operator::I32x4ExtractLane { .. }
            | Operator::I32x4ReplaceLane { .. }
            | Operator::I64x2ExtractLane { .. }
            | Operator::I64x2ReplaceLane { .. }
            | Operator::F32x4ExtractLane { .. }
            | Operator::F32x4ReplaceLane { .. }
            | Operator::F64x2ExtractLane { .. }
            | Operator::F64x2ReplaceLane { .. }
            | Operator::I8x16Swizzle
            | Operator::I8x16Splat
            | Operator::I16x8Splat
            | Operator::I32x4Splat
            | Operator::I64x2Splat
            | Operator::F32x4Splat
            | Operator::F64x2Splat
            | Operator::I8x16Eq
            | Operator::I8x16Ne
            | Operator::I8x16LtS
            | Operator::I8x16LtU
            | Operator::I8x16GtS
            | Operator::I8x16GtU
            | Operator::I8x16LeS
            | Operator::I8x16LeU
            | Operator::I8x16GeS
            | Operator::I8x16GeU
            | Operator::I16x8Eq
            | Operator::I16x8Ne
            | Operator::I16x8LtS
            | Operator::I16x8LtU
            | Operator::I16x8GtS
            | Operator::I16x8GtU
            | Operator::I16x8LeS
            | Operator::I16x8LeU
            | Operator::I16x8GeS
            | Operator::I16x8GeU
            | Operator::I32x4Eq
            | Operator::I32x4Ne
            | Operator::I32x4LtS
            | Operator::I32x4LtU
            | Operator::I32x4GtS
            | Operator::I32x4GtU
            | Operator::I32x4LeS
            | Operator::I32x4LeU
            | Operator::I32x4GeS
            | Operator::I32x4GeU
            | Operator::I64x2Eq
            | Operator::I64x2Ne
            | Operator::I64x2LtS
            | Operator::I64x2GtS
            | Operator::I64x2LeS
            | Operator::I64x2GeS
            | Operator::F32x4Eq
            | Operator::F32x4Ne
            | Operator::F32x4Lt
            | Operator::F32x4Gt
            | Operator::F32x4Le
            | Operator::F32x4Ge
            | Operator::F64x2Eq
            | Operator::F64x2Ne
            | Operator::F64x2Lt
            | Operator::F64x2Gt
            | Operator::F64x2Le
            | Operator::F64x2Ge
            | Operator::V128Not
            | Operator::V128And
            | Operator::V128AndNot
            | Operator::V128Or
            | Operator::V128Xor
            | Operator::V128Bitselect
            | Operator::V128AnyTrue
            | Operator::I8x16Abs
            | Operator::I8x16Neg
            | Operator::I8x16Popcnt
            | Operator::I8x16AllTrue
            | Operator::I8x16Bitmask
            | Operator::I8x16NarrowI16x8S
            | Operator::I8x16NarrowI16x8U
            | Operator::I8x16Shl
            | Operator::I8x16ShrS
            | Operator::I8x16ShrU
            | Operator::I8x16Add
            | Operator::I8x16AddSatS
            | Operator::I8x16AddSatU
            | Operator::I8x16Sub
            | Operator::I8x16SubSatS
            | Operator::I8x16SubSatU
            | Operator::I8x16MinS
            | Operator::I8x16MinU
            | Operator::I8x16MaxS
            | Operator::I8x16MaxU
            | Operator::I8x16RoundingAverageU
            | Operator::I16x8ExtAddPairwiseI8x16S
            | Operator::I16x8ExtAddPairwiseI8x16U
            | Operator::I16x8Abs
            | Operator::I16x8Neg
            | Operator::I16x8Q15MulrSatS
            | Operator::I16x8AllTrue
            | Operator::I16x8Bitmask
            | Operator::I16x8NarrowI32x4S
            | Operator::I16x8NarrowI32x4U
            | Operator::I16x8ExtendLowI8x16S
            | Operator::I16x8ExtendHighI8x16S
            | Operator::I16x8ExtendLowI8x16U
            | Operator::I16x8ExtendHighI8x16U
            | Operator::I16x8Shl
            | Operator::I16x8ShrS
            | Operator::I16x8ShrU
            | Operator::I16x8Add
            | Operator::I16x8AddSatS
            | Operator::I16x8AddSatU
            | Operator::I16x8Sub
            | Operator::I16x8SubSatS
            | Operator::I16x8SubSatU
            | Operator::I16x8Mul
            | Operator::I16x8MinS
            | Operator::I16x8MinU
            | Operator::I16x8MaxS
            | Operator::I16x8MaxU
            | Operator::I16x8RoundingAverageU
            | Operator::I16x8ExtMulLowI8x16S
            | Operator::I16x8ExtMulHighI8x16S
            | Operator::I16x8ExtMulLowI8x16U
            | Operator::I16x8ExtMulHighI8x16U
            | Operator::I32x4ExtAddPairwiseI16x8S
            | Operator::I32x4ExtAddPairwiseI16x8U
            | Operator::I32x4Abs
            | Operator::I32x4Neg
            | Operator::I32x4AllTrue
            | Operator::I32x4Bitmask
            | Operator::I32x4ExtendLowI16x8S
            | Operator::I32x4ExtendHighI16x8S
            | Operator::I32x4ExtendLowI16x8U
            | Operator::I32x4ExtendHighI16x8U
            | Operator::I32x4Shl
            | Operator::I32x4ShrS
            | Operator::I32x4ShrU
            | Operator::I32x4Add
            | Operator::I32x4Sub
            | Operator::I32x4Mul
            | Operator::I32x4MinS
            | Operator::I32x4MinU
            | Operator::I32x4MaxS
            | Operator::I32x4MaxU
            | Operator::I32x4DotI16x8S
            | Operator::I32x4ExtMulLowI16x8S
            | Operator::I32x4ExtMulHighI16x8S
            | Operator::I32x4ExtMulLowI16x8U
            | Operator::I32x4ExtMulHighI16x8U
            | Operator::I64x2Abs
            | Operator::I64x2Neg
            | Operator::I64x2AllTrue
            | Operator::I64x2Bitmask
            | Operator::I64x2ExtendLowI32x4S
            | Operator::I64x2ExtendHighI32x4S
            | Operator::I64x2ExtendLowI32x4U
            | Operator::I64x2ExtendHighI32x4U
            | Operator::I64x2Shl
            | Operator::I64x2ShrS
            | Operator::I64x2ShrU
            | Operator::I64x2Add
            | Operator::I64x2Sub
            | Operator::I64x2Mul
            | Operator::I64x2ExtMulLowI32x4S
            | Operator::I64x2ExtMulHighI32x4S
            | Operator::I64x2ExtMulLowI32x4U
            | Operator::I64x2ExtMulHighI32x4U
            | Operator::F32x4Ceil
            | Operator::F32x4Floor
            | Operator::F32x4Trunc
            | Operator::F32x4Nearest
            | Operator::F32x4Abs
            | Operator::F32x4Neg
            | Operator::F32x4Sqrt
            | Operator::F32x4Add
            | Operator::F32x4Sub
            | Operator::F32x4Mul
            | Operator::F32x4Div
            | Operator::F32x4Min
            | Operator::F32x4Max
            | Operator::F32x4PMin
            | Operator::F32x4PMax
            | Operator::F64x2Ceil
            | Operator::F64x2Floor
            | Operator::F64x2Trunc
            | Operator::F64x2Nearest
            | Operator::F64x2Abs
            | Operator::F64x2Neg
            | Operator::F64x2Sqrt
            | Operator::F64x2Add
            | Operator::F64x2Sub
            | Operator::F64x2Mul
            | Operator::F64x2Div
            | Operator::F64x2Min
            | Operator::F64x2Max
            | Operator::F64x2PMin
            | Operator::F64x2PMax
            | Operator::I32x4TruncSatF32x4S
            | Operator::I32x4TruncSatF32x4U
            | Operator::F32x4ConvertI32x4S
            | Operator::F32x4ConvertI32x4U
            | Operator::I32x4TruncSatF64x2SZero
            | Operator::I32x4TruncSatF64x2UZero
            | Operator::F64x2ConvertLowI32x4S
            | Operator::F64x2ConvertLowI32x4U
            | Operator::F32x4DemoteF64x2Zero
            | Operator::F64x2PromoteLowF32x4
            | Operator::I8x16SwizzleRelaxed
            | Operator::I32x4TruncSatF32x4SRelaxed
            | Operator::I32x4TruncSatF32x4URelaxed
            | Operator::I32x4TruncSatF64x2SZeroRelaxed
            | Operator::I32x4TruncSatF64x2UZeroRelaxed
            | Operator::F32x4FmaRelaxed
            | Operator::F32x4FmsRelaxed
            | Operator::F64x2FmaRelaxed
            | Operator::F64x2FmsRelaxed
            | Operator::I8x16LaneSelect
            | Operator::I16x8LaneSelect
            | Operator::I32x4LaneSelect
            | Operator::I64x2LaneSelect
            | Operator::F32x4MinRelaxed
            | Operator::F32x4MaxRelaxed
            | Operator::F64x2MinRelaxed
            | Operator::F64x2MaxRelaxed => unsupported_error(),
        }
    }
}
