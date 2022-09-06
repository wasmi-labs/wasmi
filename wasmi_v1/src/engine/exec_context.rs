use super::{
    super::{Global, Memory, Table},
    bytecode::{FuncIdx, GlobalIdx, Instruction, LocalIdx, Offset, SignatureIdx},
    cache::InstanceCache,
    code_map::Instructions,
    AsContextMut,
    CallOutcome,
    DropKeep,
    FuncFrame,
    Target,
    ValueStack,
};
use crate::{
    core::{Trap, TrapCode, F32, F64},
    Func,
};
use core::cmp;
use wasmi_core::{memory_units::Pages, ExtendInto, LittleEndianConvert, UntypedValue, WrapInto};

/// State that is used during Wasm function execution.
#[derive(Debug)]
pub struct FunctionExecutor<'engine, 'func> {
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: &'engine mut ValueStack,
    /// The function frame that is being executed.
    frame: &'func mut FuncFrame,
    /// The resolved function body of the executed function frame.
    insts: Instructions<'engine>,
}

impl<'engine, 'func> FunctionExecutor<'engine, 'func> {
    /// Creates an execution context for the given [`FuncFrame`].
    #[inline(always)]
    pub fn new(
        frame: &'func mut FuncFrame,
        insts: Instructions<'engine>,
        value_stack: &'engine mut ValueStack,
    ) -> Self {
        Self {
            frame,
            insts,
            value_stack,
        }
    }

