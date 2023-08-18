#![allow(dead_code, unused_variables)]

use crate::{
    core::TrapCode,
    engine::{
        bytecode::{BlockFuel, BranchOffset},
        bytecode2::{AnyConst32, Const32, Instruction, Register, RegisterSpanIter},
        cache::InstanceCache,
        code_map::{CodeMap2 as CodeMap, InstructionPtr2 as InstructionPtr},
        regmach::stack::{CallFrame, CallStack, ValueStack, ValueStackPtr},
    },
    store::ResourceLimiterRef,
    Func,
    Instance,
    StoreInner,
};
use core::cmp;
use wasmi_core::UntypedValue;

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
    Call { host_func: Func, instance: Instance },
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
    Call { host_func: Func, instance: Instance },
}

/// The kind of a function call.
#[derive(Debug, Copy, Clone)]
pub enum CallKind {
    /// A nested function call.
    Nested,
    /// A tailing function call.
    Tail,
}

/// The outcome of a Wasm return statement.
#[derive(Debug, Copy, Clone)]
pub enum ReturnOutcome {
    /// The call returns to a nested Wasm caller.
    Wasm,
    /// The call returns back to the host.
    Host,
}

/// Executes compiled function instructions until either
///
/// - returning from the root function
/// - calling a host function
/// - encountering a trap
///
/// # Errors
///
/// If the execution traps.
#[inline(never)]
pub fn execute_instrs<'ctx, 'engine>(
    ctx: &'ctx mut StoreInner,
    cache: &'engine mut InstanceCache,
    value_stack: &'engine mut ValueStack,
    call_stack: &'engine mut CallStack,
    code_map: &'engine CodeMap,
    resource_limiter: &'ctx mut ResourceLimiterRef<'ctx>,
) -> Result<WasmOutcome, TrapCode> {
    Executor::new(ctx, cache, value_stack, call_stack, code_map).execute(resource_limiter)
}

/// An error that can occur upon `memory.grow` or `table.grow`.
#[derive(Copy, Clone)]
pub enum EntityGrowError {
    /// Usually a [`TrapCode::OutOfFuel`] trap.
    TrapCode(TrapCode),
    /// Encountered when `memory.grow` or `table.grow` fails.
    InvalidGrow,
}

impl EntityGrowError {
    /// The WebAssembly specification demands to return this value
    /// if the `memory.grow` or `table.grow` operations fail.
    const ERROR_CODE: u32 = u32::MAX;
}

impl From<TrapCode> for EntityGrowError {
    fn from(trap_code: TrapCode) -> Self {
        Self::TrapCode(trap_code)
    }
}

