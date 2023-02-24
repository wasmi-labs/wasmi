use crate::{
    core::TrapCode,
    engine::{
        bytecode::{
            BranchParams,
            DataSegmentIdx,
            ElementSegmentIdx,
            FuncIdx,
            GlobalIdx,
            Instruction,
            LocalDepth,
            Offset,
            SignatureIdx,
            TableIdx,
        },
        cache::InstanceCache,
        code_map::InstructionPtr,
        config::FuelCosts,
        stack::ValueStackPtr,
        CallOutcome,
        DropKeep,
        FuncFrame,
        ValueStack,
    },
    table::TableEntity,
    Func,
    FuncRef,
    Memory,
    StoreInner,
    Table,
};
use core::cmp::{self};
use wasmi_core::{Pages, UntypedValue};

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
    ctx: &mut StoreInner,
    cache: &'engine mut InstanceCache,
    frame: &mut FuncFrame,
    value_stack: &'engine mut ValueStack,
) -> Result<CallOutcome, TrapCode> {
    Executor::new(ctx, cache, frame, value_stack).execute()
}

/// The function signature of Wasm load operations.
type WasmLoadOp =
    fn(memory: &[u8], address: UntypedValue, offset: u32) -> Result<UntypedValue, TrapCode>;

/// The function signature of Wasm store operations.
type WasmStoreOp = fn(
    memory: &mut [u8],
    address: UntypedValue,
    offset: u32,
    value: UntypedValue,
) -> Result<(), TrapCode>;

/// An error that can occur upon `memory.grow` or `table.grow`.
#[derive(Copy, Clone)]
pub enum EntityGrowError {
    /// Usually a [`TrapCode::OutOfFuel`] trap.
    TrapCode(TrapCode),
    /// Encountered when `memory.grow` or `table.grow` fails.
    InvalidGrow,
}

impl From<TrapCode> for EntityGrowError {
    fn from(trap_code: TrapCode) -> Self {
        Self::TrapCode(trap_code)
    }
}

/// The WebAssembly specification demands to return this value
/// if the `memory.grow` or `table.grow` operations fail.
const INVALID_GROWTH_ERRCODE: u32 = u32::MAX;

/// An execution context for executing a `wasmi` function frame.
#[derive(Debug)]
struct Executor<'ctx, 'engine, 'func> {
    /// The pointer to the currently executed instruction.
    ip: InstructionPtr,
    /// Stores the value stack of live values on the Wasm stack.
    sp: ValueStackPtr,
    /// A mutable [`StoreInner`] context.
    ///
    /// [`StoreInner`]: [`crate::StoreInner`]
    ctx: &'ctx mut StoreInner,
    /// Stores frequently used instance related data.
    cache: &'engine mut InstanceCache,
    /// The function frame that is being executed.
    frame: &'func mut FuncFrame,
    /// The value stack.
    ///
    /// # Note
    ///
    /// This reference is mainly used to synchronize back state
    /// after manipulations to the value stack via `sp`.
    value_stack: &'engine mut ValueStack,
}

impl<'ctx, 'engine, 'func> Executor<'ctx, 'engine, 'func> {
    /// Creates a new [`Executor`] for executing a `wasmi` function frame.
    #[inline(always)]
    pub fn new(
        ctx: &'ctx mut StoreInner,
        cache: &'engine mut InstanceCache,
        frame: &'func mut FuncFrame,
        value_stack: &'engine mut ValueStack,
    ) -> Self {
        cache.update_instance(frame.instance());
        let ip = frame.ip();
        let sp = value_stack.stack_ptr();
        Self {
            ip,
            sp,
            ctx,
            cache,
            frame,
            value_stack,
        }
    }

