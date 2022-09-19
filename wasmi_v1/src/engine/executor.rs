use super::{
    super::{Memory, Table},
    bytecode::{FuncIdx, GlobalIdx, Instruction, LocalDepth, Offset, SignatureIdx},
    cache::InstanceCache,
    code_map::Instructions,
    stack::Stack,
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

/// Executes the given function `frame`.
///
/// # Note
///
/// This executes instructions sequentially until either the function
/// calls into another function or the function returns to its caller.
///
/// # Errors
///
/// - If the execution of the function `frame` trapped.
#[inline(always)]
pub fn execute_frame<'engine>(
    ctx: impl AsContextMut,
    frame: FuncFrame,
    cache: &mut InstanceCache,
    insts: Instructions<'engine>,
    value_stack: &'engine mut Stack,
) -> Result<CallOutcome, Trap> {
    Executor::new(ctx, frame, cache, insts, value_stack).execute()
}

/// An execution context for executing a `wasmi` function frame.
#[derive(Debug)]
struct Executor<'engine, Ctx> {
    /// The function frame that is being executed.
    ///
    /// This frequently changes when calling a function or returning from one.
    frame: FuncFrame,
    /// The stack required for driving the execution.
    ///
    /// This hosts the value stack as well as the call stack.
    value_stack: &'engine mut ValueStack,
    /// Stores frequently used instance related data.
    cache: &'engine mut InstanceCache,
    /// A mutable [`Store`] context.
    ///
    /// [`Store`]: [`crate::v1::Store`]
    ctx: Ctx,
    /// The instructions of the executed function frame.
    insts: Instructions<'engine>,
}

