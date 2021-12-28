use super::{
    bytecode::{BrTable, FuncIdx, GlobalIdx, LocalIdx, Offset, SignatureIdx, VisitInstruction},
    AsContextMut,
    CallStack,
    DropKeep,
    EngineInner,
    ExecutionOutcome,
    FunctionExecutionOutcome,
    FunctionFrame,
    ResolvedFuncBody,
    Target,
    ValueStack,
};
use crate::{
    nan_preserving_float::{F32, F64},
    Trap,
    TrapKind,
};

/// State that is used during Wasm function execution.
#[derive(Debug)]
pub struct ExecutionContext<'engine, 'func> {
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: &'engine mut ValueStack,
    /// Stores the call stack of live function invocations.
    call_stack: &'engine mut CallStack,
    /// The function frame that is being executed.
    frame: &'func mut FunctionFrame,
    /// The resolved function body of the executed function frame.
    func_body: ResolvedFuncBody<'engine>,
}

impl<'engine, 'func> ExecutionContext<'engine, 'func> {
    /// Creates an execution context for the given [`FunctionFrame`].
    pub fn new(engine: &'engine mut EngineInner, frame: &'func mut FunctionFrame) -> Self {
        let resolved = engine.code_map.resolve(frame.func_body);
        frame.initialize(resolved, &mut engine.value_stack);
        Self {
            value_stack: &mut engine.value_stack,
            call_stack: &mut engine.call_stack,
            frame,
            func_body: resolved,
        }
    }

    pub fn execute_frame(
        &mut self,
        mut ctx: impl AsContextMut,
    ) -> Result<FunctionExecutionOutcome, Trap> {
        'outer: loop {
            let pc = self.frame.inst_ptr;
            let inst_context = InstructionExecutionContext::new(
                self.value_stack,
                self.call_stack,
                self.frame,
                ctx.as_context_mut(),
            );
            match self.func_body.visit(pc, inst_context)? {
                ExecutionOutcome::Continue => {}
                ExecutionOutcome::Branch(target) => {
                    self.value_stack.drop_keep(target.drop_keep());
                }
                ExecutionOutcome::ExecuteCall(func) => {
                    return Ok(FunctionExecutionOutcome::NestedCall(func));
                }
                ExecutionOutcome::Return(drop_keep) => {
                    self.value_stack.drop_keep(drop_keep);
                    break 'outer;
                }
            }
        }
        Ok(FunctionExecutionOutcome::Return)
    }
}

/// An execution context for executing a single `wasmi` bytecode instruction.
#[derive(Debug)]
struct InstructionExecutionContext<'engine, 'func, Ctx> {
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: &'engine mut ValueStack,
    /// Stores the call stack of live function invocations.
    call_stack: &'engine mut CallStack,
    /// The function frame that is being executed.
    frame: &'func mut FunctionFrame,
    /// A mutable [`Store`] context.
    ctx: Ctx,
}

impl<'engine, 'func, Ctx> InstructionExecutionContext<'engine, 'func, Ctx>
where
    Ctx: AsContextMut,
{
    /// Creates a new [`InstructionExecutionContext`] for executing a single `wasmi` bytecode instruction.
    pub fn new(
        value_stack: &'engine mut ValueStack,
        call_stack: &'engine mut CallStack,
        frame: &'func mut FunctionFrame,
        ctx: Ctx,
    ) -> Self {
        Self {
            value_stack,
            call_stack,
            frame,
            ctx,
        }
    }
}

