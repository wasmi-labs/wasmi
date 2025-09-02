pub use self::call::dispatch_host_func;
use super::{cache::CachedInstance, InstructionPtr, Stack};
use crate::{
    core::{hint, wasm, ReadAs, UntypedVal, WriteAs},
    engine::{
        code_map::CodeMap,
        executor::stack::{CallFrame, FrameRegisters, ValueStack},
        utils::unreachable_unchecked,
        DedupFuncType,
        EngineFunc,
    },
    ir::{index, BlockFuel, Const16, Offset64Hi, Op, Reg, ShiftAmount},
    memory::DataSegment,
    store::{PrunedStore, StoreInner},
    table::ElementSegment,
    Error,
    Func,
    Global,
    Memory,
    Ref,
    Table,
    TrapCode,
};

#[cfg(doc)]
use crate::Instance;

#[macro_use]
mod utils;

#[cfg(feature = "simd")]
mod simd;

mod binary;
mod branch;
mod call;
mod comparison;
mod conversion;
mod copy;
mod global;
mod load;
mod memory;
mod return_;
mod select;
mod store;
mod table;
mod unary;
mod wide_arithmetic;

macro_rules! forward_return {
    ($expr:expr) => {{
        if hint::unlikely($expr.is_break()) {
            return Ok(());
        }
    }};
}

/// Tells if execution loop shall continue or break (return) to the execution's caller.
type ControlFlow = ::core::ops::ControlFlow<(), ()>;

/// Executes compiled function instructions until execution returns from the root function.
///
/// # Errors
///
/// If the execution encounters a trap.
#[inline(never)]
pub fn execute_instrs<'engine>(
    store: &mut PrunedStore,
    stack: &'engine mut Stack,
    code_map: &'engine CodeMap,
) -> Result<(), Error> {
    let instance = stack.calls.instance_expect();
    let cache = CachedInstance::new(store.inner_mut(), instance);
    let mut executor = Executor::new(stack, code_map, cache);
    if let Err(error) = executor.execute(store) {
        if error.is_out_of_fuel() {
            if let Some(frame) = executor.stack.calls.peek_mut() {
                // Note: we need to update the instruction pointer to make it possible to
                //       resume execution at the current instruction after running out of fuel.
                frame.update_instr_ptr(executor.ip);
            }
        }
        return Err(error);
    }
    Ok(())
}

/// An execution context for executing a Wasmi function frame.
#[derive(Debug)]
struct Executor<'engine> {
    /// Stores the value stack of live values on the Wasm stack.
    sp: FrameRegisters,
    /// The pointer to the currently executed instruction.
    ip: InstructionPtr,
    /// The cached instance and instance related data.
    cache: CachedInstance,
    /// The value and call stacks.
    stack: &'engine mut Stack,
    /// The static resources of an [`Engine`].
    ///
    /// [`Engine`]: crate::Engine
    code_map: &'engine CodeMap,
}

impl<'engine> Executor<'engine> {
    /// Creates a new [`Executor`] for executing a Wasmi function frame.
    #[inline(always)]
    pub fn new(
        stack: &'engine mut Stack,
        code_map: &'engine CodeMap,
        cache: CachedInstance,
    ) -> Self {
        let frame = stack
            .calls
            .peek()
            .expect("must have call frame on the call stack");
        // Safety: We are using the frame's own base offset as input because it is
        //         guaranteed by the Wasm validation and translation phase to be
        //         valid for all register indices used by the associated function body.
        let sp = unsafe { stack.values.stack_ptr_at(frame.base_offset()) };
        let ip = frame.instr_ptr();
        Self {
            sp,
            ip,
            cache,
            stack,
            code_map,
        }
    }