/// An execution context for executing a `wasmi` function frame.
#[derive(Debug)]
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
        let frame = call_stack
            .pop()
            .expect("must have call frame on the call stack");
        // Safety: We are using the frame's own base offset as input because it is
        //         guaranteed by the Wasm validation and translation phase to be
        //         valid for all register indices used by the associated function body.
        let sp = unsafe { value_stack.stack_ptr_at(frame.base_offset()) };
        let ip = frame.instr_ptr();
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
    fn execute(
        mut self,
        resource_limiter: &'ctx mut ResourceLimiterRef<'ctx>,
    ) -> Result<WasmOutcome, TrapCode> {
        use Instruction as Instr;
        _ = resource_limiter.as_resource_limiter(); // TODO: remove
        loop {
            match *self.ip.get() {
                Instr::TableIdx(_)
                | Instr::DataSegmentIdx(_)
                | Instr::ElementSegmentIdx(_)
                | Instr::Const32(_)
                | Instr::I64Const32(_)
                | Instr::F64Const32(_)
                | Instr::Register(_) => self.invalid_instruction_word()?,
                Instr::Trap(_) => self.execute_unreachable()?,
                Instr::ConsumeFuel(block_fuel) => self.execute_consume_fuel(block_fuel)?,
                Instr::Return => {
                    if let ReturnOutcome::Host = self.execute_return() {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnReg { value } => {
                    if let ReturnOutcome::Host = self.execute_return_reg(value) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnImm32 { value } => {
                    if let ReturnOutcome::Host = self.execute_return_imm32(value) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnI64Imm32 { value } => {
                    if let ReturnOutcome::Host = self.execute_return_i64imm32(value) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnF64Imm32 { value } => {
                    if let ReturnOutcome::Host = self.execute_return_f64imm32(value) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnMany { values } => {
                    if let ReturnOutcome::Host = self.execute_return_many(values) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnNez { condition } => {
                    if let ReturnOutcome::Host = self.execute_return_nez(condition) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnNezReg { condition, value } => {
                    if let ReturnOutcome::Host = self.execute_return_nez_reg(condition, value) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnNezImm32 { condition, value } => {
                    if let ReturnOutcome::Host = self.execute_return_nez_imm32(condition, value) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnNezI64Imm32 { condition, value } => {
                    if let ReturnOutcome::Host = self.execute_return_nez_i64imm32(condition, value)
                    {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnNezF64Imm32 { condition, value } => {
                    if let ReturnOutcome::Host = self.execute_return_nez_f64imm32(condition, value)
                    {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::ReturnNezMany { condition, values } => {
                    if let ReturnOutcome::Host = self.execute_return_nez_many(condition, values) {
                        return Ok(WasmOutcome::Return);
                    }
                }
                Instr::Branch { offset } => self.execute_branch(offset),
                Instr::BranchEqz { condition, offset } => {
                    self.execute_branch_nez(condition, offset)
                }
                Instr::BranchNez { condition, offset } => {
                    self.execute_branch_eqz(condition, offset)
                }
                Instr::BranchTable { index, len_targets } => {
                    self.execute_branch_table(index, len_targets)
                }
                Instr::Copy { result, value } => todo!(),
                Instr::CopyImm32 { result, value } => todo!(),
                Instr::CopyI64Imm32 { result, value } => todo!(),
                Instr::CopyF64Imm32 { result, value } => todo!(),
                Instr::CopySpan {
                    results,
                    values,
                    len,
                } => todo!(),
                Instr::CallParams {
                    params,
                    len_results,
                } => todo!(),
                Instr::CallIndirectParams { index, table } => todo!(),
                Instr::CallIndirectParamsImm16 { index, table } => todo!(),
                Instr::ReturnCallInternal0 { func } => todo!(),
                Instr::ReturnCallInternal { func } => todo!(),
                Instr::ReturnCallImported0 { func } => todo!(),
                Instr::ReturnCallImported { func } => todo!(),
                Instr::ReturnCallIndirect0 { func_type } => todo!(),
                Instr::ReturnCallIndirect { func_type } => todo!(),
                Instr::CallInternal0 { results, func } => todo!(),
                Instr::CallInternal { results, func } => todo!(),
                Instr::CallImported0 { results, func } => todo!(),
                Instr::CallImported { results, func } => todo!(),
                Instr::CallIndirect0 { results, func_type } => todo!(),
                Instr::CallIndirect { results, func_type } => todo!(),
                Instr::Select {
                    result,
                    condition,
                    lhs,
                } => todo!(),
                Instr::SelectRev {
                    result,
                    condition,
                    rhs,
                } => todo!(),
                Instr::SelectImm32 {
                    result_or_condition,
                    lhs_or_rhs,
                } => todo!(),
                Instr::SelectI64Imm32 {
                    result_or_condition,
                    lhs_or_rhs,
                } => todo!(),
                Instr::SelectF64Imm32 {
                    result_or_condition,
                    lhs_or_rhs,
                } => todo!(),
                Instr::RefFunc { result, func } => todo!(),
                Instr::TableGet { result, index } => todo!(),
                Instr::TableGetImm { result, index } => todo!(),
                Instr::TableSize { result, table } => todo!(),
                Instr::TableSet { index, value } => todo!(),
                Instr::TableSetAt { index, value } => todo!(),
                Instr::TableCopy { dst, src, len } => todo!(),
                Instr::TableCopyTo { dst, src, len } => todo!(),
                Instr::TableCopyFrom { dst, src, len } => todo!(),
                Instr::TableCopyFromTo { dst, src, len } => todo!(),
                Instr::TableCopyExact { dst, src, len } => todo!(),
                Instr::TableCopyToExact { dst, src, len } => todo!(),
                Instr::TableCopyFromExact { dst, src, len } => todo!(),
                Instr::TableCopyFromToExact { dst, src, len } => todo!(),
                Instr::TableInit { dst, src, len } => todo!(),
                Instr::TableInitTo { dst, src, len } => todo!(),
                Instr::TableInitFrom { dst, src, len } => todo!(),
                Instr::TableInitFromTo { dst, src, len } => todo!(),
                Instr::TableInitExact { dst, src, len } => todo!(),
                Instr::TableInitToExact { dst, src, len } => todo!(),
                Instr::TableInitFromExact { dst, src, len } => todo!(),
                Instr::TableInitFromToExact { dst, src, len } => todo!(),
                Instr::TableFill { dst, len, value } => todo!(),
                Instr::TableFillAt { dst, len, value } => todo!(),
                Instr::TableFillExact { dst, len, value } => todo!(),
                Instr::TableFillAtExact { dst, len, value } => todo!(),
                Instr::TableGrow {
                    result,
                    delta,
                    value,
                } => todo!(),
                Instr::TableGrowImm {
                    result,
                    delta,
                    value,
                } => todo!(),
                Instr::ElemDrop(_) => todo!(),
                Instr::DataDrop(_) => todo!(),
                Instr::MemorySize { result } => todo!(),
                Instr::MemoryGrow { result, delta } => todo!(),
                Instr::MemoryGrowBy { result, delta } => todo!(),
                Instr::MemoryCopy { dst, src, len } => todo!(),
                Instr::MemoryCopyTo { dst, src, len } => todo!(),
                Instr::MemoryCopyFrom { dst, src, len } => todo!(),
                Instr::MemoryCopyFromTo { dst, src, len } => todo!(),
                Instr::MemoryCopyExact { dst, src, len } => todo!(),
                Instr::MemoryCopyToExact { dst, src, len } => todo!(),
                Instr::MemoryCopyFromExact { dst, src, len } => todo!(),
                Instr::MemoryCopyFromToExact { dst, src, len } => todo!(),
                Instr::MemoryFill { dst, value, len } => todo!(),
                Instr::MemoryFillAt { dst, value, len } => todo!(),
                Instr::MemoryFillImm { dst, value, len } => todo!(),
                Instr::MemoryFillExact { dst, value, len } => todo!(),
                Instr::MemoryFillAtImm { dst, value, len } => todo!(),
                Instr::MemoryFillAtExact { dst, value, len } => todo!(),
                Instr::MemoryFillImmExact { dst, value, len } => todo!(),
                Instr::MemoryFillAtImmExact { dst, value, len } => todo!(),
                Instr::MemoryInit { dst, src, len } => todo!(),
                Instr::MemoryInitTo { dst, src, len } => todo!(),
                Instr::MemoryInitFrom { dst, src, len } => todo!(),
                Instr::MemoryInitFromTo { dst, src, len } => todo!(),
                Instr::MemoryInitExact { dst, src, len } => todo!(),
                Instr::MemoryInitToExact { dst, src, len } => todo!(),
                Instr::MemoryInitFromExact { dst, src, len } => todo!(),
                Instr::MemoryInitFromToExact { dst, src, len } => todo!(),
                Instr::GlobalGet { result, global } => todo!(),
                Instr::GlobalSet { global, input } => todo!(),
                Instr::GlobalSetI32Imm16 { global, input } => todo!(),
                Instr::GlobalSetI64Imm16 { global, input } => todo!(),
                Instr::I32Load(_) => todo!(),
                Instr::I32LoadAt(_) => todo!(),
                Instr::I32LoadOffset16(_) => todo!(),
                Instr::I64Load(_) => todo!(),
                Instr::I64LoadAt(_) => todo!(),
                Instr::I64LoadOffset16(_) => todo!(),
                Instr::F32Load(_) => todo!(),
                Instr::F32LoadAt(_) => todo!(),
                Instr::F32LoadOffset16(_) => todo!(),
                Instr::F64Load(_) => todo!(),
                Instr::F64LoadAt(_) => todo!(),
                Instr::F64LoadOffset16(_) => todo!(),
                Instr::I32Load8s(_) => todo!(),
                Instr::I32Load8sAt(_) => todo!(),
                Instr::I32Load8sOffset16(_) => todo!(),
                Instr::I32Load8u(_) => todo!(),
                Instr::I32Load8uAt(_) => todo!(),
                Instr::I32Load8uOffset16(_) => todo!(),
                Instr::I32Load16s(_) => todo!(),
                Instr::I32Load16sAt(_) => todo!(),
                Instr::I32Load16sOffset16(_) => todo!(),
                Instr::I32Load16u(_) => todo!(),
                Instr::I32Load16uAt(_) => todo!(),
                Instr::I32Load16uOffset16(_) => todo!(),
                Instr::I64Load8s(_) => todo!(),
                Instr::I64Load8sAt(_) => todo!(),
                Instr::I64Load8sOffset16(_) => todo!(),
                Instr::I64Load8u(_) => todo!(),
                Instr::I64Load8uAt(_) => todo!(),
                Instr::I64Load8uOffset16(_) => todo!(),
                Instr::I64Load16s(_) => todo!(),
                Instr::I64Load16sAt(_) => todo!(),
                Instr::I64Load16sOffset16(_) => todo!(),
                Instr::I64Load16u(_) => todo!(),
                Instr::I64Load16uAt(_) => todo!(),
                Instr::I64Load16uOffset16(_) => todo!(),
                Instr::I64Load32s(_) => todo!(),
                Instr::I64Load32sAt(_) => todo!(),
                Instr::I64Load32sOffset16(_) => todo!(),
                Instr::I64Load32u(_) => todo!(),
                Instr::I64Load32uAt(_) => todo!(),
                Instr::I64Load32uOffset16(_) => todo!(),
                Instr::I32Store(_) => todo!(),
                Instr::I32StoreOffset16(_) => todo!(),
                Instr::I32StoreOffset16Imm16(_) => todo!(),
                Instr::I32StoreAt(_) => todo!(),
                Instr::I32StoreAtImm16(_) => todo!(),
                Instr::I32Store8(_) => todo!(),
                Instr::I32Store8Offset16(_) => todo!(),
                Instr::I32Store8Offset16Imm(_) => todo!(),
                Instr::I32Store8At(_) => todo!(),
                Instr::I32Store8AtImm(_) => todo!(),
                Instr::I32Store16(_) => todo!(),
                Instr::I32Store16Offset16(_) => todo!(),
                Instr::I32Store16Offset16Imm(_) => todo!(),
                Instr::I32Store16At(_) => todo!(),
                Instr::I32Store16AtImm(_) => todo!(),
                Instr::I64Store(_) => todo!(),
                Instr::I64StoreOffset16(_) => todo!(),
                Instr::I64StoreOffset16Imm16(_) => todo!(),
                Instr::I64StoreAt(_) => todo!(),
                Instr::I64StoreAtImm16(_) => todo!(),
                Instr::I64Store8(_) => todo!(),
                Instr::I64Store8Offset16(_) => todo!(),
                Instr::I64Store8Offset16Imm(_) => todo!(),
                Instr::I64Store8At(_) => todo!(),
                Instr::I64Store8AtImm(_) => todo!(),
                Instr::I64Store16(_) => todo!(),
                Instr::I64Store16Offset16(_) => todo!(),
                Instr::I64Store16Offset16Imm(_) => todo!(),
                Instr::I64Store16At(_) => todo!(),
                Instr::I64Store16AtImm(_) => todo!(),
                Instr::I64Store32(_) => todo!(),
                Instr::I64Store32Offset16(_) => todo!(),
                Instr::I64Store32Offset16Imm16(_) => todo!(),
                Instr::I64Store32At(_) => todo!(),
                Instr::I64Store32AtImm16(_) => todo!(),
                Instr::F32Store(_) => todo!(),
                Instr::F32StoreOffset16(_) => todo!(),
                Instr::F32StoreAt(_) => todo!(),
                Instr::F64Store(_) => todo!(),
                Instr::F64StoreOffset16(_) => todo!(),
                Instr::F64StoreAt(_) => todo!(),
                Instr::I32Eq(_) => todo!(),
                Instr::I32EqImm16(_) => todo!(),
                Instr::I64Eq(_) => todo!(),
                Instr::I64EqImm16(_) => todo!(),
                Instr::I32Ne(_) => todo!(),
                Instr::I32NeImm16(_) => todo!(),
                Instr::I64Ne(_) => todo!(),
                Instr::I64NeImm16(_) => todo!(),
                Instr::I32LtS(_) => todo!(),
                Instr::I32LtU(_) => todo!(),
                Instr::I32LtSImm16(_) => todo!(),
                Instr::I32LtUImm16(_) => todo!(),
                Instr::I64LtS(_) => todo!(),
                Instr::I64LtU(_) => todo!(),
                Instr::I64LtSImm16(_) => todo!(),
                Instr::I64LtUImm16(_) => todo!(),
                Instr::I32GtS(_) => todo!(),
                Instr::I32GtU(_) => todo!(),
                Instr::I32GtSImm16(_) => todo!(),
                Instr::I32GtUImm16(_) => todo!(),
                Instr::I64GtS(_) => todo!(),
                Instr::I64GtU(_) => todo!(),
                Instr::I64GtSImm16(_) => todo!(),
                Instr::I64GtUImm16(_) => todo!(),
                Instr::I32LeS(_) => todo!(),
                Instr::I32LeU(_) => todo!(),
                Instr::I32LeSImm16(_) => todo!(),
                Instr::I32LeUImm16(_) => todo!(),
                Instr::I64LeS(_) => todo!(),
                Instr::I64LeU(_) => todo!(),
                Instr::I64LeSImm16(_) => todo!(),
                Instr::I64LeUImm16(_) => todo!(),
                Instr::I32GeS(_) => todo!(),
                Instr::I32GeU(_) => todo!(),
                Instr::I32GeSImm16(_) => todo!(),
                Instr::I32GeUImm16(_) => todo!(),
                Instr::I64GeS(_) => todo!(),
                Instr::I64GeU(_) => todo!(),
                Instr::I64GeSImm16(_) => todo!(),
                Instr::I64GeUImm16(_) => todo!(),
                Instr::F32Eq(_) => todo!(),
                Instr::F64Eq(_) => todo!(),
                Instr::F32Ne(_) => todo!(),
                Instr::F64Ne(_) => todo!(),
                Instr::F32Lt(_) => todo!(),
                Instr::F64Lt(_) => todo!(),
                Instr::F32Le(_) => todo!(),
                Instr::F64Le(_) => todo!(),
                Instr::F32Gt(_) => todo!(),
                Instr::F64Gt(_) => todo!(),
                Instr::F32Ge(_) => todo!(),
                Instr::F64Ge(_) => todo!(),
                Instr::I32Clz(_) => todo!(),
                Instr::I64Clz(_) => todo!(),
                Instr::I32Ctz(_) => todo!(),
                Instr::I64Ctz(_) => todo!(),
                Instr::I32Popcnt(_) => todo!(),
                Instr::I64Popcnt(_) => todo!(),
                Instr::I32Add(_) => todo!(),
                Instr::I64Add(_) => todo!(),
                Instr::I32AddImm16(_) => todo!(),
                Instr::I64AddImm16(_) => todo!(),
                Instr::I32Sub(_) => todo!(),
                Instr::I64Sub(_) => todo!(),
                Instr::I32SubImm16(_) => todo!(),
                Instr::I64SubImm16(_) => todo!(),
                Instr::I32SubImm16Rev(_) => todo!(),
                Instr::I64SubImm16Rev(_) => todo!(),
                Instr::I32Mul(_) => todo!(),
                Instr::I64Mul(_) => todo!(),
                Instr::I32MulImm16(_) => todo!(),
                Instr::I64MulImm16(_) => todo!(),
                Instr::I32DivS(_) => todo!(),
                Instr::I64DivS(_) => todo!(),
                Instr::I32DivSImm16(_) => todo!(),
                Instr::I64DivSImm16(_) => todo!(),
                Instr::I32DivSImm16Rev(_) => todo!(),
                Instr::I64DivSImm16Rev(_) => todo!(),
                Instr::I32DivU(_) => todo!(),
                Instr::I64DivU(_) => todo!(),
                Instr::I32DivUImm16(_) => todo!(),
                Instr::I64DivUImm16(_) => todo!(),
                Instr::I32DivUImm16Rev(_) => todo!(),
                Instr::I64DivUImm16Rev(_) => todo!(),
                Instr::I32RemS(_) => todo!(),
                Instr::I64RemS(_) => todo!(),
                Instr::I32RemSImm16(_) => todo!(),
                Instr::I64RemSImm16(_) => todo!(),
                Instr::I32RemSImm16Rev(_) => todo!(),
                Instr::I64RemSImm16Rev(_) => todo!(),
                Instr::I32RemU(_) => todo!(),
                Instr::I64RemU(_) => todo!(),
                Instr::I32RemUImm16(_) => todo!(),
                Instr::I64RemUImm16(_) => todo!(),
                Instr::I32RemUImm16Rev(_) => todo!(),
                Instr::I64RemUImm16Rev(_) => todo!(),
                Instr::I32And(_) => todo!(),
                Instr::I64And(_) => todo!(),
                Instr::I32AndImm16(_) => todo!(),
                Instr::I64AndImm16(_) => todo!(),
                Instr::I32Or(_) => todo!(),
                Instr::I64Or(_) => todo!(),
                Instr::I32OrImm16(_) => todo!(),
                Instr::I64OrImm16(_) => todo!(),
                Instr::I32Xor(_) => todo!(),
                Instr::I64Xor(_) => todo!(),
                Instr::I32XorImm16(_) => todo!(),
                Instr::I64XorImm16(_) => todo!(),
                Instr::I32Shl(_) => todo!(),
                Instr::I64Shl(_) => todo!(),
                Instr::I32ShlImm(_) => todo!(),
                Instr::I64ShlImm(_) => todo!(),
                Instr::I32ShlImm16Rev(_) => todo!(),
                Instr::I64ShlImm16Rev(_) => todo!(),
                Instr::I32ShrU(_) => todo!(),
                Instr::I64ShrU(_) => todo!(),
                Instr::I32ShrUImm(_) => todo!(),
                Instr::I64ShrUImm(_) => todo!(),
                Instr::I32ShrUImm16Rev(_) => todo!(),
                Instr::I64ShrUImm16Rev(_) => todo!(),
                Instr::I32ShrS(_) => todo!(),
                Instr::I64ShrS(_) => todo!(),
                Instr::I32ShrSImm(_) => todo!(),
                Instr::I64ShrSImm(_) => todo!(),
                Instr::I32ShrSImm16Rev(_) => todo!(),
                Instr::I64ShrSImm16Rev(_) => todo!(),
                Instr::I32Rotl(_) => todo!(),
                Instr::I64Rotl(_) => todo!(),
                Instr::I32RotlImm(_) => todo!(),
                Instr::I64RotlImm(_) => todo!(),
                Instr::I32RotlImm16Rev(_) => todo!(),
                Instr::I64RotlImm16Rev(_) => todo!(),
                Instr::I32Rotr(_) => todo!(),
                Instr::I64Rotr(_) => todo!(),
                Instr::I32RotrImm(_) => todo!(),
                Instr::I64RotrImm(_) => todo!(),
                Instr::I32RotrImm16Rev(_) => todo!(),
                Instr::I64RotrImm16Rev(_) => todo!(),
                Instr::F32Abs(_) => todo!(),
                Instr::F64Abs(_) => todo!(),
                Instr::F32Neg(_) => todo!(),
                Instr::F64Neg(_) => todo!(),
                Instr::F32Ceil(_) => todo!(),
                Instr::F64Ceil(_) => todo!(),
                Instr::F32Floor(_) => todo!(),
                Instr::F64Floor(_) => todo!(),
                Instr::F32Trunc(_) => todo!(),
                Instr::F64Trunc(_) => todo!(),
                Instr::F32Nearest(_) => todo!(),
                Instr::F64Nearest(_) => todo!(),
                Instr::F32Sqrt(_) => todo!(),
                Instr::F64Sqrt(_) => todo!(),
                Instr::F32Add(_) => todo!(),
                Instr::F64Add(_) => todo!(),
                Instr::F32Sub(_) => todo!(),
                Instr::F64Sub(_) => todo!(),
                Instr::F32Mul(_) => todo!(),
                Instr::F64Mul(_) => todo!(),
                Instr::F32Div(_) => todo!(),
                Instr::F64Div(_) => todo!(),
                Instr::F32Min(_) => todo!(),
                Instr::F64Min(_) => todo!(),
                Instr::F32Max(_) => todo!(),
                Instr::F64Max(_) => todo!(),
                Instr::F32Copysign(_) => todo!(),
                Instr::F64Copysign(_) => todo!(),
                Instr::F32CopysignImm(_) => todo!(),
                Instr::F64CopysignImm(_) => todo!(),
                Instr::I32WrapI64(_) => todo!(),
                Instr::I64ExtendI32S(_) => todo!(),
                Instr::I64ExtendI32U(_) => todo!(),
                Instr::I32TruncF32S(_) => todo!(),
                Instr::I32TruncF32U(_) => todo!(),
                Instr::I32TruncF64S(_) => todo!(),
                Instr::I32TruncF64U(_) => todo!(),
                Instr::I64TruncF32S(_) => todo!(),
                Instr::I64TruncF32U(_) => todo!(),
                Instr::I64TruncF64S(_) => todo!(),
                Instr::I64TruncF64U(_) => todo!(),
                Instr::I32TruncSatF32S(_) => todo!(),
                Instr::I32TruncSatF32U(_) => todo!(),
                Instr::I32TruncSatF64S(_) => todo!(),
                Instr::I32TruncSatF64U(_) => todo!(),
                Instr::I64TruncSatF32S(_) => todo!(),
                Instr::I64TruncSatF32U(_) => todo!(),
                Instr::I64TruncSatF64S(_) => todo!(),
                Instr::I64TruncSatF64U(_) => todo!(),
                Instr::I32Extend8S(_) => todo!(),
                Instr::I32Extend16S(_) => todo!(),
                Instr::I64Extend8S(_) => todo!(),
                Instr::I64Extend16S(_) => todo!(),
                Instr::I64Extend32S(_) => todo!(),
                Instr::F32DemoteF64(_) => todo!(),
                Instr::F64PromoteF32(_) => todo!(),
                Instr::F32ConvertI32S(_) => todo!(),
                Instr::F32ConvertI32U(_) => todo!(),
                Instr::F32ConvertI64S(_) => todo!(),
                Instr::F32ConvertI64U(_) => todo!(),
                Instr::F64ConvertI32S(_) => todo!(),
                Instr::F64ConvertI32U(_) => todo!(),
                Instr::F64ConvertI64S(_) => todo!(),
                Instr::F64ConvertI64U(_) => todo!(),
            }
        }
    }

    /// Returns the [`Register`] value.
    fn get_register(&self, register: Register) -> UntypedValue {
        // Safety: TODO
        unsafe { self.sp.get(register) }
    }

    /// Returns the [`Register`] value.
    fn get_register_as<T>(&self, register: Register) -> T
    where
        T: From<UntypedValue>,
    {
        T::from(self.get_register(register))
    }

    /// Sets the [`Register`] value to `value`.
    fn set_register(&mut self, register: Register, value: UntypedValue) {
        // Safety: TODO
        let cell = unsafe { self.sp.get_mut(register) };
        *cell = value;
    }

    /// Shifts the instruction pointer to the next instruction.
    #[inline(always)]
    fn next_instr(&mut self) {
        self.next_instr_at(1)
    }

    /// Shifts the instruction pointer to the next instruction.
    ///
    /// Has a parameter `skip` to denote how many instruction words
    /// to skip to reach the next actual instruction.
    ///
    /// # Note
    ///
    /// This is used by `wasmi` instructions that have a fixed
    /// encoding size of two instruction words such as [`Instruction::Branch`].
    #[inline(always)]
    fn next_instr_at(&mut self, skip: usize) {
        self.ip.add(skip)
    }

    /// Shifts the instruction pointer to the next instruction and returns `Ok(())`.
    ///
    /// # Note
    ///
    /// This is a convenience function for fallible instructions.
    #[inline(always)]
    fn try_next_instr(&mut self) -> Result<(), TrapCode> {
        self.next_instr();
        Ok(())
    }

    /// Shifts the instruction pointer to the next instruction and returns `Ok(())`.
    ///
    /// Has a parameter `skip` to denote how many instruction words
    /// to skip to reach the next actual instruction.
    ///
    /// # Note
    ///
    /// This is a convenience function for fallible instructions.
    #[inline(always)]
    fn try_next_instr_at(&mut self, skip: usize) -> Result<(), TrapCode> {
        self.next_instr_at(skip);
        Ok(())
    }

    /// Branches and adjusts the value stack.
    ///
    /// # Note
    ///
    /// Offsets the instruction pointer using the given [`BranchOffset`].
    #[inline(always)]
    fn branch_to(&mut self, offset: BranchOffset) {
        self.ip.offset(offset.to_i32() as isize)
    }

    /// Initializes the [`Executor`] state for the [`CallFrame`].
    ///
    /// # Note
    ///
    /// The initialization of the [`Executor`] allows for efficient execution.
    fn init_call_frame(&mut self, frame: &CallFrame) {
        // Safety: We are using the frame's own base offset as input because it is
        //         guaranteed by the Wasm validation and translation phase to be
        //         valid for all register indices used by the associated function body.
        self.sp = unsafe { self.value_stack.stack_ptr_at(frame.base_offset()) };
        self.ip = frame.instr_ptr();
        self.cache.update_instance(frame.instance());
    }

    /// Returns the execution to the caller.
    ///
    /// Any return values are expected to already have been transferred
    /// from the returning callee to the caller.
    #[inline(always)]
    fn ret(&mut self) -> ReturnOutcome {
        match self.call_stack.pop() {
            Some(caller) => {
                self.init_call_frame(&caller);
                ReturnOutcome::Wasm
            }
            None => ReturnOutcome::Host,
        }
    }
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Used for all [`Instruction`] words that are not meant for execution.
    ///
    /// # Note
    ///
    /// This includes [`Instruction`] variants such as [`Instruction::TableIdx`]
    /// that primarily carry paramters for actually executable [`Instruction`].
    #[inline(always)]
    fn invalid_instruction_word(&mut self) -> Result<(), TrapCode> {
        self.execute_unreachable()
    }

    /// Executes a Wasm `unreachable` instruction.
    #[inline(always)]
    fn execute_unreachable(&mut self) -> Result<(), TrapCode> {
        Err(TrapCode::UnreachableCodeReached)
    }

    /// Executes an [`Instruction::ConsumeFuel`].
    #[inline(always)]
    fn execute_consume_fuel(&mut self, block_fuel: BlockFuel) -> Result<(), TrapCode> {
        // We do not have to check if fuel metering is enabled since
        // [`Instruction::ConsumeFuel`] are only generated if fuel metering
        // is enabled to begin with.
        self.ctx.fuel_mut().consume_fuel(block_fuel.to_u64())?;
        self.try_next_instr()
    }

    /// Execute an [`Instruction::Return`].
    #[inline(always)]
    fn execute_return(&mut self) -> ReturnOutcome {
        self.ret()
    }

    /// Execute a generic return [`Instruction`] returning a single value.
    #[inline(always)]
    fn execute_return_value<T>(
        &mut self,
        value: T,
        f: fn(&mut Self, T) -> UntypedValue,
    ) -> ReturnOutcome {
        match self.call_stack.peek() {
            Some(caller) => unsafe {
                // Case: we need to return the `value` back to the caller frame.
                // Safety: TODO
                let mut caller_sp = self.value_stack.stack_ptr_at(caller.base_offset());
                let result = caller_sp.get_mut(caller.results().head());
                *result = f(self, value);
            },
            None => {
                // Case: the root call frame is returning.
                todo!()
            }
        }
        self.ret()
    }

    /// Execute an [`Instruction::ReturnReg`] returning a single [`Register`] value.
    #[inline(always)]
    fn execute_return_reg(&mut self, value: Register) -> ReturnOutcome {
        self.execute_return_value(value, |this, value| this.get_register(value))
    }

    /// Execute an [`Instruction::ReturnImm32`] returning a single 32-bit value.
    #[inline(always)]
    fn execute_return_imm32(&mut self, value: AnyConst32) -> ReturnOutcome {
        self.execute_return_value(value, |_, value| value.to_u32().into())
    }

    /// Execute an [`Instruction::ReturnI64Imm32`] returning a single 32-bit encoded `i64` value.
    #[inline(always)]
    fn execute_return_i64imm32(&mut self, value: Const32<i64>) -> ReturnOutcome {
        self.execute_return_value(value, |_, value| i64::from(value).into())
    }

    /// Execute an [`Instruction::ReturnF64Imm32`] returning a single 32-bit encoded `f64` value.
    #[inline(always)]
    fn execute_return_f64imm32(&mut self, value: Const32<f64>) -> ReturnOutcome {
        self.execute_return_value(value, |_, value| f64::from(value).into())
    }

    /// Execute an [`Instruction::ReturnMany`] returning many values.
    #[inline(always)]
    fn execute_return_many(&mut self, values: RegisterSpanIter) -> ReturnOutcome {
        match self.call_stack.peek() {
            Some(caller) => unsafe {
                // Case: we need to return the `value` back to the caller frame.
                // Safety: TODO
                let mut caller_sp = self.value_stack.stack_ptr_at(caller.base_offset());
                let results = caller.results().iter(values.len());
                for (result, value) in results.zip(values) {
                    *caller_sp.get_mut(result) = self.get_register(value);
                }
            },
            None => {
                // Case: the root call frame is returning.
                todo!()
            }
        }
        self.ret()
    }

    /// Execute a generic conditional return [`Instruction`].
    #[inline(always)]
    fn execute_return_nez_impl<T>(
        &mut self,
        condition: Register,
        value: T,
        f: fn(&mut Self, T) -> ReturnOutcome,
    ) -> ReturnOutcome {
        let condition = self.get_register(condition);
        match bool::from(condition) {
            true => f(self, value),
            false => {
                self.next_instr();
                ReturnOutcome::Wasm
            }
        }
    }

    /// Execute an [`Instruction::Return`].
    #[inline(always)]
    fn execute_return_nez(&mut self, condition: Register) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, (), |this, _| this.execute_return())
    }

    /// Execute an [`Instruction::ReturnNezReg`] returning a single [`Register`] value.
    #[inline(always)]
    fn execute_return_nez_reg(&mut self, condition: Register, value: Register) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_reg)
    }

    /// Execute an [`Instruction::ReturnNezImm32`] returning a single 32-bit constant value.
    #[inline(always)]
    fn execute_return_nez_imm32(
        &mut self,
        condition: Register,
        value: AnyConst32,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_imm32)
    }

    /// Execute an [`Instruction::ReturnNezI64Imm32`] returning a single 32-bit encoded constant `i64` value.
    #[inline(always)]
    fn execute_return_nez_i64imm32(
        &mut self,
        condition: Register,
        value: Const32<i64>,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_i64imm32)
    }

    /// Execute an [`Instruction::ReturnNezF64Imm32`] returning a single 32-bit encoded constant `f64` value.
    #[inline(always)]
    fn execute_return_nez_f64imm32(
        &mut self,
        condition: Register,
        value: Const32<f64>,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_f64imm32)
    }

    /// Execute an [`Instruction::ReturnNezMany`] returning many values.
    #[inline(always)]
    fn execute_return_nez_many(
        &mut self,
        condition: Register,
        values: RegisterSpanIter,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, values, Self::execute_return_many)
    }

    #[inline(always)]
    fn execute_branch(&mut self, offset: BranchOffset) {
        self.branch_to(offset)
    }

    #[inline(always)]
    fn execute_branch_nez(&mut self, condition: Register, offset: BranchOffset) {
        let condition: bool = self.get_register_as(condition);
        match condition {
            true => {
                self.branch_to(offset);
            }
            false => {
                self.next_instr();
            }
        }
    }

    #[inline(always)]
    fn execute_branch_eqz(&mut self, condition: Register, offset: BranchOffset) {
        let condition: bool = self.get_register_as(condition);
        match condition {
            true => {
                self.next_instr();
            }
            false => {
                self.branch_to(offset);
            }
        }
    }

    #[inline(always)]
    fn execute_branch_table(&mut self, index: Register, len_targets: Const32<u32>) {
        // Safety: TODO
        let index: u32 = self.get_register_as(index);
        // The index of the default target which is the last target of the slice.
        let max_index = u32::from(len_targets) - 1;
        // A normalized index will always yield a target without panicking.
        let normalized_index = cmp::min(index, max_index);
        // Update `pc`:
        self.ip.add(normalized_index as usize + 1);
    }
}
