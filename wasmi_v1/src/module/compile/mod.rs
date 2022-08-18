pub use self::block_type::BlockType;
use super::{utils::value_type_from_wasmparser, FuncIdx, ModuleResources};
use crate::{
    engine::{FuncBody, FunctionBuilder, FunctionBuilderAllocations},
    Engine,
    ModuleError,
};
use wasmparser::{FuncValidator, FunctionBody, Operator, ValidatorResources};

mod block_type;
mod operator;

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
    allocations: &mut FunctionBuilderAllocations,
) -> Result<FuncBody, ModuleError> {
    FunctionTranslator::new(engine, func, func_body, validator, res, allocations).translate()
}

/// Translates Wasm bytecode into `wasmi` bytecode for a single Wasm function.
struct FunctionTranslator<'alloc, 'parser> {
    /// The function body that shall be translated.
    func_body: FunctionBody<'parser>,
    /// The interface to incrementally build up the `wasmi` bytecode function.
    func_builder: FunctionBuilder<'alloc, 'parser>,
    /// The Wasm validator.
    validator: FuncValidator<ValidatorResources>,
    /// The `wasmi` module resources.
    ///
    /// Provides immutable information about the translated Wasm module
    /// required for function translation to `wasmi` bytecode.
    res: ModuleResources<'parser>,
}

impl<'alloc, 'parser> FunctionTranslator<'alloc, 'parser> {
    /// Creates a new Wasm to `wasmi` bytecode function translator.
    fn new(
        engine: &Engine,
        func: FuncIdx,
        func_body: FunctionBody<'parser>,
        validator: FuncValidator<ValidatorResources>,
        res: ModuleResources<'parser>,
        allocations: &'alloc mut FunctionBuilderAllocations,
    ) -> Self {
        let func_builder = FunctionBuilder::new(engine, func, res, allocations);
        Self {
            func_body,
            func_builder,
            validator,
            res,
        }
    }

    /// Starts translation of the Wasm stream into `wasmi` bytecode.
    fn translate(mut self) -> Result<FuncBody, ModuleError> {
        self.translate_locals()?;
        self.translate_operators()?;
        let func_body = self.finish();
        Ok(func_body)
    }

    /// Finishes construction of the function and returns its [`FuncBody`].
    fn finish(self) -> FuncBody {
        self.func_builder.finish()
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
        self.validator.finish(reader.original_position())?;
        Ok(())
    }