    /// Executes the function frame until it returns or traps.
    #[inline(always)]
    fn execute(mut self) -> Result<CallOutcome, TrapCode> {
        use Instruction as Instr;
        loop {
            match *self.instr() {
                Instr::LocalGet { local_depth } => self.visit_local_get(local_depth),
                Instr::LocalSet { local_depth } => self.visit_local_set(local_depth),
                Instr::LocalTee { local_depth } => self.visit_local_tee(local_depth),
                Instr::Br(params) => self.visit_br(params),
                Instr::BrIfEqz(params) => self.visit_br_if_eqz(params),
                Instr::BrIfNez(params) => self.visit_br_if_nez(params),
                Instr::BrTable { len_targets } => self.visit_br_table(len_targets),
                Instr::Unreachable => self.visit_unreachable()?,
                Instr::ConsumeFuel { amount } => self.visit_consume_fuel(amount)?,
                Instr::Return(drop_keep) => return self.visit_ret(drop_keep),
                Instr::ReturnIfNez(drop_keep) => {
                    if let MaybeReturn::Return = self.visit_return_if_nez(drop_keep) {
                        return Ok(CallOutcome::Return);
                    }
                }
                Instr::Call(func) => return self.visit_call(func),
                Instr::CallIndirect { table, func_type } => {
                    return self.visit_call_indirect(table, func_type)
                }
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
                Instr::MemorySize => self.visit_memory_size(),
                Instr::MemoryGrow => self.visit_memory_grow()?,
                Instr::MemoryFill => self.visit_memory_fill()?,
                Instr::MemoryCopy => self.visit_memory_copy()?,
                Instr::MemoryInit(segment) => self.visit_memory_init(segment)?,
                Instr::DataDrop(segment) => self.visit_data_drop(segment),
                Instr::TableSize { table } => self.visit_table_size(table),
                Instr::TableGrow { table } => self.visit_table_grow(table)?,
                Instr::TableFill { table } => self.visit_table_fill(table)?,
                Instr::TableGet { table } => self.visit_table_get(table)?,
                Instr::TableSet { table } => self.visit_table_set(table)?,
                Instr::TableCopy { dst, src } => self.visit_table_copy(dst, src)?,
                Instr::TableInit { table, elem } => self.visit_table_init(table, elem)?,
                Instr::ElemDrop(segment) => self.visit_element_drop(segment),
                Instr::RefFunc { func_index } => self.visit_ref_func(func_index),
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
                Instr::I32TruncF32S => self.visit_i32_trunc_f32()?,
                Instr::I32TruncF32U => self.visit_u32_trunc_f32()?,
                Instr::I32TruncF64S => self.visit_i32_trunc_f64()?,
                Instr::I32TruncF64U => self.visit_u32_trunc_f64()?,
                Instr::I64ExtendI32S => self.visit_i64_extend_i32(),
                Instr::I64ExtendI32U => self.visit_i64_extend_u32(),
                Instr::I64TruncF32S => self.visit_i64_trunc_f32()?,
                Instr::I64TruncF32U => self.visit_u64_trunc_f32()?,
                Instr::I64TruncF64S => self.visit_i64_trunc_f64()?,
                Instr::I64TruncF64U => self.visit_u64_trunc_f64()?,
                Instr::F32ConvertI32S => self.visit_f32_convert_i32(),
                Instr::F32ConvertI32U => self.visit_f32_convert_u32(),
                Instr::F32ConvertI64S => self.visit_f32_convert_i64(),
                Instr::F32ConvertI64U => self.visit_f32_convert_u64(),
                Instr::F32DemoteF64 => self.visit_f32_demote_f64(),
                Instr::F64ConvertI32S => self.visit_f64_convert_i32(),
                Instr::F64ConvertI32U => self.visit_f64_convert_u32(),
                Instr::F64ConvertI64S => self.visit_f64_convert_i64(),
                Instr::F64ConvertI64U => self.visit_f64_convert_u64(),
                Instr::F64PromoteF32 => self.visit_f64_promote_f32(),
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
    fn instr(&self) -> &Instruction {
        // # Safety
        //
        // Properly constructed `wasmi` bytecode can never produce invalid `pc`.
        unsafe { self.ip.get() }
    }

    /// Returns the default linear memory.
    ///
    /// # Panics
    ///
    /// If there exists is no linear memory for the instance.
    #[inline]
    fn default_memory(&mut self) -> Memory {
        self.cache.default_memory(self.ctx)
    }

    /// Returns the global variable at the given index.
    ///
    /// # Panics
    ///
    /// If there is no global variable at the given index.
    fn global(&mut self, global_index: GlobalIdx) -> &mut UntypedValue {
        self.cache.get_global(self.ctx, global_index.into_inner())
    }

    /// Executes a generic Wasm `store[N_{s|u}]` operation.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `{i32, i64, f32, f64}.load`
    /// - `{i32, i64}.load8_s`
    /// - `{i32, i64}.load8_u`
    /// - `{i32, i64}.load16_s`
    /// - `{i32, i64}.load16_u`
    /// - `i64.load32_s`
    /// - `i64.load32_u`
    fn execute_load_extend(
        &mut self,
        offset: Offset,
        load_extend: WasmLoadOp,
    ) -> Result<(), TrapCode> {
        self.sp.try_eval_top(|address| {
            let memory = self.cache.default_memory_bytes(self.ctx).data();
            let value = load_extend(memory, address, offset.into_inner())?;
            Ok(value)
        })?;
        self.try_next_instr()
    }

    /// Executes a generic Wasm `store[N]` operation.
    ///
    /// # Note
    ///
    /// This can be used to emulate the following Wasm operands:
    ///
    /// - `{i32, i64, f32, f64}.store`
    /// - `{i32, i64}.store8`
    /// - `{i32, i64}.store16`
    /// - `i64.store32`
    fn execute_store_wrap(
        &mut self,
        offset: Offset,
        store_wrap: WasmStoreOp,
    ) -> Result<(), TrapCode> {
        let (address, value) = self.sp.pop2();
        let memory = self.cache.default_memory_bytes(self.ctx).data_mut();
        store_wrap(memory, address, offset.into_inner(), value)?;
        self.try_next_instr()
    }

    /// Executes an infallible unary `wasmi` instruction.
    #[inline]
    fn execute_unary(&mut self, f: fn(UntypedValue) -> UntypedValue) {
        self.sp.eval_top(f);
        self.next_instr()
    }

    /// Executes a fallible unary `wasmi` instruction.
    #[inline]
    fn try_execute_unary(
        &mut self,
        f: fn(UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), TrapCode> {
        self.sp.try_eval_top(f)?;
        self.try_next_instr()
    }

    /// Executes an infallible binary `wasmi` instruction.
    #[inline]
    fn execute_binary(&mut self, f: fn(UntypedValue, UntypedValue) -> UntypedValue) {
        self.sp.eval_top2(f);
        self.next_instr()
    }

    /// Executes a fallible binary `wasmi` instruction.
    #[inline]
    fn try_execute_binary(
        &mut self,
        f: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), TrapCode> {
        self.sp.try_eval_top2(f)?;
        self.try_next_instr()
    }

    /// Shifts the instruction pointer to the next instruction.
    #[inline]
    fn next_instr(&mut self) {
        self.ip_add(1)
    }

    /// Shifts the instruction pointer to the next instruction and returns `Ok(())`.
    ///
    /// # Note
    ///
    /// This is a convenience function for fallible instructions.
    #[inline]
    fn try_next_instr(&mut self) -> Result<(), TrapCode> {
        self.next_instr();
        Ok(())
    }

    /// Offsets the instruction pointer using the given [`BranchParams`].
    #[inline]
    fn branch_to(&mut self, params: BranchParams) {
        self.sp.drop_keep(params.drop_keep());
        self.ip_add(params.offset().into_i32() as isize)
    }

    /// Adjusts the [`InstructionPtr`] by `delta` in terms of [`Instruction`].
    #[inline]
    fn ip_add(&mut self, delta: isize) {
        // Safety: This is safe since we carefully constructed the `wasmi`
        //         bytecode in conjunction with Wasm validation so that the
        //         offsets of the instruction pointer within the sequence of
        //         instructions never make the instruction pointer point out
        //         of bounds of the instructions that belong to the function
        //         that is currently executed.
        unsafe {
            self.ip.offset(delta);
        }
    }

    /// Synchronizes the current stack pointer with the [`ValueStack`].
    ///
    /// # Note
    ///
    /// For performance reasons we detach the stack pointer form the [`ValueStack`].
    /// Therefore it is necessary to synchronize the [`ValueStack`] upon finishing
    /// execution of a sequence of non control flow instructions.
    #[inline]
    fn sync_stack_ptr(&mut self) {
        self.value_stack.sync_stack_ptr(self.sp);
    }

    /// Calls the given [`Func`].
    ///
    /// This also prepares the instruction pointer and stack pointer for
    /// the function call so that the stack and execution state is synchronized
    /// with the outer structures.
    fn call_func(&mut self, func: &Func) -> Result<CallOutcome, TrapCode> {
        self.next_instr();
        self.frame.update_ip(self.ip);
        self.sync_stack_ptr();
        Ok(CallOutcome::NestedCall(*func))
    }

    /// Returns to the caller.
    ///
    /// This also modifies the stack as the caller would expect it
    /// and synchronizes the execution state with the outer structures.
    fn ret(&mut self, drop_keep: DropKeep) {
        self.sp.drop_keep(drop_keep);
        self.sync_stack_ptr();
    }

    /// Consume an amount of fuel specified by `delta` if `exec` succeeds.
    ///
    /// # Note
    ///
    /// - `delta` is only evaluated if fuel metering is enabled.
    /// - `exec` is only evaluated if the remaining fuel is sufficient
    ///    for amount of required fuel determined by `delta` or if
    ///    fuel metering is disabled.
    ///
    /// Only if `exec` runs successfully and fuel metering
    /// is enabled the fuel determined by `delta` is charged.
    ///
    /// # Errors
    ///
    /// - If the [`StoreInner`] ran out of fuel.
    /// - If the `exec` closure traps.
    fn consume_fuel_on_success<T, E>(
        &mut self,
        delta: impl FnOnce(&FuelCosts) -> u64,
        exec: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<TrapCode>,
    {
        if !self.is_fuel_metering_enabled() {
            return exec(self);
        }
        // At this point we know that fuel metering is enabled.
        let delta = delta(self.fuel_costs());
        self.ctx.fuel().sufficient_fuel(delta)?;
        let result = exec(self)?;
        self.ctx
            .fuel_mut()
            .consume_fuel(delta)
            .unwrap_or_else(|error| {
                panic!("remaining fuel has already been approved prior but encountered: {error}")
            });
        Ok(result)
    }

    /// Returns `true` if fuel metering is enabled.
    fn is_fuel_metering_enabled(&self) -> bool {
        self.ctx.engine().config().get_consume_fuel()
    }

    /// Returns a shared reference to the [`FuelCosts`] of the [`Engine`].
    ///
    /// [`Engine`]: crate::Engine
    fn fuel_costs(&self) -> &FuelCosts {
        self.ctx.engine().config().fuel_costs()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MaybeReturn {
    Return,
    Continue,
}

impl<'ctx, 'engine, 'func> Executor<'ctx, 'engine, 'func> {
    fn visit_unreachable(&mut self) -> Result<(), TrapCode> {
        Err(TrapCode::UnreachableCodeReached).map_err(Into::into)
    }

    fn visit_consume_fuel(&mut self, amount: u64) -> Result<(), TrapCode> {
        // We do not have to check if fuel metering is enabled since
        // these `wasmi` instructions are only generated if fuel metering
        // is enabled to begin with.
        self.ctx.fuel_mut().consume_fuel(amount)?;
        self.try_next_instr()
    }

    fn visit_br(&mut self, params: BranchParams) {
        self.branch_to(params)
    }

    fn visit_br_if_eqz(&mut self, params: BranchParams) {
        let condition = self.sp.pop_as();
        if condition {
            self.next_instr()
        } else {
            self.branch_to(params)
        }
    }

    fn visit_br_if_nez(&mut self, params: BranchParams) {
        let condition = self.sp.pop_as();
        if condition {
            self.branch_to(params)
        } else {
            self.next_instr()
        }
    }

    fn visit_return_if_nez(&mut self, drop_keep: DropKeep) -> MaybeReturn {
        let condition = self.sp.pop_as();
        if condition {
            self.ret(drop_keep);
            MaybeReturn::Return
        } else {
            self.next_instr();
            MaybeReturn::Continue
        }
    }

    fn visit_br_table(&mut self, len_targets: usize) {
        let index: u32 = self.sp.pop_as();
        // The index of the default target which is the last target of the slice.
        let max_index = len_targets - 1;
        // A normalized index will always yield a target without panicking.
        let normalized_index = cmp::min(index as usize, max_index);
        // Update `pc`:
        unsafe {
            self.ip.offset((normalized_index + 1) as isize);
        }
    }

    fn visit_ret(&mut self, drop_keep: DropKeep) -> Result<CallOutcome, TrapCode> {
        self.ret(drop_keep);
        Ok(CallOutcome::Return)
    }

    fn visit_local_get(&mut self, local_depth: LocalDepth) {
        let value = self.sp.nth_back(local_depth.into_inner());
        self.sp.push(value);
        self.next_instr()
    }

    fn visit_local_set(&mut self, local_depth: LocalDepth) {
        let new_value = self.sp.pop();
        self.sp.set_nth_back(local_depth.into_inner(), new_value);
        self.next_instr()
    }

    fn visit_local_tee(&mut self, local_depth: LocalDepth) {
        let new_value = self.sp.last();
        self.sp.set_nth_back(local_depth.into_inner(), new_value);
        self.next_instr()
    }

    fn visit_global_get(&mut self, global_index: GlobalIdx) {
        let global_value = *self.global(global_index);
        self.sp.push(global_value);
        self.next_instr()
    }

    fn visit_global_set(&mut self, global_index: GlobalIdx) {
        let new_value = self.sp.pop();
        *self.global(global_index) = new_value;
        self.next_instr()
    }

    fn visit_call(&mut self, func_index: FuncIdx) -> Result<CallOutcome, TrapCode> {
        let callee = self.cache.get_func(self.ctx, func_index.into_inner());
        self.call_func(&callee)
    }

    fn visit_call_indirect(
        &mut self,
        table: TableIdx,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let func_index: u32 = self.sp.pop_as();
        let table = self.cache.get_table(self.ctx, table);
        let funcref = self
            .ctx
            .resolve_table(&table)
            .get_untyped(func_index)
            .map(FuncRef::from)
            .ok_or(TrapCode::TableOutOfBounds)?;
        let func = funcref.func().ok_or(TrapCode::IndirectCallToNull)?;
        let actual_signature = self.ctx.resolve_func(func).ty_dedup();
        let expected_signature = self
            .ctx
            .resolve_instance(self.frame.instance())
            .get_signature(func_type.into_inner())
            .unwrap_or_else(|| {
                panic!("missing signature for call_indirect at index: {func_type:?}")
            });
        if actual_signature != expected_signature {
            return Err(TrapCode::BadSignature).map_err(Into::into);
        }
        self.call_func(func)
    }

    fn visit_const(&mut self, bytes: UntypedValue) {
        self.sp.push(bytes);
        self.next_instr()
    }

    fn visit_drop(&mut self) {
        self.sp.drop();
        self.next_instr()
    }

    fn visit_select(&mut self) {
        self.sp.eval_top3(|e1, e2, e3| {
            let condition = <bool as From<UntypedValue>>::from(e3);
            if condition {
                e1
            } else {
                e2
            }
        });
        self.next_instr()
    }

    fn visit_memory_size(&mut self) {
        let memory = self.default_memory();
        let result: u32 = self.ctx.resolve_memory(&memory).current_pages().into();
        self.sp.push_as(result);
        self.next_instr()
    }

    fn visit_memory_grow(&mut self) -> Result<(), TrapCode> {
        let memory = self.default_memory();
        let delta: u32 = self.sp.pop_as();
        let delta = match Pages::new(delta) {
            Some(pages) => pages,
            None => {
                // Cannot grow memory so we push the expected error value.
                self.sp.push_as(INVALID_GROWTH_ERRCODE);
                return self.try_next_instr();
            }
        };
        let result = self.consume_fuel_on_success(
            |costs| {
                let delta_in_bytes = delta.to_bytes().unwrap_or(0) as u64;
                delta_in_bytes * costs.memory_per_byte
            },
            |this| {
                let new_pages = this
                    .ctx
                    .resolve_memory_mut(&memory)
                    .grow(delta)
                    .map(u32::from)
                    .map_err(|_| EntityGrowError::InvalidGrow)?;
                // The `memory.grow` operation might have invalidated the cached
                // linear memory so we need to reset it in order for the cache to
                // reload in case it is used again.
                this.cache.reset_default_memory_bytes();
                Ok(new_pages)
            },
        );
        let result = match result {
            Ok(result) => result,
            Err(EntityGrowError::InvalidGrow) => INVALID_GROWTH_ERRCODE,
            Err(EntityGrowError::TrapCode(trap_code)) => return Err(trap_code),
        };
        self.sp.push_as(result);
        self.try_next_instr()
    }

    fn visit_memory_fill(&mut self) -> Result<(), TrapCode> {
        // The `n`, `val` and `d` variable bindings are extracted from the Wasm specification.
        let (d, val, n) = self.sp.pop3();
        let n = i32::from(n) as usize;
        let offset = i32::from(d) as usize;
        let byte = u8::from(val);
        self.consume_fuel_on_success(
            |costs| n as u64 * costs.memory_per_byte,
            |this| {
                let bytes = this.cache.default_memory_bytes(this.ctx);
                let memory = bytes
                    .data_mut()
                    .get_mut(offset..)
                    .and_then(|memory| memory.get_mut(..n))
                    .ok_or(TrapCode::MemoryOutOfBounds)?;
                memory.fill(byte);
                Ok(())
            },
        )?;
        self.try_next_instr()
    }

    fn visit_memory_copy(&mut self) -> Result<(), TrapCode> {
        // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
        let (d, s, n) = self.sp.pop3();
        let n = i32::from(n) as usize;
        let src_offset = i32::from(s) as usize;
        let dst_offset = i32::from(d) as usize;
        self.consume_fuel_on_success(
            |costs| n as u64 * costs.memory_per_byte,
            |this| {
                let data = this.cache.default_memory_bytes(this.ctx).data_mut();
                // These accesses just perform the bounds checks required by the Wasm spec.
                data.get(src_offset..)
                    .and_then(|memory| memory.get(..n))
                    .ok_or(TrapCode::MemoryOutOfBounds)?;
                data.get(dst_offset..)
                    .and_then(|memory| memory.get(..n))
                    .ok_or(TrapCode::MemoryOutOfBounds)?;
                data.copy_within(src_offset..src_offset.wrapping_add(n), dst_offset);
                Ok(())
            },
        )?;
        self.try_next_instr()
    }

    fn visit_memory_init(&mut self, segment: DataSegmentIdx) -> Result<(), TrapCode> {
        // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
        let (d, s, n) = self.sp.pop3();
        let n = i32::from(n) as usize;
        let src_offset = i32::from(s) as usize;
        let dst_offset = i32::from(d) as usize;
        self.consume_fuel_on_success(
            |costs| n as u64 * costs.memory_per_byte,
            |this| {
                let (memory, data) = this
                    .cache
                    .get_default_memory_and_data_segment(this.ctx, segment);
                let memory = memory
                    .get_mut(dst_offset..)
                    .and_then(|memory| memory.get_mut(..n))
                    .ok_or(TrapCode::MemoryOutOfBounds)?;
                let data = data
                    .get(src_offset..)
                    .and_then(|data| data.get(..n))
                    .ok_or(TrapCode::MemoryOutOfBounds)?;
                memory.copy_from_slice(data);
                Ok(())
            },
        )?;
        self.try_next_instr()
    }

    fn visit_data_drop(&mut self, segment_index: DataSegmentIdx) {
        let segment = self
            .cache
            .get_data_segment(self.ctx, segment_index.into_inner());
        self.ctx.resolve_data_segment_mut(&segment).drop_bytes();
        self.next_instr();
    }

    fn visit_table_size(&mut self, table_index: TableIdx) {
        let table = self.cache.get_table(self.ctx, table_index);
        let size = self.ctx.resolve_table(&table).size();
        self.sp.push_as(size);
        self.next_instr()
    }

    fn visit_table_grow(&mut self, table_index: TableIdx) -> Result<(), TrapCode> {
        let (init, delta) = self.sp.pop2();
        let delta: u32 = delta.into();
        let result = self.consume_fuel_on_success(
            |costs| u64::from(delta) * costs.table_per_element,
            |this| {
                let table = this.cache.get_table(this.ctx, table_index);
                this.ctx
                    .resolve_table_mut(&table)
                    .grow_untyped(delta, init)
                    .map_err(|_| EntityGrowError::InvalidGrow)
            },
        );
        let result = match result {
            Ok(result) => result,
            Err(EntityGrowError::InvalidGrow) => INVALID_GROWTH_ERRCODE,
            Err(EntityGrowError::TrapCode(trap_code)) => return Err(trap_code),
        };
        self.sp.push_as(result);
        self.try_next_instr()
    }

    fn visit_table_fill(&mut self, table_index: TableIdx) -> Result<(), TrapCode> {
        // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
        let (i, val, n) = self.sp.pop3();
        let dst: u32 = i.into();
        let len: u32 = n.into();
        self.consume_fuel_on_success(
            |costs| u64::from(len) * costs.table_per_element,
            |this| {
                let table = this.cache.get_table(this.ctx, table_index);
                this.ctx
                    .resolve_table_mut(&table)
                    .fill_untyped(dst, val, len)?;
                Ok(())
            },
        )?;
        self.try_next_instr()
    }

    fn visit_table_get(&mut self, table_index: TableIdx) -> Result<(), TrapCode> {
        self.sp.try_eval_top(|index| {
            let index: u32 = index.into();
            let table = self.cache.get_table(self.ctx, table_index);
            self.ctx
                .resolve_table(&table)
                .get_untyped(index)
                .ok_or(TrapCode::TableOutOfBounds)
        })?;
        self.try_next_instr()
    }

    fn visit_table_set(&mut self, table_index: TableIdx) -> Result<(), TrapCode> {
        let (index, value) = self.sp.pop2();
        let index: u32 = index.into();
        let table = self.cache.get_table(self.ctx, table_index);
        self.ctx
            .resolve_table_mut(&table)
            .set_untyped(index, value)
            .map_err(|_| TrapCode::TableOutOfBounds)?;
        self.try_next_instr()
    }

    fn visit_table_copy(&mut self, dst: TableIdx, src: TableIdx) -> Result<(), TrapCode> {
        // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
        let (d, s, n) = self.sp.pop3();
        let len = u32::from(n);
        let src_index = u32::from(s);
        let dst_index = u32::from(d);
        self.consume_fuel_on_success(
            |costs| u64::from(len) * costs.table_per_element,
            |this| {
                // Query both tables and check if they are the same:
                let dst = this.cache.get_table(this.ctx, dst);
                let src = this.cache.get_table(this.ctx, src);
                if Table::eq(&dst, &src) {
                    // Copy within the same table:
                    let table = this.ctx.resolve_table_mut(&dst);
                    table.copy_within(dst_index, src_index, len)?;
                } else {
                    // Copy from one table to another table:
                    let (dst, src) = this.ctx.resolve_table_pair_mut(&dst, &src);
                    TableEntity::copy(dst, dst_index, src, src_index, len)?;
                }
                Ok(())
            },
        )?;
        self.try_next_instr()
    }

    fn visit_table_init(
        &mut self,
        table: TableIdx,
        elem: ElementSegmentIdx,
    ) -> Result<(), TrapCode> {
        // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
        let (d, s, n) = self.sp.pop3();
        let len = u32::from(n);
        let src_index = u32::from(s);
        let dst_index = u32::from(d);
        self.consume_fuel_on_success(
            |costs| u64::from(len) * costs.table_per_element,
            |this| {
                let (instance, table, element) = this
                    .cache
                    .get_table_and_element_segment(this.ctx, table, elem);
                table.init(dst_index, element, src_index, len, |func_index| {
                    instance
                        .get_func(func_index)
                        .unwrap_or_else(|| panic!("missing function at index {func_index}"))
                })?;
                Ok(())
            },
        )?;
        self.try_next_instr()
    }

    fn visit_element_drop(&mut self, segment_index: ElementSegmentIdx) {
        let segment = self.cache.get_element_segment(self.ctx, segment_index);
        self.ctx.resolve_element_segment_mut(&segment).drop_items();
        self.next_instr();
    }

    fn visit_ref_func(&mut self, func_index: FuncIdx) {
        let func = self.cache.get_func(self.ctx, func_index.into_inner());
        let funcref = FuncRef::new(func);
        self.sp.push_as(funcref);
        self.next_instr();
    }

    fn visit_i32_load(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i32_load)
    }

    fn visit_i64_load(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i64_load)
    }

    fn visit_f32_load(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::f32_load)
    }

    fn visit_f64_load(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::f64_load)
    }

    fn visit_i32_load_i8(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i32_load8_s)
    }

    fn visit_i32_load_u8(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i32_load8_u)
    }

    fn visit_i32_load_i16(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i32_load16_s)
    }