    /// Executes the function frame until it returns or traps.
    #[inline(always)]
    fn execute(&mut self, store: &mut PrunedStore) -> Result<(), Error> {
        use Op as Instr;
        loop {
            match *self.ip.get() {
                Instr::Trap { trap_code } => self.execute_trap(trap_code)?,
                Instr::ConsumeFuel { block_fuel } => {
                    self.execute_consume_fuel(store.inner_mut(), block_fuel)?
                }
                Instr::Return => {
                    forward_return!(self.execute_return(store.inner_mut()))
                }
                Instr::ReturnReg { value } => {
                    forward_return!(self.execute_return_reg(store.inner_mut(), value))
                }
                Instr::ReturnReg2 { values } => {
                    forward_return!(self.execute_return_reg2(store.inner_mut(), values))
                }
                Instr::ReturnReg3 { values } => {
                    forward_return!(self.execute_return_reg3(store.inner_mut(), values))
                }
                Instr::ReturnImm32 { value } => {
                    forward_return!(self.execute_return_imm32(store.inner_mut(), value))
                }
                Instr::ReturnI64Imm32 { value } => {
                    forward_return!(self.execute_return_i64imm32(store.inner_mut(), value))
                }
                Instr::ReturnF64Imm32 { value } => {
                    forward_return!(self.execute_return_f64imm32(store.inner_mut(), value))
                }
                Instr::ReturnSpan { values } => {
                    forward_return!(self.execute_return_span(store.inner_mut(), values))
                }
                Instr::ReturnMany { values } => {
                    forward_return!(self.execute_return_many(store.inner_mut(), values))
                }
                Instr::Branch { offset } => self.execute_branch(offset),
                Instr::BranchTable0 { index, len_targets } => {
                    self.execute_branch_table_0(index, len_targets)
                }
                Instr::BranchTableSpan { index, len_targets } => {
                    self.execute_branch_table_span(index, len_targets)
                }
                Instr::BranchCmpFallback { lhs, rhs, params } => {
                    self.execute_branch_cmp_fallback(lhs, rhs, params)
                }
                Instr::BranchI32And { lhs, rhs, offset } => {
                    self.execute_branch_i32_and(lhs, rhs, offset)
                }
                Instr::BranchI32AndImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i32_and_imm16(lhs, rhs, offset)
                }
                Instr::BranchI32Or { lhs, rhs, offset } => {
                    self.execute_branch_i32_or(lhs, rhs, offset)
                }
                Instr::BranchI32OrImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i32_or_imm16(lhs, rhs, offset)
                }
                Instr::BranchI32Nand { lhs, rhs, offset } => {
                    self.execute_branch_i32_nand(lhs, rhs, offset)
                }
                Instr::BranchI32NandImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i32_nand_imm16(lhs, rhs, offset)
                }
                Instr::BranchI32Nor { lhs, rhs, offset } => {
                    self.execute_branch_i32_nor(lhs, rhs, offset)
                }
                Instr::BranchI32NorImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i32_nor_imm16(lhs, rhs, offset)
                }
                Instr::BranchI32Eq { lhs, rhs, offset } => {
                    self.execute_branch_i32_eq(lhs, rhs, offset)
                }
                Instr::BranchI32EqImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i32_eq_imm16(lhs, rhs, offset)
                }
                Instr::BranchI32Ne { lhs, rhs, offset } => {
                    self.execute_branch_i32_ne(lhs, rhs, offset)
                }
                Instr::BranchI32NeImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i32_ne_imm16(lhs, rhs, offset)
                }
                Instr::BranchI32LtS { lhs, rhs, offset } => {
                    self.execute_branch_i32_lt_s(lhs, rhs, offset)
                }
                Instr::BranchI32LtSImm16Lhs { lhs, rhs, offset } => {
                    self.execute_branch_i32_lt_s_imm16_lhs(lhs, rhs, offset)
                }
                Instr::BranchI32LtSImm16Rhs { lhs, rhs, offset } => {
                    self.execute_branch_i32_lt_s_imm16_rhs(lhs, rhs, offset)
                }
                Instr::BranchI32LtU { lhs, rhs, offset } => {
                    self.execute_branch_i32_lt_u(lhs, rhs, offset)
                }
                Instr::BranchI32LtUImm16Lhs { lhs, rhs, offset } => {
                    self.execute_branch_i32_lt_u_imm16_lhs(lhs, rhs, offset)
                }
                Instr::BranchI32LtUImm16Rhs { lhs, rhs, offset } => {
                    self.execute_branch_i32_lt_u_imm16_rhs(lhs, rhs, offset)
                }
                Instr::BranchI32LeS { lhs, rhs, offset } => {
                    self.execute_branch_i32_le_s(lhs, rhs, offset)
                }
                Instr::BranchI32LeSImm16Lhs { lhs, rhs, offset } => {
                    self.execute_branch_i32_le_s_imm16_lhs(lhs, rhs, offset)
                }
                Instr::BranchI32LeSImm16Rhs { lhs, rhs, offset } => {
                    self.execute_branch_i32_le_s_imm16_rhs(lhs, rhs, offset)
                }
                Instr::BranchI32LeU { lhs, rhs, offset } => {
                    self.execute_branch_i32_le_u(lhs, rhs, offset)
                }
                Instr::BranchI32LeUImm16Lhs { lhs, rhs, offset } => {
                    self.execute_branch_i32_le_u_imm16_lhs(lhs, rhs, offset)
                }
                Instr::BranchI32LeUImm16Rhs { lhs, rhs, offset } => {
                    self.execute_branch_i32_le_u_imm16_rhs(lhs, rhs, offset)
                }
                Instr::BranchI64And { lhs, rhs, offset } => {
                    self.execute_branch_i64_and(lhs, rhs, offset)
                }
                Instr::BranchI64AndImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i64_and_imm16(lhs, rhs, offset)
                }
                Instr::BranchI64Or { lhs, rhs, offset } => {
                    self.execute_branch_i64_or(lhs, rhs, offset)
                }
                Instr::BranchI64OrImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i64_or_imm16(lhs, rhs, offset)
                }
                Instr::BranchI64Nand { lhs, rhs, offset } => {
                    self.execute_branch_i64_nand(lhs, rhs, offset)
                }
                Instr::BranchI64NandImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i64_nand_imm16(lhs, rhs, offset)
                }
                Instr::BranchI64Nor { lhs, rhs, offset } => {
                    self.execute_branch_i64_nor(lhs, rhs, offset)
                }
                Instr::BranchI64NorImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i64_nor_imm16(lhs, rhs, offset)
                }
                Instr::BranchI64Eq { lhs, rhs, offset } => {
                    self.execute_branch_i64_eq(lhs, rhs, offset)
                }
                Instr::BranchI64EqImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i64_eq_imm16(lhs, rhs, offset)
                }
                Instr::BranchI64Ne { lhs, rhs, offset } => {
                    self.execute_branch_i64_ne(lhs, rhs, offset)
                }
                Instr::BranchI64NeImm16 { lhs, rhs, offset } => {
                    self.execute_branch_i64_ne_imm16(lhs, rhs, offset)
                }
                Instr::BranchI64LtS { lhs, rhs, offset } => {
                    self.execute_branch_i64_lt_s(lhs, rhs, offset)
                }
                Instr::BranchI64LtSImm16Lhs { lhs, rhs, offset } => {
                    self.execute_branch_i64_lt_s_imm16_lhs(lhs, rhs, offset)
                }
                Instr::BranchI64LtSImm16Rhs { lhs, rhs, offset } => {
                    self.execute_branch_i64_lt_s_imm16_rhs(lhs, rhs, offset)
                }
                Instr::BranchI64LtU { lhs, rhs, offset } => {
                    self.execute_branch_i64_lt_u(lhs, rhs, offset)
                }
                Instr::BranchI64LtUImm16Lhs { lhs, rhs, offset } => {
                    self.execute_branch_i64_lt_u_imm16_lhs(lhs, rhs, offset)
                }
                Instr::BranchI64LtUImm16Rhs { lhs, rhs, offset } => {
                    self.execute_branch_i64_lt_u_imm16_rhs(lhs, rhs, offset)
                }
                Instr::BranchI64LeS { lhs, rhs, offset } => {
                    self.execute_branch_i64_le_s(lhs, rhs, offset)
                }
                Instr::BranchI64LeSImm16Lhs { lhs, rhs, offset } => {
                    self.execute_branch_i64_le_s_imm16_lhs(lhs, rhs, offset)
                }
                Instr::BranchI64LeSImm16Rhs { lhs, rhs, offset } => {
                    self.execute_branch_i64_le_s_imm16_rhs(lhs, rhs, offset)
                }
                Instr::BranchI64LeU { lhs, rhs, offset } => {
                    self.execute_branch_i64_le_u(lhs, rhs, offset)
                }
                Instr::BranchI64LeUImm16Lhs { lhs, rhs, offset } => {
                    self.execute_branch_i64_le_u_imm16_lhs(lhs, rhs, offset)
                }
                Instr::BranchI64LeUImm16Rhs { lhs, rhs, offset } => {
                    self.execute_branch_i64_le_u_imm16_rhs(lhs, rhs, offset)
                }
                Instr::BranchF32Eq { lhs, rhs, offset } => {
                    self.execute_branch_f32_eq(lhs, rhs, offset)
                }
                Instr::BranchF32Ne { lhs, rhs, offset } => {
                    self.execute_branch_f32_ne(lhs, rhs, offset)
                }
                Instr::BranchF32Lt { lhs, rhs, offset } => {
                    self.execute_branch_f32_lt(lhs, rhs, offset)
                }
                Instr::BranchF32Le { lhs, rhs, offset } => {
                    self.execute_branch_f32_le(lhs, rhs, offset)
                }
                Instr::BranchF32NotLt { lhs, rhs, offset } => {
                    self.execute_branch_f32_not_lt(lhs, rhs, offset)
                }
                Instr::BranchF32NotLe { lhs, rhs, offset } => {
                    self.execute_branch_f32_not_le(lhs, rhs, offset)
                }
                Instr::BranchF64Eq { lhs, rhs, offset } => {
                    self.execute_branch_f64_eq(lhs, rhs, offset)
                }
                Instr::BranchF64Ne { lhs, rhs, offset } => {
                    self.execute_branch_f64_ne(lhs, rhs, offset)
                }
                Instr::BranchF64Lt { lhs, rhs, offset } => {
                    self.execute_branch_f64_lt(lhs, rhs, offset)
                }
                Instr::BranchF64Le { lhs, rhs, offset } => {
                    self.execute_branch_f64_le(lhs, rhs, offset)
                }
                Instr::BranchF64NotLt { lhs, rhs, offset } => {
                    self.execute_branch_f64_not_lt(lhs, rhs, offset)
                }
                Instr::BranchF64NotLe { lhs, rhs, offset } => {
                    self.execute_branch_f64_not_le(lhs, rhs, offset)
                }
                Instr::Copy { result, value } => self.execute_copy(result, value),
                Instr::Copy2 { results, values } => self.execute_copy_2(results, values),
                Instr::CopyImm32 { result, value } => self.execute_copy_imm32(result, value),
                Instr::CopyI64Imm32 { result, value } => self.execute_copy_i64imm32(result, value),
                Instr::CopyF64Imm32 { result, value } => self.execute_copy_f64imm32(result, value),
                Instr::CopySpan {
                    results,
                    values,
                    len,
                } => self.execute_copy_span(results, values, len),
                Instr::CopyMany { results, values } => self.execute_copy_many(results, values),
                Instr::ReturnCallInternal0 { func } => {
                    self.execute_return_call_internal_0(store.inner_mut(), EngineFunc::from(func))?
                }
                Instr::ReturnCallInternal { func } => {
                    self.execute_return_call_internal(store.inner_mut(), EngineFunc::from(func))?
                }
                Instr::ReturnCallImported0 { func } => {
                    forward_return!(self.execute_return_call_imported_0(store, func)?)
                }
                Instr::ReturnCallImported { func } => {
                    forward_return!(self.execute_return_call_imported(store, func)?)
                }
                Instr::ReturnCallIndirect0 { func_type } => {
                    forward_return!(self.execute_return_call_indirect_0(store, func_type)?)
                }
                Instr::ReturnCallIndirect0Imm16 { func_type } => {
                    forward_return!(self.execute_return_call_indirect_0_imm16(store, func_type)?)
                }
                Instr::ReturnCallIndirect { func_type } => {
                    forward_return!(self.execute_return_call_indirect(store, func_type)?)
                }
                Instr::ReturnCallIndirectImm16 { func_type } => {
                    forward_return!(self.execute_return_call_indirect_imm16(store, func_type)?)
                }
                Instr::CallInternal0 { results, func } => self.execute_call_internal_0(
                    store.inner_mut(),
                    results,
                    EngineFunc::from(func),
                )?,
                Instr::CallInternal { results, func } => {
                    self.execute_call_internal(store.inner_mut(), results, EngineFunc::from(func))?
                }
                Instr::CallImported0 { results, func } => {
                    self.execute_call_imported_0(store, results, func)?
                }
                Instr::CallImported { results, func } => {
                    self.execute_call_imported(store, results, func)?
                }
                Instr::CallIndirect0 { results, func_type } => {
                    self.execute_call_indirect_0(store, results, func_type)?
                }
                Instr::CallIndirect0Imm16 { results, func_type } => {
                    self.execute_call_indirect_0_imm16(store, results, func_type)?
                }
                Instr::CallIndirect { results, func_type } => {
                    self.execute_call_indirect(store, results, func_type)?
                }
                Instr::CallIndirectImm16 { results, func_type } => {
                    self.execute_call_indirect_imm16(store, results, func_type)?
                }
                Instr::SelectI32Eq { result, lhs, rhs } => {
                    self.execute_select_i32_eq(result, lhs, rhs)
                }
                Instr::SelectI32EqImm16 { result, lhs, rhs } => {
                    self.execute_select_i32_eq_imm16(result, lhs, rhs)
                }
                Instr::SelectI32LtS { result, lhs, rhs } => {
                    self.execute_select_i32_lt_s(result, lhs, rhs)
                }
                Instr::SelectI32LtSImm16Rhs { result, lhs, rhs } => {
                    self.execute_select_i32_lt_s_imm16_rhs(result, lhs, rhs)
                }
                Instr::SelectI32LtU { result, lhs, rhs } => {
                    self.execute_select_i32_lt_u(result, lhs, rhs)
                }
                Instr::SelectI32LtUImm16Rhs { result, lhs, rhs } => {
                    self.execute_select_i32_lt_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::SelectI32LeS { result, lhs, rhs } => {
                    self.execute_select_i32_le_s(result, lhs, rhs)
                }
                Instr::SelectI32LeSImm16Rhs { result, lhs, rhs } => {
                    self.execute_select_i32_le_s_imm16_rhs(result, lhs, rhs)
                }
                Instr::SelectI32LeU { result, lhs, rhs } => {
                    self.execute_select_i32_le_u(result, lhs, rhs)
                }
                Instr::SelectI32LeUImm16Rhs { result, lhs, rhs } => {
                    self.execute_select_i32_le_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::SelectI32And { result, lhs, rhs } => {
                    self.execute_select_i32_and(result, lhs, rhs)
                }
                Instr::SelectI32AndImm16 { result, lhs, rhs } => {
                    self.execute_select_i32_and_imm16(result, lhs, rhs)
                }
                Instr::SelectI32Or { result, lhs, rhs } => {
                    self.execute_select_i32_or(result, lhs, rhs)
                }
                Instr::SelectI32OrImm16 { result, lhs, rhs } => {
                    self.execute_select_i32_or_imm16(result, lhs, rhs)
                }
                Instr::SelectI64Eq { result, lhs, rhs } => {
                    self.execute_select_i64_eq(result, lhs, rhs)
                }
                Instr::SelectI64EqImm16 { result, lhs, rhs } => {
                    self.execute_select_i64_eq_imm16(result, lhs, rhs)
                }
                Instr::SelectI64LtS { result, lhs, rhs } => {
                    self.execute_select_i64_lt_s(result, lhs, rhs)
                }
                Instr::SelectI64LtSImm16Rhs { result, lhs, rhs } => {
                    self.execute_select_i64_lt_s_imm16_rhs(result, lhs, rhs)
                }
                Instr::SelectI64LtU { result, lhs, rhs } => {
                    self.execute_select_i64_lt_u(result, lhs, rhs)
                }
                Instr::SelectI64LtUImm16Rhs { result, lhs, rhs } => {
                    self.execute_select_i64_lt_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::SelectI64LeS { result, lhs, rhs } => {
                    self.execute_select_i64_le_s(result, lhs, rhs)
                }
                Instr::SelectI64LeSImm16Rhs { result, lhs, rhs } => {
                    self.execute_select_i64_le_s_imm16_rhs(result, lhs, rhs)
                }
                Instr::SelectI64LeU { result, lhs, rhs } => {
                    self.execute_select_i64_le_u(result, lhs, rhs)
                }
                Instr::SelectI64LeUImm16Rhs { result, lhs, rhs } => {
                    self.execute_select_i64_le_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::SelectI64And { result, lhs, rhs } => {
                    self.execute_select_i64_and(result, lhs, rhs)
                }
                Instr::SelectI64AndImm16 { result, lhs, rhs } => {
                    self.execute_select_i64_and_imm16(result, lhs, rhs)
                }
                Instr::SelectI64Or { result, lhs, rhs } => {
                    self.execute_select_i64_or(result, lhs, rhs)
                }
                Instr::SelectI64OrImm16 { result, lhs, rhs } => {
                    self.execute_select_i64_or_imm16(result, lhs, rhs)
                }
                Instr::SelectF32Eq { result, lhs, rhs } => {
                    self.execute_select_f32_eq(result, lhs, rhs)
                }
                Instr::SelectF32Lt { result, lhs, rhs } => {
                    self.execute_select_f32_lt(result, lhs, rhs)
                }
                Instr::SelectF32Le { result, lhs, rhs } => {
                    self.execute_select_f32_le(result, lhs, rhs)
                }
                Instr::SelectF64Eq { result, lhs, rhs } => {
                    self.execute_select_f64_eq(result, lhs, rhs)
                }
                Instr::SelectF64Lt { result, lhs, rhs } => {
                    self.execute_select_f64_lt(result, lhs, rhs)
                }
                Instr::SelectF64Le { result, lhs, rhs } => {
                    self.execute_select_f64_le(result, lhs, rhs)
                }
                Instr::RefFunc { result, func } => self.execute_ref_func(result, func),
                Instr::GlobalGet { result, global } => {
                    self.execute_global_get(store.inner(), result, global)
                }
                Instr::GlobalSet { global, input } => {
                    self.execute_global_set(store.inner_mut(), global, input)
                }
                Instr::GlobalSetI32Imm16 { global, input } => {
                    self.execute_global_set_i32imm16(store.inner_mut(), global, input)
                }
                Instr::GlobalSetI64Imm16 { global, input } => {
                    self.execute_global_set_i64imm16(store.inner_mut(), global, input)
                }
                Instr::Load32 { result, offset_lo } => {
                    self.execute_load32(store.inner(), result, offset_lo)?
                }
                Instr::Load32At { result, address } => {
                    self.execute_load32_at(store.inner(), result, address)?
                }
                Instr::Load32Offset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_load32_offset16(result, ptr, offset)?,
                Instr::Load64 { result, offset_lo } => {
                    self.execute_load64(store.inner(), result, offset_lo)?
                }
                Instr::Load64At { result, address } => {
                    self.execute_load64_at(store.inner(), result, address)?
                }
                Instr::Load64Offset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_load64_offset16(result, ptr, offset)?,
                Instr::I32Load8s { result, offset_lo } => {
                    self.execute_i32_load8_s(store.inner(), result, offset_lo)?
                }
                Instr::I32Load8sAt { result, address } => {
                    self.execute_i32_load8_s_at(store.inner(), result, address)?
                }
                Instr::I32Load8sOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i32_load8_s_offset16(result, ptr, offset)?,
                Instr::I32Load8u { result, offset_lo } => {
                    self.execute_i32_load8_u(store.inner(), result, offset_lo)?
                }
                Instr::I32Load8uAt { result, address } => {
                    self.execute_i32_load8_u_at(store.inner(), result, address)?
                }
                Instr::I32Load8uOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i32_load8_u_offset16(result, ptr, offset)?,
                Instr::I32Load16s { result, offset_lo } => {
                    self.execute_i32_load16_s(store.inner(), result, offset_lo)?
                }
                Instr::I32Load16sAt { result, address } => {
                    self.execute_i32_load16_s_at(store.inner(), result, address)?
                }
                Instr::I32Load16sOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i32_load16_s_offset16(result, ptr, offset)?,
                Instr::I32Load16u { result, offset_lo } => {
                    self.execute_i32_load16_u(store.inner(), result, offset_lo)?
                }
                Instr::I32Load16uAt { result, address } => {
                    self.execute_i32_load16_u_at(store.inner(), result, address)?
                }
                Instr::I32Load16uOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i32_load16_u_offset16(result, ptr, offset)?,
                Instr::I64Load8s { result, offset_lo } => {
                    self.execute_i64_load8_s(store.inner(), result, offset_lo)?
                }
                Instr::I64Load8sAt { result, address } => {
                    self.execute_i64_load8_s_at(store.inner(), result, address)?
                }
                Instr::I64Load8sOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i64_load8_s_offset16(result, ptr, offset)?,
                Instr::I64Load8u { result, offset_lo } => {
                    self.execute_i64_load8_u(store.inner(), result, offset_lo)?
                }
                Instr::I64Load8uAt { result, address } => {
                    self.execute_i64_load8_u_at(store.inner(), result, address)?
                }
                Instr::I64Load8uOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i64_load8_u_offset16(result, ptr, offset)?,
                Instr::I64Load16s { result, offset_lo } => {
                    self.execute_i64_load16_s(store.inner(), result, offset_lo)?
                }
                Instr::I64Load16sAt { result, address } => {
                    self.execute_i64_load16_s_at(store.inner(), result, address)?
                }
                Instr::I64Load16sOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i64_load16_s_offset16(result, ptr, offset)?,
                Instr::I64Load16u { result, offset_lo } => {
                    self.execute_i64_load16_u(store.inner(), result, offset_lo)?
                }
                Instr::I64Load16uAt { result, address } => {
                    self.execute_i64_load16_u_at(store.inner(), result, address)?
                }
                Instr::I64Load16uOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i64_load16_u_offset16(result, ptr, offset)?,
                Instr::I64Load32s { result, offset_lo } => {
                    self.execute_i64_load32_s(store.inner(), result, offset_lo)?
                }
                Instr::I64Load32sAt { result, address } => {
                    self.execute_i64_load32_s_at(store.inner(), result, address)?
                }
                Instr::I64Load32sOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i64_load32_s_offset16(result, ptr, offset)?,
                Instr::I64Load32u { result, offset_lo } => {
                    self.execute_i64_load32_u(store.inner(), result, offset_lo)?
                }
                Instr::I64Load32uAt { result, address } => {
                    self.execute_i64_load32_u_at(store.inner(), result, address)?
                }
                Instr::I64Load32uOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_i64_load32_u_offset16(result, ptr, offset)?,
                Instr::Store32 { ptr, offset_lo } => {
                    self.execute_store32(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::Store32Offset16 { ptr, offset, value } => {
                    self.execute_store32_offset16(ptr, offset, value)?
                }
                Instr::Store32At { address, value } => {
                    self.execute_store32_at(store.inner_mut(), address, value)?
                }
                Instr::Store64 { ptr, offset_lo } => {
                    self.execute_store64(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::Store64Offset16 { ptr, offset, value } => {
                    self.execute_store64_offset16(ptr, offset, value)?
                }
                Instr::Store64At { address, value } => {
                    self.execute_store64_at(store.inner_mut(), address, value)?
                }
                Instr::I32StoreImm16 { ptr, offset_lo } => {
                    self.execute_i32_store_imm16(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I32StoreOffset16Imm16 { ptr, offset, value } => {
                    self.execute_i32_store_offset16_imm16(ptr, offset, value)?
                }
                Instr::I32StoreAtImm16 { address, value } => {
                    self.execute_i32_store_at_imm16(store.inner_mut(), address, value)?
                }
                Instr::I32Store8 { ptr, offset_lo } => {
                    self.execute_i32_store8(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I32Store8Imm { ptr, offset_lo } => {
                    self.execute_i32_store8_imm(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I32Store8Offset16 { ptr, offset, value } => {
                    self.execute_i32_store8_offset16(ptr, offset, value)?
                }
                Instr::I32Store8Offset16Imm { ptr, offset, value } => {
                    self.execute_i32_store8_offset16_imm(ptr, offset, value)?
                }
                Instr::I32Store8At { address, value } => {
                    self.execute_i32_store8_at(store.inner_mut(), address, value)?
                }
                Instr::I32Store8AtImm { address, value } => {
                    self.execute_i32_store8_at_imm(store.inner_mut(), address, value)?
                }
                Instr::I32Store16 { ptr, offset_lo } => {
                    self.execute_i32_store16(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I32Store16Imm { ptr, offset_lo } => {
                    self.execute_i32_store16_imm(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I32Store16Offset16 { ptr, offset, value } => {
                    self.execute_i32_store16_offset16(ptr, offset, value)?
                }
                Instr::I32Store16Offset16Imm { ptr, offset, value } => {
                    self.execute_i32_store16_offset16_imm(ptr, offset, value)?
                }
                Instr::I32Store16At { address, value } => {
                    self.execute_i32_store16_at(store.inner_mut(), address, value)?
                }
                Instr::I32Store16AtImm { address, value } => {
                    self.execute_i32_store16_at_imm(store.inner_mut(), address, value)?
                }
                Instr::I64StoreImm16 { ptr, offset_lo } => {
                    self.execute_i64_store_imm16(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I64StoreOffset16Imm16 { ptr, offset, value } => {
                    self.execute_i64_store_offset16_imm16(ptr, offset, value)?
                }
                Instr::I64StoreAtImm16 { address, value } => {
                    self.execute_i64_store_at_imm16(store.inner_mut(), address, value)?
                }
                Instr::I64Store8 { ptr, offset_lo } => {
                    self.execute_i64_store8(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I64Store8Imm { ptr, offset_lo } => {
                    self.execute_i64_store8_imm(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I64Store8Offset16 { ptr, offset, value } => {
                    self.execute_i64_store8_offset16(ptr, offset, value)?
                }
                Instr::I64Store8Offset16Imm { ptr, offset, value } => {
                    self.execute_i64_store8_offset16_imm(ptr, offset, value)?
                }
                Instr::I64Store8At { address, value } => {
                    self.execute_i64_store8_at(store.inner_mut(), address, value)?
                }
                Instr::I64Store8AtImm { address, value } => {
                    self.execute_i64_store8_at_imm(store.inner_mut(), address, value)?
                }
                Instr::I64Store16 { ptr, offset_lo } => {
                    self.execute_i64_store16(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I64Store16Imm { ptr, offset_lo } => {
                    self.execute_i64_store16_imm(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I64Store16Offset16 { ptr, offset, value } => {
                    self.execute_i64_store16_offset16(ptr, offset, value)?
                }
                Instr::I64Store16Offset16Imm { ptr, offset, value } => {
                    self.execute_i64_store16_offset16_imm(ptr, offset, value)?
                }
                Instr::I64Store16At { address, value } => {
                    self.execute_i64_store16_at(store.inner_mut(), address, value)?
                }
                Instr::I64Store16AtImm { address, value } => {
                    self.execute_i64_store16_at_imm(store.inner_mut(), address, value)?
                }
                Instr::I64Store32 { ptr, offset_lo } => {
                    self.execute_i64_store32(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I64Store32Imm16 { ptr, offset_lo } => {
                    self.execute_i64_store32_imm16(store.inner_mut(), ptr, offset_lo)?
                }
                Instr::I64Store32Offset16 { ptr, offset, value } => {
                    self.execute_i64_store32_offset16(ptr, offset, value)?
                }
                Instr::I64Store32Offset16Imm16 { ptr, offset, value } => {
                    self.execute_i64_store32_offset16_imm16(ptr, offset, value)?
                }
                Instr::I64Store32At { address, value } => {
                    self.execute_i64_store32_at(store.inner_mut(), address, value)?
                }
                Instr::I64Store32AtImm16 { address, value } => {
                    self.execute_i64_store32_at_imm16(store.inner_mut(), address, value)?
                }
                Instr::I32Eq { result, lhs, rhs } => self.execute_i32_eq(result, lhs, rhs),
                Instr::I32EqImm16 { result, lhs, rhs } => {
                    self.execute_i32_eq_imm16(result, lhs, rhs)
                }
                Instr::I32Ne { result, lhs, rhs } => self.execute_i32_ne(result, lhs, rhs),
                Instr::I32NeImm16 { result, lhs, rhs } => {
                    self.execute_i32_ne_imm16(result, lhs, rhs)
                }
                Instr::I32LtS { result, lhs, rhs } => self.execute_i32_lt_s(result, lhs, rhs),
                Instr::I32LtSImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_lt_s_imm16_lhs(result, lhs, rhs)
                }
                Instr::I32LtSImm16Rhs { result, lhs, rhs } => {
                    self.execute_i32_lt_s_imm16_rhs(result, lhs, rhs)
                }
                Instr::I32LtU { result, lhs, rhs } => self.execute_i32_lt_u(result, lhs, rhs),
                Instr::I32LtUImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_lt_u_imm16_lhs(result, lhs, rhs)
                }
                Instr::I32LtUImm16Rhs { result, lhs, rhs } => {
                    self.execute_i32_lt_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::I32LeS { result, lhs, rhs } => self.execute_i32_le_s(result, lhs, rhs),
                Instr::I32LeSImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_le_s_imm16_lhs(result, lhs, rhs)
                }
                Instr::I32LeSImm16Rhs { result, lhs, rhs } => {
                    self.execute_i32_le_s_imm16_rhs(result, lhs, rhs)
                }
                Instr::I32LeU { result, lhs, rhs } => self.execute_i32_le_u(result, lhs, rhs),
                Instr::I32LeUImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_le_u_imm16_lhs(result, lhs, rhs)
                }
                Instr::I32LeUImm16Rhs { result, lhs, rhs } => {
                    self.execute_i32_le_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::I64Eq { result, lhs, rhs } => self.execute_i64_eq(result, lhs, rhs),
                Instr::I64EqImm16 { result, lhs, rhs } => {
                    self.execute_i64_eq_imm16(result, lhs, rhs)
                }
                Instr::I64Ne { result, lhs, rhs } => self.execute_i64_ne(result, lhs, rhs),
                Instr::I64NeImm16 { result, lhs, rhs } => {
                    self.execute_i64_ne_imm16(result, lhs, rhs)
                }
                Instr::I64LtS { result, lhs, rhs } => self.execute_i64_lt_s(result, lhs, rhs),
                Instr::I64LtSImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_lt_s_imm16_lhs(result, lhs, rhs)
                }
                Instr::I64LtSImm16Rhs { result, lhs, rhs } => {
                    self.execute_i64_lt_s_imm16_rhs(result, lhs, rhs)
                }
                Instr::I64LtU { result, lhs, rhs } => self.execute_i64_lt_u(result, lhs, rhs),
                Instr::I64LtUImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_lt_u_imm16_lhs(result, lhs, rhs)
                }
                Instr::I64LtUImm16Rhs { result, lhs, rhs } => {
                    self.execute_i64_lt_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::I64LeS { result, lhs, rhs } => self.execute_i64_le_s(result, lhs, rhs),
                Instr::I64LeSImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_le_s_imm16_lhs(result, lhs, rhs)
                }
                Instr::I64LeSImm16Rhs { result, lhs, rhs } => {
                    self.execute_i64_le_s_imm16_rhs(result, lhs, rhs)
                }
                Instr::I64LeU { result, lhs, rhs } => self.execute_i64_le_u(result, lhs, rhs),
                Instr::I64LeUImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_le_u_imm16_lhs(result, lhs, rhs)
                }
                Instr::I64LeUImm16Rhs { result, lhs, rhs } => {
                    self.execute_i64_le_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::F32Eq { result, lhs, rhs } => self.execute_f32_eq(result, lhs, rhs),
                Instr::F32Ne { result, lhs, rhs } => self.execute_f32_ne(result, lhs, rhs),
                Instr::F32Lt { result, lhs, rhs } => self.execute_f32_lt(result, lhs, rhs),
                Instr::F32Le { result, lhs, rhs } => self.execute_f32_le(result, lhs, rhs),
                Instr::F32NotLt { result, lhs, rhs } => self.execute_f32_not_lt(result, lhs, rhs),
                Instr::F32NotLe { result, lhs, rhs } => self.execute_f32_not_le(result, lhs, rhs),
                Instr::F64Eq { result, lhs, rhs } => self.execute_f64_eq(result, lhs, rhs),
                Instr::F64Ne { result, lhs, rhs } => self.execute_f64_ne(result, lhs, rhs),
                Instr::F64Lt { result, lhs, rhs } => self.execute_f64_lt(result, lhs, rhs),
                Instr::F64Le { result, lhs, rhs } => self.execute_f64_le(result, lhs, rhs),
                Instr::F64NotLt { result, lhs, rhs } => self.execute_f64_not_lt(result, lhs, rhs),
                Instr::F64NotLe { result, lhs, rhs } => self.execute_f64_not_le(result, lhs, rhs),
                Instr::I32Clz { result, input } => self.execute_i32_clz(result, input),
                Instr::I32Ctz { result, input } => self.execute_i32_ctz(result, input),
                Instr::I32Popcnt { result, input } => self.execute_i32_popcnt(result, input),
                Instr::I32Add { result, lhs, rhs } => self.execute_i32_add(result, lhs, rhs),
                Instr::I32AddImm16 { result, lhs, rhs } => {
                    self.execute_i32_add_imm16(result, lhs, rhs)
                }
                Instr::I32Sub { result, lhs, rhs } => self.execute_i32_sub(result, lhs, rhs),
                Instr::I32SubImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_sub_imm16_lhs(result, lhs, rhs)
                }
                Instr::I32Mul { result, lhs, rhs } => self.execute_i32_mul(result, lhs, rhs),
                Instr::I32MulImm16 { result, lhs, rhs } => {
                    self.execute_i32_mul_imm16(result, lhs, rhs)
                }
                Instr::I32DivS { result, lhs, rhs } => self.execute_i32_div_s(result, lhs, rhs)?,
                Instr::I32DivSImm16Rhs { result, lhs, rhs } => {
                    self.execute_i32_div_s_imm16_rhs(result, lhs, rhs)?
                }
                Instr::I32DivSImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_div_s_imm16_lhs(result, lhs, rhs)?
                }
                Instr::I32DivU { result, lhs, rhs } => self.execute_i32_div_u(result, lhs, rhs)?,
                Instr::I32DivUImm16Rhs { result, lhs, rhs } => {
                    self.execute_i32_div_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::I32DivUImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_div_u_imm16_lhs(result, lhs, rhs)?
                }
                Instr::I32RemS { result, lhs, rhs } => self.execute_i32_rem_s(result, lhs, rhs)?,
                Instr::I32RemSImm16Rhs { result, lhs, rhs } => {
                    self.execute_i32_rem_s_imm16_rhs(result, lhs, rhs)?
                }
                Instr::I32RemSImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_rem_s_imm16_lhs(result, lhs, rhs)?
                }
                Instr::I32RemU { result, lhs, rhs } => self.execute_i32_rem_u(result, lhs, rhs)?,
                Instr::I32RemUImm16Rhs { result, lhs, rhs } => {
                    self.execute_i32_rem_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::I32RemUImm16Lhs { result, lhs, rhs } => {
                    self.execute_i32_rem_u_imm16_lhs(result, lhs, rhs)?
                }
                Instr::I32BitAnd { result, lhs, rhs } => self.execute_i32_bitand(result, lhs, rhs),
                Instr::I32BitAndImm16 { result, lhs, rhs } => {
                    self.execute_i32_bitand_imm16(result, lhs, rhs)
                }
                Instr::I32BitOr { result, lhs, rhs } => self.execute_i32_bitor(result, lhs, rhs),
                Instr::I32BitOrImm16 { result, lhs, rhs } => {
                    self.execute_i32_bitor_imm16(result, lhs, rhs)
                }
                Instr::I32BitXor { result, lhs, rhs } => self.execute_i32_bitxor(result, lhs, rhs),
                Instr::I32BitXorImm16 { result, lhs, rhs } => {
                    self.execute_i32_bitxor_imm16(result, lhs, rhs)
                }
                Instr::I32And { result, lhs, rhs } => self.execute_i32_and(result, lhs, rhs),
                Instr::I32AndImm16 { result, lhs, rhs } => {
                    self.execute_i32_and_imm16(result, lhs, rhs)
                }
                Instr::I32Or { result, lhs, rhs } => self.execute_i32_or(result, lhs, rhs),
                Instr::I32OrImm16 { result, lhs, rhs } => {
                    self.execute_i32_or_imm16(result, lhs, rhs)
                }
                Instr::I32Nand { result, lhs, rhs } => self.execute_i32_nand(result, lhs, rhs),
                Instr::I32NandImm16 { result, lhs, rhs } => {
                    self.execute_i32_nand_imm16(result, lhs, rhs)
                }
                Instr::I32Nor { result, lhs, rhs } => self.execute_i32_nor(result, lhs, rhs),
                Instr::I32NorImm16 { result, lhs, rhs } => {
                    self.execute_i32_nor_imm16(result, lhs, rhs)
                }
                Instr::I32Shl { result, lhs, rhs } => self.execute_i32_shl(result, lhs, rhs),
                Instr::I32ShlBy { result, lhs, rhs } => self.execute_i32_shl_by(result, lhs, rhs),
                Instr::I32ShlImm16 { result, lhs, rhs } => {
                    self.execute_i32_shl_imm16(result, lhs, rhs)
                }
                Instr::I32ShrU { result, lhs, rhs } => self.execute_i32_shr_u(result, lhs, rhs),
                Instr::I32ShrUBy { result, lhs, rhs } => {
                    self.execute_i32_shr_u_by(result, lhs, rhs)
                }
                Instr::I32ShrUImm16 { result, lhs, rhs } => {
                    self.execute_i32_shr_u_imm16(result, lhs, rhs)
                }
                Instr::I32ShrS { result, lhs, rhs } => self.execute_i32_shr_s(result, lhs, rhs),
                Instr::I32ShrSBy { result, lhs, rhs } => {
                    self.execute_i32_shr_s_by(result, lhs, rhs)
                }
                Instr::I32ShrSImm16 { result, lhs, rhs } => {
                    self.execute_i32_shr_s_imm16(result, lhs, rhs)
                }
                Instr::I32Rotl { result, lhs, rhs } => self.execute_i32_rotl(result, lhs, rhs),
                Instr::I32RotlBy { result, lhs, rhs } => self.execute_i32_rotl_by(result, lhs, rhs),
                Instr::I32RotlImm16 { result, lhs, rhs } => {
                    self.execute_i32_rotl_imm16(result, lhs, rhs)
                }
                Instr::I32Rotr { result, lhs, rhs } => self.execute_i32_rotr(result, lhs, rhs),
                Instr::I32RotrBy { result, lhs, rhs } => self.execute_i32_rotr_by(result, lhs, rhs),
                Instr::I32RotrImm16 { result, lhs, rhs } => {
                    self.execute_i32_rotr_imm16(result, lhs, rhs)
                }
                Instr::I64Clz { result, input } => self.execute_i64_clz(result, input),
                Instr::I64Ctz { result, input } => self.execute_i64_ctz(result, input),
                Instr::I64Popcnt { result, input } => self.execute_i64_popcnt(result, input),
                Instr::I64Add { result, lhs, rhs } => self.execute_i64_add(result, lhs, rhs),
                Instr::I64AddImm16 { result, lhs, rhs } => {
                    self.execute_i64_add_imm16(result, lhs, rhs)
                }
                Instr::I64Sub { result, lhs, rhs } => self.execute_i64_sub(result, lhs, rhs),
                Instr::I64SubImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_sub_imm16_lhs(result, lhs, rhs)
                }
                Instr::I64Mul { result, lhs, rhs } => self.execute_i64_mul(result, lhs, rhs),
                Instr::I64MulImm16 { result, lhs, rhs } => {
                    self.execute_i64_mul_imm16(result, lhs, rhs)
                }
                Instr::I64DivS { result, lhs, rhs } => self.execute_i64_div_s(result, lhs, rhs)?,
                Instr::I64DivSImm16Rhs { result, lhs, rhs } => {
                    self.execute_i64_div_s_imm16_rhs(result, lhs, rhs)?
                }
                Instr::I64DivSImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_div_s_imm16_lhs(result, lhs, rhs)?
                }
                Instr::I64DivU { result, lhs, rhs } => self.execute_i64_div_u(result, lhs, rhs)?,
                Instr::I64DivUImm16Rhs { result, lhs, rhs } => {
                    self.execute_i64_div_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::I64DivUImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_div_u_imm16_lhs(result, lhs, rhs)?
                }
                Instr::I64RemS { result, lhs, rhs } => self.execute_i64_rem_s(result, lhs, rhs)?,
                Instr::I64RemSImm16Rhs { result, lhs, rhs } => {
                    self.execute_i64_rem_s_imm16_rhs(result, lhs, rhs)?
                }
                Instr::I64RemSImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_rem_s_imm16_lhs(result, lhs, rhs)?
                }
                Instr::I64RemU { result, lhs, rhs } => self.execute_i64_rem_u(result, lhs, rhs)?,
                Instr::I64RemUImm16Rhs { result, lhs, rhs } => {
                    self.execute_i64_rem_u_imm16_rhs(result, lhs, rhs)
                }
                Instr::I64RemUImm16Lhs { result, lhs, rhs } => {
                    self.execute_i64_rem_u_imm16_lhs(result, lhs, rhs)?
                }
                Instr::I64BitAnd { result, lhs, rhs } => self.execute_i64_bitand(result, lhs, rhs),
                Instr::I64BitAndImm16 { result, lhs, rhs } => {
                    self.execute_i64_bitand_imm16(result, lhs, rhs)
                }
                Instr::I64BitOr { result, lhs, rhs } => self.execute_i64_bitor(result, lhs, rhs),
                Instr::I64BitOrImm16 { result, lhs, rhs } => {
                    self.execute_i64_bitor_imm16(result, lhs, rhs)
                }
                Instr::I64BitXor { result, lhs, rhs } => self.execute_i64_bitxor(result, lhs, rhs),
                Instr::I64BitXorImm16 { result, lhs, rhs } => {
                    self.execute_i64_bitxor_imm16(result, lhs, rhs)
                }
                Instr::I64And { result, lhs, rhs } => self.execute_i64_and(result, lhs, rhs),
                Instr::I64AndImm16 { result, lhs, rhs } => {
                    self.execute_i64_and_imm16(result, lhs, rhs)
                }
                Instr::I64Or { result, lhs, rhs } => self.execute_i64_or(result, lhs, rhs),
                Instr::I64OrImm16 { result, lhs, rhs } => {
                    self.execute_i64_or_imm16(result, lhs, rhs)
                }
                Instr::I64Nand { result, lhs, rhs } => self.execute_i64_nand(result, lhs, rhs),
                Instr::I64NandImm16 { result, lhs, rhs } => {
                    self.execute_i64_nand_imm16(result, lhs, rhs)
                }
                Instr::I64Nor { result, lhs, rhs } => self.execute_i64_nor(result, lhs, rhs),
                Instr::I64NorImm16 { result, lhs, rhs } => {
                    self.execute_i64_nor_imm16(result, lhs, rhs)
                }
                Instr::I64Shl { result, lhs, rhs } => self.execute_i64_shl(result, lhs, rhs),
                Instr::I64ShlBy { result, lhs, rhs } => self.execute_i64_shl_by(result, lhs, rhs),
                Instr::I64ShlImm16 { result, lhs, rhs } => {
                    self.execute_i64_shl_imm16(result, lhs, rhs)
                }
                Instr::I64ShrU { result, lhs, rhs } => self.execute_i64_shr_u(result, lhs, rhs),
                Instr::I64ShrUBy { result, lhs, rhs } => {
                    self.execute_i64_shr_u_by(result, lhs, rhs)
                }
                Instr::I64ShrUImm16 { result, lhs, rhs } => {
                    self.execute_i64_shr_u_imm16(result, lhs, rhs)
                }
                Instr::I64ShrS { result, lhs, rhs } => self.execute_i64_shr_s(result, lhs, rhs),
                Instr::I64ShrSBy { result, lhs, rhs } => {
                    self.execute_i64_shr_s_by(result, lhs, rhs)
                }
                Instr::I64ShrSImm16 { result, lhs, rhs } => {
                    self.execute_i64_shr_s_imm16(result, lhs, rhs)
                }
                Instr::I64Rotl { result, lhs, rhs } => self.execute_i64_rotl(result, lhs, rhs),
                Instr::I64RotlBy { result, lhs, rhs } => self.execute_i64_rotl_by(result, lhs, rhs),
                Instr::I64RotlImm16 { result, lhs, rhs } => {
                    self.execute_i64_rotl_imm16(result, lhs, rhs)
                }
                Instr::I64Rotr { result, lhs, rhs } => self.execute_i64_rotr(result, lhs, rhs),
                Instr::I64RotrBy { result, lhs, rhs } => self.execute_i64_rotr_by(result, lhs, rhs),
                Instr::I64RotrImm16 { result, lhs, rhs } => {
                    self.execute_i64_rotr_imm16(result, lhs, rhs)
                }
                Instr::I64Add128 { results, lhs_lo } => self.execute_i64_add128(results, lhs_lo),
                Instr::I64Sub128 { results, lhs_lo } => self.execute_i64_sub128(results, lhs_lo),
                Instr::I64MulWideS { results, lhs, rhs } => {
                    self.execute_i64_mul_wide_s(results, lhs, rhs)
                }
                Instr::I64MulWideU { results, lhs, rhs } => {
                    self.execute_i64_mul_wide_u(results, lhs, rhs)
                }
                Instr::I32WrapI64 { result, input } => self.execute_i32_wrap_i64(result, input),
                Instr::I32Extend8S { result, input } => self.execute_i32_extend8_s(result, input),
                Instr::I32Extend16S { result, input } => self.execute_i32_extend16_s(result, input),
                Instr::I64Extend8S { result, input } => self.execute_i64_extend8_s(result, input),
                Instr::I64Extend16S { result, input } => self.execute_i64_extend16_s(result, input),
                Instr::I64Extend32S { result, input } => self.execute_i64_extend32_s(result, input),
                Instr::F32Abs { result, input } => self.execute_f32_abs(result, input),
                Instr::F32Neg { result, input } => self.execute_f32_neg(result, input),
                Instr::F32Ceil { result, input } => self.execute_f32_ceil(result, input),
                Instr::F32Floor { result, input } => self.execute_f32_floor(result, input),
                Instr::F32Trunc { result, input } => self.execute_f32_trunc(result, input),
                Instr::F32Nearest { result, input } => self.execute_f32_nearest(result, input),
                Instr::F32Sqrt { result, input } => self.execute_f32_sqrt(result, input),
                Instr::F32Add { result, lhs, rhs } => self.execute_f32_add(result, lhs, rhs),
                Instr::F32Sub { result, lhs, rhs } => self.execute_f32_sub(result, lhs, rhs),
                Instr::F32Mul { result, lhs, rhs } => self.execute_f32_mul(result, lhs, rhs),
                Instr::F32Div { result, lhs, rhs } => self.execute_f32_div(result, lhs, rhs),
                Instr::F32Min { result, lhs, rhs } => self.execute_f32_min(result, lhs, rhs),
                Instr::F32Max { result, lhs, rhs } => self.execute_f32_max(result, lhs, rhs),
                Instr::F32Copysign { result, lhs, rhs } => {
                    self.execute_f32_copysign(result, lhs, rhs)
                }
                Instr::F32CopysignImm { result, lhs, rhs } => {
                    self.execute_f32_copysign_imm(result, lhs, rhs)
                }
                Instr::F64Abs { result, input } => self.execute_f64_abs(result, input),
                Instr::F64Neg { result, input } => self.execute_f64_neg(result, input),
                Instr::F64Ceil { result, input } => self.execute_f64_ceil(result, input),
                Instr::F64Floor { result, input } => self.execute_f64_floor(result, input),
                Instr::F64Trunc { result, input } => self.execute_f64_trunc(result, input),
                Instr::F64Nearest { result, input } => self.execute_f64_nearest(result, input),
                Instr::F64Sqrt { result, input } => self.execute_f64_sqrt(result, input),
                Instr::F64Add { result, lhs, rhs } => self.execute_f64_add(result, lhs, rhs),
                Instr::F64Sub { result, lhs, rhs } => self.execute_f64_sub(result, lhs, rhs),
                Instr::F64Mul { result, lhs, rhs } => self.execute_f64_mul(result, lhs, rhs),
                Instr::F64Div { result, lhs, rhs } => self.execute_f64_div(result, lhs, rhs),
                Instr::F64Min { result, lhs, rhs } => self.execute_f64_min(result, lhs, rhs),
                Instr::F64Max { result, lhs, rhs } => self.execute_f64_max(result, lhs, rhs),
                Instr::F64Copysign { result, lhs, rhs } => {
                    self.execute_f64_copysign(result, lhs, rhs)
                }
                Instr::F64CopysignImm { result, lhs, rhs } => {
                    self.execute_f64_copysign_imm(result, lhs, rhs)
                }
                Instr::I32TruncF32S { result, input } => {
                    self.execute_i32_trunc_f32_s(result, input)?
                }
                Instr::I32TruncF32U { result, input } => {
                    self.execute_i32_trunc_f32_u(result, input)?
                }
                Instr::I32TruncF64S { result, input } => {
                    self.execute_i32_trunc_f64_s(result, input)?
                }
                Instr::I32TruncF64U { result, input } => {
                    self.execute_i32_trunc_f64_u(result, input)?
                }
                Instr::I64TruncF32S { result, input } => {
                    self.execute_i64_trunc_f32_s(result, input)?
                }
                Instr::I64TruncF32U { result, input } => {
                    self.execute_i64_trunc_f32_u(result, input)?
                }
                Instr::I64TruncF64S { result, input } => {
                    self.execute_i64_trunc_f64_s(result, input)?
                }
                Instr::I64TruncF64U { result, input } => {
                    self.execute_i64_trunc_f64_u(result, input)?
                }
                Instr::I32TruncSatF32S { result, input } => {
                    self.execute_i32_trunc_sat_f32_s(result, input)
                }
                Instr::I32TruncSatF32U { result, input } => {
                    self.execute_i32_trunc_sat_f32_u(result, input)
                }
                Instr::I32TruncSatF64S { result, input } => {
                    self.execute_i32_trunc_sat_f64_s(result, input)
                }
                Instr::I32TruncSatF64U { result, input } => {
                    self.execute_i32_trunc_sat_f64_u(result, input)
                }
                Instr::I64TruncSatF32S { result, input } => {
                    self.execute_i64_trunc_sat_f32_s(result, input)
                }
                Instr::I64TruncSatF32U { result, input } => {
                    self.execute_i64_trunc_sat_f32_u(result, input)
                }
                Instr::I64TruncSatF64S { result, input } => {
                    self.execute_i64_trunc_sat_f64_s(result, input)
                }
                Instr::I64TruncSatF64U { result, input } => {
                    self.execute_i64_trunc_sat_f64_u(result, input)
                }
                Instr::F32DemoteF64 { result, input } => self.execute_f32_demote_f64(result, input),
                Instr::F64PromoteF32 { result, input } => {
                    self.execute_f64_promote_f32(result, input)
                }
                Instr::F32ConvertI32S { result, input } => {
                    self.execute_f32_convert_i32_s(result, input)
                }
                Instr::F32ConvertI32U { result, input } => {
                    self.execute_f32_convert_i32_u(result, input)
                }
                Instr::F32ConvertI64S { result, input } => {
                    self.execute_f32_convert_i64_s(result, input)
                }
                Instr::F32ConvertI64U { result, input } => {
                    self.execute_f32_convert_i64_u(result, input)
                }
                Instr::F64ConvertI32S { result, input } => {
                    self.execute_f64_convert_i32_s(result, input)
                }
                Instr::F64ConvertI32U { result, input } => {
                    self.execute_f64_convert_i32_u(result, input)
                }
                Instr::F64ConvertI64S { result, input } => {
                    self.execute_f64_convert_i64_s(result, input)
                }
                Instr::F64ConvertI64U { result, input } => {
                    self.execute_f64_convert_i64_u(result, input)
                }
                Instr::TableGet { result, index } => {
                    self.execute_table_get(store.inner(), result, index)?
                }
                Instr::TableGetImm { result, index } => {
                    self.execute_table_get_imm(store.inner(), result, index)?
                }
                Instr::TableSize { result, table } => {
                    self.execute_table_size(store.inner(), result, table)
                }
                Instr::TableSet { index, value } => {
                    self.execute_table_set(store.inner_mut(), index, value)?
                }
                Instr::TableSetAt { index, value } => {
                    self.execute_table_set_at(store.inner_mut(), index, value)?
                }
                Instr::TableCopy { dst, src, len } => {
                    self.execute_table_copy(store.inner_mut(), dst, src, len)?
                }
                Instr::TableInit { dst, src, len } => {
                    self.execute_table_init(store.inner_mut(), dst, src, len)?
                }
                Instr::TableFill { dst, len, value } => {
                    self.execute_table_fill(store.inner_mut(), dst, len, value)?
                }
                Instr::TableGrow {
                    result,
                    delta,
                    value,
                } => self.execute_table_grow(store, result, delta, value)?,
                Instr::ElemDrop { index } => self.execute_element_drop(store.inner_mut(), index),
                Instr::DataDrop { index } => self.execute_data_drop(store.inner_mut(), index),
                Instr::MemorySize { result, memory } => {
                    self.execute_memory_size(store.inner(), result, memory)
                }
                Instr::MemoryGrow { result, delta } => {
                    self.execute_memory_grow(store, result, delta)?
                }
                Instr::MemoryCopy { dst, src, len } => {
                    self.execute_memory_copy(store.inner_mut(), dst, src, len)?
                }
                Instr::MemoryFill { dst, value, len } => {
                    self.execute_memory_fill(store.inner_mut(), dst, value, len)?
                }
                Instr::MemoryFillImm { dst, value, len } => {
                    self.execute_memory_fill_imm(store.inner_mut(), dst, value, len)?
                }
                Instr::MemoryInit { dst, src, len } => {
                    self.execute_memory_init(store.inner_mut(), dst, src, len)?
                }
                Instr::TableIndex { .. }
                | Instr::MemoryIndex { .. }
                | Instr::DataIndex { .. }
                | Instr::ElemIndex { .. }
                | Instr::Const32 { .. }
                | Instr::I64Const32 { .. }
                | Instr::F64Const32 { .. }
                | Instr::BranchTableTarget { .. }
                | Instr::Register { .. }
                | Instr::Register2 { .. }
                | Instr::Register3 { .. }
                | Instr::RegisterAndImm32 { .. }
                | Instr::Imm16AndImm32 { .. }
                | Instr::RegisterSpan { .. }
                | Instr::RegisterList { .. }
                | Instr::CallIndirectParams { .. }
                | Instr::CallIndirectParamsImm16 { .. } => self.invalid_instruction_word()?,
                #[cfg(feature = "simd")]
                Instr::I8x16Splat { result, value } => self.execute_i8x16_splat(result, value),
                #[cfg(feature = "simd")]
                Instr::I16x8Splat { result, value } => self.execute_i16x8_splat(result, value),
                #[cfg(feature = "simd")]
                Instr::I32x4Splat { result, value } => self.execute_i32x4_splat(result, value),
                #[cfg(feature = "simd")]
                Instr::I64x2Splat { result, value } => self.execute_i64x2_splat(result, value),
                #[cfg(feature = "simd")]
                Instr::F32x4Splat { result, value } => self.execute_f32x4_splat(result, value),
                #[cfg(feature = "simd")]
                Instr::F64x2Splat { result, value } => self.execute_f64x2_splat(result, value),
                #[cfg(feature = "simd")]
                Instr::I8x16ExtractLaneS {
                    result,
                    value,
                    lane,
                } => self.i8x16_extract_lane_s(result, value, lane),
                #[cfg(feature = "simd")]
                Instr::I8x16ExtractLaneU {
                    result,
                    value,
                    lane,
                } => self.i8x16_extract_lane_u(result, value, lane),
                #[cfg(feature = "simd")]
                Instr::I16x8ExtractLaneS {
                    result,
                    value,
                    lane,
                } => self.i16x8_extract_lane_s(result, value, lane),
                #[cfg(feature = "simd")]
                Instr::I16x8ExtractLaneU {
                    result,
                    value,
                    lane,
                } => self.i16x8_extract_lane_u(result, value, lane),
                #[cfg(feature = "simd")]
                Instr::I32x4ExtractLane {
                    result,
                    value,
                    lane,
                } => self.i32x4_extract_lane(result, value, lane),
                #[cfg(feature = "simd")]
                Instr::I64x2ExtractLane {
                    result,
                    value,
                    lane,
                } => self.i64x2_extract_lane(result, value, lane),
                #[cfg(feature = "simd")]
                Instr::F32x4ExtractLane {
                    result,
                    value,
                    lane,
                } => self.f32x4_extract_lane(result, value, lane),
                #[cfg(feature = "simd")]
                Instr::F64x2ExtractLane {
                    result,
                    value,
                    lane,
                } => self.f64x2_extract_lane(result, value, lane),
                #[cfg(feature = "simd")]
                Instr::I8x16ReplaceLane {
                    result,
                    input,
                    lane,
                } => self.execute_i8x16_replace_lane(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::I8x16ReplaceLaneImm {
                    result,
                    input,
                    lane,
                    value,
                } => self.execute_i8x16_replace_lane_imm(result, input, lane, value),
                #[cfg(feature = "simd")]
                Instr::I16x8ReplaceLane {
                    result,
                    input,
                    lane,
                } => self.execute_i16x8_replace_lane(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::I16x8ReplaceLaneImm {
                    result,
                    input,
                    lane,
                } => self.execute_i16x8_replace_lane_imm(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::I32x4ReplaceLane {
                    result,
                    input,
                    lane,
                } => self.execute_i32x4_replace_lane(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::I32x4ReplaceLaneImm {
                    result,
                    input,
                    lane,
                } => self.execute_i32x4_replace_lane_imm(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::I64x2ReplaceLane {
                    result,
                    input,
                    lane,
                } => self.execute_i64x2_replace_lane(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::I64x2ReplaceLaneImm32 {
                    result,
                    input,
                    lane,
                } => self.execute_i64x2_replace_lane_imm32(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::F32x4ReplaceLane {
                    result,
                    input,
                    lane,
                } => self.execute_f32x4_replace_lane(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::F32x4ReplaceLaneImm {
                    result,
                    input,
                    lane,
                } => self.execute_f32x4_replace_lane_imm(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::F64x2ReplaceLane {
                    result,
                    input,
                    lane,
                } => self.execute_f64x2_replace_lane(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::F64x2ReplaceLaneImm32 {
                    result,
                    input,
                    lane,
                } => self.execute_f64x2_replace_lane_imm32(result, input, lane),
                #[cfg(feature = "simd")]
                Instr::I8x16Shuffle { result, lhs, rhs } => {
                    self.execute_i8x16_shuffle(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16Swizzle {
                    result,
                    input,
                    selector,
                } => self.execute_i8x16_swizzle(result, input, selector),
                #[cfg(feature = "simd")]
                Instr::I8x16Add { result, lhs, rhs } => self.execute_i8x16_add(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8Add { result, lhs, rhs } => self.execute_i16x8_add(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4Add { result, lhs, rhs } => self.execute_i32x4_add(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2Add { result, lhs, rhs } => self.execute_i64x2_add(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16Sub { result, lhs, rhs } => self.execute_i8x16_sub(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8Sub { result, lhs, rhs } => self.execute_i16x8_sub(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4Sub { result, lhs, rhs } => self.execute_i32x4_sub(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2Sub { result, lhs, rhs } => self.execute_i64x2_sub(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8Mul { result, lhs, rhs } => self.execute_i16x8_mul(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4Mul { result, lhs, rhs } => self.execute_i32x4_mul(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2Mul { result, lhs, rhs } => self.execute_i64x2_mul(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4DotI16x8S { result, lhs, rhs } => {
                    self.execute_i32x4_dot_i16x8_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16Neg { result, input } => self.execute_i8x16_neg(result, input),
                #[cfg(feature = "simd")]
                Instr::I16x8Neg { result, input } => self.execute_i16x8_neg(result, input),
                #[cfg(feature = "simd")]
                Instr::I32x4Neg { result, input } => self.execute_i32x4_neg(result, input),
                #[cfg(feature = "simd")]
                Instr::I64x2Neg { result, input } => self.execute_i64x2_neg(result, input),
                #[cfg(feature = "simd")]
                Instr::I16x8ExtmulLowI8x16S { result, lhs, rhs } => {
                    self.execute_i16x8_extmul_low_i8x16_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtmulHighI8x16S { result, lhs, rhs } => {
                    self.execute_i16x8_extmul_high_i8x16_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtmulLowI8x16U { result, lhs, rhs } => {
                    self.execute_i16x8_extmul_low_i8x16_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtmulHighI8x16U { result, lhs, rhs } => {
                    self.execute_i16x8_extmul_high_i8x16_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtmulLowI16x8S { result, lhs, rhs } => {
                    self.execute_i32x4_extmul_low_i16x8_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtmulHighI16x8S { result, lhs, rhs } => {
                    self.execute_i32x4_extmul_high_i16x8_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtmulLowI16x8U { result, lhs, rhs } => {
                    self.execute_i32x4_extmul_low_i16x8_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtmulHighI16x8U { result, lhs, rhs } => {
                    self.execute_i32x4_extmul_high_i16x8_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ExtmulLowI32x4S { result, lhs, rhs } => {
                    self.execute_i64x2_extmul_low_i32x4_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ExtmulHighI32x4S { result, lhs, rhs } => {
                    self.execute_i64x2_extmul_high_i32x4_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ExtmulLowI32x4U { result, lhs, rhs } => {
                    self.execute_i64x2_extmul_low_i32x4_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ExtmulHighI32x4U { result, lhs, rhs } => {
                    self.execute_i64x2_extmul_high_i32x4_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtaddPairwiseI8x16S { result, input } => {
                    self.execute_i16x8_extadd_pairwise_i8x16_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtaddPairwiseI8x16U { result, input } => {
                    self.execute_i16x8_extadd_pairwise_i8x16_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtaddPairwiseI16x8S { result, input } => {
                    self.execute_i32x4_extadd_pairwise_i16x8_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtaddPairwiseI16x8U { result, input } => {
                    self.execute_i32x4_extadd_pairwise_i16x8_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16AddSatS { result, lhs, rhs } => {
                    self.execute_i8x16_add_sat_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16AddSatU { result, lhs, rhs } => {
                    self.execute_i8x16_add_sat_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8AddSatS { result, lhs, rhs } => {
                    self.execute_i16x8_add_sat_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8AddSatU { result, lhs, rhs } => {
                    self.execute_i16x8_add_sat_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16SubSatS { result, lhs, rhs } => {
                    self.execute_i8x16_sub_sat_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16SubSatU { result, lhs, rhs } => {
                    self.execute_i8x16_sub_sat_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8SubSatS { result, lhs, rhs } => {
                    self.execute_i16x8_sub_sat_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8SubSatU { result, lhs, rhs } => {
                    self.execute_i16x8_sub_sat_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8Q15MulrSatS { result, lhs, rhs } => {
                    self.execute_i16x8_q15mulr_sat_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16MinS { result, lhs, rhs } => self.execute_i8x16_min_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16MinU { result, lhs, rhs } => self.execute_i8x16_min_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8MinS { result, lhs, rhs } => self.execute_i16x8_min_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8MinU { result, lhs, rhs } => self.execute_i16x8_min_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4MinS { result, lhs, rhs } => self.execute_i32x4_min_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4MinU { result, lhs, rhs } => self.execute_i32x4_min_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16MaxS { result, lhs, rhs } => self.execute_i8x16_max_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16MaxU { result, lhs, rhs } => self.execute_i8x16_max_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8MaxS { result, lhs, rhs } => self.execute_i16x8_max_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8MaxU { result, lhs, rhs } => self.execute_i16x8_max_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4MaxS { result, lhs, rhs } => self.execute_i32x4_max_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4MaxU { result, lhs, rhs } => self.execute_i32x4_max_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16AvgrU { result, lhs, rhs } => {
                    self.execute_i8x16_avgr_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8AvgrU { result, lhs, rhs } => {
                    self.execute_i16x8_avgr_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16Abs { result, input } => self.execute_i8x16_abs(result, input),
                #[cfg(feature = "simd")]
                Instr::I16x8Abs { result, input } => self.execute_i16x8_abs(result, input),
                #[cfg(feature = "simd")]
                Instr::I32x4Abs { result, input } => self.execute_i32x4_abs(result, input),
                #[cfg(feature = "simd")]
                Instr::I64x2Abs { result, input } => self.execute_i64x2_abs(result, input),
                #[cfg(feature = "simd")]
                Instr::I8x16Shl { result, lhs, rhs } => self.execute_i8x16_shl(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16ShlBy { result, lhs, rhs } => {
                    self.execute_i8x16_shl_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8Shl { result, lhs, rhs } => self.execute_i16x8_shl(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8ShlBy { result, lhs, rhs } => {
                    self.execute_i16x8_shl_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4Shl { result, lhs, rhs } => self.execute_i32x4_shl(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4ShlBy { result, lhs, rhs } => {
                    self.execute_i32x4_shl_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2Shl { result, lhs, rhs } => self.execute_i64x2_shl(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2ShlBy { result, lhs, rhs } => {
                    self.execute_i64x2_shl_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16ShrS { result, lhs, rhs } => self.execute_i8x16_shr_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16ShrSBy { result, lhs, rhs } => {
                    self.execute_i8x16_shr_s_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16ShrU { result, lhs, rhs } => self.execute_i8x16_shr_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16ShrUBy { result, lhs, rhs } => {
                    self.execute_i8x16_shr_u_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ShrS { result, lhs, rhs } => self.execute_i16x8_shr_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8ShrSBy { result, lhs, rhs } => {
                    self.execute_i16x8_shr_s_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ShrU { result, lhs, rhs } => self.execute_i16x8_shr_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8ShrUBy { result, lhs, rhs } => {
                    self.execute_i16x8_shr_u_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ShrS { result, lhs, rhs } => self.execute_i32x4_shr_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4ShrSBy { result, lhs, rhs } => {
                    self.execute_i32x4_shr_s_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ShrU { result, lhs, rhs } => self.execute_i32x4_shr_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4ShrUBy { result, lhs, rhs } => {
                    self.execute_i32x4_shr_u_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ShrS { result, lhs, rhs } => self.execute_i64x2_shr_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2ShrSBy { result, lhs, rhs } => {
                    self.execute_i64x2_shr_s_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ShrU { result, lhs, rhs } => self.execute_i64x2_shr_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2ShrUBy { result, lhs, rhs } => {
                    self.execute_i64x2_shr_u_by(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::V128And { result, lhs, rhs } => self.execute_v128_and(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::V128Or { result, lhs, rhs } => self.execute_v128_or(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::V128Xor { result, lhs, rhs } => self.execute_v128_xor(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::V128Andnot { result, lhs, rhs } => {
                    self.execute_v128_andnot(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::V128Not { result, input } => self.execute_v128_not(result, input),
                #[cfg(feature = "simd")]
                Instr::V128Bitselect { result, lhs, rhs } => {
                    self.execute_v128_bitselect(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16Popcnt { result, input } => self.execute_i8x16_popcnt(result, input),
                #[cfg(feature = "simd")]
                Instr::V128AnyTrue { result, input } => self.execute_v128_any_true(result, input),
                #[cfg(feature = "simd")]
                Instr::I8x16AllTrue { result, input } => self.execute_i8x16_all_true(result, input),
                #[cfg(feature = "simd")]
                Instr::I16x8AllTrue { result, input } => self.execute_i16x8_all_true(result, input),
                #[cfg(feature = "simd")]
                Instr::I32x4AllTrue { result, input } => self.execute_i32x4_all_true(result, input),
                #[cfg(feature = "simd")]
                Instr::I64x2AllTrue { result, input } => self.execute_i64x2_all_true(result, input),
                #[cfg(feature = "simd")]
                Instr::I8x16Bitmask { result, input } => self.execute_i8x16_bitmask(result, input),
                #[cfg(feature = "simd")]
                Instr::I16x8Bitmask { result, input } => self.execute_i16x8_bitmask(result, input),
                #[cfg(feature = "simd")]
                Instr::I32x4Bitmask { result, input } => self.execute_i32x4_bitmask(result, input),
                #[cfg(feature = "simd")]
                Instr::I64x2Bitmask { result, input } => self.execute_i64x2_bitmask(result, input),
                #[cfg(feature = "simd")]
                Instr::I8x16Eq { result, lhs, rhs } => self.execute_i8x16_eq(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8Eq { result, lhs, rhs } => self.execute_i16x8_eq(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4Eq { result, lhs, rhs } => self.execute_i32x4_eq(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2Eq { result, lhs, rhs } => self.execute_i64x2_eq(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Eq { result, lhs, rhs } => self.execute_f32x4_eq(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Eq { result, lhs, rhs } => self.execute_f64x2_eq(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16Ne { result, lhs, rhs } => self.execute_i8x16_ne(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8Ne { result, lhs, rhs } => self.execute_i16x8_ne(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4Ne { result, lhs, rhs } => self.execute_i32x4_ne(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2Ne { result, lhs, rhs } => self.execute_i64x2_ne(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Ne { result, lhs, rhs } => self.execute_f32x4_ne(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Ne { result, lhs, rhs } => self.execute_f64x2_ne(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16LtS { result, lhs, rhs } => self.execute_i8x16_lt_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16LtU { result, lhs, rhs } => self.execute_i8x16_lt_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8LtS { result, lhs, rhs } => self.execute_i16x8_lt_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8LtU { result, lhs, rhs } => self.execute_i16x8_lt_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4LtS { result, lhs, rhs } => self.execute_i32x4_lt_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4LtU { result, lhs, rhs } => self.execute_i32x4_lt_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2LtS { result, lhs, rhs } => self.execute_i64x2_lt_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Lt { result, lhs, rhs } => self.execute_f32x4_lt(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Lt { result, lhs, rhs } => self.execute_f64x2_lt(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16LeS { result, lhs, rhs } => self.execute_i8x16_le_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I8x16LeU { result, lhs, rhs } => self.execute_i8x16_le_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8LeS { result, lhs, rhs } => self.execute_i16x8_le_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I16x8LeU { result, lhs, rhs } => self.execute_i16x8_le_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4LeS { result, lhs, rhs } => self.execute_i32x4_le_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I32x4LeU { result, lhs, rhs } => self.execute_i32x4_le_u(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::I64x2LeS { result, lhs, rhs } => self.execute_i64x2_le_s(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Le { result, lhs, rhs } => self.execute_f32x4_le(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Le { result, lhs, rhs } => self.execute_f64x2_le(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Neg { result, input } => self.execute_f32x4_neg(result, input),
                #[cfg(feature = "simd")]
                Instr::F64x2Neg { result, input } => self.execute_f64x2_neg(result, input),
                #[cfg(feature = "simd")]
                Instr::F32x4Abs { result, input } => self.execute_f32x4_abs(result, input),
                #[cfg(feature = "simd")]
                Instr::F64x2Abs { result, input } => self.execute_f64x2_abs(result, input),
                #[cfg(feature = "simd")]
                Instr::F32x4Min { result, lhs, rhs } => self.execute_f32x4_min(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Min { result, lhs, rhs } => self.execute_f64x2_min(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Max { result, lhs, rhs } => self.execute_f32x4_max(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Max { result, lhs, rhs } => self.execute_f64x2_max(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Pmin { result, lhs, rhs } => self.execute_f32x4_pmin(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Pmin { result, lhs, rhs } => self.execute_f64x2_pmin(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Pmax { result, lhs, rhs } => self.execute_f32x4_pmax(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Pmax { result, lhs, rhs } => self.execute_f64x2_pmax(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Add { result, lhs, rhs } => self.execute_f32x4_add(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Add { result, lhs, rhs } => self.execute_f64x2_add(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Sub { result, lhs, rhs } => self.execute_f32x4_sub(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Sub { result, lhs, rhs } => self.execute_f64x2_sub(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Div { result, lhs, rhs } => self.execute_f32x4_div(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Div { result, lhs, rhs } => self.execute_f64x2_div(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Mul { result, lhs, rhs } => self.execute_f32x4_mul(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F64x2Mul { result, lhs, rhs } => self.execute_f64x2_mul(result, lhs, rhs),
                #[cfg(feature = "simd")]
                Instr::F32x4Sqrt { result, input } => self.execute_f32x4_sqrt(result, input),
                #[cfg(feature = "simd")]
                Instr::F64x2Sqrt { result, input } => self.execute_f64x2_sqrt(result, input),
                #[cfg(feature = "simd")]
                Instr::F32x4Ceil { result, input } => self.execute_f32x4_ceil(result, input),
                #[cfg(feature = "simd")]
                Instr::F64x2Ceil { result, input } => self.execute_f64x2_ceil(result, input),
                #[cfg(feature = "simd")]
                Instr::F32x4Floor { result, input } => self.execute_f32x4_floor(result, input),
                #[cfg(feature = "simd")]
                Instr::F64x2Floor { result, input } => self.execute_f64x2_floor(result, input),
                #[cfg(feature = "simd")]
                Instr::F32x4Trunc { result, input } => self.execute_f32x4_trunc(result, input),
                #[cfg(feature = "simd")]
                Instr::F64x2Trunc { result, input } => self.execute_f64x2_trunc(result, input),
                #[cfg(feature = "simd")]
                Instr::F32x4Nearest { result, input } => self.execute_f32x4_nearest(result, input),
                #[cfg(feature = "simd")]
                Instr::F64x2Nearest { result, input } => self.execute_f64x2_nearest(result, input),
                #[cfg(feature = "simd")]
                Instr::F32x4ConvertI32x4S { result, input } => {
                    self.execute_f32x4_convert_i32x4_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::F32x4ConvertI32x4U { result, input } => {
                    self.execute_f32x4_convert_i32x4_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::F64x2ConvertLowI32x4S { result, input } => {
                    self.execute_f64x2_convert_low_i32x4_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::F64x2ConvertLowI32x4U { result, input } => {
                    self.execute_f64x2_convert_low_i32x4_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4TruncSatF32x4S { result, input } => {
                    self.execute_i32x4_trunc_sat_f32x4_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4TruncSatF32x4U { result, input } => {
                    self.execute_i32x4_trunc_sat_f32x4_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4TruncSatF64x2SZero { result, input } => {
                    self.execute_i32x4_trunc_sat_f64x2_s_zero(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4TruncSatF64x2UZero { result, input } => {
                    self.execute_i32x4_trunc_sat_f64x2_u_zero(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::F32x4DemoteF64x2Zero { result, input } => {
                    self.execute_f32x4_demote_f64x2_zero(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::F64x2PromoteLowF32x4 { result, input } => {
                    self.execute_f64x2_promote_low_f32x4(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16NarrowI16x8S { result, lhs, rhs } => {
                    self.execute_i8x16_narrow_i16x8_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I8x16NarrowI16x8U { result, lhs, rhs } => {
                    self.execute_i8x16_narrow_i16x8_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8NarrowI32x4S { result, lhs, rhs } => {
                    self.execute_i16x8_narrow_i32x4_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8NarrowI32x4U { result, lhs, rhs } => {
                    self.execute_i16x8_narrow_i32x4_u(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtendLowI8x16S { result, input } => {
                    self.execute_i16x8_extend_low_i8x16_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtendHighI8x16S { result, input } => {
                    self.execute_i16x8_extend_high_i8x16_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtendLowI8x16U { result, input } => {
                    self.execute_i16x8_extend_low_i8x16_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I16x8ExtendHighI8x16U { result, input } => {
                    self.execute_i16x8_extend_high_i8x16_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtendLowI16x8S { result, input } => {
                    self.execute_i32x4_extend_low_i16x8_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtendHighI16x8S { result, input } => {
                    self.execute_i32x4_extend_high_i16x8_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtendLowI16x8U { result, input } => {
                    self.execute_i32x4_extend_low_i16x8_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4ExtendHighI16x8U { result, input } => {
                    self.execute_i32x4_extend_high_i16x8_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ExtendLowI32x4S { result, input } => {
                    self.execute_i64x2_extend_low_i32x4_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ExtendHighI32x4S { result, input } => {
                    self.execute_i64x2_extend_high_i32x4_s(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ExtendLowI32x4U { result, input } => {
                    self.execute_i64x2_extend_low_i32x4_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::I64x2ExtendHighI32x4U { result, input } => {
                    self.execute_i64x2_extend_high_i32x4_u(result, input)
                }
                #[cfg(feature = "simd")]
                Instr::V128Store { ptr, offset_lo } => {
                    self.execute_v128_store(store.inner_mut(), ptr, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128StoreOffset16 { ptr, value, offset } => {
                    self.execute_v128_store_offset16(ptr, offset, value)?
                }
                #[cfg(feature = "simd")]
                Instr::V128StoreAt { value, address } => {
                    self.execute_v128_store_at(store.inner_mut(), address, value)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Store8Lane { ptr, offset_lo } => {
                    self.execute_v128_store8_lane(store.inner_mut(), ptr, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Store8LaneOffset8 {
                    ptr,
                    value,
                    offset,
                    lane,
                } => self.execute_v128_store8_lane_offset8(ptr, value, offset, lane)?,
                #[cfg(feature = "simd")]
                Instr::V128Store8LaneAt { value, address } => {
                    self.execute_v128_store8_lane_at(store.inner_mut(), value, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Store16Lane { ptr, offset_lo } => {
                    self.execute_v128_store16_lane(store.inner_mut(), ptr, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Store16LaneOffset8 {
                    ptr,
                    value,
                    offset,
                    lane,
                } => self.execute_v128_store16_lane_offset8(ptr, value, offset, lane)?,
                #[cfg(feature = "simd")]
                Instr::V128Store16LaneAt { value, address } => {
                    self.execute_v128_store16_lane_at(store.inner_mut(), value, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Store32Lane { ptr, offset_lo } => {
                    self.execute_v128_store32_lane(store.inner_mut(), ptr, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Store32LaneOffset8 {
                    ptr,
                    value,
                    offset,
                    lane,
                } => self.execute_v128_store32_lane_offset8(ptr, value, offset, lane)?,
                #[cfg(feature = "simd")]
                Instr::V128Store32LaneAt { value, address } => {
                    self.execute_v128_store32_lane_at(store.inner_mut(), value, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Store64Lane { ptr, offset_lo } => {
                    self.execute_v128_store64_lane(store.inner_mut(), ptr, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Store64LaneOffset8 {
                    ptr,
                    value,
                    offset,
                    lane,
                } => self.execute_v128_store64_lane_offset8(ptr, value, offset, lane)?,
                #[cfg(feature = "simd")]
                Instr::V128Store64LaneAt { value, address } => {
                    self.execute_v128_store64_lane_at(store.inner_mut(), value, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load { result, offset_lo } => {
                    self.execute_v128_load(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128LoadAt { result, address } => {
                    self.execute_v128_load_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128LoadOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load32Zero { result, offset_lo } => {
                    self.execute_v128_load32_zero(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32ZeroAt { result, address } => {
                    self.execute_v128_load32_zero_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32ZeroOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load32_zero_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load64Zero { result, offset_lo } => {
                    self.execute_v128_load64_zero(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load64ZeroAt { result, address } => {
                    self.execute_v128_load64_zero_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load64ZeroOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load64_zero_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load8Splat { result, offset_lo } => {
                    self.execute_v128_load8_splat(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load8SplatAt { result, address } => {
                    self.execute_v128_load8_splat_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load8SplatOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load8_splat_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load16Splat { result, offset_lo } => {
                    self.execute_v128_load16_splat(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load16SplatAt { result, address } => {
                    self.execute_v128_load16_splat_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load16SplatOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load16_splat_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load32Splat { result, offset_lo } => {
                    self.execute_v128_load32_splat(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32SplatAt { result, address } => {
                    self.execute_v128_load32_splat_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32SplatOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load32_splat_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load64Splat { result, offset_lo } => {
                    self.execute_v128_load64_splat(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load64SplatAt { result, address } => {
                    self.execute_v128_load64_splat_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load64SplatOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load64_splat_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load8x8S { result, offset_lo } => {
                    self.execute_v128_load8x8_s(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load8x8SAt { result, address } => {
                    self.execute_v128_load8x8_s_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load8x8SOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load8x8_s_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load8x8U { result, offset_lo } => {
                    self.execute_v128_load8x8_u(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load8x8UAt { result, address } => {
                    self.execute_v128_load8x8_u_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load8x8UOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load8x8_u_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load16x4S { result, offset_lo } => {
                    self.execute_v128_load16x4_s(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load16x4SAt { result, address } => {
                    self.execute_v128_load16x4_s_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load16x4SOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load16x4_s_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load16x4U { result, offset_lo } => {
                    self.execute_v128_load16x4_u(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load16x4UAt { result, address } => {
                    self.execute_v128_load16x4_u_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load16x4UOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load16x4_u_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load32x2S { result, offset_lo } => {
                    self.execute_v128_load32x2_s(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32x2SAt { result, address } => {
                    self.execute_v128_load32x2_s_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32x2SOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load32x2_s_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load32x2U { result, offset_lo } => {
                    self.execute_v128_load32x2_u(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32x2UAt { result, address } => {
                    self.execute_v128_load32x2_u_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32x2UOffset16 {
                    result,
                    ptr,
                    offset,
                } => self.execute_v128_load32x2_u_offset16(result, ptr, offset)?,
                #[cfg(feature = "simd")]
                Instr::V128Load8Lane { result, offset_lo } => {
                    self.execute_v128_load8_lane(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load8LaneAt { result, address } => {
                    self.execute_v128_load8_lane_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load16Lane { result, offset_lo } => {
                    self.execute_v128_load16_lane(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load16LaneAt { result, address } => {
                    self.execute_v128_load16_lane_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32Lane { result, offset_lo } => {
                    self.execute_v128_load32_lane(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load32LaneAt { result, address } => {
                    self.execute_v128_load32_lane_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load64Lane { result, offset_lo } => {
                    self.execute_v128_load64_lane(store.inner(), result, offset_lo)?
                }
                #[cfg(feature = "simd")]
                Instr::V128Load64LaneAt { result, address } => {
                    self.execute_v128_load64_lane_at(store.inner(), result, address)?
                }
                #[cfg(feature = "simd")]
                Instr::I16x8RelaxedDotI8x16I7x16S { result, lhs, rhs } => {
                    self.execute_i16x8_relaxed_dot_i8x16_i7x16_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::I32x4RelaxedDotI8x16I7x16AddS { result, lhs, rhs } => {
                    self.execute_i32x4_relaxed_dot_i8x16_i7x16_add_s(result, lhs, rhs)
                }
                #[cfg(feature = "simd")]
                Instr::F32x4RelaxedMadd { result, a, b } => {
                    self.execute_f32x4_relaxed_madd(result, a, b)
                }
                #[cfg(feature = "simd")]
                Instr::F32x4RelaxedNmadd { result, a, b } => {
                    self.execute_f32x4_relaxed_nmadd(result, a, b)
                }
                #[cfg(feature = "simd")]
                Instr::F64x2RelaxedMadd { result, a, b } => {
                    self.execute_f64x2_relaxed_madd(result, a, b)
                }
                #[cfg(feature = "simd")]
                Instr::F64x2RelaxedNmadd { result, a, b } => {
                    self.execute_f64x2_relaxed_nmadd(result, a, b)
                }
                unsupported => panic!("encountered unsupported Wasmi instruction: {unsupported:?}"),
            }
        }
    }
}

macro_rules! get_entity {
    (
        $(
            fn $name:ident(&self, index: $index_ty:ty) -> $id_ty:ty;
        )*
    ) => {
        $(
            #[doc = ::core::concat!(
                "Returns the [`",
                ::core::stringify!($id_ty),
                "`] at `index` for the currently used [`Instance`].\n\n",
                "# Panics\n\n",
                "- If there is no [`",
                ::core::stringify!($id_ty),
                "`] at `index` for the currently used [`Instance`] in `store`."
            )]
            #[inline]
            fn $name(&self, index: $index_ty) -> $id_ty {
                unsafe { self.cache.$name(index) }
                    .unwrap_or_else(|| {
                        const ENTITY_NAME: &'static str = ::core::stringify!($id_ty);
                        // Safety: within the Wasmi executor it is assumed that store entity
                        //         indices within the Wasmi bytecode are always valid for the
                        //         store. This is an invariant of the Wasmi translation.
                        unsafe {
                            unreachable_unchecked!(
                                "missing {ENTITY_NAME} at index {index:?} for the currently used instance",
                            )
                        }
                    })
            }
        )*
    }
}

impl Executor<'_> {
    get_entity! {
        fn get_func(&self, index: index::Func) -> Func;
        fn get_func_type_dedup(&self, index: index::FuncType) -> DedupFuncType;
        fn get_memory(&self, index: index::Memory) -> Memory;
        fn get_table(&self, index: index::Table) -> Table;
        fn get_global(&self, index: index::Global) -> Global;
        fn get_data_segment(&self, index: index::Data) -> DataSegment;
        fn get_element_segment(&self, index: index::Elem) -> ElementSegment;
    }

    /// Returns the [`Reg`] value.
    fn get_register(&self, register: Reg) -> UntypedVal {
        // Safety: - It is the responsibility of the `Executor`
        //           implementation to keep the `sp` pointer valid
        //           whenever this method is accessed.
        //         - This is done by updating the `sp` pointer whenever
        //           the heap underlying the value stack is changed.
        unsafe { self.sp.get(register) }
    }

    /// Returns the [`Reg`] value.
    fn get_register_as<T>(&self, register: Reg) -> T
    where
        UntypedVal: ReadAs<T>,
    {
        // Safety: - It is the responsibility of the `Executor`
        //           implementation to keep the `sp` pointer valid
        //           whenever this method is accessed.
        //         - This is done by updating the `sp` pointer whenever
        //           the heap underlying the value stack is changed.
        unsafe { self.sp.read_as::<T>(register) }
    }

    /// Sets the [`Reg`] value to `value`.
    fn set_register(&mut self, register: Reg, value: impl Into<UntypedVal>) {
        // Safety: - It is the responsibility of the `Executor`
        //           implementation to keep the `sp` pointer valid
        //           whenever this method is accessed.
        //         - This is done by updating the `sp` pointer whenever
        //           the heap underlying the value stack is changed.
        unsafe { self.sp.set(register, value.into()) };
    }

    /// Sets the [`Reg`] value to `value`.
    fn set_register_as<T>(&mut self, register: Reg, value: T)
    where
        UntypedVal: WriteAs<T>,
    {
        // Safety: - It is the responsibility of the `Executor`
        //           implementation to keep the `sp` pointer valid
        //           whenever this method is accessed.
        //         - This is done by updating the `sp` pointer whenever
        //           the heap underlying the value stack is changed.
        unsafe { self.sp.write_as::<T>(register, value) };
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
    /// This is used by Wasmi instructions that have a fixed
    /// encoding size of two instruction words such as [`Op::Branch`].
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
    fn try_next_instr(&mut self) -> Result<(), Error> {
        self.try_next_instr_at(1)
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
    fn try_next_instr_at(&mut self, skip: usize) -> Result<(), Error> {
        self.next_instr_at(skip);
        Ok(())
    }

    /// Returns the [`FrameRegisters`] of the [`CallFrame`].
    fn frame_stack_ptr_impl(value_stack: &mut ValueStack, frame: &CallFrame) -> FrameRegisters {
        // Safety: We are using the frame's own base offset as input because it is
        //         guaranteed by the Wasm validation and translation phase to be
        //         valid for all register indices used by the associated function body.
        unsafe { value_stack.stack_ptr_at(frame.base_offset()) }
    }

    /// Initializes the [`Executor`] state for the [`CallFrame`].
    ///
    /// # Note
    ///
    /// The initialization of the [`Executor`] allows for efficient execution.
    fn init_call_frame(&mut self, frame: &CallFrame) {
        Self::init_call_frame_impl(&mut self.stack.values, &mut self.sp, &mut self.ip, frame)
    }

    /// Initializes the [`Executor`] state for the [`CallFrame`].
    ///
    /// # Note
    ///
    /// The initialization of the [`Executor`] allows for efficient execution.
    fn init_call_frame_impl(
        value_stack: &mut ValueStack,
        sp: &mut FrameRegisters,
        ip: &mut InstructionPtr,
        frame: &CallFrame,
    ) {
        *sp = Self::frame_stack_ptr_impl(value_stack, frame);
        *ip = frame.instr_ptr();
    }

    /// Executes a generic unary [`Op`].
    #[inline(always)]
    fn execute_unary<P, R>(&mut self, result: Reg, input: Reg, op: fn(P) -> R)
    where
        UntypedVal: ReadAs<P> + WriteAs<R>,
    {
        let value = self.get_register_as::<P>(input);
        self.set_register_as::<R>(result, op(value));
        self.next_instr();
    }

    /// Executes a fallible generic unary [`Op`].
    #[inline(always)]
    fn try_execute_unary<P, R>(
        &mut self,
        result: Reg,
        input: Reg,
        op: fn(P) -> Result<R, TrapCode>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<P> + WriteAs<R>,
    {
        let value = self.get_register_as::<P>(input);
        self.set_register_as::<R>(result, op(value)?);
        self.try_next_instr()
    }

    /// Executes a generic binary [`Op`].
    #[inline(always)]
    fn execute_binary<Lhs, Rhs, Result>(
        &mut self,
        result: Reg,
        lhs: Reg,
        rhs: Reg,
        op: fn(Lhs, Rhs) -> Result,
    ) where
        UntypedVal: ReadAs<Lhs> + ReadAs<Rhs> + WriteAs<Result>,
    {
        let lhs = self.get_register_as::<Lhs>(lhs);
        let rhs = self.get_register_as::<Rhs>(rhs);
        self.set_register_as::<Result>(result, op(lhs, rhs));
        self.next_instr();
    }

    /// Executes a generic binary [`Op`].
    #[inline(always)]
    fn execute_binary_imm16_rhs<Lhs, Rhs, T>(
        &mut self,
        result: Reg,
        lhs: Reg,
        rhs: Const16<Rhs>,
        op: fn(Lhs, Rhs) -> T,
    ) where
        Rhs: From<Const16<Rhs>>,
        UntypedVal: ReadAs<Lhs> + ReadAs<Rhs> + WriteAs<T>,
    {
        let lhs = self.get_register_as::<Lhs>(lhs);
        let rhs = Rhs::from(rhs);
        self.set_register_as::<T>(result, op(lhs, rhs));
        self.next_instr();
    }

    /// Executes a generic binary [`Op`] with reversed operands.
    #[inline(always)]
    fn execute_binary_imm16_lhs<Lhs, Rhs, T>(
        &mut self,
        result: Reg,
        lhs: Const16<Lhs>,
        rhs: Reg,
        op: fn(Lhs, Rhs) -> T,
    ) where
        Lhs: From<Const16<Lhs>>,
        UntypedVal: ReadAs<Rhs> + WriteAs<T>,
    {
        let lhs = Lhs::from(lhs);
        let rhs = self.get_register_as::<Rhs>(rhs);
        self.set_register_as::<T>(result, op(lhs, rhs));
        self.next_instr();
    }

    /// Executes a generic shift or rotate [`Op`].
    #[inline(always)]
    fn execute_shift_by<Lhs, Rhs, T>(
        &mut self,
        result: Reg,
        lhs: Reg,
        rhs: ShiftAmount<Rhs>,
        op: fn(Lhs, Rhs) -> T,
    ) where
        Rhs: From<ShiftAmount<Rhs>>,
        UntypedVal: ReadAs<Lhs> + ReadAs<Rhs> + WriteAs<T>,
    {
        let lhs = self.get_register_as::<Lhs>(lhs);
        let rhs = Rhs::from(rhs);
        self.set_register_as::<T>(result, op(lhs, rhs));
        self.next_instr();
    }

    /// Executes a fallible generic binary [`Op`].
    #[inline(always)]
    fn try_execute_binary<Lhs, Rhs, T>(
        &mut self,
        result: Reg,
        lhs: Reg,
        rhs: Reg,
        op: fn(Lhs, Rhs) -> Result<T, TrapCode>,
    ) -> Result<(), Error>
    where
        UntypedVal: ReadAs<Lhs> + ReadAs<Rhs> + WriteAs<T>,
    {
        let lhs = self.get_register_as::<Lhs>(lhs);
        let rhs = self.get_register_as::<Rhs>(rhs);
        self.set_register_as::<T>(result, op(lhs, rhs)?);
        self.try_next_instr()
    }

    /// Executes a fallible generic binary [`Op`].
    #[inline(always)]
    fn try_execute_divrem_imm16_rhs<Lhs, Rhs, T>(
        &mut self,
        result: Reg,
        lhs: Reg,
        rhs: Const16<Rhs>,
        op: fn(Lhs, Rhs) -> Result<T, Error>,
    ) -> Result<(), Error>
    where
        Rhs: From<Const16<Rhs>>,
        UntypedVal: ReadAs<Lhs> + WriteAs<T>,
    {
        let lhs = self.get_register_as::<Lhs>(lhs);
        let rhs = Rhs::from(rhs);
        self.set_register_as::<T>(result, op(lhs, rhs)?);
        self.try_next_instr()
    }

    /// Executes a fallible generic binary [`Op`].
    #[inline(always)]
    fn execute_divrem_imm16_rhs<Lhs, NonZeroT, T>(
        &mut self,
        result: Reg,
        lhs: Reg,
        rhs: Const16<NonZeroT>,
        op: fn(Lhs, NonZeroT) -> T,
    ) where
        NonZeroT: From<Const16<NonZeroT>>,
        UntypedVal: ReadAs<Lhs> + WriteAs<T>,
    {
        let lhs = self.get_register_as::<Lhs>(lhs);
        let rhs = <NonZeroT>::from(rhs);
        self.set_register_as::<T>(result, op(lhs, rhs));
        self.next_instr()
    }

    /// Executes a fallible generic binary [`Op`] with reversed operands.
    #[inline(always)]
    fn try_execute_binary_imm16_lhs<Lhs, Rhs, T>(
        &mut self,
        result: Reg,
        lhs: Const16<Lhs>,
        rhs: Reg,
        op: fn(Lhs, Rhs) -> Result<T, TrapCode>,
    ) -> Result<(), Error>
    where
        Lhs: From<Const16<Lhs>>,
        UntypedVal: ReadAs<Rhs> + WriteAs<T>,
    {
        let lhs = Lhs::from(lhs);
        let rhs = self.get_register_as::<Rhs>(rhs);
        self.set_register_as::<T>(result, op(lhs, rhs)?);
        self.try_next_instr()
    }

    /// Returns the optional `memory` parameter for a `load_at` [`Op`].
    ///
    /// # Note
    ///
    /// - Returns the default [`index::Memory`] if the parameter is missing.
    /// - Bumps `self.ip` if a [`Op::MemoryIndex`] parameter was found.
    #[inline(always)]
    fn fetch_optional_memory(&mut self, delta: usize) -> index::Memory {
        let mut addr: InstructionPtr = self.ip;
        addr.add(delta);
        match *addr.get() {
            Op::MemoryIndex { index } => {
                hint::cold();
                self.ip.add(1);
                index
            }
            _ => index::Memory::from(0),
        }
    }

    /// Fetches the [`Reg`] and [`Offset64Hi`] parameters for a load or store [`Op`].
    unsafe fn fetch_reg_and_offset_hi(&self) -> (Reg, Offset64Hi) {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match addr.get().filter_register_and_offset_hi() {
            Ok(value) => value,
            Err(instr) => unsafe {
                unreachable_unchecked!("expected an `Op::RegisterAndImm32` but found: {instr:?}")
            },
        }
    }
}

impl Executor<'_> {
    /// Used for all [`Op`] words that are not meant for execution.
    ///
    /// # Note
    ///
    /// This includes [`Op`] variants such as [`Op::TableIndex`]
    /// that primarily carry parameters for actually executable [`Op`].
    fn invalid_instruction_word(&mut self) -> Result<(), Error> {
        // Safety: Wasmi translation guarantees that branches are never taken to instruction parameters directly.
        unsafe {
            unreachable_unchecked!(
                "expected instruction but found instruction parameter: {:?}",
                *self.ip.get()
            )
        }
    }

    /// Executes a Wasm `unreachable` instruction.
    fn execute_trap(&mut self, trap_code: TrapCode) -> Result<(), Error> {
        Err(Error::from(trap_code))
    }

    /// Executes an [`Op::ConsumeFuel`].
    fn execute_consume_fuel(
        &mut self,
        store: &mut StoreInner,
        block_fuel: BlockFuel,
    ) -> Result<(), Error> {
        // We do not have to check if fuel metering is enabled since
        // [`Op::ConsumeFuel`] are only generated if fuel metering
        // is enabled to begin with.
        store
            .fuel_mut()
            .consume_fuel_unchecked(block_fuel.to_u64())?;
        self.try_next_instr()
    }

    /// Executes an [`Op::RefFunc`].
    fn execute_ref_func(&mut self, result: Reg, func_index: index::Func) {
        let func = self.get_func(func_index);
        let funcref = <Ref<Func>>::from(func);
        self.set_register(result, funcref);
        self.next_instr();
    }
}

/// Extension method for [`UntypedVal`] required by the [`Executor`].
trait UntypedValueExt: Sized {
    /// Executes a logical `i{32,64}.and` instruction.
    fn and(x: Self, y: Self) -> bool;

    /// Executes a logical `i{32,64}.or` instruction.
    fn or(x: Self, y: Self) -> bool;

    /// Executes a fused `i{32,64}.and` + `i{32,64}.eqz` instruction.
    fn nand(x: Self, y: Self) -> bool {
        !Self::and(x, y)
    }

    /// Executes a fused `i{32,64}.or` + `i{32,64}.eqz` instruction.
    fn nor(x: Self, y: Self) -> bool {
        !Self::or(x, y)
    }
}

impl UntypedValueExt for i32 {
    fn and(x: Self, y: Self) -> bool {
        wasm::i32_bitand(x, y) != 0
    }

    fn or(x: Self, y: Self) -> bool {
        wasm::i32_bitor(x, y) != 0
    }
}

impl UntypedValueExt for i64 {
    fn and(x: Self, y: Self) -> bool {
        wasm::i64_bitand(x, y) != 0
    }

    fn or(x: Self, y: Self) -> bool {
        wasm::i64_bitor(x, y) != 0
    }
}

/// Extension method for [`UntypedVal`] required by the [`Executor`].
trait UntypedValueCmpExt: Sized {
    fn not_le(lhs: Self, rhs: Self) -> bool;
    fn not_lt(lhs: Self, rhs: Self) -> bool;
}

impl UntypedValueCmpExt for f32 {
    fn not_le(x: Self, y: Self) -> bool {
        !wasm::f32_le(x, y)
    }

    fn not_lt(x: Self, y: Self) -> bool {
        !wasm::f32_lt(x, y)
    }
}

impl UntypedValueCmpExt for f64 {
    fn not_le(x: Self, y: Self) -> bool {
        !wasm::f64_le(x, y)
    }

    fn not_lt(x: Self, y: Self) -> bool {
        !wasm::f64_lt(x, y)
    }
}
