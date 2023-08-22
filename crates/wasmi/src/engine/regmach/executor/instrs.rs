#![allow(dead_code, unused_variables)]

use crate::{
    core::TrapCode,
    engine::{
        bytecode::{BlockFuel, BranchOffset, FuncIdx},
        bytecode2::{
            AnyConst32,
            BinInstr,
            BinInstrImm16,
            Const16,
            Const32,
            Instruction,
            Register,
            RegisterSpan,
            UnaryInstr,
        },
        cache::InstanceCache,
        code_map::{CodeMap2 as CodeMap, InstructionPtr2 as InstructionPtr},
        config::FuelCosts,
        regmach::stack::{CallFrame, CallStack, ValueStack, ValueStackPtr},
    },
    store::ResourceLimiterRef,
    FuelConsumptionMode,
    Func,
    FuncRef,
    Instance,
    StoreInner,
};
use core::cmp;
use wasmi_core::UntypedValue;

mod binary;
mod call;
mod comparison;
mod conversion;
mod global;
mod load;
mod memory;
mod return_;
mod select;
mod store;
mod table;
mod unary;

macro_rules! forward_call {
    ($expr:expr) => {{
        if let CallOutcome::Call {
            host_func,
            instance,
        } = $expr?
        {
            return Ok(WasmOutcome::Call {
                host_func,
                instance,
            });
        }
    }};
}

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