    /// Executes the current function frame.
    ///
    /// # Note
    ///
    /// This executes instructions sequentially until either the function
    /// calls into another function or the function returns to its caller.
    #[inline(always)]
    #[rustfmt::skip]
    pub fn execute_frame(self, mut ctx: impl AsContextMut, cache: &mut InstanceCache) -> Result<CallOutcome, Trap> {
        use Instruction as Instr;
        cache.update_instance(self.frame.instance());
        let mut exec_ctx = ExecutionContext::new(self.value_stack, self.frame, cache, &mut ctx, self.frame.pc());
        let mut top = UntypedValue::default();
        loop {
            // # Safety
            //
            // Properly constructed `wasmi` bytecode can never produce invalid `pc`.
            let instr = unsafe {
                self.insts.get_release_unchecked(exec_ctx.pc)
            };
            // println!("\
            //     \ttop = {:?}\n\
            //     \tstack = {:?}\n\
            //     \tinstr = {:?}\n\
            // ", top, exec_ctx.value_stack, instr);
            match instr {
                Instr::LocalGet { local_depth } => {
                    top = exec_ctx.visit_local_get(*local_depth, top)
                }
                Instr::LocalGetEmpty { local_depth } => {
                    top = exec_ctx.visit_local_get_empty(*local_depth)
                }
                Instr::LocalSet { local_depth } => { top = exec_ctx.visit_local_set(*local_depth, top); }
                Instr::LocalTee { local_depth } => { exec_ctx.visit_local_tee(*local_depth, top) }
                Instr::Br(target) => { top = exec_ctx.visit_br(top, *target); }
                Instr::BrEmpty(target) => { exec_ctx.visit_br_empty(*target) }
                Instr::BrIfEqz(target) => { top = exec_ctx.visit_br_if_eqz(*target, top); }
                Instr::BrIfNez(target) => { top = exec_ctx.visit_br_if_nez(*target, top); }
                Instr::ReturnIfNez(drop_keep)  => {
                    match exec_ctx.visit_return_if_nez(*drop_keep, top) {
                        MaybeReturn::Return => {
                            return Ok(CallOutcome::Return)
                        }
                        MaybeReturn::Continue { new_top } => {
                            top = new_top;
                        },
                    }
                }
                Instr::BrTable { len_targets } => {
                    top = exec_ctx.visit_br_table(*len_targets, top);
                }
                Instr::Unreachable => { exec_ctx.visit_unreachable()?; }
                Instr::Return(drop_keep) => {
                    exec_ctx.visit_return(*drop_keep, top);
                    return Ok(CallOutcome::Return)
                }
                Instr::ReturnEmpty(drop_keep) => {
                    exec_ctx.visit_return_empty(*drop_keep);
                    return Ok(CallOutcome::Return)
                }
                Instr::Call(func) => {
                    return exec_ctx.visit_call(*func, top)
                }
                Instr::CallEmpty(func) => {
                    return exec_ctx.visit_call_empty(*func)
                }
                Instr::CallIndirect(signature)  => {
                    return exec_ctx.visit_call_indirect(*signature, top)
                }
                Instr::Drop => { top = exec_ctx.visit_drop(); }
                Instr::Select => { top = exec_ctx.visit_select(top); }
                Instr::GlobalGet(global_idx)  => {
                    top = exec_ctx.visit_global_get(*global_idx, top);
                }
                Instr::GlobalGetEmpty(global_idx) => {
                    top = exec_ctx.visit_global_get_empty(*global_idx);
                }
                Instr::GlobalSet(global_idx)  => {
                    top = exec_ctx.visit_global_set(*global_idx, top);
                }
                Instr::I32Load(offset)  => { top = exec_ctx.visit_i32_load(top, *offset)?; }
                Instr::I64Load(offset)  => { top = exec_ctx.visit_i64_load(top, *offset)?; }
                Instr::F32Load(offset)  => { top = exec_ctx.visit_f32_load(top, *offset)?; }
                Instr::F64Load(offset)  => { top = exec_ctx.visit_f64_load(top, *offset)?; }
                Instr::I32Load8S(offset)  => { top = exec_ctx.visit_i32_load_i8(top, *offset)?; }
                Instr::I32Load8U(offset)  => { top = exec_ctx.visit_i32_load_u8(top, *offset)?; }
                Instr::I32Load16S(offset)  => { top = exec_ctx.visit_i32_load_i16(top, *offset)?; }
                Instr::I32Load16U(offset)  => { top = exec_ctx.visit_i32_load_u16(top, *offset)?; }
                Instr::I64Load8S(offset)  => { top = exec_ctx.visit_i64_load_i8(top, *offset)?; }
                Instr::I64Load8U(offset)  => { top = exec_ctx.visit_i64_load_u8(top, *offset)?; }
                Instr::I64Load16S(offset)  => { top = exec_ctx.visit_i64_load_i16(top, *offset)?; }
                Instr::I64Load16U(offset)  => { top = exec_ctx.visit_i64_load_u16(top, *offset)?; }
                Instr::I64Load32S(offset)  => { top = exec_ctx.visit_i64_load_i32(top, *offset)?; }
                Instr::I64Load32U(offset)  => { top = exec_ctx.visit_i64_load_u32(top, *offset)?; }
                Instr::I32Store(offset)  => { top = exec_ctx.visit_i32_store(top, *offset)?; }
                Instr::I64Store(offset)  => { top = exec_ctx.visit_i64_store(top, *offset)?; }
                Instr::F32Store(offset)  => { top = exec_ctx.visit_f32_store(top, *offset)?; }
                Instr::F64Store(offset)  => { top = exec_ctx.visit_f64_store(top, *offset)?; }
                Instr::I32Store8(offset)  => { top = exec_ctx.visit_i32_store_8(top, *offset)?; }
                Instr::I32Store16(offset)  => { top = exec_ctx.visit_i32_store_16(top, *offset)?; }
                Instr::I64Store8(offset)  => { top = exec_ctx.visit_i64_store_8(top, *offset)?; }
                Instr::I64Store16(offset)  => { top = exec_ctx.visit_i64_store_16(top, *offset)?; }
                Instr::I64Store32(offset)  => { top = exec_ctx.visit_i64_store_32(top, *offset)?; }
                Instr::MemorySize => { top = exec_ctx.visit_memory_size(top); }
                Instr::MemorySizeEmpty => { top = exec_ctx.visit_memory_size_empty(); }
                Instr::MemoryGrow => { top = exec_ctx.visit_memory_grow(top); }
                Instr::Const(bytes)  => { top = exec_ctx.visit_const(*bytes, top); }
                Instr::ConstEmpty(bytes)  => { top = exec_ctx.visit_const_empty(*bytes); }
                Instr::I32Eqz => { top = exec_ctx.visit_i32_eqz(top); }
                Instr::I32Eq => { top = exec_ctx.visit_i32_eq(top); }
                Instr::I32Ne => { top = exec_ctx.visit_i32_ne(top); }
                Instr::I32LtS => { top = exec_ctx.visit_i32_lt_s(top); }
                Instr::I32LtU => { top = exec_ctx.visit_i32_lt_u(top); }
                Instr::I32GtS => { top = exec_ctx.visit_i32_gt_s(top); }
                Instr::I32GtU => { top = exec_ctx.visit_i32_gt_u(top); }
                Instr::I32LeS => { top = exec_ctx.visit_i32_le_s(top); }
                Instr::I32LeU => { top = exec_ctx.visit_i32_le_u(top); }
                Instr::I32GeS => { top = exec_ctx.visit_i32_ge_s(top); }
                Instr::I32GeU => { top = exec_ctx.visit_i32_ge_u(top); }
                Instr::I64Eqz => { top = exec_ctx.visit_i64_eqz(top); }
                Instr::I64Eq => { top = exec_ctx.visit_i64_eq(top); }
                Instr::I64Ne => { top = exec_ctx.visit_i64_ne(top); }
                Instr::I64LtS => { top = exec_ctx.visit_i64_lt_s(top); }
                Instr::I64LtU => { top = exec_ctx.visit_i64_lt_u(top); }
                Instr::I64GtS => { top = exec_ctx.visit_i64_gt_s(top); }
                Instr::I64GtU => { top = exec_ctx.visit_i64_gt_u(top); }
                Instr::I64LeS => { top = exec_ctx.visit_i64_le_s(top); }
                Instr::I64LeU => { top = exec_ctx.visit_i64_le_u(top); }
                Instr::I64GeS => { top = exec_ctx.visit_i64_ge_s(top); }
                Instr::I64GeU => { top = exec_ctx.visit_i64_ge_u(top); }
                Instr::F32Eq => { top = exec_ctx.visit_f32_eq(top); }
                Instr::F32Ne => { top = exec_ctx.visit_f32_ne(top); }
                Instr::F32Lt => { top = exec_ctx.visit_f32_lt(top); }
                Instr::F32Gt => { top = exec_ctx.visit_f32_gt(top); }
                Instr::F32Le => { top = exec_ctx.visit_f32_le(top); }
                Instr::F32Ge => { top = exec_ctx.visit_f32_ge(top); }
                Instr::F64Eq => { top = exec_ctx.visit_f64_eq(top); }
                Instr::F64Ne => { top = exec_ctx.visit_f64_ne(top); }
                Instr::F64Lt => { top = exec_ctx.visit_f64_lt(top); }
                Instr::F64Gt => { top = exec_ctx.visit_f64_gt(top); }
                Instr::F64Le => { top = exec_ctx.visit_f64_le(top); }
                Instr::F64Ge => { top = exec_ctx.visit_f64_ge(top); }
                Instr::I32Clz => { top = exec_ctx.visit_i32_clz(top); }
                Instr::I32Ctz => { top = exec_ctx.visit_i32_ctz(top); }
                Instr::I32Popcnt => { top = exec_ctx.visit_i32_popcnt(top); }
                Instr::I32Add => { top = exec_ctx.visit_i32_add(top); }
                Instr::I32Sub => { top = exec_ctx.visit_i32_sub(top); }
                Instr::I32Mul => { top = exec_ctx.visit_i32_mul(top); }
                Instr::I32DivS => { top = exec_ctx.visit_i32_div_s(top)?; }
                Instr::I32DivU => { top = exec_ctx.visit_i32_div_u(top)?; }
                Instr::I32RemS => { top = exec_ctx.visit_i32_rem_s(top)?; }
                Instr::I32RemU => { top = exec_ctx.visit_i32_rem_u(top)?; }
                Instr::I32And => { top = exec_ctx.visit_i32_and(top); }
                Instr::I32Or => { top = exec_ctx.visit_i32_or(top); }
                Instr::I32Xor => { top = exec_ctx.visit_i32_xor(top); }
                Instr::I32Shl => { top = exec_ctx.visit_i32_shl(top); }
                Instr::I32ShrS => { top = exec_ctx.visit_i32_shr_s(top); }
                Instr::I32ShrU => { top = exec_ctx.visit_i32_shr_u(top); }
                Instr::I32Rotl => { top = exec_ctx.visit_i32_rotl(top); }
                Instr::I32Rotr => { top = exec_ctx.visit_i32_rotr(top); }
                Instr::I64Clz => { top = exec_ctx.visit_i64_clz(top); }
                Instr::I64Ctz => { top = exec_ctx.visit_i64_ctz(top); }
                Instr::I64Popcnt => { top = exec_ctx.visit_i64_popcnt(top); }
                Instr::I64Add => { top = exec_ctx.visit_i64_add(top); }
                Instr::I64Sub => { top = exec_ctx.visit_i64_sub(top); }
                Instr::I64Mul => { top = exec_ctx.visit_i64_mul(top); }
                Instr::I64DivS => { top = exec_ctx.visit_i64_div_s(top)?; }
                Instr::I64DivU => { top = exec_ctx.visit_i64_div_u(top)?; }
                Instr::I64RemS => { top = exec_ctx.visit_i64_rem_s(top)?; }
                Instr::I64RemU => { top = exec_ctx.visit_i64_rem_u(top)?; }
                Instr::I64And => { top = exec_ctx.visit_i64_and(top); }
                Instr::I64Or => { top = exec_ctx.visit_i64_or(top); }
                Instr::I64Xor => { top = exec_ctx.visit_i64_xor(top); }
                Instr::I64Shl => { top = exec_ctx.visit_i64_shl(top); }
                Instr::I64ShrS => { top = exec_ctx.visit_i64_shr_s(top); }
                Instr::I64ShrU => { top = exec_ctx.visit_i64_shr_u(top); }
                Instr::I64Rotl => { top = exec_ctx.visit_i64_rotl(top); }
                Instr::I64Rotr => { top = exec_ctx.visit_i64_rotr(top); }
                Instr::F32Abs => { top = exec_ctx.visit_f32_abs(top); }
                Instr::F32Neg => { top = exec_ctx.visit_f32_neg(top); }
                Instr::F32Ceil => { top = exec_ctx.visit_f32_ceil(top); }
                Instr::F32Floor => { top = exec_ctx.visit_f32_floor(top); }
                Instr::F32Trunc => { top = exec_ctx.visit_f32_trunc(top); }
                Instr::F32Nearest => { top = exec_ctx.visit_f32_nearest(top); }
                Instr::F32Sqrt => { top = exec_ctx.visit_f32_sqrt(top); }
                Instr::F32Add => { top = exec_ctx.visit_f32_add(top); }
                Instr::F32Sub => { top = exec_ctx.visit_f32_sub(top); }
                Instr::F32Mul => { top = exec_ctx.visit_f32_mul(top); }
                Instr::F32Div => { top = exec_ctx.visit_f32_div(top)?; }
                Instr::F32Min => { top = exec_ctx.visit_f32_min(top); }
                Instr::F32Max => { top = exec_ctx.visit_f32_max(top); }
                Instr::F32Copysign => { top = exec_ctx.visit_f32_copysign(top); }
                Instr::F64Abs => { top = exec_ctx.visit_f64_abs(top); }
                Instr::F64Neg => { top = exec_ctx.visit_f64_neg(top); }
                Instr::F64Ceil => { top = exec_ctx.visit_f64_ceil(top); }
                Instr::F64Floor => { top = exec_ctx.visit_f64_floor(top); }
                Instr::F64Trunc => { top = exec_ctx.visit_f64_trunc(top); }
                Instr::F64Nearest => { top = exec_ctx.visit_f64_nearest(top); }
                Instr::F64Sqrt => { top = exec_ctx.visit_f64_sqrt(top); }
                Instr::F64Add => { top = exec_ctx.visit_f64_add(top); }
                Instr::F64Sub => { top = exec_ctx.visit_f64_sub(top); }
                Instr::F64Mul => { top = exec_ctx.visit_f64_mul(top); }
                Instr::F64Div => { top = exec_ctx.visit_f64_div(top)?; }
                Instr::F64Min => { top = exec_ctx.visit_f64_min(top); }
                Instr::F64Max => { top = exec_ctx.visit_f64_max(top); }
                Instr::F64Copysign => { top = exec_ctx.visit_f64_copysign(top); }
                Instr::I32WrapI64 => { top = exec_ctx.visit_i32_wrap_i64(top); }
                Instr::I32TruncSF32 => { top = exec_ctx.visit_i32_trunc_f32(top)?; }
                Instr::I32TruncUF32 => { top = exec_ctx.visit_u32_trunc_f32(top)?; }
                Instr::I32TruncSF64 => { top = exec_ctx.visit_i32_trunc_f64(top)?; }
                Instr::I32TruncUF64 => { top = exec_ctx.visit_u32_trunc_f64(top)?; }
                Instr::I64ExtendSI32 => { top = exec_ctx.visit_i64_extend_i32(top); }
                Instr::I64ExtendUI32 => { top = exec_ctx.visit_i64_extend_u32(top); }
                Instr::I64TruncSF32 => { top = exec_ctx.visit_i64_trunc_f32(top)?; }
                Instr::I64TruncUF32 => { top = exec_ctx.visit_u64_trunc_f32(top)?; }
                Instr::I64TruncSF64 => { top = exec_ctx.visit_i64_trunc_f64(top)?; }
                Instr::I64TruncUF64 => { top = exec_ctx.visit_u64_trunc_f64(top)?; }
                Instr::F32ConvertSI32 => { top = exec_ctx.visit_f32_convert_i32(top); }
                Instr::F32ConvertUI32 => { top = exec_ctx.visit_f32_convert_u32(top); }
                Instr::F32ConvertSI64 => { top = exec_ctx.visit_f32_convert_i64(top); }
                Instr::F32ConvertUI64 => { top = exec_ctx.visit_f32_convert_u64(top); }
                Instr::F32DemoteF64 => { top = exec_ctx.visit_f32_demote_f64(top); }
                Instr::F64ConvertSI32 => { top = exec_ctx.visit_f64_convert_i32(top); }
                Instr::F64ConvertUI32 => { top = exec_ctx.visit_f64_convert_u32(top); }
                Instr::F64ConvertSI64 => { top = exec_ctx.visit_f64_convert_i64(top); }
                Instr::F64ConvertUI64 => { top = exec_ctx.visit_f64_convert_u64(top); }
                Instr::F64PromoteF32 => { top = exec_ctx.visit_f64_promote_f32(top); }
                Instr::I32ReinterpretF32 => { top = exec_ctx.visit_i32_reinterpret_f32(top); }
                Instr::I64ReinterpretF64 => { top = exec_ctx.visit_i64_reinterpret_f64(top); }
                Instr::F32ReinterpretI32 => { top = exec_ctx.visit_f32_reinterpret_i32(top); }
                Instr::F64ReinterpretI64 => { top = exec_ctx.visit_f64_reinterpret_i64(top); }
                Instr::I32TruncSatF32S => { top = exec_ctx.visit_i32_trunc_sat_f32(top); }
                Instr::I32TruncSatF32U => { top = exec_ctx.visit_u32_trunc_sat_f32(top); }
                Instr::I32TruncSatF64S => { top = exec_ctx.visit_i32_trunc_sat_f64(top); }
                Instr::I32TruncSatF64U => { top = exec_ctx.visit_u32_trunc_sat_f64(top); }
                Instr::I64TruncSatF32S => { top = exec_ctx.visit_i64_trunc_sat_f32(top); }
                Instr::I64TruncSatF32U => { top = exec_ctx.visit_u64_trunc_sat_f32(top); }
                Instr::I64TruncSatF64S => { top = exec_ctx.visit_i64_trunc_sat_f64(top); }
                Instr::I64TruncSatF64U => { top = exec_ctx.visit_u64_trunc_sat_f64(top); }
                Instr::I32Extend8S => { top = exec_ctx.visit_i32_sign_extend8(top); }
                Instr::I32Extend16S => { top = exec_ctx.visit_i32_sign_extend16(top); }
                Instr::I64Extend8S => { top = exec_ctx.visit_i64_sign_extend8(top); }
                Instr::I64Extend16S => { top = exec_ctx.visit_i64_sign_extend16(top); }
                Instr::I64Extend32S => { top = exec_ctx.visit_i64_sign_extend32(top); }
            }
        }
    }
}