#[rustfmt::skip]
impl<'engine, 'func, Ctx> VisitInstruction for InstructionExecutionContext<'engine, 'func, Ctx>
where
    Ctx: AsContextMut,
{
    type Outcome = Result<ExecutionOutcome, TrapKind>;

    fn visit_br(&mut self, _target: Target) -> Self::Outcome { todo!() }
    fn visit_br_if_eqz(&mut self, _target: Target) -> Self::Outcome { todo!() }
    fn visit_br_if_nez(&mut self, _target: Target) -> Self::Outcome { todo!() }
    fn visit_br_table(&mut self, _br_table: BrTable) -> Self::Outcome { todo!() }
    fn visit_ret(&mut self, _drop_keep: DropKeep) -> Self::Outcome { todo!() }
    fn visit_get_local(&mut self, _local_depth: LocalIdx) -> Self::Outcome { todo!() }
    fn visit_set_local(&mut self, _local_depth: LocalIdx) -> Self::Outcome { todo!() }
    fn visit_tee_local(&mut self, _local_depth: LocalIdx) -> Self::Outcome { todo!() }
    fn visit_get_global(&mut self, _global_idx: GlobalIdx) -> Self::Outcome { todo!() }
    fn visit_set_global(&mut self, _global_idx: GlobalIdx) -> Self::Outcome { todo!() }
    fn visit_call(&mut self, _func: FuncIdx) -> Self::Outcome { todo!() }
    fn visit_call_indirect(&mut self, _signature: SignatureIdx) -> Self::Outcome { todo!() }
    fn visit_i32_const(&mut self, _value: i32) -> Self::Outcome { todo!() }
    fn visit_i64_const(&mut self, _value: i64) -> Self::Outcome { todo!() }
    fn visit_f32_const(&mut self, _value: F32) -> Self::Outcome { todo!() }
    fn visit_f64_const(&mut self, _value: F64) -> Self::Outcome { todo!() }
    fn visit_unreachable(&mut self) -> Self::Outcome {
        Err(TrapKind::Unreachable)
    }
    fn visit_drop(&mut self) -> Self::Outcome { todo!() }
    fn visit_select(&mut self) -> Self::Outcome { todo!() }
    fn visit_current_memory(&mut self) -> Self::Outcome { todo!() }
    fn visit_grow_memory(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_load(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_load(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_f32_load(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_f64_load(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i32_load_i8(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i32_load_u8(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i32_load_i16(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i32_load_u16(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_load_i8(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_load_u8(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_load_i16(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_load_u16(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_load_i32(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_load_u32(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i32_store(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_store(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_f32_store(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_f64_store(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i32_store_8(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i32_store_16(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_store_8(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_store_16(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i64_store_32(&mut self, _offset: Offset) -> Self::Outcome { todo!() }
    fn visit_i32_eqz(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_eq(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_ne(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_lt_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_lt_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_gt_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_gt_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_le_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_le_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_ge_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_ge_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_eqz(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_eq(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_ne(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_lt_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_lt_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_gt_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_gt_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_le_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_le_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_ge_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_ge_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_eq(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_ne(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_lt(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_gt(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_le(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_ge(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_eq(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_ne(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_lt(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_gt(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_le(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_ge(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_clz(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_ctz(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_popcnt(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_add(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_sub(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_mul(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_div_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_div_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_rem_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_rem_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_and(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_or(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_xor(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_shl(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_shr_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_shr_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_rotl(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_rotr(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_clz(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_ctz(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_popcnt(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_add(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_sub(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_mul(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_div_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_div_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_rem_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_rem_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_and(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_or(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_xor(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_shl(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_shr_s(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_shr_u(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_rotl(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_rotr(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_abs(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_neg(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_ceil(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_floor(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_trunc(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_nearest(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_sqrt(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_add(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_sub(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_mul(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_div(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_min(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_max(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_copysign(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_abs(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_neg(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_ceil(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_floor(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_trunc(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_nearest(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_sqrt(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_add(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_sub(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_mul(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_div(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_min(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_max(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_copysign(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_wrap_i64(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_trunc_f32(&mut self) -> Self::Outcome { todo!() }
    fn visit_u32_trunc_f32(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_trunc_f64(&mut self) -> Self::Outcome { todo!() }
    fn visit_u32_trunc_f64(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_extend_i32(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_extend_u32(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_trunc_f32(&mut self) -> Self::Outcome { todo!() }
    fn visit_u64_trunc_f32(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_trunc_f64(&mut self) -> Self::Outcome { todo!() }
    fn visit_u64_trunc_f64(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_convert_i32(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_convert_u32(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_convert_i64(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_convert_u64(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_demote_f64(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_convert_i32(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_convert_u32(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_convert_i64(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_convert_u64(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_promote_f32(&mut self) -> Self::Outcome { todo!() }
    fn visit_i32_reinterpret_f32(&mut self) -> Self::Outcome { todo!() }
    fn visit_i64_reinterpret_f64(&mut self) -> Self::Outcome { todo!() }
    fn visit_f32_reinterpret_i32(&mut self) -> Self::Outcome { todo!() }
    fn visit_f64_reinterpret_i64(&mut self) -> Self::Outcome { todo!() }
}