    /// Translate a single Wasm operator of the Wasm function.
    fn translate_operator(&mut self, operator: Operator) -> Result<(), ModuleError> {
        let unsupported_error = || Err(ModuleError::unsupported(&operator));
        match operator {
            Operator::Unreachable => self.translate_unreachable(),
            Operator::Nop => self.translate_nop(),
            Operator::Block { ty } => self.translate_block(ty),
            Operator::Loop { ty } => self.translate_loop(ty),
            Operator::If { ty } => self.translate_if(ty),
            Operator::Else => self.translate_else(),
            Operator::Try { .. }
            | Operator::Catch { .. }
            | Operator::Throw { .. }
            | Operator::Rethrow { .. } => unsupported_error(),
            Operator::End => self.translate_end(),
            Operator::Br { relative_depth } => self.translate_br(relative_depth),
            Operator::BrIf { relative_depth } => self.translate_br_if(relative_depth),
            Operator::BrTable { table } => self.translate_br_table(table),
            Operator::Return => self.translate_return(),
            Operator::Call { function_index } => self.translate_call(function_index),
            Operator::CallIndirect { index, table_index } => {
                self.translate_call_indirect(index, table_index)
            }
            Operator::ReturnCall { .. }
            | Operator::ReturnCallIndirect { .. }
            | Operator::Delegate { .. }
            | Operator::CatchAll => unsupported_error(),
            Operator::Drop => self.translate_drop(),
            Operator::Select => self.translate_select(),
            Operator::TypedSelect { ty: _ } => unsupported_error(),
            Operator::LocalGet { local_index } => self.translate_local_get(local_index),
            Operator::LocalSet { local_index } => self.translate_local_set(local_index),
            Operator::LocalTee { local_index } => self.translate_local_tee(local_index),
            Operator::GlobalGet { global_index } => self.translate_global_get(global_index),
            Operator::GlobalSet { global_index } => self.translate_global_set(global_index),
            Operator::I32Load { memarg } => self.translate_i32_load(memarg),
            Operator::I64Load { memarg } => self.translate_i64_load(memarg),
            Operator::F32Load { memarg } => self.translate_f32_load(memarg),
            Operator::F64Load { memarg } => self.translate_f64_load(memarg),
            Operator::I32Load8S { memarg } => self.translate_i32_load_i8(memarg),
            Operator::I32Load8U { memarg } => self.translate_i32_load_u8(memarg),
            Operator::I32Load16S { memarg } => self.translate_i32_load_i16(memarg),
            Operator::I32Load16U { memarg } => self.translate_i32_load_u16(memarg),
            Operator::I64Load8S { memarg } => self.translate_i64_load_i8(memarg),
            Operator::I64Load8U { memarg } => self.translate_i64_load_u8(memarg),
            Operator::I64Load16S { memarg } => self.translate_i64_load_i16(memarg),
            Operator::I64Load16U { memarg } => self.translate_i64_load_u16(memarg),
            Operator::I64Load32S { memarg } => self.translate_i64_load_i32(memarg),
            Operator::I64Load32U { memarg } => self.translate_i64_load_u32(memarg),
            Operator::I32Store { memarg } => self.translate_i32_store(memarg),
            Operator::I64Store { memarg } => self.translate_i64_store(memarg),
            Operator::F32Store { memarg } => self.translate_f32_store(memarg),
            Operator::F64Store { memarg } => self.translate_f64_store(memarg),
            Operator::I32Store8 { memarg } => self.translate_i32_store_i8(memarg),
            Operator::I32Store16 { memarg } => self.translate_i32_store_i16(memarg),
            Operator::I64Store8 { memarg } => self.translate_i64_store_i8(memarg),
            Operator::I64Store16 { memarg } => self.translate_i64_store_i16(memarg),
            Operator::I64Store32 { memarg } => self.translate_i64_store_i32(memarg),
            Operator::MemorySize { mem, mem_byte } => self.translate_memory_size(mem, mem_byte),
            Operator::MemoryGrow { mem, mem_byte } => self.translate_memory_grow(mem, mem_byte),
            Operator::I32Const { value } => self.translate_i32_const(value),
            Operator::I64Const { value } => self.translate_i64_const(value),
            Operator::F32Const { value } => self.translate_f32_const(value),
            Operator::F64Const { value } => self.translate_f64_const(value),
            Operator::RefNull { .. } | Operator::RefIsNull | Operator::RefFunc { .. } => {
                unsupported_error()
            }
            Operator::I32Eqz => self.translate_i32_eqz(),
            Operator::I32Eq => self.translate_i32_eq(),
            Operator::I32Ne => self.translate_i32_ne(),
            Operator::I32LtS => self.translate_i32_lt(),
            Operator::I32LtU => self.translate_u32_lt(),
            Operator::I32GtS => self.translate_i32_gt(),
            Operator::I32GtU => self.translate_u32_gt(),
            Operator::I32LeS => self.translate_i32_le(),
            Operator::I32LeU => self.translate_u32_le(),
            Operator::I32GeS => self.translate_i32_ge(),
            Operator::I32GeU => self.translate_u32_ge(),
            Operator::I64Eqz => self.translate_i64_eqz(),
            Operator::I64Eq => self.translate_i64_eq(),
            Operator::I64Ne => self.translate_i64_ne(),
            Operator::I64LtS => self.translate_i64_lt(),
            Operator::I64LtU => self.translate_u64_lt(),
            Operator::I64GtS => self.translate_i64_gt(),
            Operator::I64GtU => self.translate_u64_gt(),
            Operator::I64LeS => self.translate_i64_le(),
            Operator::I64LeU => self.translate_u64_le(),
            Operator::I64GeS => self.translate_i64_ge(),
            Operator::I64GeU => self.translate_u64_ge(),
            Operator::F32Eq => self.translate_f32_eq(),
            Operator::F32Ne => self.translate_f32_ne(),
            Operator::F32Lt => self.translate_f32_lt(),
            Operator::F32Gt => self.translate_f32_gt(),
            Operator::F32Le => self.translate_f32_le(),
            Operator::F32Ge => self.translate_f32_ge(),
            Operator::F64Eq => self.translate_f64_eq(),
            Operator::F64Ne => self.translate_f64_ne(),
            Operator::F64Lt => self.translate_f64_lt(),
            Operator::F64Gt => self.translate_f64_gt(),
            Operator::F64Le => self.translate_f64_le(),
            Operator::F64Ge => self.translate_f64_ge(),
            Operator::I32Clz => self.translate_i32_clz(),
            Operator::I32Ctz => self.translate_i32_ctz(),
            Operator::I32Popcnt => self.translate_i32_popcnt(),
            Operator::I32Add => self.translate_i32_add(),
            Operator::I32Sub => self.translate_i32_sub(),
            Operator::I32Mul => self.translate_i32_mul(),
            Operator::I32DivS => self.translate_i32_div(),
            Operator::I32DivU => self.translate_u32_div(),
            Operator::I32RemS => self.translate_i32_rem(),
            Operator::I32RemU => self.translate_u32_rem(),
            Operator::I32And => self.translate_i32_and(),
            Operator::I32Or => self.translate_i32_or(),
            Operator::I32Xor => self.translate_i32_xor(),
            Operator::I32Shl => self.translate_i32_shl(),
            Operator::I32ShrS => self.translate_i32_shr(),
            Operator::I32ShrU => self.translate_u32_shr(),
            Operator::I32Rotl => self.translate_i32_rotl(),
            Operator::I32Rotr => self.translate_i32_rotr(),
            Operator::I64Clz => self.translate_i64_clz(),
            Operator::I64Ctz => self.translate_i64_ctz(),
            Operator::I64Popcnt => self.translate_i64_popcnt(),
            Operator::I64Add => self.translate_i64_add(),
            Operator::I64Sub => self.translate_i64_sub(),
            Operator::I64Mul => self.translate_i64_mul(),
            Operator::I64DivS => self.translate_i64_div(),
            Operator::I64DivU => self.translate_u64_div(),
            Operator::I64RemS => self.translate_i64_rem(),
            Operator::I64RemU => self.translate_u64_rem(),
            Operator::I64And => self.translate_i64_and(),
            Operator::I64Or => self.translate_i64_or(),
            Operator::I64Xor => self.translate_i64_xor(),
            Operator::I64Shl => self.translate_i64_shl(),
            Operator::I64ShrS => self.translate_i64_shr(),
            Operator::I64ShrU => self.translate_u64_shr(),
            Operator::I64Rotl => self.translate_i64_rotl(),
            Operator::I64Rotr => self.translate_i64_rotr(),
            Operator::F32Abs => self.translate_f32_abs(),
            Operator::F32Neg => self.translate_f32_neg(),
            Operator::F32Ceil => self.translate_f32_ceil(),
            Operator::F32Floor => self.translate_f32_floor(),
            Operator::F32Trunc => self.translate_f32_trunc(),
            Operator::F32Nearest => self.translate_f32_nearest(),
            Operator::F32Sqrt => self.translate_f32_sqrt(),
            Operator::F32Add => self.translate_f32_add(),
            Operator::F32Sub => self.translate_f32_sub(),
            Operator::F32Mul => self.translate_f32_mul(),
            Operator::F32Div => self.translate_f32_div(),
            Operator::F32Min => self.translate_f32_min(),
            Operator::F32Max => self.translate_f32_max(),
            Operator::F32Copysign => self.translate_f32_copysign(),
            Operator::F64Abs => self.translate_f64_abs(),
            Operator::F64Neg => self.translate_f64_neg(),
            Operator::F64Ceil => self.translate_f64_ceil(),
            Operator::F64Floor => self.translate_f64_floor(),
            Operator::F64Trunc => self.translate_f64_trunc(),
            Operator::F64Nearest => self.translate_f64_nearest(),
            Operator::F64Sqrt => self.translate_f64_sqrt(),
            Operator::F64Add => self.translate_f64_add(),
            Operator::F64Sub => self.translate_f64_sub(),
            Operator::F64Mul => self.translate_f64_mul(),
            Operator::F64Div => self.translate_f64_div(),
            Operator::F64Min => self.translate_f64_min(),
            Operator::F64Max => self.translate_f64_max(),
            Operator::F64Copysign => self.translate_f64_copysign(),
            Operator::I32WrapI64 => self.translate_i32_wrap_i64(),
            Operator::I32TruncF32S => self.translate_i32_trunc_f32(),
            Operator::I32TruncF32U => self.translate_u32_trunc_f32(),
            Operator::I32TruncF64S => self.translate_i32_trunc_f64(),
            Operator::I32TruncF64U => self.translate_u32_trunc_f64(),
            Operator::I64ExtendI32S => self.translate_i64_extend_i32(),
            Operator::I64ExtendI32U => self.translate_u64_extend_i32(),
            Operator::I64TruncF32S => self.translate_i64_trunc_f32(),
            Operator::I64TruncF32U => self.translate_u64_trunc_f32(),
            Operator::I64TruncF64S => self.translate_i64_trunc_f64(),
            Operator::I64TruncF64U => self.translate_u64_trunc_f64(),
            Operator::F32ConvertI32S => self.translate_f32_convert_i32(),
            Operator::F32ConvertI32U => self.translate_f32_convert_u32(),
            Operator::F32ConvertI64S => self.translate_f32_convert_i64(),
            Operator::F32ConvertI64U => self.translate_f32_convert_u64(),
            Operator::F32DemoteF64 => self.translate_f32_demote_f64(),
            Operator::F64ConvertI32S => self.translate_f64_convert_i32(),
            Operator::F64ConvertI32U => self.translate_f64_convert_u32(),
            Operator::F64ConvertI64S => self.translate_f64_convert_i64(),
            Operator::F64ConvertI64U => self.translate_f64_convert_u64(),
            Operator::F64PromoteF32 => self.translate_f64_promote_f32(),
            Operator::I32ReinterpretF32 => self.translate_i32_reinterpret_f32(),
            Operator::I64ReinterpretF64 => self.translate_i64_reinterpret_f64(),
            Operator::F32ReinterpretI32 => self.translate_f32_reinterpret_i32(),
            Operator::F64ReinterpretI64 => self.translate_f64_reinterpret_i64(),
            Operator::I32TruncSatF32S => self.translate_i32_truncate_saturate_f32(),
            Operator::I32TruncSatF32U => self.translate_u32_truncate_saturate_f32(),
            Operator::I32TruncSatF64S => self.translate_i32_truncate_saturate_f64(),
            Operator::I32TruncSatF64U => self.translate_u32_truncate_saturate_f64(),
            Operator::I64TruncSatF32S => self.translate_i64_truncate_saturate_f32(),
            Operator::I64TruncSatF32U => self.translate_u64_truncate_saturate_f32(),
            Operator::I64TruncSatF64S => self.translate_i64_truncate_saturate_f64(),
            Operator::I64TruncSatF64U => self.translate_u64_truncate_saturate_f64(),
            Operator::I32Extend8S => self.translate_i32_sign_extend8(),
            Operator::I32Extend16S => self.translate_i32_sign_extend16(),
            Operator::I64Extend8S => self.translate_i64_sign_extend8(),
            Operator::I64Extend16S => self.translate_i64_sign_extend16(),
            Operator::I64Extend32S => self.translate_i64_sign_extend32(),
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
            | Operator::I8x16RelaxedSwizzle
            | Operator::I32x4RelaxedTruncSatF32x4S
            | Operator::I32x4RelaxedTruncSatF32x4U
            | Operator::I32x4RelaxedTruncSatF64x2SZero
            | Operator::I32x4RelaxedTruncSatF64x2UZero
            | Operator::F32x4Fma
            | Operator::F32x4Fms
            | Operator::F64x2Fma
            | Operator::F64x2Fms
            | Operator::I8x16LaneSelect
            | Operator::I16x8LaneSelect
            | Operator::I32x4LaneSelect
            | Operator::I64x2LaneSelect
            | Operator::F32x4RelaxedMin
            | Operator::F32x4RelaxedMax
            | Operator::F64x2RelaxedMin
            | Operator::F64x2RelaxedMax => unsupported_error(),
        }
    }
}
