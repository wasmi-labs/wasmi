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
        code_map::{CodeMap, InstructionPtr},
        config::FuelCosts,
        stack::{CallStack, ValueStackPtr},
        DropKeep,
        FuncFrame,
        ValueStack,
    },
    func::FuncEntity,
    table::TableEntity,
    Func,
    FuncRef,
    StoreInner,
    Table,
};
use core::cmp::{self};
use wasmi_core::{Pages, UntypedValue};

/// The outcome of a Wasm execution.
///
/// # Note
///
/// A Wasm execution includes everything but host calls.
/// In other words: Everything in between host calls is a Wasm execution.
#[derive(Debug, Copy, Clone)]
pub enum WasmOutcome {
    /// The Wasm execution has ended and returns to the host side.
    Return,
    /// The Wasm execution calls a host function.
    Call(Func),
}

/// The outcome of a Wasm execution.
///
/// # Note
///
/// A Wasm execution includes everything but host calls.
/// In other words: Everything in between host calls is a Wasm execution.
#[derive(Debug, Copy, Clone)]
pub enum CallOutcome {
    /// The Wasm execution continues in Wasm.
    Continue,
    /// The Wasm execution calls a host function.
    Call(Func),
}

/// The outcome of a Wasm return statement.
#[derive(Debug, Copy, Clone)]
pub enum ReturnOutcome {
    /// The call returns to a nested Wasm caller.
    Wasm,
    /// The call returns back to the host.
    Host,
}

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
#[inline(never)]
pub fn execute_frame<'engine>(
    ctx: &mut StoreInner,
    cache: &'engine mut InstanceCache,
    value_stack: &'engine mut ValueStack,
    call_stack: &'engine mut CallStack,
    code_map: &'engine CodeMap,
) -> Result<WasmOutcome, TrapCode> {
    Executor::new(ctx, cache, value_stack, call_stack, code_map).execute()
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

/// Shifts the instruction pointer to the next instruction.
#[inline]
fn next_instr(ip: &mut InstructionPtr) {
    ip.add(1)
}

/// Shifts the instruction pointer to the next instruction and returns `Ok(())`.
///
/// # Note
///
/// This is a convenience function for fallible instructions.
#[inline]
fn try_next_instr(ip: &mut InstructionPtr) -> Result<(), TrapCode> {
    next_instr(ip);
    Ok(())
}

/// Offsets the instruction pointer using the given [`BranchParams`].
#[inline]
fn branch_to(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, params: BranchParams) {
    sp.drop_keep(params.drop_keep());
    ip.offset(params.offset().into_i32() as isize)
}

/// Synchronizes the current stack pointer with the [`ValueStack`].
///
/// # Note
///
/// For performance reasons we detach the stack pointer form the [`ValueStack`].
/// Therefore it is necessary to synchronize the [`ValueStack`] upon finishing
/// execution of a sequence of non control flow instructions.
#[inline]
fn sync_stack_ptr(sp: &mut ValueStackPtr, value_stack: &mut ValueStack) {
    value_stack.sync_stack_ptr(*sp);
}

/// Returns to the caller.
///
/// This also modifies the stack as the caller would expect it
/// and synchronizes the execution state with the outer structures.
#[inline]
fn ret(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    cache: &mut InstanceCache,
    value_stack: &mut ValueStack,
    call_stack: &mut CallStack,
    drop_keep: DropKeep,
) -> ReturnOutcome {
    sp.drop_keep(drop_keep);
    sync_stack_ptr(sp, value_stack);
    match call_stack.pop() {
        Some(caller) => {
            *ip = caller.ip();
            cache.update_instance(caller.instance());
            ReturnOutcome::Wasm
        }
        None => ReturnOutcome::Host,
    }
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
#[inline]
fn execute_load_extend(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    offset: Offset,
    load_extend: WasmLoadOp,
) -> Result<(), TrapCode> {
    sp.try_eval_top(|address| {
        let memory = cache.default_memory_bytes(ctx);
        let value = load_extend(memory, address, offset.into_inner())?;
        Ok(value)
    })?;
    try_next_instr(ip)
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
#[inline]
fn execute_store_wrap(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    offset: Offset,
    store_wrap: WasmStoreOp,
) -> Result<(), TrapCode> {
    let (address, value) = sp.pop2();
    let memory = cache.default_memory_bytes(ctx);
    store_wrap(memory, address, offset.into_inner(), value)?;
    try_next_instr(ip)
}

/// Executes an infallible unary `wasmi` instruction.
#[inline]
fn execute_unary(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    f: fn(UntypedValue) -> UntypedValue,
) {
    sp.eval_top(f);
    next_instr(ip)
}

/// Executes a fallible unary `wasmi` instruction.
#[inline]
fn try_execute_unary(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    f: fn(UntypedValue) -> Result<UntypedValue, TrapCode>,
) -> Result<(), TrapCode> {
    sp.try_eval_top(f)?;
    try_next_instr(ip)
}

/// Executes an infallible binary `wasmi` instruction.
#[inline]
fn execute_binary(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    f: fn(UntypedValue, UntypedValue) -> UntypedValue,
) {
    sp.eval_top2(f);
    next_instr(ip)
}

/// Executes a fallible binary `wasmi` instruction.
#[inline]
fn try_execute_binary(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    f: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
) -> Result<(), TrapCode> {
    sp.try_eval_top2(f)?;
    try_next_instr(ip)
}

/// Calls the given [`Func`].
///
/// This also prepares the instruction pointer and stack pointer for
/// the function call so that the stack and execution state is synchronized
/// with the outer structures.
#[inline(always)]
fn call_func(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    cache: &mut InstanceCache,
    ctx: &mut StoreInner,
    value_stack: &mut ValueStack,
    call_stack: &mut CallStack,
    code_map: &CodeMap,
    func: &Func,
) -> Result<CallOutcome, TrapCode> {
    next_instr(ip);
    sync_stack_ptr(sp, value_stack);
    call_stack.push(FuncFrame::new(*ip, cache.instance()))?;
    let wasm_func = match ctx.resolve_func(func) {
        FuncEntity::Wasm(wasm_func) => wasm_func,
        FuncEntity::Host(_host_func) => {
            cache.reset();
            return Ok(CallOutcome::Call(*func));
        }
    };
    let header = code_map.header(wasm_func.func_body());
    value_stack.prepare_wasm_call(header)?;
    *sp = value_stack.stack_ptr();
    cache.update_instance(wasm_func.instance());
    *ip = code_map.instr_ptr(header.iref());
    Ok(CallOutcome::Continue)
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
    ctx: &mut StoreInner,
    delta: impl FnOnce(&FuelCosts) -> u64,
    exec: impl FnOnce(&mut StoreInner) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<TrapCode>,
{
    if !is_fuel_metering_enabled(ctx) {
        return exec(ctx);
    }
    // At this point we know that fuel metering is enabled.
    let delta = delta(fuel_costs(ctx));
    ctx.fuel().sufficient_fuel(delta)?;
    let result = exec(ctx)?;
    ctx.fuel_mut().consume_fuel(delta).unwrap_or_else(|error| {
        panic!("remaining fuel has already been approved prior but encountered: {error}")
    });
    Ok(result)
}

/// Returns `true` if fuel metering is enabled.
#[inline]
fn is_fuel_metering_enabled(ctx: &StoreInner) -> bool {
    ctx.engine().config().get_consume_fuel()
}

/// Returns a shared reference to the [`FuelCosts`] of the [`Engine`].
///
/// [`Engine`]: crate::Engine
#[inline]
fn fuel_costs(ctx: &StoreInner) -> &FuelCosts {
    ctx.engine().config().fuel_costs()
}

/// An execution context for executing a `wasmi` function frame.
#[derive(Debug)]
#[repr(C)]
struct Executor<'ctx, 'engine> {
    /// Stores the value stack of live values on the Wasm stack.
    sp: ValueStackPtr,
    /// The pointer to the currently executed instruction.
    ip: InstructionPtr,
    /// Stores frequently used instance related data.
    cache: &'engine mut InstanceCache,
    /// A mutable [`StoreInner`] context.
    ///
    /// [`StoreInner`]: [`crate::StoreInner`]
    ctx: &'ctx mut StoreInner,
    /// The value stack.
    ///
    /// # Note
    ///
    /// This reference is mainly used to synchronize back state
    /// after manipulations to the value stack via `sp`.
    value_stack: &'engine mut ValueStack,
    /// The call stack.
    ///
    /// # Note
    ///
    /// This is used to store the stack of nested function calls.
    call_stack: &'engine mut CallStack,
    /// The Wasm function code map.
    ///
    /// # Note
    ///
    /// This is used to lookup Wasm function information.
    code_map: &'engine CodeMap,
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Creates a new [`Executor`] for executing a `wasmi` function frame.
    #[inline(always)]
    pub fn new(
        ctx: &'ctx mut StoreInner,
        cache: &'engine mut InstanceCache,
        value_stack: &'engine mut ValueStack,
        call_stack: &'engine mut CallStack,
        code_map: &'engine CodeMap,
    ) -> Self {
        let frame = call_stack.pop().expect("must have frame on the call stack");
        let ip = frame.ip();
        let sp = value_stack.stack_ptr();
        Self {
            sp,
            ip,
            cache,
            ctx,
            value_stack,
            call_stack,
            code_map,
        }
    }

    /// Executes the function frame until it returns or traps.
    #[inline(always)]
    fn execute(mut self) -> Result<WasmOutcome, TrapCode> {
        let Self {
            sp,
            ip,
            cache,
            ctx,
            value_stack,
            call_stack,
            code_map,
        } = &mut self;
        use Instruction as Instr;
        loop {
            match *ip.get() {
                Instr::LocalGet { local_depth } => visit_local_get(ip, sp, local_depth),
                Instr::LocalSet { local_depth } => visit_local_set(ip, sp, local_depth),
                Instr::LocalTee { local_depth } => visit_local_tee(ip, sp, local_depth),
                Instr::Br(params) => visit_br(ip, sp, params),
                Instr::BrIfEqz(params) => visit_br_if_eqz(ip, sp, params),
                Instr::BrIfNez(params) => visit_br_if_nez(ip, sp, params),
                Instr::BrTable { len_targets } => visit_br_table(ip, sp, len_targets),
                Instr::Unreachable => visit_unreachable()?,
                Instr::ConsumeFuel { amount } => visit_consume_fuel(ip, ctx, amount)?,
                Instr::Return(drop_keep) => {
                    if let ReturnOutcome::Host =
                        visit_ret(ip, sp, cache, value_stack, call_stack, drop_keep)
                    {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnIfNez(drop_keep) => {
                    if let ReturnOutcome::Host =
                        visit_return_if_nez(ip, sp, cache, value_stack, call_stack, drop_keep)
                    {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::Call(func) => {
                    if let CallOutcome::Call(host_func) =
                        visit_call(ip, sp, cache, ctx, value_stack, call_stack, code_map, func)?
                    {
                        return Ok(WasmOutcome::Call(host_func));
                    }
                }
                Instr::CallIndirect { table, func_type } => {
                    if let CallOutcome::Call(host_func) = visit_call_indirect(
                        ip,
                        sp,
                        cache,
                        ctx,
                        value_stack,
                        call_stack,
                        code_map,
                        table,
                        func_type,
                    )? {
                        return Ok(WasmOutcome::Call(host_func));
                    }
                }
                Instr::Drop => visit_drop(ip, sp),
                Instr::Select => visit_select(ip, sp),
                Instr::GlobalGet(global_idx) => visit_global_get(ip, sp, ctx, cache, global_idx),
                Instr::GlobalSet(global_idx) => visit_global_set(ip, sp, ctx, cache, global_idx),
                Instr::I32Load(offset) => visit_i32_load(ip, sp, ctx, cache, offset)?,
                Instr::I64Load(offset) => visit_i64_load(ip, sp, ctx, cache, offset)?,
                Instr::F32Load(offset) => visit_f32_load(ip, sp, ctx, cache, offset)?,
                Instr::F64Load(offset) => visit_f64_load(ip, sp, ctx, cache, offset)?,
                Instr::I32Load8S(offset) => visit_i32_load_i8_s(ip, sp, ctx, cache, offset)?,
                Instr::I32Load8U(offset) => visit_i32_load_i8_u(ip, sp, ctx, cache, offset)?,
                Instr::I32Load16S(offset) => visit_i32_load_i16_s(ip, sp, ctx, cache, offset)?,
                Instr::I32Load16U(offset) => visit_i32_load_i16_u(ip, sp, ctx, cache, offset)?,
                Instr::I64Load8S(offset) => visit_i64_load_i8_s(ip, sp, ctx, cache, offset)?,
                Instr::I64Load8U(offset) => visit_i64_load_i8_u(ip, sp, ctx, cache, offset)?,
                Instr::I64Load16S(offset) => visit_i64_load_i16_s(ip, sp, ctx, cache, offset)?,
                Instr::I64Load16U(offset) => visit_i64_load_i16_u(ip, sp, ctx, cache, offset)?,
                Instr::I64Load32S(offset) => visit_i64_load_i32_s(ip, sp, ctx, cache, offset)?,
                Instr::I64Load32U(offset) => visit_i64_load_i32_u(ip, sp, ctx, cache, offset)?,
                Instr::I32Store(offset) => visit_i32_store(ip, sp, ctx, cache, offset)?,
                Instr::I64Store(offset) => visit_i64_store(ip, sp, ctx, cache, offset)?,
                Instr::F32Store(offset) => visit_f32_store(ip, sp, ctx, cache, offset)?,
                Instr::F64Store(offset) => visit_f64_store(ip, sp, ctx, cache, offset)?,
                Instr::I32Store8(offset) => visit_i32_store_8(ip, sp, ctx, cache, offset)?,
                Instr::I32Store16(offset) => visit_i32_store_16(ip, sp, ctx, cache, offset)?,
                Instr::I64Store8(offset) => visit_i64_store_8(ip, sp, ctx, cache, offset)?,
                Instr::I64Store16(offset) => visit_i64_store_16(ip, sp, ctx, cache, offset)?,
                Instr::I64Store32(offset) => visit_i64_store_32(ip, sp, ctx, cache, offset)?,
                Instr::MemorySize => visit_memory_size(ip, sp, ctx, cache),
                Instr::MemoryGrow => visit_memory_grow(ip, sp, ctx, cache)?,
                Instr::MemoryFill => visit_memory_fill(ip, sp, ctx, cache)?,
                Instr::MemoryCopy => visit_memory_copy(ip, sp, ctx, cache)?,
                Instr::MemoryInit(segment) => visit_memory_init(ip, sp, ctx, cache, segment)?,
                Instr::DataDrop(segment) => visit_data_drop(ip, ctx, cache, segment),
                Instr::TableSize { table } => visit_table_size(ip, sp, ctx, cache, table),
                Instr::TableGrow { table } => visit_table_grow(ip, sp, ctx, cache, table)?,
                Instr::TableFill { table } => visit_table_fill(ip, sp, ctx, cache, table)?,
                Instr::TableGet { table } => visit_table_get(ip, sp, ctx, cache, table)?,
                Instr::TableSet { table } => visit_table_set(ip, sp, ctx, cache, table)?,
                Instr::TableCopy { dst, src } => visit_table_copy(ip, sp, ctx, cache, dst, src)?,
                Instr::TableInit { table, elem } => {
                    visit_table_init(ip, sp, ctx, cache, table, elem)?
                }
                Instr::ElemDrop(segment) => visit_element_drop(ip, ctx, cache, segment),
                Instr::RefFunc { func_index } => visit_ref_func(ip, sp, ctx, cache, func_index),
                Instr::Const(bytes) => visit_const(ip, sp, bytes),
                Instr::I32Eqz => visit_i32_eqz(ip, sp),
                Instr::I32Eq => visit_i32_eq(ip, sp),
                Instr::I32Ne => visit_i32_ne(ip, sp),
                Instr::I32LtS => visit_i32_lt_s(ip, sp),
                Instr::I32LtU => visit_i32_lt_u(ip, sp),
                Instr::I32GtS => visit_i32_gt_s(ip, sp),
                Instr::I32GtU => visit_i32_gt_u(ip, sp),
                Instr::I32LeS => visit_i32_le_s(ip, sp),
                Instr::I32LeU => visit_i32_le_u(ip, sp),
                Instr::I32GeS => visit_i32_ge_s(ip, sp),
                Instr::I32GeU => visit_i32_ge_u(ip, sp),
                Instr::I64Eqz => visit_i64_eqz(ip, sp),
                Instr::I64Eq => visit_i64_eq(ip, sp),
                Instr::I64Ne => visit_i64_ne(ip, sp),
                Instr::I64LtS => visit_i64_lt_s(ip, sp),
                Instr::I64LtU => visit_i64_lt_u(ip, sp),
                Instr::I64GtS => visit_i64_gt_s(ip, sp),
                Instr::I64GtU => visit_i64_gt_u(ip, sp),
                Instr::I64LeS => visit_i64_le_s(ip, sp),
                Instr::I64LeU => visit_i64_le_u(ip, sp),
                Instr::I64GeS => visit_i64_ge_s(ip, sp),
                Instr::I64GeU => visit_i64_ge_u(ip, sp),
                Instr::F32Eq => visit_f32_eq(ip, sp),
                Instr::F32Ne => visit_f32_ne(ip, sp),
                Instr::F32Lt => visit_f32_lt(ip, sp),
                Instr::F32Gt => visit_f32_gt(ip, sp),
                Instr::F32Le => visit_f32_le(ip, sp),
                Instr::F32Ge => visit_f32_ge(ip, sp),
                Instr::F64Eq => visit_f64_eq(ip, sp),
                Instr::F64Ne => visit_f64_ne(ip, sp),
                Instr::F64Lt => visit_f64_lt(ip, sp),
                Instr::F64Gt => visit_f64_gt(ip, sp),
                Instr::F64Le => visit_f64_le(ip, sp),
                Instr::F64Ge => visit_f64_ge(ip, sp),
                Instr::I32Clz => visit_i32_clz(ip, sp),
                Instr::I32Ctz => visit_i32_ctz(ip, sp),
                Instr::I32Popcnt => visit_i32_popcnt(ip, sp),
                Instr::I32Add => visit_i32_add(ip, sp),
                Instr::I32Sub => visit_i32_sub(ip, sp),
                Instr::I32Mul => visit_i32_mul(ip, sp),
                Instr::I32DivS => visit_i32_div_s(ip, sp)?,
                Instr::I32DivU => visit_i32_div_u(ip, sp)?,
                Instr::I32RemS => visit_i32_rem_s(ip, sp)?,
                Instr::I32RemU => visit_i32_rem_u(ip, sp)?,
                Instr::I32And => visit_i32_and(ip, sp),
                Instr::I32Or => visit_i32_or(ip, sp),
                Instr::I32Xor => visit_i32_xor(ip, sp),
                Instr::I32Shl => visit_i32_shl(ip, sp),
                Instr::I32ShrS => visit_i32_shr_s(ip, sp),
                Instr::I32ShrU => visit_i32_shr_u(ip, sp),
                Instr::I32Rotl => visit_i32_rotl(ip, sp),
                Instr::I32Rotr => visit_i32_rotr(ip, sp),
                Instr::I64Clz => visit_i64_clz(ip, sp),
                Instr::I64Ctz => visit_i64_ctz(ip, sp),
                Instr::I64Popcnt => visit_i64_popcnt(ip, sp),
                Instr::I64Add => visit_i64_add(ip, sp),
                Instr::I64Sub => visit_i64_sub(ip, sp),
                Instr::I64Mul => visit_i64_mul(ip, sp),
                Instr::I64DivS => visit_i64_div_s(ip, sp)?,
                Instr::I64DivU => visit_i64_div_u(ip, sp)?,
                Instr::I64RemS => visit_i64_rem_s(ip, sp)?,
                Instr::I64RemU => visit_i64_rem_u(ip, sp)?,
                Instr::I64And => visit_i64_and(ip, sp),
                Instr::I64Or => visit_i64_or(ip, sp),
                Instr::I64Xor => visit_i64_xor(ip, sp),
                Instr::I64Shl => visit_i64_shl(ip, sp),
                Instr::I64ShrS => visit_i64_shr_s(ip, sp),
                Instr::I64ShrU => visit_i64_shr_u(ip, sp),
                Instr::I64Rotl => visit_i64_rotl(ip, sp),
                Instr::I64Rotr => visit_i64_rotr(ip, sp),
                Instr::F32Abs => visit_f32_abs(ip, sp),
                Instr::F32Neg => visit_f32_neg(ip, sp),
                Instr::F32Ceil => visit_f32_ceil(ip, sp),
                Instr::F32Floor => visit_f32_floor(ip, sp),
                Instr::F32Trunc => visit_f32_trunc(ip, sp),
                Instr::F32Nearest => visit_f32_nearest(ip, sp),
                Instr::F32Sqrt => visit_f32_sqrt(ip, sp),
                Instr::F32Add => visit_f32_add(ip, sp),
                Instr::F32Sub => visit_f32_sub(ip, sp),
                Instr::F32Mul => visit_f32_mul(ip, sp),
                Instr::F32Div => visit_f32_div(ip, sp),
                Instr::F32Min => visit_f32_min(ip, sp),
                Instr::F32Max => visit_f32_max(ip, sp),
                Instr::F32Copysign => visit_f32_copysign(ip, sp),
                Instr::F64Abs => visit_f64_abs(ip, sp),
                Instr::F64Neg => visit_f64_neg(ip, sp),
                Instr::F64Ceil => visit_f64_ceil(ip, sp),
                Instr::F64Floor => visit_f64_floor(ip, sp),
                Instr::F64Trunc => visit_f64_trunc(ip, sp),
                Instr::F64Nearest => visit_f64_nearest(ip, sp),
                Instr::F64Sqrt => visit_f64_sqrt(ip, sp),
                Instr::F64Add => visit_f64_add(ip, sp),
                Instr::F64Sub => visit_f64_sub(ip, sp),
                Instr::F64Mul => visit_f64_mul(ip, sp),
                Instr::F64Div => visit_f64_div(ip, sp),
                Instr::F64Min => visit_f64_min(ip, sp),
                Instr::F64Max => visit_f64_max(ip, sp),
                Instr::F64Copysign => visit_f64_copysign(ip, sp),
                Instr::I32WrapI64 => visit_i32_wrap_i64(ip, sp),
                Instr::I32TruncF32S => visit_i32_trunc_f32_s(ip, sp)?,
                Instr::I32TruncF32U => visit_i32_trunc_f32_u(ip, sp)?,
                Instr::I32TruncF64S => visit_i32_trunc_f64_s(ip, sp)?,
                Instr::I32TruncF64U => visit_i32_trunc_f64_u(ip, sp)?,
                Instr::I64ExtendI32S => visit_i64_extend_i32_s(ip, sp),
                Instr::I64ExtendI32U => visit_i64_extend_i32_u(ip, sp),
                Instr::I64TruncF32S => visit_i64_trunc_f32_s(ip, sp)?,
                Instr::I64TruncF32U => visit_i64_trunc_f32_u(ip, sp)?,
                Instr::I64TruncF64S => visit_i64_trunc_f64_s(ip, sp)?,
                Instr::I64TruncF64U => visit_i64_trunc_f64_u(ip, sp)?,
                Instr::F32ConvertI32S => visit_f32_convert_i32_s(ip, sp),
                Instr::F32ConvertI32U => visit_f32_convert_i32_u(ip, sp),
                Instr::F32ConvertI64S => visit_f32_convert_i64_s(ip, sp),
                Instr::F32ConvertI64U => visit_f32_convert_i64_u(ip, sp),
                Instr::F32DemoteF64 => visit_f32_demote_f64(ip, sp),
                Instr::F64ConvertI32S => visit_f64_convert_i32_s(ip, sp),
                Instr::F64ConvertI32U => visit_f64_convert_i32_u(ip, sp),
                Instr::F64ConvertI64S => visit_f64_convert_i64_s(ip, sp),
                Instr::F64ConvertI64U => visit_f64_convert_i64_u(ip, sp),
                Instr::F64PromoteF32 => visit_f64_promote_f32(ip, sp),
                Instr::I32TruncSatF32S => visit_i32_trunc_sat_f32_s(ip, sp),
                Instr::I32TruncSatF32U => visit_i32_trunc_sat_f32_u(ip, sp),
                Instr::I32TruncSatF64S => visit_i32_trunc_sat_f64_s(ip, sp),
                Instr::I32TruncSatF64U => visit_i32_trunc_sat_f64_u(ip, sp),
                Instr::I64TruncSatF32S => visit_i64_trunc_sat_f32_s(ip, sp),
                Instr::I64TruncSatF32U => visit_i64_trunc_sat_f32_u(ip, sp),
                Instr::I64TruncSatF64S => visit_i64_trunc_sat_f64_s(ip, sp),
                Instr::I64TruncSatF64U => visit_i64_trunc_sat_f64_u(ip, sp),
                Instr::I32Extend8S => visit_i32_extend8_s(ip, sp),
                Instr::I32Extend16S => visit_i32_extend16_s(ip, sp),
                Instr::I64Extend8S => visit_i64_extend8_s(ip, sp),
                Instr::I64Extend16S => visit_i64_extend16_s(ip, sp),
                Instr::I64Extend32S => visit_i64_extend32_s(ip, sp),
            }
        }
    }
}

#[inline(always)]
fn visit_unreachable() -> Result<(), TrapCode> {
    Err(TrapCode::UnreachableCodeReached).map_err(Into::into)
}

#[inline(always)]
fn visit_consume_fuel(
    ip: &mut InstructionPtr,
    ctx: &mut StoreInner,
    amount: u64,
) -> Result<(), TrapCode> {
    // We do not have to check if fuel metering is enabled since
    // these `wasmi` instructions are only generated if fuel metering
    // is enabled to begin with.
    ctx.fuel_mut().consume_fuel(amount)?;
    try_next_instr(ip)
}

#[inline(always)]
fn visit_br(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, params: BranchParams) {
    branch_to(ip, sp, params)
}

#[inline(always)]
fn visit_br_if_eqz(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, params: BranchParams) {
    let condition = sp.pop_as();
    if condition {
        next_instr(ip)
    } else {
        branch_to(ip, sp, params)
    }
}

#[inline(always)]
fn visit_br_if_nez(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, params: BranchParams) {
    let condition = sp.pop_as();
    if condition {
        branch_to(ip, sp, params)
    } else {
        next_instr(ip)
    }
}

#[inline(always)]
fn visit_return_if_nez(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    cache: &mut InstanceCache,
    value_stack: &mut ValueStack,
    call_stack: &mut CallStack,
    drop_keep: DropKeep,
) -> ReturnOutcome {
    let condition = sp.pop_as();
    if condition {
        ret(ip, sp, cache, value_stack, call_stack, drop_keep)
    } else {
        next_instr(ip);
        ReturnOutcome::Wasm
    }
}

#[inline(always)]
fn visit_br_table(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, len_targets: usize) {
    let index: u32 = sp.pop_as();
    // The index of the default target which is the last target of the slice.
    let max_index = len_targets - 1;
    // A normalized index will always yield a target without panicking.
    let normalized_index = cmp::min(index as usize, max_index);
    // Update `pc`:
    ip.add(normalized_index + 1);
}

#[inline(always)]
fn visit_ret(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    cache: &mut InstanceCache,
    value_stack: &mut ValueStack,
    call_stack: &mut CallStack,
    drop_keep: DropKeep,
) -> ReturnOutcome {
    ret(ip, sp, cache, value_stack, call_stack, drop_keep)
}

#[inline(always)]
fn visit_local_get(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, local_depth: LocalDepth) {
    let value = sp.nth_back(local_depth.into_inner());
    sp.push(value);
    next_instr(ip)
}

#[inline(always)]
fn visit_local_set(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, local_depth: LocalDepth) {
    let new_value = sp.pop();
    sp.set_nth_back(local_depth.into_inner(), new_value);
    next_instr(ip)
}

#[inline(always)]
fn visit_local_tee(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, local_depth: LocalDepth) {
    let new_value = sp.last();
    sp.set_nth_back(local_depth.into_inner(), new_value);
    next_instr(ip)
}

#[inline(always)]
fn visit_global_get(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    global_index: GlobalIdx,
) {
    let global_value = cache.get_global(ctx, global_index);
    sp.push(global_value);
    next_instr(ip)
}

#[inline(always)]
fn visit_global_set(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    global_index: GlobalIdx,
) {
    let new_value = sp.pop();
    cache.set_global(ctx, global_index, new_value);
    next_instr(ip)
}

#[inline(always)]
fn visit_call(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    cache: &mut InstanceCache,
    ctx: &mut StoreInner,
    value_stack: &mut ValueStack,
    call_stack: &mut CallStack,
    code_map: &CodeMap,
    func_index: FuncIdx,
) -> Result<CallOutcome, TrapCode> {
    let callee = cache.get_func(ctx, func_index);
    call_func(
        ip,
        sp,
        cache,
        ctx,
        value_stack,
        call_stack,
        code_map,
        &callee,
    )
}

#[inline(always)]
fn visit_call_indirect(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    cache: &mut InstanceCache,
    ctx: &mut StoreInner,
    value_stack: &mut ValueStack,
    call_stack: &mut CallStack,
    code_map: &CodeMap,
    table: TableIdx,
    func_type: SignatureIdx,
) -> Result<CallOutcome, TrapCode> {
    let func_index: u32 = sp.pop_as();
    let table = cache.get_table(ctx, table);
    let funcref = ctx
        .resolve_table(&table)
        .get_untyped(func_index)
        .map(FuncRef::from)
        .ok_or(TrapCode::TableOutOfBounds)?;
    let func = funcref.func().ok_or(TrapCode::IndirectCallToNull)?;
    let actual_signature = ctx.resolve_func(func).ty_dedup();
    let expected_signature = ctx
        .resolve_instance(cache.instance())
        .get_signature(func_type.into_inner())
        .unwrap_or_else(|| panic!("missing signature for call_indirect at index: {func_type:?}"));
    if actual_signature != expected_signature {
        return Err(TrapCode::BadSignature).map_err(Into::into);
    }
    call_func(ip, sp, cache, ctx, value_stack, call_stack, code_map, func)
}

#[inline(always)]
fn visit_const(ip: &mut InstructionPtr, sp: &mut ValueStackPtr, bytes: UntypedValue) {
    sp.push(bytes);
    next_instr(ip)
}

#[inline(always)]
fn visit_drop(ip: &mut InstructionPtr, sp: &mut ValueStackPtr) {
    sp.drop();
    next_instr(ip)
}

#[inline(always)]
fn visit_select(ip: &mut InstructionPtr, sp: &mut ValueStackPtr) {
    sp.eval_top3(|e1, e2, e3| {
        let condition = <bool as From<UntypedValue>>::from(e3);
        if condition {
            e1
        } else {
            e2
        }
    });
    next_instr(ip)
}

#[inline(always)]
fn visit_memory_size(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
) {
    let memory = cache.default_memory(ctx);
    let result: u32 = ctx.resolve_memory(memory).current_pages().into();
    sp.push_as(result);
    next_instr(ip)
}

#[inline(always)]
fn visit_memory_grow(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
) -> Result<(), TrapCode> {
    let delta: u32 = sp.pop_as();
    let delta = match Pages::new(delta) {
        Some(pages) => pages,
        None => {
            // Cannot grow memory so we push the expected error value.
            sp.push_as(INVALID_GROWTH_ERRCODE);
            return try_next_instr(ip);
        }
    };
    let result = consume_fuel_on_success(
        ctx,
        |costs| {
            let delta_in_bytes = delta.to_bytes().unwrap_or(0) as u64;
            delta_in_bytes * costs.memory_per_byte
        },
        |ctx| {
            let memory = cache.default_memory(ctx);
            let new_pages = ctx
                .resolve_memory_mut(memory)
                .grow(delta)
                .map(u32::from)
                .map_err(|_| EntityGrowError::InvalidGrow)?;
            // The `memory.grow` operation might have invalidated the cached
            // linear memory so we need to reset it in order for the cache to
            // reload in case it is used again.
            cache.reset_default_memory_bytes();
            Ok(new_pages)
        },
    );
    let result = match result {
        Ok(result) => result,
        Err(EntityGrowError::InvalidGrow) => INVALID_GROWTH_ERRCODE,
        Err(EntityGrowError::TrapCode(trap_code)) => return Err(trap_code),
    };
    sp.push_as(result);
    try_next_instr(ip)
}

#[inline(always)]
fn visit_memory_fill(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
) -> Result<(), TrapCode> {
    // The `n`, `val` and `d` variable bindings are extracted from the Wasm specification.
    let (d, val, n) = sp.pop3();
    let n = i32::from(n) as usize;
    let offset = i32::from(d) as usize;
    let byte = u8::from(val);
    consume_fuel_on_success(
        ctx,
        |costs| n as u64 * costs.memory_per_byte,
        |ctx| {
            let memory = cache
                .default_memory_bytes(ctx)
                .get_mut(offset..)
                .and_then(|memory| memory.get_mut(..n))
                .ok_or(TrapCode::MemoryOutOfBounds)?;
            memory.fill(byte);
            Ok(())
        },
    )?;
    try_next_instr(ip)
}

#[inline(always)]
fn visit_memory_copy(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
) -> Result<(), TrapCode> {
    // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
    let (d, s, n) = sp.pop3();
    let n = i32::from(n) as usize;
    let src_offset = i32::from(s) as usize;
    let dst_offset = i32::from(d) as usize;
    consume_fuel_on_success(
        ctx,
        |costs| n as u64 * costs.memory_per_byte,
        |ctx| {
            let data = cache.default_memory_bytes(ctx);
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
    try_next_instr(ip)
}

#[inline(always)]
fn visit_memory_init(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    segment: DataSegmentIdx,
) -> Result<(), TrapCode> {
    // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
    let (d, s, n) = sp.pop3();
    let n = i32::from(n) as usize;
    let src_offset = i32::from(s) as usize;
    let dst_offset = i32::from(d) as usize;
    consume_fuel_on_success(
        ctx,
        |costs| n as u64 * costs.memory_per_byte,
        |ctx| {
            let (memory, data) = cache.get_default_memory_and_data_segment(ctx, segment);
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
    try_next_instr(ip)
}

#[inline(always)]
fn visit_data_drop(
    ip: &mut InstructionPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    segment_index: DataSegmentIdx,
) {
    let segment = cache.get_data_segment(ctx, segment_index.into_inner());
    ctx.resolve_data_segment_mut(&segment).drop_bytes();
    next_instr(ip);
}

#[inline(always)]
fn visit_table_size(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    table_index: TableIdx,
) {
    let table = cache.get_table(ctx, table_index);
    let size = ctx.resolve_table(&table).size();
    sp.push_as(size);
    next_instr(ip)
}

#[inline(always)]
fn visit_table_grow(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    table_index: TableIdx,
) -> Result<(), TrapCode> {
    let (init, delta) = sp.pop2();
    let delta: u32 = delta.into();
    let result = consume_fuel_on_success(
        ctx,
        |costs| u64::from(delta) * costs.table_per_element,
        |ctx| {
            let table = cache.get_table(ctx, table_index);
            ctx.resolve_table_mut(&table)
                .grow_untyped(delta, init)
                .map_err(|_| EntityGrowError::InvalidGrow)
        },
    );
    let result = match result {
        Ok(result) => result,
        Err(EntityGrowError::InvalidGrow) => INVALID_GROWTH_ERRCODE,
        Err(EntityGrowError::TrapCode(trap_code)) => return Err(trap_code),
    };
    sp.push_as(result);
    try_next_instr(ip)
}

#[inline(always)]
fn visit_table_fill(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    table_index: TableIdx,
) -> Result<(), TrapCode> {
    // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
    let (i, val, n) = sp.pop3();
    let dst: u32 = i.into();
    let len: u32 = n.into();
    consume_fuel_on_success(
        ctx,
        |costs| u64::from(len) * costs.table_per_element,
        |ctx| {
            let table = cache.get_table(ctx, table_index);
            ctx.resolve_table_mut(&table).fill_untyped(dst, val, len)?;
            Ok(())
        },
    )?;
    try_next_instr(ip)
}

#[inline(always)]
fn visit_table_get(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    table_index: TableIdx,
) -> Result<(), TrapCode> {
    sp.try_eval_top(|index| {
        let index: u32 = index.into();
        let table = cache.get_table(ctx, table_index);
        ctx.resolve_table(&table)
            .get_untyped(index)
            .ok_or(TrapCode::TableOutOfBounds)
    })?;
    try_next_instr(ip)
}

#[inline(always)]
fn visit_table_set(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    table_index: TableIdx,
) -> Result<(), TrapCode> {
    let (index, value) = sp.pop2();
    let index: u32 = index.into();
    let table = cache.get_table(ctx, table_index);
    ctx.resolve_table_mut(&table)
        .set_untyped(index, value)
        .map_err(|_| TrapCode::TableOutOfBounds)?;
    try_next_instr(ip)
}

#[inline(always)]
fn visit_table_copy(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    dst: TableIdx,
    src: TableIdx,
) -> Result<(), TrapCode> {
    // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
    let (d, s, n) = sp.pop3();
    let len = u32::from(n);
    let src_index = u32::from(s);
    let dst_index = u32::from(d);
    consume_fuel_on_success(
        ctx,
        |costs| u64::from(len) * costs.table_per_element,
        |ctx| {
            // Query both tables and check if they are the same:
            let dst = cache.get_table(ctx, dst);
            let src = cache.get_table(ctx, src);
            if Table::eq(&dst, &src) {
                // Copy within the same table:
                let table = ctx.resolve_table_mut(&dst);
                table.copy_within(dst_index, src_index, len)?;
            } else {
                // Copy from one table to another table:
                let (dst, src) = ctx.resolve_table_pair_mut(&dst, &src);
                TableEntity::copy(dst, dst_index, src, src_index, len)?;
            }
            Ok(())
        },
    )?;
    try_next_instr(ip)
}

#[inline(always)]
fn visit_table_init(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    table: TableIdx,
    elem: ElementSegmentIdx,
) -> Result<(), TrapCode> {
    // The `n`, `s` and `d` variable bindings are extracted from the Wasm specification.
    let (d, s, n) = sp.pop3();
    let len = u32::from(n);
    let src_index = u32::from(s);
    let dst_index = u32::from(d);
    consume_fuel_on_success(
        ctx,
        |costs| u64::from(len) * costs.table_per_element,
        |ctx| {
            let (instance, table, element) = cache.get_table_and_element_segment(ctx, table, elem);
            table.init(dst_index, element, src_index, len, |func_index| {
                instance
                    .get_func(func_index)
                    .unwrap_or_else(|| panic!("missing function at index {func_index}"))
            })?;
            Ok(())
        },
    )?;
    try_next_instr(ip)
}

#[inline(always)]
fn visit_element_drop(
    ip: &mut InstructionPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    segment_index: ElementSegmentIdx,
) {
    let segment = cache.get_element_segment(ctx, segment_index);
    ctx.resolve_element_segment_mut(&segment).drop_items();
    next_instr(ip);
}

#[inline(always)]
fn visit_ref_func(
    ip: &mut InstructionPtr,
    sp: &mut ValueStackPtr,
    ctx: &mut StoreInner,
    cache: &mut InstanceCache,
    func_index: FuncIdx,
) {
    let func = cache.get_func(ctx, func_index);
    let funcref = FuncRef::new(func);
    sp.push_as(funcref);
    next_instr(ip);
}

macro_rules! impl_visit_load {
    ( $( fn $visit_ident:ident($untyped_ident:ident); )* ) => {
        $(
            #[inline(always)]
            fn $visit_ident(
                ip: &mut InstructionPtr,
                sp: &mut ValueStackPtr,
                ctx: &mut StoreInner,
                cache: &mut InstanceCache,
                offset: Offset,
            ) -> Result<(), TrapCode> {
                execute_load_extend(ip, sp, ctx, cache, offset, UntypedValue::$untyped_ident)
            }
        )*
    }
}
impl_visit_load! {
    fn visit_i32_load(i32_load);
    fn visit_i64_load(i64_load);
    fn visit_f32_load(f32_load);
    fn visit_f64_load(f64_load);

    fn visit_i32_load_i8_s(i32_load8_s);
    fn visit_i32_load_i8_u(i32_load8_u);
    fn visit_i32_load_i16_s(i32_load16_s);
    fn visit_i32_load_i16_u(i32_load16_u);

    fn visit_i64_load_i8_s(i64_load8_s);
    fn visit_i64_load_i8_u(i64_load8_u);
    fn visit_i64_load_i16_s(i64_load16_s);
    fn visit_i64_load_i16_u(i64_load16_u);
    fn visit_i64_load_i32_s(i64_load32_s);
    fn visit_i64_load_i32_u(i64_load32_u);
}

macro_rules! impl_visit_store {
    ( $( fn $visit_ident:ident($untyped_ident:ident); )* ) => {
        $(
            #[inline(always)]
            fn $visit_ident(
                ip: &mut InstructionPtr,
                sp: &mut ValueStackPtr,
                ctx: &mut StoreInner,
                cache: &mut InstanceCache,
                offset: Offset,
            ) -> Result<(), TrapCode> {
                execute_store_wrap(ip, sp, ctx, cache, offset, UntypedValue::$untyped_ident)
            }
        )*
    }
}
impl_visit_store! {
    fn visit_i32_store(i32_store);
    fn visit_i64_store(i64_store);
    fn visit_f32_store(f32_store);
    fn visit_f64_store(f64_store);

    fn visit_i32_store_8(i32_store8);
    fn visit_i32_store_16(i32_store16);

    fn visit_i64_store_8(i64_store8);
    fn visit_i64_store_16(i64_store16);
    fn visit_i64_store_32(i64_store32);
}

macro_rules! impl_visit_unary {
    ( $( fn $visit_ident:ident($untyped_ident:ident); )* ) => {
        $(
            #[inline(always)]
            fn $visit_ident(ip: &mut InstructionPtr, sp: &mut ValueStackPtr) {
                execute_unary(ip, sp, UntypedValue::$untyped_ident)
            }
        )*
    }
}
impl_visit_unary! {
    fn visit_i32_eqz(i32_eqz);
    fn visit_i64_eqz(i64_eqz);

    fn visit_i32_clz(i32_clz);
    fn visit_i32_ctz(i32_ctz);
    fn visit_i32_popcnt(i32_popcnt);

    fn visit_i64_clz(i64_clz);
    fn visit_i64_ctz(i64_ctz);
    fn visit_i64_popcnt(i64_popcnt);

    fn visit_f32_abs(f32_abs);
    fn visit_f32_neg(f32_neg);
    fn visit_f32_ceil(f32_ceil);
    fn visit_f32_floor(f32_floor);
    fn visit_f32_trunc(f32_trunc);
    fn visit_f32_nearest(f32_nearest);
    fn visit_f32_sqrt(f32_sqrt);

    fn visit_f64_abs(f64_abs);
    fn visit_f64_neg(f64_neg);
    fn visit_f64_ceil(f64_ceil);
    fn visit_f64_floor(f64_floor);
    fn visit_f64_trunc(f64_trunc);
    fn visit_f64_nearest(f64_nearest);
    fn visit_f64_sqrt(f64_sqrt);

    fn visit_i32_wrap_i64(i32_wrap_i64);
    fn visit_i64_extend_i32_s(i64_extend_i32_s);
    fn visit_i64_extend_i32_u(i64_extend_i32_u);

    fn visit_f32_convert_i32_s(f32_convert_i32_s);
    fn visit_f32_convert_i32_u(f32_convert_i32_u);
    fn visit_f32_convert_i64_s(f32_convert_i64_s);
    fn visit_f32_convert_i64_u(f32_convert_i64_u);
    fn visit_f32_demote_f64(f32_demote_f64);
    fn visit_f64_convert_i32_s(f64_convert_i32_s);
    fn visit_f64_convert_i32_u(f64_convert_i32_u);
    fn visit_f64_convert_i64_s(f64_convert_i64_s);
    fn visit_f64_convert_i64_u(f64_convert_i64_u);
    fn visit_f64_promote_f32(f64_promote_f32);

    fn visit_i32_extend8_s(i32_extend8_s);
    fn visit_i32_extend16_s(i32_extend16_s);
    fn visit_i64_extend8_s(i64_extend8_s);
    fn visit_i64_extend16_s(i64_extend16_s);
    fn visit_i64_extend32_s(i64_extend32_s);

    fn visit_i32_trunc_sat_f32_s(i32_trunc_sat_f32_s);
    fn visit_i32_trunc_sat_f32_u(i32_trunc_sat_f32_u);
    fn visit_i32_trunc_sat_f64_s(i32_trunc_sat_f64_s);
    fn visit_i32_trunc_sat_f64_u(i32_trunc_sat_f64_u);
    fn visit_i64_trunc_sat_f32_s(i64_trunc_sat_f32_s);
    fn visit_i64_trunc_sat_f32_u(i64_trunc_sat_f32_u);
    fn visit_i64_trunc_sat_f64_s(i64_trunc_sat_f64_s);
    fn visit_i64_trunc_sat_f64_u(i64_trunc_sat_f64_u);
}

macro_rules! impl_visit_fallible_unary {
    ( $( fn $visit_ident:ident($untyped_ident:ident); )* ) => {
        $(
            #[inline(always)]
            fn $visit_ident(ip: &mut InstructionPtr, sp: &mut ValueStackPtr) -> Result<(), TrapCode> {
                try_execute_unary(ip, sp, UntypedValue::$untyped_ident)
            }
        )*
    }
}
impl_visit_fallible_unary! {
    fn visit_i32_trunc_f32_s(i32_trunc_f32_s);
    fn visit_i32_trunc_f32_u(i32_trunc_f32_u);
    fn visit_i32_trunc_f64_s(i32_trunc_f64_s);
    fn visit_i32_trunc_f64_u(i32_trunc_f64_u);

    fn visit_i64_trunc_f32_s(i64_trunc_f32_s);
    fn visit_i64_trunc_f32_u(i64_trunc_f32_u);
    fn visit_i64_trunc_f64_s(i64_trunc_f64_s);
    fn visit_i64_trunc_f64_u(i64_trunc_f64_u);
}

macro_rules! impl_visit_binary {
    ( $( fn $visit_ident:ident($untyped_ident:ident); )* ) => {
        $(
            #[inline(always)]
            fn $visit_ident(ip: &mut InstructionPtr, sp: &mut ValueStackPtr) {
                execute_binary(ip, sp, UntypedValue::$untyped_ident)
            }
        )*
    }
}
impl_visit_binary! {
    fn visit_i32_eq(i32_eq);
    fn visit_i32_ne(i32_ne);
    fn visit_i32_lt_s(i32_lt_s);
    fn visit_i32_lt_u(i32_lt_u);
    fn visit_i32_gt_s(i32_gt_s);
    fn visit_i32_gt_u(i32_gt_u);
    fn visit_i32_le_s(i32_le_s);
    fn visit_i32_le_u(i32_le_u);
    fn visit_i32_ge_s(i32_ge_s);
    fn visit_i32_ge_u(i32_ge_u);

    fn visit_i64_eq(i64_eq);
    fn visit_i64_ne(i64_ne);
    fn visit_i64_lt_s(i64_lt_s);
    fn visit_i64_lt_u(i64_lt_u);
    fn visit_i64_gt_s(i64_gt_s);
    fn visit_i64_gt_u(i64_gt_u);
    fn visit_i64_le_s(i64_le_s);
    fn visit_i64_le_u(i64_le_u);
    fn visit_i64_ge_s(i64_ge_s);
    fn visit_i64_ge_u(i64_ge_u);

    fn visit_f32_eq(f32_eq);
    fn visit_f32_ne(f32_ne);
    fn visit_f32_lt(f32_lt);
    fn visit_f32_gt(f32_gt);
    fn visit_f32_le(f32_le);
    fn visit_f32_ge(f32_ge);

    fn visit_f64_eq(f64_eq);
    fn visit_f64_ne(f64_ne);
    fn visit_f64_lt(f64_lt);
    fn visit_f64_gt(f64_gt);
    fn visit_f64_le(f64_le);
    fn visit_f64_ge(f64_ge);

    fn visit_i32_add(i32_add);
    fn visit_i32_sub(i32_sub);
    fn visit_i32_mul(i32_mul);
    fn visit_i32_and(i32_and);
    fn visit_i32_or(i32_or);
    fn visit_i32_xor(i32_xor);
    fn visit_i32_shl(i32_shl);
    fn visit_i32_shr_s(i32_shr_s);
    fn visit_i32_shr_u(i32_shr_u);
    fn visit_i32_rotl(i32_rotl);
    fn visit_i32_rotr(i32_rotr);

    fn visit_i64_add(i64_add);
    fn visit_i64_sub(i64_sub);
    fn visit_i64_mul(i64_mul);
    fn visit_i64_and(i64_and);
    fn visit_i64_or(i64_or);
    fn visit_i64_xor(i64_xor);
    fn visit_i64_shl(i64_shl);
    fn visit_i64_shr_s(i64_shr_s);
    fn visit_i64_shr_u(i64_shr_u);
    fn visit_i64_rotl(i64_rotl);
    fn visit_i64_rotr(i64_rotr);

    fn visit_f32_add(f32_add);
    fn visit_f32_sub(f32_sub);
    fn visit_f32_mul(f32_mul);
    fn visit_f32_div(f32_div);
    fn visit_f32_min(f32_min);
    fn visit_f32_max(f32_max);
    fn visit_f32_copysign(f32_copysign);

    fn visit_f64_add(f64_add);
    fn visit_f64_sub(f64_sub);
    fn visit_f64_mul(f64_mul);
    fn visit_f64_div(f64_div);
    fn visit_f64_min(f64_min);
    fn visit_f64_max(f64_max);
    fn visit_f64_copysign(f64_copysign);
}

macro_rules! impl_visit_fallible_binary {
    ( $( fn $visit_ident:ident($untyped_ident:ident); )* ) => {
        $(
            #[inline(always)]
            fn $visit_ident(ip: &mut InstructionPtr, sp: &mut ValueStackPtr) -> Result<(), TrapCode> {
                try_execute_binary(ip, sp, UntypedValue::$untyped_ident)
            }
        )*
    }
}
impl_visit_fallible_binary! {
    fn visit_i32_div_s(i32_div_s);
    fn visit_i32_div_u(i32_div_u);
    fn visit_i32_rem_s(i32_rem_s);
    fn visit_i32_rem_u(i32_rem_u);

    fn visit_i64_div_s(i64_div_s);
    fn visit_i64_div_u(i64_div_u);
    fn visit_i64_rem_s(i64_rem_s);
    fn visit_i64_rem_u(i64_rem_u);
}