impl<'engine, Ctx> Executor<'engine, Ctx>
where
    Ctx: AsContextMut,
{
    /// Creates a new [`Executor`] for executing a `wasmi` function frame.
    #[inline(always)]
    pub fn new(
        ctx: Ctx,
        frame: FuncFrame,
        cache: &'engine mut InstanceCache,
        insts: Instructions<'engine>,
        value_stack: &'engine mut ValueStack,
    ) -> Self {
        cache.update_instance(frame.instance());
        Self {
            value_stack,
            frame,
            cache,
            ctx,
            insts,
        }
    }

    /// Executes the function frame until it returns or traps.
    #[inline(always)]
    fn execute(mut self) -> Result<CallOutcome, Trap> {
        use Instruction as Instr;
        loop {
            match *self.instr() {
                Instr::LocalGet { local_depth } => self.visit_local_get(local_depth),
                Instr::LocalSet { local_depth } => self.visit_local_set(local_depth),
                Instr::LocalTee { local_depth } => self.visit_local_tee(local_depth),
                Instr::Br(target) => self.visit_br(target),
                Instr::BrIfEqz(target) => self.visit_br_if_eqz(target),
                Instr::BrIfNez(target) => self.visit_br_if_nez(target),
                Instr::ReturnIfNez(drop_keep) => {
                    if let MaybeReturn::Return = self.visit_return_if_nez(drop_keep) {
                        return Ok(CallOutcome::Return);
                    }
                }
                Instr::BrTable { len_targets } => self.visit_br_table(len_targets),
                Instr::Unreachable => self.visit_unreachable()?,
                Instr::Return(drop_keep) => return self.visit_ret(drop_keep),
                Instr::Call(func) => return self.visit_call(func),
                Instr::CallIndirect(signature) => return self.visit_call_indirect(signature),
                Instr::Drop => self.visit_drop(),
                Instr::Select => self.visit_select(),
                Instr::GlobalGet(global_idx) => self.visit_global_get(global_idx),
                Instr::GlobalSet(global_idx) => self.visit_global_set(global_idx),
                Instr::I32Load(offset) => self.visit_i32_load(offset)?,
                Instr::I64Load(offset) => self.visit_i64_load(offset)?,
                Instr::F32Load(offset) => self.visit_f32_load(offset)?,
                Instr::F64Load(offset) => self.visit_f64_load(offset)?,
                Instr::I32Load8S(offset) => self.visit_i32_load_i8(offset)?,
                Instr::I32Load8U(offset) => self.visit_i32_load_u8(offset)?,
                Instr::I32Load16S(offset) => self.visit_i32_load_i16(offset)?,
                Instr::I32Load16U(offset) => self.visit_i32_load_u16(offset)?,
                Instr::I64Load8S(offset) => self.visit_i64_load_i8(offset)?,
                Instr::I64Load8U(offset) => self.visit_i64_load_u8(offset)?,
                Instr::I64Load16S(offset) => self.visit_i64_load_i16(offset)?,
                Instr::I64Load16U(offset) => self.visit_i64_load_u16(offset)?,
                Instr::I64Load32S(offset) => self.visit_i64_load_i32(offset)?,
                Instr::I64Load32U(offset) => self.visit_i64_load_u32(offset)?,
                Instr::I32Store(offset) => self.visit_i32_store(offset)?,
                Instr::I64Store(offset) => self.visit_i64_store(offset)?,
                Instr::F32Store(offset) => self.visit_f32_store(offset)?,
                Instr::F64Store(offset) => self.visit_f64_store(offset)?,
                Instr::I32Store8(offset) => self.visit_i32_store_8(offset)?,
                Instr::I32Store16(offset) => self.visit_i32_store_16(offset)?,
                Instr::I64Store8(offset) => self.visit_i64_store_8(offset)?,
                Instr::I64Store16(offset) => self.visit_i64_store_16(offset)?,
                Instr::I64Store32(offset) => self.visit_i64_store_32(offset)?,
                Instr::MemorySize => self.visit_current_memory(),
                Instr::MemoryGrow => self.visit_grow_memory(),
                Instr::Const(bytes) => self.visit_const(bytes),
                Instr::I32Eqz => self.visit_i32_eqz(),
                Instr::I32Eq => self.visit_i32_eq(),
                Instr::I32Ne => self.visit_i32_ne(),
                Instr::I32LtS => self.visit_i32_lt_s(),
                Instr::I32LtU => self.visit_i32_lt_u(),
                Instr::I32GtS => self.visit_i32_gt_s(),
                Instr::I32GtU => self.visit_i32_gt_u(),
                Instr::I32LeS => self.visit_i32_le_s(),
                Instr::I32LeU => self.visit_i32_le_u(),
                Instr::I32GeS => self.visit_i32_ge_s(),
                Instr::I32GeU => self.visit_i32_ge_u(),
                Instr::I64Eqz => self.visit_i64_eqz(),
                Instr::I64Eq => self.visit_i64_eq(),
                Instr::I64Ne => self.visit_i64_ne(),
                Instr::I64LtS => self.visit_i64_lt_s(),
                Instr::I64LtU => self.visit_i64_lt_u(),
                Instr::I64GtS => self.visit_i64_gt_s(),
                Instr::I64GtU => self.visit_i64_gt_u(),
                Instr::I64LeS => self.visit_i64_le_s(),
                Instr::I64LeU => self.visit_i64_le_u(),
                Instr::I64GeS => self.visit_i64_ge_s(),
                Instr::I64GeU => self.visit_i64_ge_u(),
                Instr::F32Eq => self.visit_f32_eq(),
                Instr::F32Ne => self.visit_f32_ne(),
                Instr::F32Lt => self.visit_f32_lt(),
                Instr::F32Gt => self.visit_f32_gt(),
                Instr::F32Le => self.visit_f32_le(),
                Instr::F32Ge => self.visit_f32_ge(),
                Instr::F64Eq => self.visit_f64_eq(),
                Instr::F64Ne => self.visit_f64_ne(),
                Instr::F64Lt => self.visit_f64_lt(),
                Instr::F64Gt => self.visit_f64_gt(),
                Instr::F64Le => self.visit_f64_le(),
                Instr::F64Ge => self.visit_f64_ge(),
                Instr::I32Clz => self.visit_i32_clz(),
                Instr::I32Ctz => self.visit_i32_ctz(),
                Instr::I32Popcnt => self.visit_i32_popcnt(),
                Instr::I32Add => self.visit_i32_add(),
                Instr::I32Sub => self.visit_i32_sub(),
                Instr::I32Mul => self.visit_i32_mul(),
                Instr::I32DivS => self.visit_i32_div_s()?,
                Instr::I32DivU => self.visit_i32_div_u()?,
                Instr::I32RemS => self.visit_i32_rem_s()?,
                Instr::I32RemU => self.visit_i32_rem_u()?,
                Instr::I32And => self.visit_i32_and(),
                Instr::I32Or => self.visit_i32_or(),
                Instr::I32Xor => self.visit_i32_xor(),
                Instr::I32Shl => self.visit_i32_shl(),
                Instr::I32ShrS => self.visit_i32_shr_s(),
                Instr::I32ShrU => self.visit_i32_shr_u(),
                Instr::I32Rotl => self.visit_i32_rotl(),
                Instr::I32Rotr => self.visit_i32_rotr(),
                Instr::I64Clz => self.visit_i64_clz(),
                Instr::I64Ctz => self.visit_i64_ctz(),
                Instr::I64Popcnt => self.visit_i64_popcnt(),
                Instr::I64Add => self.visit_i64_add(),
                Instr::I64Sub => self.visit_i64_sub(),
                Instr::I64Mul => self.visit_i64_mul(),
                Instr::I64DivS => self.visit_i64_div_s()?,
                Instr::I64DivU => self.visit_i64_div_u()?,
                Instr::I64RemS => self.visit_i64_rem_s()?,
                Instr::I64RemU => self.visit_i64_rem_u()?,
                Instr::I64And => self.visit_i64_and(),
                Instr::I64Or => self.visit_i64_or(),
                Instr::I64Xor => self.visit_i64_xor(),
                Instr::I64Shl => self.visit_i64_shl(),
                Instr::I64ShrS => self.visit_i64_shr_s(),
                Instr::I64ShrU => self.visit_i64_shr_u(),
                Instr::I64Rotl => self.visit_i64_rotl(),
                Instr::I64Rotr => self.visit_i64_rotr(),
                Instr::F32Abs => self.visit_f32_abs(),
                Instr::F32Neg => self.visit_f32_neg(),
                Instr::F32Ceil => self.visit_f32_ceil(),
                Instr::F32Floor => self.visit_f32_floor(),
                Instr::F32Trunc => self.visit_f32_trunc(),
                Instr::F32Nearest => self.visit_f32_nearest(),
                Instr::F32Sqrt => self.visit_f32_sqrt(),
                Instr::F32Add => self.visit_f32_add(),
                Instr::F32Sub => self.visit_f32_sub(),
                Instr::F32Mul => self.visit_f32_mul(),
                Instr::F32Div => self.visit_f32_div(),
                Instr::F32Min => self.visit_f32_min(),
                Instr::F32Max => self.visit_f32_max(),
                Instr::F32Copysign => self.visit_f32_copysign(),
                Instr::F64Abs => self.visit_f64_abs(),
                Instr::F64Neg => self.visit_f64_neg(),
                Instr::F64Ceil => self.visit_f64_ceil(),
                Instr::F64Floor => self.visit_f64_floor(),
                Instr::F64Trunc => self.visit_f64_trunc(),
                Instr::F64Nearest => self.visit_f64_nearest(),
                Instr::F64Sqrt => self.visit_f64_sqrt(),
                Instr::F64Add => self.visit_f64_add(),
                Instr::F64Sub => self.visit_f64_sub(),
                Instr::F64Mul => self.visit_f64_mul(),
                Instr::F64Div => self.visit_f64_div(),
                Instr::F64Min => self.visit_f64_min(),
                Instr::F64Max => self.visit_f64_max(),
                Instr::F64Copysign => self.visit_f64_copysign(),
                Instr::I32WrapI64 => self.visit_i32_wrap_i64(),
                Instr::I32TruncSF32 => self.visit_i32_trunc_f32()?,
                Instr::I32TruncUF32 => self.visit_u32_trunc_f32()?,
                Instr::I32TruncSF64 => self.visit_i32_trunc_f64()?,
                Instr::I32TruncUF64 => self.visit_u32_trunc_f64()?,
                Instr::I64ExtendSI32 => self.visit_i64_extend_i32(),
                Instr::I64ExtendUI32 => self.visit_i64_extend_u32(),
                Instr::I64TruncSF32 => self.visit_i64_trunc_f32()?,
                Instr::I64TruncUF32 => self.visit_u64_trunc_f32()?,
                Instr::I64TruncSF64 => self.visit_i64_trunc_f64()?,
                Instr::I64TruncUF64 => self.visit_u64_trunc_f64()?,
                Instr::F32ConvertSI32 => self.visit_f32_convert_i32(),
                Instr::F32ConvertUI32 => self.visit_f32_convert_u32(),
                Instr::F32ConvertSI64 => self.visit_f32_convert_i64(),
                Instr::F32ConvertUI64 => self.visit_f32_convert_u64(),
                Instr::F32DemoteF64 => self.visit_f32_demote_f64(),
                Instr::F64ConvertSI32 => self.visit_f64_convert_i32(),
                Instr::F64ConvertUI32 => self.visit_f64_convert_u32(),
                Instr::F64ConvertSI64 => self.visit_f64_convert_i64(),
                Instr::F64ConvertUI64 => self.visit_f64_convert_u64(),
                Instr::F64PromoteF32 => self.visit_f64_promote_f32(),
                Instr::I32ReinterpretF32 => self.visit_i32_reinterpret_f32(),
                Instr::I64ReinterpretF64 => self.visit_i64_reinterpret_f64(),
                Instr::F32ReinterpretI32 => self.visit_f32_reinterpret_i32(),
                Instr::F64ReinterpretI64 => self.visit_f64_reinterpret_i64(),
                Instr::I32TruncSatF32S => self.visit_i32_trunc_sat_f32(),
                Instr::I32TruncSatF32U => self.visit_u32_trunc_sat_f32(),
                Instr::I32TruncSatF64S => self.visit_i32_trunc_sat_f64(),
                Instr::I32TruncSatF64U => self.visit_u32_trunc_sat_f64(),
                Instr::I64TruncSatF32S => self.visit_i64_trunc_sat_f32(),
                Instr::I64TruncSatF32U => self.visit_u64_trunc_sat_f32(),
                Instr::I64TruncSatF64S => self.visit_i64_trunc_sat_f64(),
                Instr::I64TruncSatF64U => self.visit_u64_trunc_sat_f64(),
                Instr::I32Extend8S => self.visit_i32_sign_extend8(),
                Instr::I32Extend16S => self.visit_i32_sign_extend16(),
                Instr::I64Extend8S => self.visit_i64_sign_extend8(),
                Instr::I64Extend16S => self.visit_i64_sign_extend16(),
                Instr::I64Extend32S => self.visit_i64_sign_extend32(),
            }
        }
    }

    /// Returns the [`Instruction`] at the current program counter.
    #[inline(always)]
    fn instr(&self) -> &'engine Instruction {
        // # Safety
        //
        // Properly constructed `wasmi` bytecode can never produce invalid `pc`.
        unsafe { self.insts.get_release_unchecked(self.frame.pc()) }
    }

    /// Returns the default linear memory.
    ///
    /// # Panics
    ///
    /// If there exists is no linear memory for the instance.
    #[inline]
    fn default_memory(&mut self) -> Memory {
        self.cache.default_memory(self.ctx.as_context())
    }

    /// Returns the default table.
    ///
    /// # Panics
    ///
    /// If there exists is no table for the instance.
    #[inline]
    fn default_table(&mut self) -> Table {
        self.cache.default_table(self.ctx.as_context())
    }

    /// Returns the global variable at the given index.
    ///
    /// # Panics
    ///
    /// If there is no global variable at the given index.
    fn global(&mut self, global_index: GlobalIdx) -> &mut UntypedValue {
        self.cache
            .get_global(self.ctx.as_context_mut(), global_index.into_inner())
    }

    /// Calculates the effective address of a linear memory access.
    ///
    /// # Errors
    ///
    /// If the resulting effective address overflows.
    fn effective_address(offset: Offset, address: u32) -> Result<usize, TrapCode> {
        offset
            .into_inner()
            .checked_add(address)
            .map(|address| address as usize)
            .ok_or(TrapCode::MemoryAccessOutOfBounds)
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
    fn execute_load<T>(&mut self, offset: Offset) -> Result<(), Trap>
    where
        UntypedValue: From<T>,
        T: LittleEndianConvert,
    {
        self.value_stack.try_eval_top(|address| {
            let raw_address = u32::from(address);
            let address = Self::effective_address(offset, raw_address)?;
            let mut bytes = <<T as LittleEndianConvert>::Bytes as Default>::default();
            self.cache
                .default_memory_bytes(self.ctx.as_context_mut())
                .read(address, bytes.as_mut())?;
            let value = <T as LittleEndianConvert>::from_le_bytes(bytes);
            Ok(value.into())
        })?;
        self.next_instr();
        Ok(())
    }

    /// Loads a value of type `U` from the default memory at the given address offset and extends it into `T`.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
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
    fn execute_load_extend<T, U>(&mut self, offset: Offset) -> Result<(), Trap>
    where
        T: ExtendInto<U> + LittleEndianConvert,
        UntypedValue: From<U>,
    {
        self.value_stack.try_eval_top(|address| {
            let raw_address = u32::from(address);
            let address = Self::effective_address(offset, raw_address)?;
            let mut bytes = <<T as LittleEndianConvert>::Bytes as Default>::default();
            self.cache
                .default_memory_bytes(self.ctx.as_context_mut())
                .read(address, bytes.as_mut())?;
            let value = <T as LittleEndianConvert>::from_le_bytes(bytes).extend_into();
            Ok(value.into())
        })?;
        self.next_instr();
        Ok(())
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
    fn execute_store<T>(&mut self, offset: Offset) -> Result<(), Trap>
    where
        T: LittleEndianConvert + From<UntypedValue>,
    {
        let (address, value) = self.value_stack.pop2();
        let value = T::from(value);
        let address = Self::effective_address(offset, u32::from(address))?;
        let bytes = <T as LittleEndianConvert>::into_le_bytes(value);
        self.cache
            .default_memory_bytes(self.ctx.as_context_mut())
            .write(address, bytes.as_ref())?;
        self.next_instr();
        Ok(())
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
    fn execute_store_wrap<T, U>(&mut self, offset: Offset) -> Result<(), Trap>
    where
        T: WrapInto<U> + From<UntypedValue>,
        U: LittleEndianConvert,
    {
        let (address, value) = self.value_stack.pop2();
        let wrapped_value = T::from(value).wrap_into();
        let address = Self::effective_address(offset, u32::from(address))?;
        let bytes = <U as LittleEndianConvert>::into_le_bytes(wrapped_value);
        self.cache
            .default_memory_bytes(self.ctx.as_context_mut())
            .write(address, bytes.as_ref())?;
        self.next_instr();
        Ok(())
    }

    fn execute_unary(&mut self, f: fn(UntypedValue) -> UntypedValue) {
        self.value_stack.eval_top(f);
        self.next_instr()
    }

    fn try_execute_unary(
        &mut self,
        f: fn(UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), Trap> {
        self.value_stack.try_eval_top(f)?;
        self.try_next_instr()
    }

    fn execute_binary(&mut self, f: fn(UntypedValue, UntypedValue) -> UntypedValue) {
        self.value_stack.eval_top2(f);
        self.next_instr()
    }

    fn try_execute_binary(
        &mut self,
        f: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), Trap> {
        self.value_stack.try_eval_top2(f)?;
        self.try_next_instr()
    }

    fn execute_reinterpret<T, U>(&mut self)
    where
        UntypedValue: From<U>,
        T: From<UntypedValue>,
    {
        // Nothing to do for `wasmi` bytecode.
        self.next_instr()
    }

    fn try_next_instr(&mut self) -> Result<(), Trap> {
        self.frame.bump_pc();
        Ok(())
    }

    fn next_instr(&mut self) {
        self.frame.bump_pc();
    }

    fn branch_to(&mut self, target: Target) {
        self.value_stack.drop_keep(target.drop_keep());
        self.frame.update_pc(target.destination_pc().into_usize());
    }

    fn call_func(&mut self, func: Func) -> Result<CallOutcome, Trap> {
        self.frame.bump_pc();
        Ok(CallOutcome::NestedCall(func))
    }

    fn ret(&mut self, drop_keep: DropKeep) {
        self.value_stack.drop_keep(drop_keep)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MaybeReturn {
    Return,
    Continue,
}

impl<'engine, Ctx> Executor<'engine, Ctx>
where
    Ctx: AsContextMut,
{
    fn visit_unreachable(&mut self) -> Result<(), Trap> {
        Err(TrapCode::Unreachable).map_err(Into::into)
    }

    fn visit_br(&mut self, target: Target) {
        self.branch_to(target)
    }

    fn visit_br_if_eqz(&mut self, target: Target) {
        let condition = self.value_stack.pop_as();
        if condition {
            self.next_instr()
        } else {
            self.branch_to(target)
        }
    }

    fn visit_br_if_nez(&mut self, target: Target) {
        let condition = self.value_stack.pop_as();
        if condition {
            self.branch_to(target)
        } else {
            self.next_instr()
        }
    }

    fn visit_return_if_nez(&mut self, drop_keep: DropKeep) -> MaybeReturn {
        let condition = self.value_stack.pop_as();
        if condition {
            self.ret(drop_keep);
            MaybeReturn::Return
        } else {
            self.next_instr();
            MaybeReturn::Continue
        }
    }

    fn visit_br_table(&mut self, len_targets: usize) {
        let index: u32 = self.value_stack.pop_as();
        // The index of the default target which is the last target of the slice.
        let max_index = len_targets - 1;
        // A normalized index will always yield a target without panicking.
        let normalized_index = cmp::min(index as usize, max_index);
        // Update `pc`:
        self.frame.bump_pc_by(normalized_index + 1)
    }

    fn visit_ret(&mut self, drop_keep: DropKeep) -> Result<CallOutcome, Trap> {
        self.ret(drop_keep);
        Ok(CallOutcome::Return)
    }

    fn visit_local_get(&mut self, local_depth: LocalDepth) {
        let value = self.value_stack.peek(local_depth.into_inner());
        self.value_stack.push(value);
        self.next_instr()
    }

    fn visit_local_set(&mut self, local_depth: LocalDepth) {
        let new_value = self.value_stack.pop();
        *self.value_stack.peek_mut(local_depth.into_inner()) = new_value;
        self.next_instr()
    }

    fn visit_local_tee(&mut self, local_depth: LocalDepth) {
        let new_value = self.value_stack.last();
        *self.value_stack.peek_mut(local_depth.into_inner()) = new_value;
        self.next_instr()
    }

    fn visit_global_get(&mut self, global_index: GlobalIdx) {
        let global_value = *self.global(global_index);
        self.value_stack.push(global_value);
        self.next_instr()
    }

    fn visit_global_set(&mut self, global_index: GlobalIdx) {
        let new_value = self.value_stack.pop();
        *self.global(global_index) = new_value;
        self.next_instr()
    }

    fn visit_call(&mut self, func_index: FuncIdx) -> Result<CallOutcome, Trap> {
        let callee = self
            .cache
            .get_func(self.ctx.as_context_mut(), func_index.into_inner());
        self.call_func(callee)
    }

    fn visit_call_indirect(&mut self, signature_index: SignatureIdx) -> Result<CallOutcome, Trap> {
        let func_index: u32 = self.value_stack.pop_as();
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
        self.call_func(func)
    }

    fn visit_const(&mut self, bytes: UntypedValue) {
        self.value_stack.push(bytes);
        self.next_instr()
    }

    fn visit_drop(&mut self) {
        let _ = self.value_stack.pop();
        self.next_instr()
    }

    fn visit_select(&mut self) {
        self.value_stack.pop2_eval(|e1, e2, e3| {
            let condition = <bool as From<UntypedValue>>::from(e3);
            let result = if condition { *e1 } else { e2 };
            *e1 = result;
        });
        self.next_instr()
    }

    fn visit_current_memory(&mut self) {
        let memory = self.default_memory();
        let result = memory.current_pages(self.ctx.as_context()).0 as u32;
        self.value_stack.push(result);
        self.next_instr()
    }

    fn visit_grow_memory(&mut self) {
        let pages: u32 = self.value_stack.pop_as();
        let memory = self.default_memory();
        let new_size = match memory.grow(self.ctx.as_context_mut(), Pages(pages as usize)) {
            Ok(Pages(old_size)) => old_size as u32,
            Err(_) => {
                // Note: The WebAssembly spec demands to return `0xFFFF_FFFF`
                //       in case of failure for this instruction.
                u32::MAX
            }
        };
        // The memory grow might have invalidated the cached linear memory
        // so we need to reset it in order for the cache to reload in case it
        // is used again.
        self.cache.reset_default_memory_bytes();
        self.value_stack.push(new_size);
        self.next_instr()
    }

    fn visit_i32_load(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load::<i32>(offset)
    }

    fn visit_i64_load(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load::<i64>(offset)
    }

    fn visit_f32_load(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load::<F32>(offset)
    }

    fn visit_f64_load(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load::<F64>(offset)
    }

    fn visit_i32_load_i8(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<i8, i32>(offset)
    }

    fn visit_i32_load_u8(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<u8, i32>(offset)
    }

    fn visit_i32_load_i16(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<i16, i32>(offset)
    }

    fn visit_i32_load_u16(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<u16, i32>(offset)
    }

    fn visit_i64_load_i8(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<i8, i64>(offset)
    }

    fn visit_i64_load_u8(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<u8, i64>(offset)
    }

    fn visit_i64_load_i16(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<i16, i64>(offset)
    }

    fn visit_i64_load_u16(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<u16, i64>(offset)
    }

    fn visit_i64_load_i32(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<i32, i64>(offset)
    }

    fn visit_i64_load_u32(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_load_extend::<u32, i64>(offset)
    }

    fn visit_i32_store(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store::<i32>(offset)
    }

    fn visit_i64_store(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store::<i64>(offset)
    }

    fn visit_f32_store(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store::<F32>(offset)
    }

    fn visit_f64_store(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store::<F64>(offset)
    }

    fn visit_i32_store_8(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store_wrap::<i32, i8>(offset)
    }

    fn visit_i32_store_16(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store_wrap::<i32, i16>(offset)
    }

    fn visit_i64_store_8(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store_wrap::<i64, i8>(offset)
    }

    fn visit_i64_store_16(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store_wrap::<i64, i16>(offset)
    }

    fn visit_i64_store_32(&mut self, offset: Offset) -> Result<(), Trap> {
        self.execute_store_wrap::<i64, i32>(offset)
    }

    fn visit_i32_eqz(&mut self) {
        self.execute_unary(UntypedValue::i32_eqz)
    }

    fn visit_i32_eq(&mut self) {
        self.execute_binary(UntypedValue::i32_eq)
    }

    fn visit_i32_ne(&mut self) {
        self.execute_binary(UntypedValue::i32_ne)
    }

    fn visit_i32_lt_s(&mut self) {
        self.execute_binary(UntypedValue::i32_lt_s)
    }

    fn visit_i32_lt_u(&mut self) {
        self.execute_binary(UntypedValue::i32_lt_u)
    }

    fn visit_i32_gt_s(&mut self) {
        self.execute_binary(UntypedValue::i32_gt_s)
    }

    fn visit_i32_gt_u(&mut self) {
        self.execute_binary(UntypedValue::i32_gt_u)
    }

    fn visit_i32_le_s(&mut self) {
        self.execute_binary(UntypedValue::i32_le_s)
    }

    fn visit_i32_le_u(&mut self) {
        self.execute_binary(UntypedValue::i32_le_u)
    }

    fn visit_i32_ge_s(&mut self) {
        self.execute_binary(UntypedValue::i32_ge_s)
    }

    fn visit_i32_ge_u(&mut self) {
        self.execute_binary(UntypedValue::i32_ge_u)
    }

    fn visit_i64_eqz(&mut self) {
        self.execute_unary(UntypedValue::i64_eqz)
    }

    fn visit_i64_eq(&mut self) {
        self.execute_binary(UntypedValue::i64_eq)
    }

    fn visit_i64_ne(&mut self) {
        self.execute_binary(UntypedValue::i64_ne)
    }

    fn visit_i64_lt_s(&mut self) {
        self.execute_binary(UntypedValue::i64_lt_s)
    }

    fn visit_i64_lt_u(&mut self) {
        self.execute_binary(UntypedValue::i64_lt_u)
    }

    fn visit_i64_gt_s(&mut self) {
        self.execute_binary(UntypedValue::i64_gt_s)
    }

    fn visit_i64_gt_u(&mut self) {
        self.execute_binary(UntypedValue::i64_gt_u)
    }

    fn visit_i64_le_s(&mut self) {
        self.execute_binary(UntypedValue::i64_le_s)
    }

    fn visit_i64_le_u(&mut self) {
        self.execute_binary(UntypedValue::i64_le_u)
    }

    fn visit_i64_ge_s(&mut self) {
        self.execute_binary(UntypedValue::i64_ge_s)
    }

    fn visit_i64_ge_u(&mut self) {
        self.execute_binary(UntypedValue::i64_ge_u)
    }

    fn visit_f32_eq(&mut self) {
        self.execute_binary(UntypedValue::f32_eq)
    }

    fn visit_f32_ne(&mut self) {
        self.execute_binary(UntypedValue::f32_ne)
    }

    fn visit_f32_lt(&mut self) {
        self.execute_binary(UntypedValue::f32_lt)
    }

    fn visit_f32_gt(&mut self) {
        self.execute_binary(UntypedValue::f32_gt)
    }

    fn visit_f32_le(&mut self) {
        self.execute_binary(UntypedValue::f32_le)
    }

    fn visit_f32_ge(&mut self) {
        self.execute_binary(UntypedValue::f32_ge)
    }

    fn visit_f64_eq(&mut self) {
        self.execute_binary(UntypedValue::f64_eq)
    }

    fn visit_f64_ne(&mut self) {
        self.execute_binary(UntypedValue::f64_ne)
    }

    fn visit_f64_lt(&mut self) {
        self.execute_binary(UntypedValue::f64_lt)
    }

    fn visit_f64_gt(&mut self) {
        self.execute_binary(UntypedValue::f64_gt)
    }

    fn visit_f64_le(&mut self) {
        self.execute_binary(UntypedValue::f64_le)
    }

    fn visit_f64_ge(&mut self) {
        self.execute_binary(UntypedValue::f64_ge)
    }

    fn visit_i32_clz(&mut self) {
        self.execute_unary(UntypedValue::i32_clz)
    }

    fn visit_i32_ctz(&mut self) {
        self.execute_unary(UntypedValue::i32_ctz)
    }

    fn visit_i32_popcnt(&mut self) {
        self.execute_unary(UntypedValue::i32_popcnt)
    }

    fn visit_i32_add(&mut self) {
        self.execute_binary(UntypedValue::i32_add)
    }

    fn visit_i32_sub(&mut self) {
        self.execute_binary(UntypedValue::i32_sub)
    }

    fn visit_i32_mul(&mut self) {
        self.execute_binary(UntypedValue::i32_mul)
    }

    fn visit_i32_div_s(&mut self) -> Result<(), Trap> {
        self.try_execute_binary(UntypedValue::i32_div_s)
    }

    fn visit_i32_div_u(&mut self) -> Result<(), Trap> {
        self.try_execute_binary(UntypedValue::i32_div_u)
    }

    fn visit_i32_rem_s(&mut self) -> Result<(), Trap> {
        self.try_execute_binary(UntypedValue::i32_rem_s)
    }

    fn visit_i32_rem_u(&mut self) -> Result<(), Trap> {
        self.try_execute_binary(UntypedValue::i32_rem_u)
    }

    fn visit_i32_and(&mut self) {
        self.execute_binary(UntypedValue::i32_and)
    }

    fn visit_i32_or(&mut self) {
        self.execute_binary(UntypedValue::i32_or)
    }

    fn visit_i32_xor(&mut self) {
        self.execute_binary(UntypedValue::i32_xor)
    }

    fn visit_i32_shl(&mut self) {
        self.execute_binary(UntypedValue::i32_shl)
    }

    fn visit_i32_shr_s(&mut self) {
        self.execute_binary(UntypedValue::i32_shr_s)
    }

    fn visit_i32_shr_u(&mut self) {
        self.execute_binary(UntypedValue::i32_shr_u)
    }

    fn visit_i32_rotl(&mut self) {
        self.execute_binary(UntypedValue::i32_rotl)
    }

    fn visit_i32_rotr(&mut self) {
        self.execute_binary(UntypedValue::i32_rotr)
    }

    fn visit_i64_clz(&mut self) {
        self.execute_unary(UntypedValue::i64_clz)
    }

    fn visit_i64_ctz(&mut self) {
        self.execute_unary(UntypedValue::i64_ctz)
    }

    fn visit_i64_popcnt(&mut self) {
        self.execute_unary(UntypedValue::i64_popcnt)
    }

    fn visit_i64_add(&mut self) {
        self.execute_binary(UntypedValue::i64_add)
    }

    fn visit_i64_sub(&mut self) {
        self.execute_binary(UntypedValue::i64_sub)
    }

    fn visit_i64_mul(&mut self) {
        self.execute_binary(UntypedValue::i64_mul)
    }

    fn visit_i64_div_s(&mut self) -> Result<(), Trap> {
        self.try_execute_binary(UntypedValue::i64_div_s)
    }

    fn visit_i64_div_u(&mut self) -> Result<(), Trap> {
        self.try_execute_binary(UntypedValue::i64_div_u)
    }

    fn visit_i64_rem_s(&mut self) -> Result<(), Trap> {
        self.try_execute_binary(UntypedValue::i64_rem_s)
    }

    fn visit_i64_rem_u(&mut self) -> Result<(), Trap> {
        self.try_execute_binary(UntypedValue::i64_rem_u)
    }

    fn visit_i64_and(&mut self) {
        self.execute_binary(UntypedValue::i64_and)
    }

    fn visit_i64_or(&mut self) {
        self.execute_binary(UntypedValue::i64_or)
    }

    fn visit_i64_xor(&mut self) {
        self.execute_binary(UntypedValue::i64_xor)
    }

    fn visit_i64_shl(&mut self) {
        self.execute_binary(UntypedValue::i64_shl)
    }

    fn visit_i64_shr_s(&mut self) {
        self.execute_binary(UntypedValue::i64_shr_s)
    }

    fn visit_i64_shr_u(&mut self) {
        self.execute_binary(UntypedValue::i64_shr_u)
    }

    fn visit_i64_rotl(&mut self) {
        self.execute_binary(UntypedValue::i64_rotl)
    }

    fn visit_i64_rotr(&mut self) {
        self.execute_binary(UntypedValue::i64_rotr)
    }

    fn visit_f32_abs(&mut self) {
        self.execute_unary(UntypedValue::f32_abs)
    }

    fn visit_f32_neg(&mut self) {
        self.execute_unary(UntypedValue::f32_neg)
    }

    fn visit_f32_ceil(&mut self) {
        self.execute_unary(UntypedValue::f32_ceil)
    }

    fn visit_f32_floor(&mut self) {
        self.execute_unary(UntypedValue::f32_floor)
    }

    fn visit_f32_trunc(&mut self) {
        self.execute_unary(UntypedValue::f32_trunc)
    }

    fn visit_f32_nearest(&mut self) {
        self.execute_unary(UntypedValue::f32_nearest)
    }

    fn visit_f32_sqrt(&mut self) {
        self.execute_unary(UntypedValue::f32_sqrt)
    }

    fn visit_f32_add(&mut self) {
        self.execute_binary(UntypedValue::f32_add)
    }

    fn visit_f32_sub(&mut self) {
        self.execute_binary(UntypedValue::f32_sub)
    }

    fn visit_f32_mul(&mut self) {
        self.execute_binary(UntypedValue::f32_mul)
    }

    fn visit_f32_div(&mut self) {
        self.execute_binary(UntypedValue::f32_div)
    }

    fn visit_f32_min(&mut self) {
        self.execute_binary(UntypedValue::f32_min)
    }

    fn visit_f32_max(&mut self) {
        self.execute_binary(UntypedValue::f32_max)
    }

    fn visit_f32_copysign(&mut self) {
        self.execute_binary(UntypedValue::f32_copysign)
    }

    fn visit_f64_abs(&mut self) {
        self.execute_unary(UntypedValue::f64_abs)
    }

    fn visit_f64_neg(&mut self) {
        self.execute_unary(UntypedValue::f64_neg)
    }

    fn visit_f64_ceil(&mut self) {
        self.execute_unary(UntypedValue::f64_ceil)
    }

    fn visit_f64_floor(&mut self) {
        self.execute_unary(UntypedValue::f64_floor)
    }

    fn visit_f64_trunc(&mut self) {
        self.execute_unary(UntypedValue::f64_trunc)
    }

    fn visit_f64_nearest(&mut self) {
        self.execute_unary(UntypedValue::f64_nearest)
    }

    fn visit_f64_sqrt(&mut self) {
        self.execute_unary(UntypedValue::f64_sqrt)
    }

    fn visit_f64_add(&mut self) {
        self.execute_binary(UntypedValue::f64_add)
    }

    fn visit_f64_sub(&mut self) {
        self.execute_binary(UntypedValue::f64_sub)
    }

    fn visit_f64_mul(&mut self) {
        self.execute_binary(UntypedValue::f64_mul)
    }

    fn visit_f64_div(&mut self) {
        self.execute_binary(UntypedValue::f64_div)
    }

    fn visit_f64_min(&mut self) {
        self.execute_binary(UntypedValue::f64_min)
    }

    fn visit_f64_max(&mut self) {
        self.execute_binary(UntypedValue::f64_max)
    }

    fn visit_f64_copysign(&mut self) {
        self.execute_binary(UntypedValue::f64_copysign)
    }

    fn visit_i32_wrap_i64(&mut self) {
        self.execute_unary(UntypedValue::i32_wrap_i64)
    }

    fn visit_i32_trunc_f32(&mut self) -> Result<(), Trap> {
        self.try_execute_unary(UntypedValue::i32_trunc_f32_s)
    }

    fn visit_u32_trunc_f32(&mut self) -> Result<(), Trap> {
        self.try_execute_unary(UntypedValue::i32_trunc_f32_u)
    }

    fn visit_i32_trunc_f64(&mut self) -> Result<(), Trap> {
        self.try_execute_unary(UntypedValue::i32_trunc_f64_s)
    }

    fn visit_u32_trunc_f64(&mut self) -> Result<(), Trap> {
        self.try_execute_unary(UntypedValue::i32_trunc_f64_u)
    }

    fn visit_i64_extend_i32(&mut self) {
        self.execute_unary(UntypedValue::i64_extend_i32_s)
    }

    fn visit_i64_extend_u32(&mut self) {
        self.execute_unary(UntypedValue::i64_extend_i32_u)
    }

    fn visit_i64_trunc_f32(&mut self) -> Result<(), Trap> {
        self.try_execute_unary(UntypedValue::i64_trunc_f32_s)
    }

    fn visit_u64_trunc_f32(&mut self) -> Result<(), Trap> {
        self.try_execute_unary(UntypedValue::i64_trunc_f32_u)
    }

    fn visit_i64_trunc_f64(&mut self) -> Result<(), Trap> {
        self.try_execute_unary(UntypedValue::i64_trunc_f64_s)
    }

    fn visit_u64_trunc_f64(&mut self) -> Result<(), Trap> {
        self.try_execute_unary(UntypedValue::i64_trunc_f64_u)
    }

    fn visit_f32_convert_i32(&mut self) {
        self.execute_unary(UntypedValue::f32_convert_i32_s)
    }

    fn visit_f32_convert_u32(&mut self) {
        self.execute_unary(UntypedValue::f32_convert_i32_u)
    }

    fn visit_f32_convert_i64(&mut self) {
        self.execute_unary(UntypedValue::f32_convert_i64_s)
    }

    fn visit_f32_convert_u64(&mut self) {
        self.execute_unary(UntypedValue::f32_convert_i64_u)
    }

    fn visit_f32_demote_f64(&mut self) {
        self.execute_unary(UntypedValue::f32_demote_f64)
    }

    fn visit_f64_convert_i32(&mut self) {
        self.execute_unary(UntypedValue::f64_convert_i32_s)
    }

    fn visit_f64_convert_u32(&mut self) {
        self.execute_unary(UntypedValue::f64_convert_i32_u)
    }

    fn visit_f64_convert_i64(&mut self) {
        self.execute_unary(UntypedValue::f64_convert_i64_s)
    }

    fn visit_f64_convert_u64(&mut self) {
        self.execute_unary(UntypedValue::f64_convert_i64_u)
    }

    fn visit_f64_promote_f32(&mut self) {
        self.execute_unary(UntypedValue::f64_promote_f32)
    }

    fn visit_i32_reinterpret_f32(&mut self) {
        self.execute_reinterpret::<F32, i32>()
    }

    fn visit_i64_reinterpret_f64(&mut self) {
        self.execute_reinterpret::<F64, i64>()
    }

    fn visit_f32_reinterpret_i32(&mut self) {
        self.execute_reinterpret::<i32, F32>()
    }

    fn visit_f64_reinterpret_i64(&mut self) {
        self.execute_reinterpret::<i64, F64>()
    }

    fn visit_i32_sign_extend8(&mut self) {
        self.execute_unary(UntypedValue::i32_extend8_s)
    }

    fn visit_i32_sign_extend16(&mut self) {
        self.execute_unary(UntypedValue::i32_extend16_s)
    }

    fn visit_i64_sign_extend8(&mut self) {
        self.execute_unary(UntypedValue::i64_extend8_s)
    }

    fn visit_i64_sign_extend16(&mut self) {
        self.execute_unary(UntypedValue::i64_extend16_s)
    }

    fn visit_i64_sign_extend32(&mut self) {
        self.execute_unary(UntypedValue::i64_extend32_s)
    }

    fn visit_i32_trunc_sat_f32(&mut self) {
        self.execute_unary(UntypedValue::i32_trunc_sat_f32_s)
    }

    fn visit_u32_trunc_sat_f32(&mut self) {
        self.execute_unary(UntypedValue::i32_trunc_sat_f32_u)
    }

    fn visit_i32_trunc_sat_f64(&mut self) {
        self.execute_unary(UntypedValue::i32_trunc_sat_f64_s)
    }

    fn visit_u32_trunc_sat_f64(&mut self) {
        self.execute_unary(UntypedValue::i32_trunc_sat_f64_u)
    }

    fn visit_i64_trunc_sat_f32(&mut self) {
        self.execute_unary(UntypedValue::i64_trunc_sat_f32_s)
    }

    fn visit_u64_trunc_sat_f32(&mut self) {
        self.execute_unary(UntypedValue::i64_trunc_sat_f32_u)
    }

    fn visit_i64_trunc_sat_f64(&mut self) {
        self.execute_unary(UntypedValue::i64_trunc_sat_f64_s)
    }

    fn visit_u64_trunc_sat_f64(&mut self) {
        self.execute_unary(UntypedValue::i64_trunc_sat_f64_u)
    }
}