    fn visit_i32_load_u16(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i32_load16_u)
    }

    fn visit_i64_load_i8(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i64_load8_s)
    }

    fn visit_i64_load_u8(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i64_load8_u)
    }

    fn visit_i64_load_i16(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i64_load16_s)
    }

    fn visit_i64_load_u16(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i64_load16_u)
    }

    fn visit_i64_load_i32(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i64_load32_s)
    }

    fn visit_i64_load_u32(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_load_extend(offset, UntypedValue::i64_load32_u)
    }

    fn visit_i32_store(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::i32_store)
    }

    fn visit_i64_store(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::i64_store)
    }

    fn visit_f32_store(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::f32_store)
    }

    fn visit_f64_store(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::f64_store)
    }

    fn visit_i32_store_8(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::i32_store8)
    }

    fn visit_i32_store_16(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::i32_store16)
    }

    fn visit_i64_store_8(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::i64_store8)
    }

    fn visit_i64_store_16(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::i64_store16)
    }

    fn visit_i64_store_32(&mut self, offset: Offset) -> Result<(), TrapCode> {
        self.execute_store_wrap(offset, UntypedValue::i64_store32)
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

    fn visit_i32_div_s(&mut self) -> Result<(), TrapCode> {
        self.try_execute_binary(UntypedValue::i32_div_s)
    }

    fn visit_i32_div_u(&mut self) -> Result<(), TrapCode> {
        self.try_execute_binary(UntypedValue::i32_div_u)
    }

    fn visit_i32_rem_s(&mut self) -> Result<(), TrapCode> {
        self.try_execute_binary(UntypedValue::i32_rem_s)
    }

    fn visit_i32_rem_u(&mut self) -> Result<(), TrapCode> {
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

    fn visit_i64_div_s(&mut self) -> Result<(), TrapCode> {
        self.try_execute_binary(UntypedValue::i64_div_s)
    }

    fn visit_i64_div_u(&mut self) -> Result<(), TrapCode> {
        self.try_execute_binary(UntypedValue::i64_div_u)
    }

    fn visit_i64_rem_s(&mut self) -> Result<(), TrapCode> {
        self.try_execute_binary(UntypedValue::i64_rem_s)
    }

    fn visit_i64_rem_u(&mut self) -> Result<(), TrapCode> {
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

    fn visit_i32_trunc_f32(&mut self) -> Result<(), TrapCode> {
        self.try_execute_unary(UntypedValue::i32_trunc_f32_s)
    }

    fn visit_u32_trunc_f32(&mut self) -> Result<(), TrapCode> {
        self.try_execute_unary(UntypedValue::i32_trunc_f32_u)
    }

    fn visit_i32_trunc_f64(&mut self) -> Result<(), TrapCode> {
        self.try_execute_unary(UntypedValue::i32_trunc_f64_s)
    }

    fn visit_u32_trunc_f64(&mut self) -> Result<(), TrapCode> {
        self.try_execute_unary(UntypedValue::i32_trunc_f64_u)
    }

    fn visit_i64_extend_i32(&mut self) {
        self.execute_unary(UntypedValue::i64_extend_i32_s)
    }

    fn visit_i64_extend_u32(&mut self) {
        self.execute_unary(UntypedValue::i64_extend_i32_u)
    }

    fn visit_i64_trunc_f32(&mut self) -> Result<(), TrapCode> {
        self.try_execute_unary(UntypedValue::i64_trunc_f32_s)
    }

    fn visit_u64_trunc_f32(&mut self) -> Result<(), TrapCode> {
        self.try_execute_unary(UntypedValue::i64_trunc_f32_u)
    }

    fn visit_i64_trunc_f64(&mut self) -> Result<(), TrapCode> {
        self.try_execute_unary(UntypedValue::i64_trunc_f64_s)
    }

    fn visit_u64_trunc_f64(&mut self) -> Result<(), TrapCode> {
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