impl CallOutcome {
    /// Creates a new [`CallOutcome::Call`].
    pub fn call(host_func: Func, instance: Instance) -> Self {
        Self::Call {
            host_func,
            instance,
        }
    }
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
            .peek()
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
                Instr::Copy { result, value } => self.execute_copy(result, value),
                Instr::CopyImm32 { result, value } => self.execute_copy_imm32(result, value),
                Instr::CopyI64Imm32 { result, value } => self.execute_copy_i64imm32(result, value),
                Instr::CopyF64Imm32 { result, value } => self.execute_copy_f64imm32(result, value),
                Instr::CopySpan {
                    results,
                    values,
                    len,
                } => self.execute_copy_span(results, values, len),
                Instr::CallParams(_) => self.invalid_instruction_word()?,
                Instr::CallIndirectParams(_) => self.invalid_instruction_word()?,
                Instr::CallIndirectParamsImm16(_) => self.invalid_instruction_word()?,
                Instr::ReturnCallInternal0 { func } => self.execute_return_call_internal_0(func)?,
                Instr::ReturnCallInternal { func } => self.execute_return_call_internal(func)?,
                Instr::ReturnCallImported0 { func } => {
                    forward_call!(self.execute_return_call_imported_0(func))
                }
                Instr::ReturnCallImported { func } => {
                    forward_call!(self.execute_return_call_imported(func))
                }
                Instr::ReturnCallIndirect0 { func_type } => {
                    forward_call!(self.execute_return_call_indirect_0(func_type))
                }
                Instr::ReturnCallIndirect { func_type } => {
                    forward_call!(self.execute_return_call_indirect(func_type))
                }
                Instr::CallInternal0 { results, func } => {
                    self.execute_call_internal_0(results, func)?
                }
                Instr::CallInternal { results, func } => {
                    self.execute_call_internal(results, func)?
                }
                Instr::CallImported0 { results, func } => {
                    forward_call!(self.execute_call_imported_0(results, func))
                }
                Instr::CallImported { results, func } => {
                    forward_call!(self.execute_call_imported(results, func))
                }
                Instr::CallIndirect0 { results, func_type } => {
                    forward_call!(self.execute_call_indirect_0(results, func_type))
                }
                Instr::CallIndirect { results, func_type } => {
                    forward_call!(self.execute_call_indirect(results, func_type))
                }
                Instr::Select {
                    result,
                    condition,
                    lhs,
                } => self.execute_select(result, condition, lhs),
                Instr::SelectRev {
                    result,
                    condition,
                    rhs,
                } => self.execute_select_rev(result, condition, rhs),
                Instr::SelectImm32 {
                    result_or_condition,
                    lhs_or_rhs,
                } => self.execute_select_imm32(result_or_condition, lhs_or_rhs),
                Instr::SelectI64Imm32 {
                    result_or_condition,
                    lhs_or_rhs,
                } => self.execute_select_i64imm32(result_or_condition, lhs_or_rhs),
                Instr::SelectF64Imm32 {
                    result_or_condition,
                    lhs_or_rhs,
                } => self.execute_select_f64imm32(result_or_condition, lhs_or_rhs),
                Instr::RefFunc { result, func } => self.execute_ref_func(result, func),
                Instr::TableGet { result, index } => self.execute_table_get(result, index)?,
                Instr::TableGetImm { result, index } => {
                    self.execute_table_get_imm(result, index)?
                }
                Instr::TableSize { result, table } => self.execute_table_size(result, table),
                Instr::TableSet { index, value } => self.execute_table_set(index, value)?,
                Instr::TableSetAt { index, value } => self.execute_table_set_at(index, value)?,
                Instr::TableCopy { dst, src, len } => self.execute_table_copy(dst, src, len)?,
                Instr::TableCopyTo { dst, src, len } => {
                    self.execute_table_copy_to(dst, src, len)?
                }
                Instr::TableCopyFrom { dst, src, len } => {
                    self.execute_table_copy_from(dst, src, len)?
                }
                Instr::TableCopyFromTo { dst, src, len } => {
                    self.execute_table_copy_from_to(dst, src, len)?
                }
                Instr::TableCopyExact { dst, src, len } => {
                    self.execute_table_copy_exact(dst, src, len)?
                }
                Instr::TableCopyToExact { dst, src, len } => {
                    self.execute_table_copy_to_exact(dst, src, len)?
                }
                Instr::TableCopyFromExact { dst, src, len } => {
                    self.execute_table_copy_from_exact(dst, src, len)?
                }
                Instr::TableCopyFromToExact { dst, src, len } => {
                    self.execute_table_copy_from_to_exact(dst, src, len)?
                }
                Instr::TableInit { dst, src, len } => self.execute_table_init(dst, src, len)?,
                Instr::TableInitTo { dst, src, len } => {
                    self.execute_table_init_to(dst, src, len)?
                }
                Instr::TableInitFrom { dst, src, len } => {
                    self.execute_table_init_from(dst, src, len)?
                }
                Instr::TableInitFromTo { dst, src, len } => {
                    self.execute_table_init_from_to(dst, src, len)?
                }
                Instr::TableInitExact { dst, src, len } => {
                    self.execute_table_init_exact(dst, src, len)?
                }
                Instr::TableInitToExact { dst, src, len } => {
                    self.execute_table_init_to_exact(dst, src, len)?
                }
                Instr::TableInitFromExact { dst, src, len } => {
                    self.execute_table_init_from_exact(dst, src, len)?
                }
                Instr::TableInitFromToExact { dst, src, len } => {
                    self.execute_table_init_from_to_exact(dst, src, len)?
                }
                Instr::TableFill { dst, len, value } => self.execute_table_fill(dst, len, value)?,
                Instr::TableFillAt { dst, len, value } => {
                    self.execute_table_fill_at(dst, len, value)?
                }
                Instr::TableFillExact { dst, len, value } => {
                    self.execute_table_fill_exact(dst, len, value)?
                }
                Instr::TableFillAtExact { dst, len, value } => {
                    self.execute_table_fill_at_exact(dst, len, value)?
                }
                Instr::TableGrow {
                    result,
                    delta,
                    value,
                } => self.execute_table_grow(result, delta, value, &mut *resource_limiter)?,
                Instr::TableGrowImm {
                    result,
                    delta,
                    value,
                } => self.execute_table_grow_imm(result, delta, value, &mut *resource_limiter)?,
                Instr::ElemDrop(element_index) => self.execute_element_drop(element_index),
                Instr::DataDrop(data_index) => self.execute_data_drop(data_index),
                Instr::MemorySize { result } => self.execute_memory_size(result),
                Instr::MemoryGrow { result, delta } => {
                    self.execute_memory_grow(result, delta, &mut *resource_limiter)?
                }
                Instr::MemoryGrowBy { result, delta } => {
                    self.execute_memory_grow_by(result, delta, &mut *resource_limiter)?
                }
                Instr::MemoryCopy { dst, src, len } => self.execute_memory_copy(dst, src, len)?,
                Instr::MemoryCopyTo { dst, src, len } => {
                    self.execute_memory_copy_to(dst, src, len)?
                }
                Instr::MemoryCopyFrom { dst, src, len } => {
                    self.execute_memory_copy_from(dst, src, len)?
                }
                Instr::MemoryCopyFromTo { dst, src, len } => {
                    self.execute_memory_copy_from_to(dst, src, len)?
                }
                Instr::MemoryCopyExact { dst, src, len } => {
                    self.execute_memory_copy_exact(dst, src, len)?
                }
                Instr::MemoryCopyToExact { dst, src, len } => {
                    self.execute_memory_copy_to_exact(dst, src, len)?
                }
                Instr::MemoryCopyFromExact { dst, src, len } => {
                    self.execute_memory_copy_from_exact(dst, src, len)?
                }
                Instr::MemoryCopyFromToExact { dst, src, len } => {
                    self.execute_memory_copy_from_to_exact(dst, src, len)?
                }
                Instr::MemoryFill { dst, value, len } => {
                    self.execute_memory_fill(dst, value, len)?
                }
                Instr::MemoryFillAt { dst, value, len } => {
                    self.execute_memory_fill_at(dst, value, len)?
                }
                Instr::MemoryFillImm { dst, value, len } => {
                    self.execute_memory_fill_imm(dst, value, len)?
                }
                Instr::MemoryFillExact { dst, value, len } => {
                    self.execute_memory_fill_exact(dst, value, len)?
                }
                Instr::MemoryFillAtImm { dst, value, len } => {
                    self.execute_memory_fill_at_imm(dst, value, len)?
                }
                Instr::MemoryFillAtExact { dst, value, len } => {
                    self.execute_memory_fill_at_exact(dst, value, len)?
                }
                Instr::MemoryFillImmExact { dst, value, len } => {
                    self.execute_memory_fill_imm_exact(dst, value, len)?
                }
                Instr::MemoryFillAtImmExact { dst, value, len } => {
                    self.execute_memory_fill_at_imm_exact(dst, value, len)?
                }
                Instr::MemoryInit { dst, src, len } => self.execute_memory_init(dst, src, len)?,
                Instr::MemoryInitTo { dst, src, len } => {
                    self.execute_memory_init_to(dst, src, len)?
                }
                Instr::MemoryInitFrom { dst, src, len } => {
                    self.execute_memory_init_from(dst, src, len)?
                }
                Instr::MemoryInitFromTo { dst, src, len } => {
                    self.execute_memory_init_from_to(dst, src, len)?
                }
                Instr::MemoryInitExact { dst, src, len } => {
                    self.execute_memory_init_exact(dst, src, len)?
                }
                Instr::MemoryInitToExact { dst, src, len } => {
                    self.execute_memory_init_to_exact(dst, src, len)?
                }
                Instr::MemoryInitFromExact { dst, src, len } => {
                    self.execute_memory_init_from_exact(dst, src, len)?
                }
                Instr::MemoryInitFromToExact { dst, src, len } => {
                    self.execute_memory_init_from_to_exact(dst, src, len)?
                }
                Instr::GlobalGet { result, global } => self.execute_global_get(result, global),
                Instr::GlobalSet { global, input } => self.execute_global_set(global, input),
                Instr::GlobalSetI32Imm16 { global, input } => {
                    self.execute_global_set_i32imm16(global, input)
                }
                Instr::GlobalSetI64Imm16 { global, input } => {
                    self.execute_global_set_i64imm16(global, input)
                }
                Instr::I32Load(instr) => self.execute_i32_load(instr)?,
                Instr::I32LoadAt(instr) => self.execute_i32_load_at(instr)?,
                Instr::I32LoadOffset16(instr) => self.execute_i32_load_offset16(instr)?,
                Instr::I64Load(instr) => self.execute_i64_load(instr)?,
                Instr::I64LoadAt(instr) => self.execute_i64_load_at(instr)?,
                Instr::I64LoadOffset16(instr) => self.execute_i64_load_offset16(instr)?,
                Instr::F32Load(instr) => self.execute_f32_load(instr)?,
                Instr::F32LoadAt(instr) => self.execute_f32_load_at(instr)?,
                Instr::F32LoadOffset16(instr) => self.execute_f32_load_offset16(instr)?,
                Instr::F64Load(instr) => self.execute_f64_load(instr)?,
                Instr::F64LoadAt(instr) => self.execute_f64_load_at(instr)?,
                Instr::F64LoadOffset16(instr) => self.execute_f64_load_offset16(instr)?,
                Instr::I32Load8s(instr) => self.execute_i32_load8_s(instr)?,
                Instr::I32Load8sAt(instr) => self.execute_i32_load8_s_at(instr)?,
                Instr::I32Load8sOffset16(instr) => self.execute_i32_load8_s_offset16(instr)?,
                Instr::I32Load8u(instr) => self.execute_i32_load8_u(instr)?,
                Instr::I32Load8uAt(instr) => self.execute_i32_load8_u_at(instr)?,
                Instr::I32Load8uOffset16(instr) => self.execute_i32_load8_u_offset16(instr)?,
                Instr::I32Load16s(instr) => self.execute_i32_load16_s(instr)?,
                Instr::I32Load16sAt(instr) => self.execute_i32_load16_s_at(instr)?,
                Instr::I32Load16sOffset16(instr) => self.execute_i32_load16_s_offset16(instr)?,
                Instr::I32Load16u(instr) => self.execute_i32_load16_u(instr)?,
                Instr::I32Load16uAt(instr) => self.execute_i32_load16_u_at(instr)?,
                Instr::I32Load16uOffset16(instr) => self.execute_i32_load16_u_offset16(instr)?,
                Instr::I64Load8s(instr) => self.execute_i64_load8_s(instr)?,
                Instr::I64Load8sAt(instr) => self.execute_i64_load8_s_at(instr)?,
                Instr::I64Load8sOffset16(instr) => self.execute_i64_load8_s_offset16(instr)?,
                Instr::I64Load8u(instr) => self.execute_i64_load8_u(instr)?,
                Instr::I64Load8uAt(instr) => self.execute_i64_load8_u_at(instr)?,
                Instr::I64Load8uOffset16(instr) => self.execute_i64_load8_u_offset16(instr)?,
                Instr::I64Load16s(instr) => self.execute_i64_load16_s(instr)?,
                Instr::I64Load16sAt(instr) => self.execute_i64_load16_s_at(instr)?,
                Instr::I64Load16sOffset16(instr) => self.execute_i64_load16_s_offset16(instr)?,
                Instr::I64Load16u(instr) => self.execute_i64_load16_u(instr)?,
                Instr::I64Load16uAt(instr) => self.execute_i64_load16_u_at(instr)?,
                Instr::I64Load16uOffset16(instr) => self.execute_i64_load16_u_offset16(instr)?,
                Instr::I64Load32s(instr) => self.execute_i64_load32_s(instr)?,
                Instr::I64Load32sAt(instr) => self.execute_i64_load32_s_at(instr)?,
                Instr::I64Load32sOffset16(instr) => self.execute_i64_load32_s_offset16(instr)?,
                Instr::I64Load32u(instr) => self.execute_i64_load32_u(instr)?,
                Instr::I64Load32uAt(instr) => self.execute_i64_load32_u_at(instr)?,
                Instr::I64Load32uOffset16(instr) => self.execute_i64_load32_u_offset16(instr)?,
                Instr::I32Store(instr) => self.execute_i32_store(instr)?,
                Instr::I32StoreOffset16(instr) => self.execute_i32_store_offset16(instr)?,
                Instr::I32StoreOffset16Imm16(instr) => {
                    self.execute_i32_store_offset16_imm16(instr)?
                }
                Instr::I32StoreAt(instr) => self.execute_i32_store_at(instr)?,
                Instr::I32StoreAtImm16(instr) => self.execute_i32_store_at_imm16(instr)?,
                Instr::I32Store8(instr) => self.execute_i32_store8(instr)?,
                Instr::I32Store8Offset16(instr) => self.execute_i32_store8_offset16(instr)?,
                Instr::I32Store8Offset16Imm(instr) => {
                    self.execute_i32_store8_offset16_imm(instr)?
                }
                Instr::I32Store8At(instr) => self.execute_i32_store8_at(instr)?,
                Instr::I32Store8AtImm(instr) => self.execute_i32_store8_at_imm(instr)?,
                Instr::I32Store16(instr) => self.execute_i32_store16(instr)?,
                Instr::I32Store16Offset16(instr) => self.execute_i32_store16_offset16(instr)?,
                Instr::I32Store16Offset16Imm(instr) => {
                    self.execute_i32_store16_offset16_imm(instr)?
                }
                Instr::I32Store16At(instr) => self.execute_i32_store16_at(instr)?,
                Instr::I32Store16AtImm(instr) => self.execute_i32_store16_at_imm(instr)?,
                Instr::I64Store(instr) => self.execute_i64_store(instr)?,
                Instr::I64StoreOffset16(instr) => self.execute_i64_store_offset16(instr)?,
                Instr::I64StoreOffset16Imm16(instr) => {
                    self.execute_i64_store_offset16_imm16(instr)?
                }
                Instr::I64StoreAt(instr) => self.execute_i64_store_at(instr)?,
                Instr::I64StoreAtImm16(instr) => self.execute_i64_store_at_imm16(instr)?,
                Instr::I64Store8(instr) => self.execute_i64_store8(instr)?,
                Instr::I64Store8Offset16(instr) => self.execute_i64_store8_offset16(instr)?,
                Instr::I64Store8Offset16Imm(instr) => {
                    self.execute_i64_store8_offset16_imm(instr)?
                }
                Instr::I64Store8At(instr) => self.execute_i64_store8_at(instr)?,
                Instr::I64Store8AtImm(instr) => self.execute_i64_store8_at_imm(instr)?,
                Instr::I64Store16(instr) => self.execute_i64_store16(instr)?,
                Instr::I64Store16Offset16(instr) => self.execute_i64_store16_offset16(instr)?,
                Instr::I64Store16Offset16Imm(instr) => {
                    self.execute_i64_store16_offset16_imm(instr)?
                }
                Instr::I64Store16At(instr) => self.execute_i64_store16_at(instr)?,
                Instr::I64Store16AtImm(instr) => self.execute_i64_store16_at_imm(instr)?,
                Instr::I64Store32(instr) => self.execute_i64_store32(instr)?,
                Instr::I64Store32Offset16(instr) => self.execute_i64_store32_offset16(instr)?,
                Instr::I64Store32Offset16Imm16(instr) => {
                    self.execute_i64_store32_offset16_imm16(instr)?
                }
                Instr::I64Store32At(instr) => self.execute_i64_store32_at(instr)?,
                Instr::I64Store32AtImm16(instr) => self.execute_i64_store32_at_imm16(instr)?,
                Instr::F32Store(instr) => self.execute_f32_store(instr)?,
                Instr::F32StoreOffset16(instr) => self.execute_f32_store_offset16(instr)?,
                Instr::F32StoreAt(instr) => self.execute_f32_store_at(instr)?,
                Instr::F64Store(instr) => self.execute_f64_store(instr)?,
                Instr::F64StoreOffset16(instr) => self.execute_f64_store_offset16(instr)?,
                Instr::F64StoreAt(instr) => self.execute_f64_store_at(instr)?,
                Instr::I32Eq(instr) => self.execute_i32_eq(instr),
                Instr::I32EqImm16(instr) => self.execute_i32_eq_imm16(instr),
                Instr::I32Ne(instr) => self.execute_i32_ne(instr),
                Instr::I32NeImm16(instr) => self.execute_i32_ne_imm16(instr),
                Instr::I32LtS(instr) => self.execute_i32_lt_s(instr),
                Instr::I32LtSImm16(instr) => self.execute_i32_lt_s_imm16(instr),
                Instr::I32LtU(instr) => self.execute_i32_lt_u(instr),
                Instr::I32LtUImm16(instr) => self.execute_i32_lt_u_imm16(instr),
                Instr::I32GtS(instr) => self.execute_i32_le_s(instr),
                Instr::I32GtSImm16(instr) => self.execute_i32_le_s_imm16(instr),
                Instr::I32GtU(instr) => self.execute_i32_le_u(instr),
                Instr::I32GtUImm16(instr) => self.execute_i32_le_u_imm16(instr),
                Instr::I32LeS(instr) => self.execute_i32_gt_s(instr),
                Instr::I32LeSImm16(instr) => self.execute_i32_gt_s_imm16(instr),
                Instr::I32LeU(instr) => self.execute_i32_gt_u(instr),
                Instr::I32LeUImm16(instr) => self.execute_i32_gt_u_imm16(instr),
                Instr::I32GeS(instr) => self.execute_i32_ge_s(instr),
                Instr::I32GeSImm16(instr) => self.execute_i32_ge_s_imm16(instr),
                Instr::I32GeU(instr) => self.execute_i32_ge_u(instr),
                Instr::I32GeUImm16(instr) => self.execute_i32_ge_u_imm16(instr),
                Instr::I64Eq(instr) => self.execute_i64_eq(instr),
                Instr::I64EqImm16(instr) => self.execute_i64_eq_imm16(instr),
                Instr::I64Ne(instr) => self.execute_i64_ne(instr),
                Instr::I64NeImm16(instr) => self.execute_i64_ne_imm16(instr),
                Instr::I64LtS(instr) => self.execute_i64_lt_s(instr),
                Instr::I64LtSImm16(instr) => self.execute_i64_lt_s_imm16(instr),
                Instr::I64LtU(instr) => self.execute_i64_lt_u(instr),
                Instr::I64LtUImm16(instr) => self.execute_i64_lt_u_imm16(instr),
                Instr::I64GtS(instr) => self.execute_i64_le_s(instr),
                Instr::I64GtSImm16(instr) => self.execute_i64_le_s_imm16(instr),
                Instr::I64GtU(instr) => self.execute_i64_le_u(instr),
                Instr::I64GtUImm16(instr) => self.execute_i64_le_u_imm16(instr),
                Instr::I64LeS(instr) => self.execute_i64_gt_s(instr),
                Instr::I64LeSImm16(instr) => self.execute_i64_gt_s_imm16(instr),
                Instr::I64LeU(instr) => self.execute_i64_gt_u(instr),
                Instr::I64LeUImm16(instr) => self.execute_i64_gt_u_imm16(instr),
                Instr::I64GeS(instr) => self.execute_i64_ge_s(instr),
                Instr::I64GeSImm16(instr) => self.execute_i64_ge_s_imm16(instr),
                Instr::I64GeU(instr) => self.execute_i64_ge_u(instr),
                Instr::I64GeUImm16(instr) => self.execute_i64_ge_u_imm16(instr),
                Instr::F32Eq(instr) => self.execute_f32_eq(instr),
                Instr::F32Ne(instr) => self.execute_f32_ne(instr),
                Instr::F32Lt(instr) => self.execute_f32_lt(instr),
                Instr::F32Le(instr) => self.execute_f32_le(instr),
                Instr::F32Gt(instr) => self.execute_f32_gt(instr),
                Instr::F32Ge(instr) => self.execute_f32_ge(instr),
                Instr::F64Eq(instr) => self.execute_f64_eq(instr),
                Instr::F64Ne(instr) => self.execute_f64_ne(instr),
                Instr::F64Lt(instr) => self.execute_f64_lt(instr),
                Instr::F64Le(instr) => self.execute_f64_le(instr),
                Instr::F64Gt(instr) => self.execute_f64_gt(instr),
                Instr::F64Ge(instr) => self.execute_f64_ge(instr),
                Instr::I32Clz(instr) => self.execute_i32_clz(instr),
                Instr::I64Clz(instr) => self.execute_i64_clz(instr),
                Instr::I32Ctz(instr) => self.execute_i32_ctz(instr),
                Instr::I64Ctz(instr) => self.execute_i64_ctz(instr),
                Instr::I32Popcnt(instr) => self.execute_i32_popcnt(instr),
                Instr::I64Popcnt(instr) => self.execute_i64_popcnt(instr),
                Instr::I32Add(instr) => self.execute_i32_add(instr),
                Instr::I32AddImm16(instr) => self.execute_i32_add_imm16(instr),
                Instr::I32Sub(instr) => self.execute_i32_sub(instr),
                Instr::I32SubImm16(instr) => self.execute_i32_sub_imm16(instr),
                Instr::I32SubImm16Rev(instr) => self.execute_i32_sub_imm16_rev(instr),
                Instr::I32Mul(instr) => self.execute_i32_mul(instr),
                Instr::I32MulImm16(instr) => self.execute_i32_mul_imm16(instr),
                Instr::I32DivS(instr) => self.execute_i32_div_s(instr)?,
                Instr::I32DivSImm16(instr) => self.execute_i32_div_s_imm16(instr)?,
                Instr::I32DivSImm16Rev(instr) => self.execute_i32_div_s_imm16_rev(instr)?,
                Instr::I32DivU(instr) => self.execute_i32_div_u(instr)?,
                Instr::I32DivUImm16(instr) => self.execute_i32_div_u_imm16(instr)?,
                Instr::I32DivUImm16Rev(instr) => self.execute_i32_div_u_imm16_rev(instr)?,
                Instr::I32RemS(instr) => self.execute_i32_rem_s(instr)?,
                Instr::I32RemSImm16(instr) => self.execute_i32_rem_s_imm16(instr)?,
                Instr::I32RemSImm16Rev(instr) => self.execute_i32_rem_s_imm16_rev(instr)?,
                Instr::I32RemU(instr) => self.execute_i32_rem_u(instr)?,
                Instr::I32RemUImm16(instr) => self.execute_i32_rem_u_imm16(instr)?,
                Instr::I32RemUImm16Rev(instr) => self.execute_i32_rem_u_imm16_rev(instr)?,
                Instr::I32And(instr) => self.execute_i32_and(instr),
                Instr::I32AndImm16(instr) => self.execute_i32_and_imm16(instr),
                Instr::I32Or(instr) => self.execute_i32_or(instr),
                Instr::I32OrImm16(instr) => self.execute_i32_or_imm16(instr),
                Instr::I32Xor(instr) => self.execute_i32_xor(instr),
                Instr::I32XorImm16(instr) => self.execute_i32_xor_imm16(instr),
                Instr::I32Shl(instr) => self.execute_i32_shl(instr),
                Instr::I32ShlImm(instr) => self.execute_i32_shl_imm(instr),
                Instr::I32ShlImm16Rev(instr) => self.execute_i32_shl_imm16_rev(instr),
                Instr::I32ShrU(instr) => self.execute_i32_shr_u(instr),
                Instr::I32ShrUImm(instr) => self.execute_i32_shr_u_imm(instr),
                Instr::I32ShrUImm16Rev(instr) => self.execute_i32_shr_u_imm16_rev(instr),
                Instr::I32ShrS(instr) => self.execute_i32_shr_s(instr),
                Instr::I32ShrSImm(instr) => self.execute_i32_shr_s_imm(instr),
                Instr::I32ShrSImm16Rev(instr) => self.execute_i32_shr_s_imm16_rev(instr),
                Instr::I32Rotl(instr) => self.execute_i32_rotl(instr),
                Instr::I32RotlImm(instr) => self.execute_i32_rotl_imm(instr),
                Instr::I32RotlImm16Rev(instr) => self.execute_i32_rotl_imm16_rev(instr),
                Instr::I32Rotr(instr) => self.execute_i32_rotr(instr),
                Instr::I32RotrImm(instr) => self.execute_i32_rotr_imm(instr),
                Instr::I32RotrImm16Rev(instr) => self.execute_i32_rotr_imm16_rev(instr),
                Instr::I64Add(instr) => self.execute_i64_add(instr),
                Instr::I64AddImm16(instr) => self.execute_i64_add_imm16(instr),
                Instr::I64Sub(instr) => self.execute_i64_sub(instr),
                Instr::I64SubImm16(instr) => self.execute_i64_sub_imm16(instr),
                Instr::I64SubImm16Rev(instr) => self.execute_i64_sub_imm16_rev(instr),
                Instr::I64Mul(instr) => self.execute_i64_mul(instr),
                Instr::I64MulImm16(instr) => self.execute_i64_mul_imm16(instr),
                Instr::I64DivS(instr) => self.execute_i64_div_s(instr)?,
                Instr::I64DivSImm16(instr) => self.execute_i64_div_s_imm16(instr)?,
                Instr::I64DivSImm16Rev(instr) => self.execute_i64_div_s_imm16_rev(instr)?,
                Instr::I64DivU(instr) => self.execute_i64_div_u(instr)?,
                Instr::I64DivUImm16(instr) => self.execute_i64_div_u_imm16(instr)?,
                Instr::I64DivUImm16Rev(instr) => self.execute_i64_div_u_imm16_rev(instr)?,
                Instr::I64RemS(instr) => self.execute_i64_rem_s(instr)?,
                Instr::I64RemSImm16(instr) => self.execute_i64_rem_s_imm16(instr)?,
                Instr::I64RemSImm16Rev(instr) => self.execute_i64_rem_s_imm16_rev(instr)?,
                Instr::I64RemU(instr) => self.execute_i64_rem_u(instr)?,
                Instr::I64RemUImm16(instr) => self.execute_i64_rem_u_imm16(instr)?,
                Instr::I64RemUImm16Rev(instr) => self.execute_i64_rem_u_imm16_rev(instr)?,
                Instr::I64And(instr) => self.execute_i64_and(instr),
                Instr::I64AndImm16(instr) => self.execute_i64_and_imm16(instr),
                Instr::I64Or(instr) => self.execute_i64_or(instr),
                Instr::I64OrImm16(instr) => self.execute_i64_or_imm16(instr),
                Instr::I64Xor(instr) => self.execute_i64_xor(instr),
                Instr::I64XorImm16(instr) => self.execute_i64_xor_imm16(instr),
                Instr::I64Shl(instr) => self.execute_i64_shl(instr),
                Instr::I64ShlImm(instr) => self.execute_i64_shl_imm(instr),
                Instr::I64ShlImm16Rev(instr) => self.execute_i64_shl_imm16_rev(instr),
                Instr::I64ShrU(instr) => self.execute_i64_shr_u(instr),
                Instr::I64ShrUImm(instr) => self.execute_i64_shr_u_imm(instr),
                Instr::I64ShrUImm16Rev(instr) => self.execute_i64_shr_u_imm16_rev(instr),
                Instr::I64ShrS(instr) => self.execute_i64_shr_s(instr),
                Instr::I64ShrSImm(instr) => self.execute_i64_shr_s_imm(instr),
                Instr::I64ShrSImm16Rev(instr) => self.execute_i64_shr_s_imm16_rev(instr),
                Instr::I64Rotl(instr) => self.execute_i64_rotl(instr),
                Instr::I64RotlImm(instr) => self.execute_i64_rotl_imm(instr),
                Instr::I64RotlImm16Rev(instr) => self.execute_i64_rotl_imm16_rev(instr),
                Instr::I64Rotr(instr) => self.execute_i64_rotr(instr),
                Instr::I64RotrImm(instr) => self.execute_i64_rotr_imm(instr),
                Instr::I64RotrImm16Rev(instr) => self.execute_i64_rotr_imm16_rev(instr),
                Instr::F32Abs(instr) => self.execute_f32_abs(instr),
                Instr::F32Neg(instr) => self.execute_f32_neg(instr),
                Instr::F32Ceil(instr) => self.execute_f32_ceil(instr),
                Instr::F32Floor(instr) => self.execute_f32_floor(instr),
                Instr::F32Trunc(instr) => self.execute_f32_trunc(instr),
                Instr::F32Nearest(instr) => self.execute_f32_nearest(instr),
                Instr::F32Sqrt(instr) => self.execute_f32_sqrt(instr),
                Instr::F64Abs(instr) => self.execute_f64_abs(instr),
                Instr::F64Neg(instr) => self.execute_f64_neg(instr),
                Instr::F64Ceil(instr) => self.execute_f64_ceil(instr),
                Instr::F64Floor(instr) => self.execute_f64_floor(instr),
                Instr::F64Trunc(instr) => self.execute_f64_trunc(instr),
                Instr::F64Nearest(instr) => self.execute_f64_nearest(instr),
                Instr::F64Sqrt(instr) => self.execute_f64_sqrt(instr),
                Instr::F32Add(instr) => self.execute_f32_add(instr),
                Instr::F32Sub(instr) => self.execute_f32_sub(instr),
                Instr::F32Mul(instr) => self.execute_f32_mul(instr),
                Instr::F32Div(instr) => self.execute_f32_div(instr),
                Instr::F32Min(instr) => self.execute_f32_min(instr),
                Instr::F32Max(instr) => self.execute_f32_max(instr),
                Instr::F32Copysign(instr) => self.execute_f32_copysign(instr),
                Instr::F32CopysignImm(instr) => self.execute_f32_copysign_imm(instr),
                Instr::F64Add(instr) => self.execute_f64_add(instr),
                Instr::F64Sub(instr) => self.execute_f64_sub(instr),
                Instr::F64Mul(instr) => self.execute_f64_mul(instr),
                Instr::F64Div(instr) => self.execute_f64_div(instr),
                Instr::F64Min(instr) => self.execute_f64_min(instr),
                Instr::F64Max(instr) => self.execute_f64_max(instr),
                Instr::F64Copysign(instr) => self.execute_f64_copysign(instr),
                Instr::F64CopysignImm(instr) => self.execute_f64_copysign_imm(instr),
                Instr::I32WrapI64(instr) => self.execute_i32_wrap_i64(instr),
                Instr::I64ExtendI32S(instr) => self.execute_i64_extend_i32_s(instr),
                Instr::I64ExtendI32U(instr) => self.execute_i64_extend_i32_u(instr),
                Instr::I32TruncF32S(instr) => self.execute_i32_trunc_f32_s(instr)?,
                Instr::I32TruncF32U(instr) => self.execute_i32_trunc_f32_u(instr)?,
                Instr::I32TruncF64S(instr) => self.execute_i32_trunc_f64_s(instr)?,
                Instr::I32TruncF64U(instr) => self.execute_i32_trunc_f64_u(instr)?,
                Instr::I64TruncF32S(instr) => self.execute_i64_trunc_f32_s(instr)?,
                Instr::I64TruncF32U(instr) => self.execute_i64_trunc_f32_u(instr)?,
                Instr::I64TruncF64S(instr) => self.execute_i64_trunc_f64_s(instr)?,
                Instr::I64TruncF64U(instr) => self.execute_i64_trunc_f64_u(instr)?,
                Instr::I32TruncSatF32S(instr) => self.execute_i32_trunc_sat_f32_s(instr),
                Instr::I32TruncSatF32U(instr) => self.execute_i32_trunc_sat_f32_u(instr),
                Instr::I32TruncSatF64S(instr) => self.execute_i32_trunc_sat_f64_s(instr),
                Instr::I32TruncSatF64U(instr) => self.execute_i32_trunc_sat_f64_u(instr),
                Instr::I64TruncSatF32S(instr) => self.execute_i64_trunc_sat_f32_s(instr),
                Instr::I64TruncSatF32U(instr) => self.execute_i64_trunc_sat_f32_u(instr),
                Instr::I64TruncSatF64S(instr) => self.execute_i64_trunc_sat_f64_s(instr),
                Instr::I64TruncSatF64U(instr) => self.execute_i64_trunc_sat_f64_u(instr),
                Instr::I32Extend8S(instr) => self.execute_i32_extend8_s(instr),
                Instr::I32Extend16S(instr) => self.execute_i32_extend16_s(instr),
                Instr::I64Extend8S(instr) => self.execute_i64_extend8_s(instr),
                Instr::I64Extend16S(instr) => self.execute_i64_extend16_s(instr),
                Instr::I64Extend32S(instr) => self.execute_i64_extend32_s(instr),
                Instr::F32DemoteF64(instr) => self.execute_f32_demote_f64(instr),
                Instr::F64PromoteF32(instr) => self.execute_f64_promote_f32(instr),
                Instr::F32ConvertI32S(instr) => self.execute_f32_convert_i32_s(instr),
                Instr::F32ConvertI32U(instr) => self.execute_f32_convert_i32_u(instr),
                Instr::F32ConvertI64S(instr) => self.execute_f32_convert_i64_s(instr),
                Instr::F32ConvertI64U(instr) => self.execute_f32_convert_i64_u(instr),
                Instr::F64ConvertI32S(instr) => self.execute_f64_convert_i32_s(instr),
                Instr::F64ConvertI32U(instr) => self.execute_f64_convert_i32_u(instr),
                Instr::F64ConvertI64S(instr) => self.execute_f64_convert_i64_s(instr),
                Instr::F64ConvertI64U(instr) => self.execute_f64_convert_i64_u(instr),
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
    fn set_register(&mut self, register: Register, value: impl Into<UntypedValue>) {
        // Safety: TODO
        let cell = unsafe { self.sp.get_mut(register) };
        *cell = value.into();
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

    /// Returns the [`ValueStackPtr`] of the [`CallFrame`].
    fn frame_stack_ptr(&mut self, frame: &CallFrame) -> ValueStackPtr {
        // Safety: We are using the frame's own base offset as input because it is
        //         guaranteed by the Wasm validation and translation phase to be
        //         valid for all register indices used by the associated function body.
        unsafe { self.value_stack.stack_ptr_at(frame.base_offset()) }
    }

    /// Initializes the [`Executor`] state for the [`CallFrame`].
    ///
    /// # Note
    ///
    /// The initialization of the [`Executor`] allows for efficient execution.
    fn init_call_frame(&mut self, frame: &CallFrame) {
        self.sp = self.frame_stack_ptr(frame);
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
                self.value_stack.truncate(caller.frame_offset());
                self.init_call_frame(&caller);
                ReturnOutcome::Wasm
            }
            None => ReturnOutcome::Host,
        }
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
    /// # Errors
    ///
    /// - If the [`StoreInner`] ran out of fuel.
    /// - If the `exec` closure traps.
    #[inline(always)]
    fn consume_fuel_with<T, E>(
        &mut self,
        delta: impl FnOnce(&FuelCosts) -> u64,
        exec: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<TrapCode>,
    {
        match self.get_fuel_consumption_mode() {
            None => exec(self),
            Some(mode) => self.consume_fuel_with_mode(mode, delta, exec),
        }
    }

    /// Consume an amount of fuel specified by `delta` and executes `exec`.
    ///
    /// The `mode` determines when and if the fuel determined by `delta` is charged.
    ///
    /// # Errors
    ///
    /// - If the [`StoreInner`] ran out of fuel.
    /// - If the `exec` closure traps.
    #[inline(always)]
    fn consume_fuel_with_mode<T, E>(
        &mut self,
        mode: FuelConsumptionMode,
        delta: impl FnOnce(&FuelCosts) -> u64,
        exec: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<TrapCode>,
    {
        let delta = delta(self.fuel_costs());
        match mode {
            FuelConsumptionMode::Lazy => self.consume_fuel_with_lazy(delta, exec),
            FuelConsumptionMode::Eager => self.consume_fuel_with_eager(delta, exec),
        }
    }

    /// Consume an amount of fuel specified by `delta` if `exec` succeeds.
    ///
    /// Prior to executing `exec` it is checked if enough fuel is remaining
    /// determined by `delta`. The fuel is charged only after `exec` has been
    /// finished successfully.
    ///
    /// # Errors
    ///
    /// - If the [`StoreInner`] ran out of fuel.
    /// - If the `exec` closure traps.
    #[inline(always)]
    fn consume_fuel_with_lazy<T, E>(
        &mut self,
        delta: u64,
        exec: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<TrapCode>,
    {
        self.ctx.fuel().sufficient_fuel(delta)?;
        let result = exec(self)?;
        self.ctx
            .fuel_mut()
            .consume_fuel(delta)
            .expect("remaining fuel has already been approved prior");
        Ok(result)
    }

    /// Consume an amount of fuel specified by `delta` and executes `exec`.
    ///
    /// # Errors
    ///
    /// - If the [`StoreInner`] ran out of fuel.
    /// - If the `exec` closure traps.
    #[inline(always)]
    fn consume_fuel_with_eager<T, E>(
        &mut self,
        delta: u64,
        exec: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<TrapCode>,
    {
        self.ctx.fuel_mut().consume_fuel(delta)?;
        exec(self)
    }

    /// Returns a shared reference to the [`FuelCosts`] of the [`Engine`].
    ///
    /// [`Engine`]: crate::Engine
    #[inline]
    fn fuel_costs(&self) -> &FuelCosts {
        self.ctx.engine().config().fuel_costs()
    }

    /// Returns the [`FuelConsumptionMode`] of the [`Engine`].
    ///
    /// [`Engine`]: crate::Engine
    #[inline]
    fn get_fuel_consumption_mode(&self) -> Option<FuelConsumptionMode> {
        self.ctx.engine().config().get_fuel_consumption_mode()
    }

    /// Returns the [`Instruction::Const32`] parameter for an [`Instruction`].
    fn fetch_const32(&self, offset: usize) -> AnyConst32 {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::Const32(value) => value,
            _ => unreachable!("expected an Instruction::Const32 instruction word"),
        }
    }

    /// Returns the [`Instruction::Const32`] parameter for an [`Instruction`].
    fn fetch_address_offset(&self, offset: usize) -> u32 {
        self.fetch_const32(offset).to_u32()
    }

    /// Executes a generic unary [`Instruction`].
    fn execute_unary(&mut self, instr: UnaryInstr, op: fn(UntypedValue) -> UntypedValue) {
        let value = self.get_register(instr.input);
        self.set_register(instr.result, op(value));
        self.next_instr();
    }

    /// Executes a fallible generic unary [`Instruction`].
    fn try_execute_unary(
        &mut self,
        instr: UnaryInstr,
        op: fn(UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), TrapCode> {
        let value = self.get_register(instr.input);
        self.set_register(instr.result, op(value)?);
        self.try_next_instr()
    }

    /// Executes a generic binary [`Instruction`].
    fn execute_binary(
        &mut self,
        instr: BinInstr,
        op: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) {
        let lhs = self.get_register(instr.lhs);
        let rhs = self.get_register(instr.rhs);
        self.set_register(instr.result, op(lhs, rhs));
        self.next_instr();
    }

    /// Executes a generic binary [`Instruction`].
    fn execute_binary_imm16<T>(
        &mut self,
        instr: BinInstrImm16<T>,
        op: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) where
        T: From<Const16<T>>,
        UntypedValue: From<T>,
    {
        let lhs = self.get_register(instr.reg_in);
        let rhs = UntypedValue::from(<T>::from(instr.imm_in));
        self.set_register(instr.result, op(lhs, rhs));
        self.next_instr();
    }

    /// Executes a generic binary [`Instruction`] with reversed operands.
    fn execute_binary_imm16_rev<T>(
        &mut self,
        instr: BinInstrImm16<T>,
        op: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) where
        T: From<Const16<T>>,
        UntypedValue: From<T>,
    {
        let lhs = UntypedValue::from(<T>::from(instr.imm_in));
        let rhs = self.get_register(instr.reg_in);
        self.set_register(instr.result, op(lhs, rhs));
        self.next_instr();
    }

    /// Executes a fallible generic binary [`Instruction`].
    fn try_execute_binary(
        &mut self,
        instr: BinInstr,
        op: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), TrapCode> {
        let lhs = self.get_register(instr.lhs);
        let rhs = self.get_register(instr.rhs);
        self.set_register(instr.result, op(lhs, rhs)?);
        self.try_next_instr()
    }

    /// Executes a fallible generic binary [`Instruction`].
    fn try_execute_binary_imm16<T>(
        &mut self,
        instr: BinInstrImm16<T>,
        op: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), TrapCode>
    where
        T: From<Const16<T>>,
        UntypedValue: From<T>,
    {
        let lhs = self.get_register(instr.reg_in);
        let rhs = UntypedValue::from(<T>::from(instr.imm_in));
        self.set_register(instr.result, op(lhs, rhs)?);
        self.try_next_instr()
    }

    /// Executes a fallible generic binary [`Instruction`] with reversed operands.
    fn try_execute_binary_imm16_rev<T>(
        &mut self,
        instr: BinInstrImm16<T>,
        op: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<(), TrapCode>
    where
        T: From<Const16<T>>,
        UntypedValue: From<T>,
    {
        let lhs = UntypedValue::from(<T>::from(instr.imm_in));
        let rhs = self.get_register(instr.reg_in);
        self.set_register(instr.result, op(lhs, rhs)?);
        self.try_next_instr()
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

    fn execute_copy_impl<T>(
        &mut self,
        result: Register,
        value: T,
        f: fn(&mut Self, T) -> UntypedValue,
    ) {
        let value = f(self, value);
        // Safety: TODO
        let result = unsafe { self.sp.get_mut(result) };
        *result = value;
    }

    #[inline(always)]
    fn execute_copy(&mut self, result: Register, value: Register) {
        self.execute_copy_impl(result, value, |this, value| {
            // Safety: TODO
            unsafe { this.sp.get(value) }
        })
    }

    #[inline(always)]
    fn execute_copy_imm32(&mut self, result: Register, value: AnyConst32) {
        self.execute_copy_impl(result, value, |_, value| UntypedValue::from(value.to_u32()))
    }

    #[inline(always)]
    fn execute_copy_i64imm32(&mut self, result: Register, value: Const32<i64>) {
        self.execute_copy_impl(result, value, |_, value| {
            UntypedValue::from(i64::from(value))
        })
    }

    #[inline(always)]
    fn execute_copy_f64imm32(&mut self, result: Register, value: Const32<f64>) {
        self.execute_copy_impl(result, value, |_, value| {
            UntypedValue::from(f64::from(value))
        })
    }

    #[inline(always)]
    fn execute_copy_span(&mut self, results: RegisterSpan, values: RegisterSpan, len: u16) {
        let len = len as usize;
        let results = results.iter(len);
        let values = values.iter(len);
        for (result, value) in results.zip(values) {
            let value = self.get_register(value);
            self.set_register(result, value);
        }
    }

    /// Executes an [`Instruction::RefFunc`].
    #[inline(always)]
    fn execute_ref_func(&mut self, result: Register, func_index: FuncIdx) {
        let func = self.cache.get_func(self.ctx, func_index);
        let funcref = FuncRef::new(func);
        self.set_register(result, funcref);
        self.next_instr();
    }
}