/// An execution context for executing a single `wasmi` bytecode instruction.
#[derive(Debug)]
struct ExecutionContext<'engine, 'func, Ctx> {
    /// The program counter.
    pc: usize,
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: &'engine mut ValueStack,
    /// The function frame that is being executed.
    frame: &'func mut FuncFrame,
    /// Stores frequently used instance related data.
    cache: &'engine mut InstanceCache,
    /// A mutable [`Store`] context.
    ///
    /// [`Store`]: [`crate::v1::Store`]
    ctx: Ctx,
}

impl<'engine, 'func, Ctx> ExecutionContext<'engine, 'func, Ctx>
where
    Ctx: AsContextMut,
{
    /// Creates a new [`ExecutionContext`] for executing a single `wasmi` bytecode instruction.
    pub fn new(
        value_stack: &'engine mut ValueStack,
        frame: &'func mut FuncFrame,
        cache: &'engine mut InstanceCache,
        ctx: Ctx,
        pc: usize,
    ) -> Self {
        Self {
            value_stack,
            frame,
            ctx,
            pc,
            cache,
        }
    }

    /// Returns the default linear memory.
    ///
    /// # Panics
    ///
    /// If there exists is no linear memory for the instance.
    #[inline]
    fn default_memory(&mut self) -> Memory {
        self.cache.default_memory(&self.ctx)
    }

    /// Returns the default table.
    ///
    /// # Panics
    ///
    /// If there exists is no table for the instance.
    #[inline]
    fn default_table(&mut self) -> Table {
        self.cache.default_table(&self.ctx)
    }

    /// Returns the global variable at the given index.
    ///
    /// # Panics
    ///
    /// If there is no global variable at the given index.
    fn global(&self, global_index: GlobalIdx) -> Global {
        self.frame
            .instance()
            .get_global(self.ctx.as_context(), global_index.into_inner())
            .unwrap_or_else(|| panic!("missing global at index {:?}", global_index))
    }

    /// Returns the local depth as `usize`.
    fn convert_local_depth(local_depth: LocalIdx) -> usize {
        local_depth.into_inner() as usize
    }

    /// Calculates the effective address of a linear memory access.
    ///
    /// # Errors
    ///
    /// If the resulting effective address overflows.
    fn effective_address(offset: Offset, address: u32) -> Result<usize, Trap> {
        offset
            .into_inner()
            .checked_add(address)
            .map(|address| address as usize)
            .ok_or(TrapCode::MemoryAccessOutOfBounds)
            .map_err(Into::into)
    }

    /// Loads a value of type `T` from the default memory at the given address offset.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `i32.load`
    /// - `i64.load`
    /// - `f32.load`
    /// - `f64.load`
    fn execute_load<T>(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap>
    where
        UntypedValue: From<T>,
        T: LittleEndianConvert,
    {
        let raw_address = u32::from(top);
        let address = Self::effective_address(offset, raw_address)?;
        let mut bytes = <<T as LittleEndianConvert>::Bytes as Default>::default();
        self.cache
            .default_memory_bytes(self.ctx.as_context_mut())
            .read(address, bytes.as_mut())?;
        let value = <T as LittleEndianConvert>::from_le_bytes(bytes);
        let result = value.into();
        self.next_instr();
        Ok(result)
    }

    /// Loads a vaoue of type `U` from the default memory at the given address offset and extends it into `T`.
    ///
    /// # Note
    ///
    /// This can be used to emuate the following Wasm operands:
    ///
    /// - `i32.load_8s`
    /// - `i32.load_8u`
    /// - `i32.load_16s`
    /// - `i32.load_16u`
    /// - `i64.load_8s`
    /// - `i64.load_8u`
    /// - `i64.load_16s`
    /// - `i64.load_16u`
    /// - `i64.load_32s`
    /// - `i64.load_32u`
    fn execute_load_extend<T, U>(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap>
    where
        T: ExtendInto<U> + LittleEndianConvert,
        UntypedValue: From<U>,
    {
        let raw_address = u32::from(top);
        let address = Self::effective_address(offset, raw_address)?;
        let mut bytes = <<T as LittleEndianConvert>::Bytes as Default>::default();
        self.cache
            .default_memory_bytes(self.ctx.as_context_mut())
            .read(address, bytes.as_mut())?;
        let extended = <T as LittleEndianConvert>::from_le_bytes(bytes).extend_into();
        let result = extended.into();
        self.next_instr();
        Ok(result)
    }

    /// Stores a value of type `T` into the default memory at the given address offset.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `i32.store`
    /// - `i64.store`
    /// - `f32.store`
    /// - `f64.store`
    fn execute_store<T>(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap>
    where
        T: LittleEndianConvert + From<UntypedValue>,
    {
        let stack_value = top.into();
        let raw_address = self.value_stack.pop_as::<u32>();
        let address = Self::effective_address(offset, raw_address)?;
        let bytes = <T as LittleEndianConvert>::into_le_bytes(stack_value);
        self.cache
            .default_memory_bytes(self.ctx.as_context_mut())
            .write(address, bytes.as_ref())?;
        self.next_instr();
        let new_top = self.value_stack.try_pop().unwrap_or_default();
        Ok(new_top)
    }

    /// Stores a value of type `T` wrapped to type `U` into the default memory at the given address offset.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `i32.store8`
    /// - `i32.store16`
    /// - `i64.store8`
    /// - `i64.store16`
    /// - `i64.store32`
    fn execute_store_wrap<T, U>(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap>
    where
        T: WrapInto<U> + From<UntypedValue>,
        U: LittleEndianConvert,
    {
        let wrapped_value = <T>::from(top).wrap_into();
        let raw_address = self.value_stack.pop_as::<u32>();
        let address = Self::effective_address(offset, raw_address)?;
        let bytes = <U as LittleEndianConvert>::into_le_bytes(wrapped_value);
        self.cache
            .default_memory_bytes(self.ctx.as_context_mut())
            .write(address, bytes.as_ref())?;
        self.next_instr();
        let new_top = self.value_stack.try_pop().unwrap_or_default();
        Ok(new_top)
    }

    fn execute_unary(
        &mut self,
        top: UntypedValue,
        f: fn(UntypedValue) -> UntypedValue,
    ) -> UntypedValue {
        let result = f(top);
        self.next_instr();
        result
    }

    fn try_execute_unary(
        &mut self,
        top: UntypedValue,
        f: fn(UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<UntypedValue, Trap> {
        let result = f(top)?;
        self.next_instr();
        Ok(result)
    }

    fn execute_binary(
        &mut self,
        top: UntypedValue,
        f: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> UntypedValue {
        let lhs = self.value_stack.pop();
        let rhs = top;
        let result = f(lhs, rhs);
        self.next_instr();
        result
    }

    fn try_execute_binary(
        &mut self,
        top: UntypedValue,
        f: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<UntypedValue, Trap> {
        let lhs = self.value_stack.pop();
        let rhs = top;
        let result = f(lhs, rhs)?;
        self.next_instr();
        Ok(result)
    }

    fn execute_reinterpret<T, U>(&mut self, top: UntypedValue) -> UntypedValue
    where
        UntypedValue: From<U>,
        T: From<UntypedValue>,
    {
        // Nothing to do for `wasmi` bytecode.
        self.next_instr();
        top
    }

    fn next_instr(&mut self) {
        self.pc += 1;
    }

    fn branch_to(&mut self, top: Option<UntypedValue>, target: Target) {
        if let Some(top) = top {
            self.value_stack.push(top);
        }
        self.value_stack.drop_keep(target.drop_keep());
        self.pc = target.destination_pc().into_usize();
    }

    fn call_func(&mut self, top: Option<UntypedValue>, func: Func) -> Result<CallOutcome, Trap> {
        self.pc += 1;
        self.frame.update_pc(self.pc);
        if let Some(top) = top {
            // We push the inline cached top most value from the value stack
            // back to the original value stack before we conduct the call.
            self.value_stack.push(top);
        }
        Ok(CallOutcome::NestedCall(func))
    }

    fn ret(&mut self, top: Option<UntypedValue>, drop_keep: DropKeep) {
        if let Some(top) = top {
            self.value_stack.push(top);
        }
        self.value_stack.drop_keep(drop_keep)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MaybeReturn {
    Return,
    Continue { new_top: UntypedValue },
}

impl<'engine, 'func, Ctx> ExecutionContext<'engine, 'func, Ctx>
where
    Ctx: AsContextMut,
{
    fn visit_unreachable(&mut self) -> Result<(), Trap> {
        Err(TrapCode::Unreachable).map_err(Into::into)
    }

    fn visit_br(&mut self, top: UntypedValue, target: Target) -> UntypedValue {
        self.branch_to(Some(top), target);
        self.value_stack.try_pop().unwrap_or_default()
    }

    fn visit_br_empty(&mut self, target: Target) {
        self.branch_to(None, target);
    }

    fn visit_br_if_eqz(&mut self, target: Target, top: UntypedValue) -> UntypedValue {
        let condition = bool::from(top);
        let new_top = self.value_stack.try_pop();
        if condition {
            self.next_instr()
        } else {
            self.branch_to(new_top, target)
        }
        new_top.unwrap_or_default()
    }

    fn visit_br_if_nez(&mut self, target: Target, top: UntypedValue) -> UntypedValue {
        let condition = bool::from(top);
        let new_top = self.value_stack.try_pop();
        if condition {
            self.branch_to(new_top, target);
        } else {
            self.next_instr();
        }
        new_top.unwrap_or_default()
    }

    fn visit_return_if_nez(&mut self, drop_keep: DropKeep, top: UntypedValue) -> MaybeReturn {
        let new_top = self.value_stack.try_pop();
        let condition = bool::from(top);
        if condition {
            self.ret(new_top, drop_keep);
            MaybeReturn::Return
        } else {
            self.pc += 1;
            let new_top = new_top.unwrap_or_default();
            MaybeReturn::Continue { new_top }
        }
    }

    fn visit_br_table(&mut self, len_targets: usize, top: UntypedValue) -> UntypedValue {
        let new_top = self.value_stack.try_pop().unwrap_or_default();
        let index = u32::from(top);
        // The index of the default target which is the last target of the slice.
        let max_index = len_targets - 1;
        // A normalized index will always yield a target without panicking.
        let normalized_index = cmp::min(index as usize, max_index);
        // Update `pc`:
        self.pc += normalized_index + 1;
        new_top
    }

    fn visit_return(&mut self, drop_keep: DropKeep, top: UntypedValue) {
        self.ret(Some(top), drop_keep)
    }

    fn visit_return_empty(&mut self, drop_keep: DropKeep) {
        self.ret(None, drop_keep)
    }

    fn visit_local_get(&mut self, local_depth: LocalIdx, top: UntypedValue) -> UntypedValue {
        let local_depth = Self::convert_local_depth(local_depth) - 1;
        let value = self.value_stack.peek(local_depth);
        self.value_stack.push(top);
        self.next_instr();
        value
    }

    fn visit_local_get_empty(&mut self, local_depth: LocalIdx) -> UntypedValue {
        let local_depth = Self::convert_local_depth(local_depth);
        let value = self.value_stack.peek(local_depth);
        self.next_instr();
        value
    }

    fn visit_local_set(&mut self, local_depth: LocalIdx, _top: UntypedValue) -> UntypedValue {
        let local_depth = Self::convert_local_depth(local_depth) - 1;
        let new_value = self.value_stack.pop();
        let new_top = self.value_stack.try_pop().unwrap_or_default();
        *self.value_stack.peek_mut(local_depth) = new_value;
        self.next_instr();
        new_top
    }

    fn visit_local_tee(&mut self, local_depth: LocalIdx, top: UntypedValue) {
        let local_depth = Self::convert_local_depth(local_depth) - 1;
        *self.value_stack.peek_mut(local_depth) = top;
        self.next_instr()
    }

    fn visit_global_get(&mut self, global_index: GlobalIdx, top: UntypedValue) -> UntypedValue {
        let global_value = self.global(global_index).get_untyped(self.ctx.as_context());
        self.value_stack.push(top);
        self.next_instr();
        global_value
    }

    fn visit_global_get_empty(&mut self, global_index: GlobalIdx) -> UntypedValue {
        let global_value = self.global(global_index).get_untyped(self.ctx.as_context());
        self.next_instr();
        global_value
    }

    fn visit_global_set(&mut self, global_index: GlobalIdx, top: UntypedValue) -> UntypedValue {
        let global = self.global(global_index);
        let new_value = top;
        let new_top = self.value_stack.try_pop().unwrap_or_default();
        global.set_untyped(self.ctx.as_context_mut(), new_value);
        self.next_instr();
        new_top
    }

    fn visit_call(&mut self, func_index: FuncIdx, top: UntypedValue) -> Result<CallOutcome, Trap> {
        let callee = self.cache.get_func(&mut self.ctx, func_index.into_inner());
        self.call_func(Some(top), callee)
    }

    fn visit_call_empty(&mut self, func_index: FuncIdx) -> Result<CallOutcome, Trap> {
        let callee = self.cache.get_func(&mut self.ctx, func_index.into_inner());
        self.call_func(None, callee)
    }

    fn visit_call_indirect(
        &mut self,
        signature_index: SignatureIdx,
        top: UntypedValue,
    ) -> Result<CallOutcome, Trap> {
        let func_index: u32 = u32::from(top);
        let table = self.default_table();
        let func = table
            .get(self.ctx.as_context(), func_index as usize)
            .map_err(|_| TrapCode::TableAccessOutOfBounds)?
            .ok_or(TrapCode::ElemUninitialized)?;
        let actual_signature = func.signature(self.ctx.as_context());
        let expected_signature = self
            .frame
            .instance()
            .get_signature(self.ctx.as_context(), signature_index.into_inner())
            .unwrap_or_else(|| {
                panic!(
                    "missing signature for call_indirect at index: {:?}",
                    signature_index,
                )
            });
        if actual_signature != expected_signature {
            return Err(TrapCode::UnexpectedSignature).map_err(Into::into);
        }
        let top = self.value_stack.try_pop();
        self.call_func(top, func)
    }

    fn visit_const(&mut self, bytes: UntypedValue, top: UntypedValue) -> UntypedValue {
        self.value_stack.push(top);
        self.next_instr();
        bytes
    }

    fn visit_const_empty(&mut self, bytes: UntypedValue) -> UntypedValue {
        self.next_instr();
        bytes
    }

    fn visit_drop(&mut self) -> UntypedValue {
        let new_top = self.value_stack.pop();
        self.next_instr();
        new_top
    }

    fn visit_select(&mut self, top: UntypedValue) -> UntypedValue {
        let (condition, if_true) = self.value_stack.pop2();
        let condition = <bool as From<UntypedValue>>::from(condition);
        let if_false = top;
        let result = if condition { if_true } else { if_false };
        self.next_instr();
        result
    }

    fn visit_memory_size(&mut self, top: UntypedValue) -> UntypedValue {
        let memory = self.default_memory();
        let result = (memory.current_pages(self.ctx.as_context()).0 as u32).into();
        self.value_stack.push(top);
        self.next_instr();
        result
    }

    fn visit_memory_size_empty(&mut self) -> UntypedValue {
        let memory = self.default_memory();
        let result = (memory.current_pages(self.ctx.as_context()).0 as u32).into();
        self.next_instr();
        result
    }

    fn visit_memory_grow(&mut self, top: UntypedValue) -> UntypedValue {
        let pages: u32 = top.into();
        let memory = self.default_memory();
        let new_size = match memory.grow(self.ctx.as_context_mut(), Pages(pages as usize)) {
            Ok(Pages(old_size)) => old_size as u32,
            Err(_) => {
                // Note: The WebAssembly spec demands to return `0xFFFF_FFFF`
                //       in case of failure for this instruction.
                u32::MAX
            }
        }
        .into();
        // The memory grow might have invalidated the cached linear memory
        // so we need to reset it in order for the cache to reload in case it
        // is used again.
        self.cache.reset_default_memory_bytes();
        self.next_instr();
        new_size
    }

    fn visit_i32_load(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap> {
        self.execute_load::<i32>(top, offset)
    }

    fn visit_i64_load(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap> {
        self.execute_load::<i64>(top, offset)
    }

    fn visit_f32_load(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap> {
        self.execute_load::<F32>(top, offset)
    }

    fn visit_f64_load(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap> {
        self.execute_load::<F64>(top, offset)
    }

    fn visit_i32_load_i8(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<i8, i32>(top, offset)
    }

    fn visit_i32_load_u8(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<u8, i32>(top, offset)
    }

    fn visit_i32_load_i16(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<i16, i32>(top, offset)
    }

    fn visit_i32_load_u16(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<u16, i32>(top, offset)
    }

    fn visit_i64_load_i8(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<i8, i64>(top, offset)
    }

    fn visit_i64_load_u8(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<u8, i64>(top, offset)
    }

    fn visit_i64_load_i16(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<i16, i64>(top, offset)
    }

    fn visit_i64_load_u16(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<u16, i64>(top, offset)
    }

    fn visit_i64_load_i32(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<i32, i64>(top, offset)
    }

    fn visit_i64_load_u32(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_load_extend::<u32, i64>(top, offset)
    }

    fn visit_i32_store(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap> {
        self.execute_store::<i32>(top, offset)
    }

    fn visit_i64_store(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap> {
        self.execute_store::<i64>(top, offset)
    }

    fn visit_f32_store(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap> {
        self.execute_store::<F32>(top, offset)
    }

    fn visit_f64_store(&mut self, top: UntypedValue, offset: Offset) -> Result<UntypedValue, Trap> {
        self.execute_store::<F64>(top, offset)
    }

    fn visit_i32_store_8(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_store_wrap::<i32, i8>(top, offset)
    }

    fn visit_i32_store_16(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_store_wrap::<i32, i16>(top, offset)
    }

    fn visit_i64_store_8(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_store_wrap::<i64, i8>(top, offset)
    }

    fn visit_i64_store_16(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_store_wrap::<i64, i16>(top, offset)
    }

    fn visit_i64_store_32(
        &mut self,
        top: UntypedValue,
        offset: Offset,
    ) -> Result<UntypedValue, Trap> {
        self.execute_store_wrap::<i64, i32>(top, offset)
    }

    fn visit_i32_eqz(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_eqz)
    }

    fn visit_i32_eq(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_eq)
    }

    fn visit_i32_ne(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_ne)
    }

    fn visit_i32_lt_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_lt_s)
    }

    fn visit_i32_lt_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_lt_u)
    }

    fn visit_i32_gt_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_gt_s)
    }

    fn visit_i32_gt_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_gt_u)
    }

    fn visit_i32_le_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_le_s)
    }

    fn visit_i32_le_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_le_u)
    }

    fn visit_i32_ge_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_ge_s)
    }

    fn visit_i32_ge_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_ge_u)
    }

    fn visit_i64_eqz(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_eqz)
    }

    fn visit_i64_eq(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_eq)
    }

    fn visit_i64_ne(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_ne)
    }

    fn visit_i64_lt_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_lt_s)
    }

    fn visit_i64_lt_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_lt_u)
    }

    fn visit_i64_gt_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_gt_s)
    }

    fn visit_i64_gt_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_gt_u)
    }

    fn visit_i64_le_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_le_s)
    }

    fn visit_i64_le_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_le_u)
    }

    fn visit_i64_ge_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_ge_s)
    }

    fn visit_i64_ge_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_ge_u)
    }

    fn visit_f32_eq(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_eq)
    }

    fn visit_f32_ne(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_ne)
    }

    fn visit_f32_lt(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_lt)
    }

    fn visit_f32_gt(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_gt)
    }

    fn visit_f32_le(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_le)
    }

    fn visit_f32_ge(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_ge)
    }

    fn visit_f64_eq(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_eq)
    }

    fn visit_f64_ne(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_ne)
    }

    fn visit_f64_lt(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_lt)
    }

    fn visit_f64_gt(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_gt)
    }

    fn visit_f64_le(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_le)
    }

    fn visit_f64_ge(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_ge)
    }

    fn visit_i32_clz(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_clz)
    }

    fn visit_i32_ctz(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_ctz)
    }

    fn visit_i32_popcnt(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_popcnt)
    }

    fn visit_i32_add(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_add)
    }

    fn visit_i32_sub(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_sub)
    }

    fn visit_i32_mul(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_mul)
    }

    fn visit_i32_div_s(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::i32_div_s)
    }

    fn visit_i32_div_u(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::i32_div_u)
    }

    fn visit_i32_rem_s(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::i32_rem_s)
    }

    fn visit_i32_rem_u(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::i32_rem_u)
    }

    fn visit_i32_and(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_and)
    }

    fn visit_i32_or(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_or)
    }

    fn visit_i32_xor(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_xor)
    }

    fn visit_i32_shl(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_shl)
    }

    fn visit_i32_shr_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_shr_s)
    }

    fn visit_i32_shr_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_shr_u)
    }

    fn visit_i32_rotl(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_rotl)
    }

    fn visit_i32_rotr(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i32_rotr)
    }

    fn visit_i64_clz(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_clz)
    }

    fn visit_i64_ctz(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_ctz)
    }

    fn visit_i64_popcnt(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_popcnt)
    }

    fn visit_i64_add(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_add)
    }

    fn visit_i64_sub(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_sub)
    }

    fn visit_i64_mul(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_mul)
    }

    fn visit_i64_div_s(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::i64_div_s)
    }

    fn visit_i64_div_u(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::i64_div_u)
    }

    fn visit_i64_rem_s(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::i64_rem_s)
    }

    fn visit_i64_rem_u(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::i64_rem_u)
    }

    fn visit_i64_and(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_and)
    }

    fn visit_i64_or(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_or)
    }

    fn visit_i64_xor(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_xor)
    }

    fn visit_i64_shl(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_shl)
    }

    fn visit_i64_shr_s(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_shr_s)
    }

    fn visit_i64_shr_u(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_shr_u)
    }

    fn visit_i64_rotl(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_rotl)
    }

    fn visit_i64_rotr(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::i64_rotr)
    }

    fn visit_f32_abs(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_abs)
    }

    fn visit_f32_neg(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_neg)
    }

    fn visit_f32_ceil(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_ceil)
    }

    fn visit_f32_floor(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_floor)
    }

    fn visit_f32_trunc(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_trunc)
    }

    fn visit_f32_nearest(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_nearest)
    }

    fn visit_f32_sqrt(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_sqrt)
    }

    fn visit_f32_add(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_add)
    }

    fn visit_f32_sub(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_sub)
    }

    fn visit_f32_mul(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_mul)
    }

    fn visit_f32_div(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::f32_div)
    }

    fn visit_f32_min(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_min)
    }

    fn visit_f32_max(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_max)
    }

    fn visit_f32_copysign(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f32_copysign)
    }

    fn visit_f64_abs(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_abs)
    }

    fn visit_f64_neg(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_neg)
    }

    fn visit_f64_ceil(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_ceil)
    }

    fn visit_f64_floor(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_floor)
    }

    fn visit_f64_trunc(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_trunc)
    }

    fn visit_f64_nearest(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_nearest)
    }

    fn visit_f64_sqrt(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_sqrt)
    }

    fn visit_f64_add(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_add)
    }

    fn visit_f64_sub(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_sub)
    }

    fn visit_f64_mul(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_mul)
    }

    fn visit_f64_div(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_binary(top, UntypedValue::f64_div)
    }

    fn visit_f64_min(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_min)
    }

    fn visit_f64_max(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_max)
    }

    fn visit_f64_copysign(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_binary(top, UntypedValue::f64_copysign)
    }

    fn visit_i32_wrap_i64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_wrap_i64)
    }

    fn visit_i32_trunc_f32(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_unary(top, UntypedValue::i32_trunc_f32_s)
    }

    fn visit_u32_trunc_f32(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_unary(top, UntypedValue::i32_trunc_f32_u)
    }

    fn visit_i32_trunc_f64(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_unary(top, UntypedValue::i32_trunc_f64_s)
    }

    fn visit_u32_trunc_f64(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_unary(top, UntypedValue::i32_trunc_f64_u)
    }

    fn visit_i64_extend_i32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_extend_i32_s)
    }

    fn visit_i64_extend_u32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_extend_i32_u)
    }

    fn visit_i64_trunc_f32(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_unary(top, UntypedValue::i64_trunc_f32_s)
    }

    fn visit_u64_trunc_f32(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_unary(top, UntypedValue::i64_trunc_f32_u)
    }

    fn visit_i64_trunc_f64(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_unary(top, UntypedValue::i64_trunc_f64_s)
    }

    fn visit_u64_trunc_f64(&mut self, top: UntypedValue) -> Result<UntypedValue, Trap> {
        self.try_execute_unary(top, UntypedValue::i64_trunc_f64_u)
    }

    fn visit_f32_convert_i32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_convert_i32_s)
    }

    fn visit_f32_convert_u32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_convert_i32_u)
    }

    fn visit_f32_convert_i64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_convert_i64_s)
    }

    fn visit_f32_convert_u64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_convert_i64_u)
    }

    fn visit_f32_demote_f64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f32_demote_f64)
    }

    fn visit_f64_convert_i32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_convert_i32_s)
    }

    fn visit_f64_convert_u32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_convert_i32_u)
    }

    fn visit_f64_convert_i64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_convert_i64_s)
    }

    fn visit_f64_convert_u64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_convert_i64_u)
    }

    fn visit_f64_promote_f32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::f64_promote_f32)
    }

    fn visit_i32_reinterpret_f32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_reinterpret::<F32, i32>(top)
    }

    fn visit_i64_reinterpret_f64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_reinterpret::<F64, i64>(top)
    }

    fn visit_f32_reinterpret_i32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_reinterpret::<i32, F32>(top)
    }

    fn visit_f64_reinterpret_i64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_reinterpret::<i64, F64>(top)
    }

    fn visit_i32_sign_extend8(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_extend8_s)
    }

    fn visit_i32_sign_extend16(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_extend16_s)
    }

    fn visit_i64_sign_extend8(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_extend8_s)
    }

    fn visit_i64_sign_extend16(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_extend16_s)
    }

    fn visit_i64_sign_extend32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_extend32_s)
    }

    fn visit_i32_trunc_sat_f32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_trunc_sat_f32_s)
    }

    fn visit_u32_trunc_sat_f32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_trunc_sat_f32_u)
    }

    fn visit_i32_trunc_sat_f64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_trunc_sat_f64_s)
    }

    fn visit_u32_trunc_sat_f64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i32_trunc_sat_f64_u)
    }

    fn visit_i64_trunc_sat_f32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_trunc_sat_f32_s)
    }

    fn visit_u64_trunc_sat_f32(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_trunc_sat_f32_u)
    }

    fn visit_i64_trunc_sat_f64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_trunc_sat_f64_s)
    }

    fn visit_u64_trunc_sat_f64(&mut self, top: UntypedValue) -> UntypedValue {
        self.execute_unary(top, UntypedValue::i64_trunc_sat_f64_u)
    }
}
