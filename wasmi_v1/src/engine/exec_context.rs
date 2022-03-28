use super::{
    super::{Global, Memory, Table},
    bytecode::{BrTable, FuncIdx, GlobalIdx, Instruction, LocalIdx, Offset, SignatureIdx},
    AsContextMut,
    DropKeep,
    EngineInner,
    FunctionExecutionOutcome,
    FunctionFrame,
    ResolvedFuncBody,
    Target,
    ValueStack,
    VisitInstruction,
};
use crate::{
    core::{Trap, TrapCode, F32, F64},
    Func,
};
use wasmi_core::{memory_units::Pages, ExtendInto, LittleEndianConvert, UntypedValue, WrapInto};

/// The outcome of a `wasmi` instruction execution.
///
/// # Note
///
/// This signals to the `wasmi` interpreter what to do after the
/// instruction has been successfully executed.
#[derive(Debug, Copy, Clone)]
pub enum ExecutionOutcome {
    /// Continue with next instruction.
    Continue,
    /// Branch to an instruction at the given position.
    Branch(Target),
    /// Execute function call.
    ExecuteCall(Func),
    /// Return from current function block.
    Return(DropKeep),
}

/// State that is used during Wasm function execution.
#[derive(Debug)]
pub struct ExecutionContext<'engine, 'func> {
    /// Stores the value stack of live values on the Wasm stack.
    value_stack: &'engine mut ValueStack,
    /// The function frame that is being executed.
    frame: &'func mut FunctionFrame,
    /// The resolved function body of the executed function frame.
    func_body: ResolvedFuncBody<'engine>,
}

impl<'engine, 'func> ExecutionContext<'engine, 'func> {
    /// Creates an execution context for the given [`FunctionFrame`].
    pub fn new(
        engine: &'engine mut EngineInner,
        frame: &'func mut FunctionFrame,
    ) -> Result<Self, Trap> {
        let resolved = engine.code_map.resolve(frame.func_body);
        frame.initialize(resolved, &mut engine.value_stack)?;
        Ok(Self {
            value_stack: &mut engine.value_stack,
            frame,
            func_body: resolved,
        })
    }

    /// Executes the current function frame.
    ///
    /// # Note
    ///
    /// This executes instructions sequentially until either the function
    /// calls into another function or the function returns to its caller.
    #[inline(always)]
    pub fn execute_frame(
        self,
        mut ctx: impl AsContextMut,
    ) -> Result<FunctionExecutionOutcome, Trap> {
        'outer: loop {
            let pc = self.frame.inst_ptr;
            let inst_context =
                InstructionExecutionContext::new(self.value_stack, self.frame, &mut ctx);
            match self.func_body.visit(pc, inst_context)? {
                ExecutionOutcome::Continue => {
                    // Advance instruction pointer.
                    self.frame.inst_ptr += 1;
                }
                ExecutionOutcome::Branch(target) => {
                    self.value_stack.drop_keep(target.drop_keep());
                    // Set instruction pointer to the branch target.
                    self.frame.inst_ptr = target.destination_pc().into_usize();
                }
                ExecutionOutcome::ExecuteCall(func) => {
                    // Advance instruction pointer.
                    self.frame.inst_ptr += 1;
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
    /// The function frame that is being executed.
    frame: &'func mut FunctionFrame,
    /// A mutable [`Store`] context.
    ///
    /// [`Store`]: [`crate::v1::Store`]
    ctx: Ctx,
}

impl<'engine, 'func, Ctx> InstructionExecutionContext<'engine, 'func, Ctx>
where
    Ctx: AsContextMut,
{
    /// Creates a new [`InstructionExecutionContext`] for executing a single `wasmi` bytecode instruction.
    pub fn new(
        value_stack: &'engine mut ValueStack,
        frame: &'func mut FunctionFrame,
        ctx: Ctx,
    ) -> Self {
        Self {
            value_stack,
            frame,
            ctx,
        }
    }

    /// Returns the default linear memory.
    ///
    /// # Panics
    ///
    /// If there is no default linear memory.
    fn default_memory(&mut self) -> Memory {
        self.frame.default_memory(self.ctx.as_context())
    }

    /// Returns the default linear memory.
    ///
    /// # Panics
    ///
    /// If there is no default linear memory.
    fn default_table(&mut self) -> Table {
        self.frame.default_table(self.ctx.as_context())
    }

    /// Returns the global variable at the given index.
    ///
    /// # Panics
    ///
    /// If there is no global variable at the given index.
    fn global(&self, global_index: GlobalIdx) -> Global {
        self.frame
            .instance
            .get_global(self.ctx.as_context(), global_index.into_inner())
            .unwrap_or_else(|| panic!("missing global at index {:?}", global_index))
    }

    /// Returns the local depth as `usize`.
    fn convert_local_depth(local_depth: LocalIdx) -> usize {
        // TODO: calculate the -1 offset at module compilation time.
        (local_depth.into_inner() - 1) as usize
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
    fn execute_load<T>(&mut self, offset: Offset) -> Result<ExecutionOutcome, Trap>
    where
        UntypedValue: From<T>,
        T: LittleEndianConvert,
    {
        let memory = self.default_memory();
        let entry = self.value_stack.last_mut();
        let raw_address = u32::from(*entry);
        let address = Self::effective_address(offset, raw_address)?;
        let mut bytes = <<T as LittleEndianConvert>::Bytes as Default>::default();
        memory
            .read(self.ctx.as_context(), address, bytes.as_mut())
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        let value = <T as LittleEndianConvert>::from_le_bytes(bytes);
        *entry = value.into();
        Ok(ExecutionOutcome::Continue)
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
    fn execute_load_extend<T, U>(&mut self, offset: Offset) -> Result<ExecutionOutcome, Trap>
    where
        T: ExtendInto<U> + LittleEndianConvert,
        UntypedValue: From<U>,
    {
        let memory = self.default_memory();
        let entry = self.value_stack.last_mut();
        let raw_address = u32::from(*entry);
        let address = Self::effective_address(offset, raw_address)?;
        let mut bytes = <<T as LittleEndianConvert>::Bytes as Default>::default();
        memory
            .read(self.ctx.as_context(), address, bytes.as_mut())
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        let extended = <T as LittleEndianConvert>::from_le_bytes(bytes).extend_into();
        *entry = extended.into();
        Ok(ExecutionOutcome::Continue)
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
    fn execute_store<T>(&mut self, offset: Offset) -> Result<ExecutionOutcome, Trap>
    where
        T: LittleEndianConvert + From<UntypedValue>,
    {
        let stack_value = self.value_stack.pop_as::<T>();
        let raw_address = self.value_stack.pop_as::<u32>();
        let address = Self::effective_address(offset, raw_address)?;
        let memory = self.default_memory();
        let bytes = <T as LittleEndianConvert>::into_le_bytes(stack_value);
        memory
            .write(self.ctx.as_context_mut(), address, bytes.as_ref())
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        Ok(ExecutionOutcome::Continue)
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
    fn execute_store_wrap<T, U>(&mut self, offset: Offset) -> Result<ExecutionOutcome, Trap>
    where
        T: WrapInto<U> + From<UntypedValue>,
        U: LittleEndianConvert,
    {
        let wrapped_value = self.value_stack.pop_as::<T>().wrap_into();
        let raw_address = self.value_stack.pop_as::<u32>();
        let address = Self::effective_address(offset, raw_address)?;
        let memory = self.default_memory();
        let bytes = <U as LittleEndianConvert>::into_le_bytes(wrapped_value);
        memory
            .write(self.ctx.as_context_mut(), address, bytes.as_ref())
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        Ok(ExecutionOutcome::Continue)
    }

    fn execute_unary(
        &mut self,
        f: fn(UntypedValue) -> UntypedValue,
    ) -> Result<ExecutionOutcome, Trap> {
        let entry = self.value_stack.last_mut();
        *entry = f(*entry);
        Ok(ExecutionOutcome::Continue)
    }

    fn try_execute_unary(
        &mut self,
        f: fn(UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<ExecutionOutcome, Trap> {
        let entry = self.value_stack.last_mut();
        *entry = f(*entry)?;
        Ok(ExecutionOutcome::Continue)
    }

    fn execute_binary(
        &mut self,
        f: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> Result<ExecutionOutcome, Trap> {
        let right = self.value_stack.pop();
        let entry = self.value_stack.last_mut();
        let left = *entry;
        *entry = f(left, right);
        Ok(ExecutionOutcome::Continue)
    }

    fn try_execute_binary(
        &mut self,
        f: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<ExecutionOutcome, Trap> {
        let right = self.value_stack.pop();
        let entry = self.value_stack.last_mut();
        let left = *entry;
        *entry = f(left, right)?;
        Ok(ExecutionOutcome::Continue)
    }

    fn execute_reinterpret<T, U>(&mut self) -> Result<ExecutionOutcome, Trap>
    where
        UntypedValue: From<U>,
        T: From<UntypedValue>,
    {
        // Nothing to do for `wasmi` bytecode.
        Ok(ExecutionOutcome::Continue)
    }
}

impl<'engine, 'func, Ctx> VisitInstruction for InstructionExecutionContext<'engine, 'func, Ctx>
where
    Ctx: AsContextMut,
{
    type Outcome = Result<ExecutionOutcome, Trap>;

    fn visit_unreachable(&mut self) -> Self::Outcome {
        Err(TrapCode::Unreachable).map_err(Into::into)
    }

    fn visit_br(&mut self, target: Target) -> Self::Outcome {
        Ok(ExecutionOutcome::Branch(target))
    }

    fn visit_br_if_eqz(&mut self, target: Target) -> Self::Outcome {
        let condition = self.value_stack.pop_as();
        if condition {
            Ok(ExecutionOutcome::Continue)
        } else {
            Ok(ExecutionOutcome::Branch(target))
        }
    }

    fn visit_br_if_nez(&mut self, target: Target) -> Self::Outcome {
        let condition = self.value_stack.pop_as();
        if condition {
            Ok(ExecutionOutcome::Branch(target))
        } else {
            Ok(ExecutionOutcome::Continue)
        }
    }

    fn visit_return_if_nez(&mut self, drop_keep: DropKeep) -> Self::Outcome {
        let condition = self.value_stack.pop_as();
        if condition {
            Ok(ExecutionOutcome::Return(drop_keep))
        } else {
            Ok(ExecutionOutcome::Continue)
        }
    }

    fn visit_br_table(&mut self, br_table: BrTable) -> Self::Outcome {
        let index: u32 = self.value_stack.pop_as();
        match br_table.branch_or_default(index as usize) {
            Instruction::Br(target) => Ok(ExecutionOutcome::Branch(*target)),
            Instruction::Return(drop_keep) => Ok(ExecutionOutcome::Return(*drop_keep)),
            unexpected => panic!(
                "encountered unexpected `br_table` branch arm: {:?}",
                unexpected
            ),
        }
    }

    fn visit_ret(&mut self, drop_keep: DropKeep) -> Self::Outcome {
        Ok(ExecutionOutcome::Return(drop_keep))
    }

    fn visit_get_local(&mut self, local_depth: LocalIdx) -> Self::Outcome {
        let local_depth = Self::convert_local_depth(local_depth);
        let value = self.value_stack.peek(local_depth);
        self.value_stack.push(value);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_set_local(&mut self, local_depth: LocalIdx) -> Self::Outcome {
        let local_depth = Self::convert_local_depth(local_depth);
        let new_value = self.value_stack.pop();
        *self.value_stack.peek_mut(local_depth) = new_value;
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_tee_local(&mut self, local_depth: LocalIdx) -> Self::Outcome {
        let local_depth = Self::convert_local_depth(local_depth);
        let new_value = self.value_stack.last();
        *self.value_stack.peek_mut(local_depth) = new_value;
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_get_global(&mut self, global_index: GlobalIdx) -> Self::Outcome {
        let global_value = self.global(global_index).get(self.ctx.as_context());
        self.value_stack.push(global_value);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_set_global(&mut self, global_index: GlobalIdx) -> Self::Outcome {
        let global = self.global(global_index);
        let new_value = self
            .value_stack
            .pop()
            .with_type(global.value_type(self.ctx.as_context()));
        global
            .set(self.ctx.as_context_mut(), new_value)
            .unwrap_or_else(|error| panic!("encountered type mismatch upon global_set: {}", error));
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_call(&mut self, func_index: FuncIdx) -> Self::Outcome {
        let func = self
            .frame
            .instance
            .get_func(self.ctx.as_context_mut(), func_index.into_inner())
            .unwrap_or_else(|| panic!("missing function at index {:?}", func_index));
        Ok(ExecutionOutcome::ExecuteCall(func))
    }

    fn visit_call_indirect(&mut self, signature_index: SignatureIdx) -> Self::Outcome {
        let func_index: u32 = self.value_stack.pop_as();
        let table = self.default_table();
        let func = table
            .get(self.ctx.as_context(), func_index as usize)
            .map_err(|_| TrapCode::TableAccessOutOfBounds)?
            .ok_or(TrapCode::ElemUninitialized)?;
        let actual_signature = func.signature(self.ctx.as_context());
        let expected_signature = self
            .frame
            .instance
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
        Ok(ExecutionOutcome::ExecuteCall(func))
    }

    fn visit_const(&mut self, bytes: UntypedValue) -> Self::Outcome {
        self.value_stack.push(bytes);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_drop(&mut self) -> Self::Outcome {
        let _ = self.value_stack.pop();
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_select(&mut self) -> Self::Outcome {
        self.value_stack.pop2_eval(|e1, e2, e3| {
            let condition = <bool as From<UntypedValue>>::from(e3);
            let result = if condition { *e1 } else { e2 };
            *e1 = result;
        });
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_current_memory(&mut self) -> Self::Outcome {
        let memory = self.default_memory();
        let result = memory.current_pages(self.ctx.as_context()).0 as u32;
        self.value_stack.push(result);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_grow_memory(&mut self) -> Self::Outcome {
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
        self.value_stack.push(new_size);
        Ok(ExecutionOutcome::Continue)
    }

    fn visit_i32_load(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load::<i32>(offset)
    }

    fn visit_i64_load(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load::<i64>(offset)
    }

    fn visit_f32_load(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load::<F32>(offset)
    }

    fn visit_f64_load(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load::<F64>(offset)
    }

    fn visit_i32_load_i8(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<i8, i32>(offset)
    }

    fn visit_i32_load_u8(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<u8, i32>(offset)
    }

    fn visit_i32_load_i16(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<i16, i32>(offset)
    }

    fn visit_i32_load_u16(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<u16, i32>(offset)
    }

    fn visit_i64_load_i8(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<i8, i64>(offset)
    }

    fn visit_i64_load_u8(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<u8, i64>(offset)
    }

    fn visit_i64_load_i16(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<i16, i64>(offset)
    }

    fn visit_i64_load_u16(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<u16, i64>(offset)
    }

    fn visit_i64_load_i32(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<i32, i64>(offset)
    }

    fn visit_i64_load_u32(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_load_extend::<u32, i64>(offset)
    }

    fn visit_i32_store(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store::<i32>(offset)
    }

    fn visit_i64_store(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store::<i64>(offset)
    }

    fn visit_f32_store(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store::<F32>(offset)
    }

    fn visit_f64_store(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store::<F64>(offset)
    }

    fn visit_i32_store_8(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store_wrap::<i32, i8>(offset)
    }

    fn visit_i32_store_16(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store_wrap::<i32, i16>(offset)
    }

    fn visit_i64_store_8(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store_wrap::<i64, i8>(offset)
    }

    fn visit_i64_store_16(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store_wrap::<i64, i16>(offset)
    }

    fn visit_i64_store_32(&mut self, offset: Offset) -> Self::Outcome {
        self.execute_store_wrap::<i64, i32>(offset)
    }

    fn visit_i32_eqz(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_eqz)
    }

    fn visit_i32_eq(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_eq)
    }

    fn visit_i32_ne(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_ne)
    }

    fn visit_i32_lt_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_lt_s)
    }

    fn visit_i32_lt_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_lt_u)
    }

    fn visit_i32_gt_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_gt_s)
    }

    fn visit_i32_gt_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_gt_u)
    }

    fn visit_i32_le_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_le_s)
    }

    fn visit_i32_le_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_le_u)
    }

    fn visit_i32_ge_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_ge_s)
    }

    fn visit_i32_ge_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_ge_u)
    }

    fn visit_i64_eqz(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_eqz)
    }

    fn visit_i64_eq(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_eq)
    }

    fn visit_i64_ne(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_ne)
    }

    fn visit_i64_lt_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_lt_s)
    }

    fn visit_i64_lt_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_lt_u)
    }

    fn visit_i64_gt_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_gt_s)
    }

    fn visit_i64_gt_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_gt_u)
    }

    fn visit_i64_le_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_le_s)
    }

    fn visit_i64_le_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_le_u)
    }

    fn visit_i64_ge_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_ge_s)
    }

    fn visit_i64_ge_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_ge_u)
    }

    fn visit_f32_eq(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_eq)
    }

    fn visit_f32_ne(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_ne)
    }

    fn visit_f32_lt(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_lt)
    }

    fn visit_f32_gt(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_gt)
    }

    fn visit_f32_le(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_le)
    }

    fn visit_f32_ge(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_ge)
    }

    fn visit_f64_eq(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_eq)
    }

    fn visit_f64_ne(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_ne)
    }

    fn visit_f64_lt(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_lt)
    }

    fn visit_f64_gt(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_gt)
    }

    fn visit_f64_le(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_le)
    }

    fn visit_f64_ge(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_ge)
    }

    fn visit_i32_clz(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_clz)
    }

    fn visit_i32_ctz(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_ctz)
    }

    fn visit_i32_popcnt(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_popcnt)
    }

    fn visit_i32_add(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_add)
    }

    fn visit_i32_sub(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_sub)
    }

    fn visit_i32_mul(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_mul)
    }

    fn visit_i32_div_s(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::i32_div_s)
    }

    fn visit_i32_div_u(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::i32_div_u)
    }

    fn visit_i32_rem_s(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::i32_rem_s)
    }

    fn visit_i32_rem_u(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::i32_rem_u)
    }

    fn visit_i32_and(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_and)
    }

    fn visit_i32_or(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_or)
    }

    fn visit_i32_xor(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_xor)
    }

    fn visit_i32_shl(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_shl)
    }

    fn visit_i32_shr_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_shr_s)
    }

    fn visit_i32_shr_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_shr_u)
    }

    fn visit_i32_rotl(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_rotl)
    }

    fn visit_i32_rotr(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i32_rotr)
    }

    fn visit_i64_clz(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_clz)
    }

    fn visit_i64_ctz(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_ctz)
    }

    fn visit_i64_popcnt(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_popcnt)
    }

    fn visit_i64_add(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_add)
    }

    fn visit_i64_sub(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_sub)
    }

    fn visit_i64_mul(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_mul)
    }

    fn visit_i64_div_s(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::i64_div_s)
    }

    fn visit_i64_div_u(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::i64_div_u)
    }

    fn visit_i64_rem_s(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::i64_rem_s)
    }

    fn visit_i64_rem_u(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::i64_rem_u)
    }

    fn visit_i64_and(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_and)
    }

    fn visit_i64_or(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_or)
    }

    fn visit_i64_xor(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_xor)
    }

    fn visit_i64_shl(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_shl)
    }

    fn visit_i64_shr_s(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_shr_s)
    }

    fn visit_i64_shr_u(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_shr_u)
    }

    fn visit_i64_rotl(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_rotl)
    }

    fn visit_i64_rotr(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::i64_rotr)
    }

    fn visit_f32_abs(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_abs)
    }

    fn visit_f32_neg(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_neg)
    }

    fn visit_f32_ceil(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_ceil)
    }

    fn visit_f32_floor(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_floor)
    }

    fn visit_f32_trunc(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_trunc)
    }

    fn visit_f32_nearest(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_nearest)
    }

    fn visit_f32_sqrt(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_sqrt)
    }

    fn visit_f32_add(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_add)
    }

    fn visit_f32_sub(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_sub)
    }

    fn visit_f32_mul(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_mul)
    }

    fn visit_f32_div(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::f32_div)
    }

    fn visit_f32_min(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_min)
    }

    fn visit_f32_max(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_max)
    }

    fn visit_f32_copysign(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f32_copysign)
    }

    fn visit_f64_abs(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_abs)
    }

    fn visit_f64_neg(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_neg)
    }

    fn visit_f64_ceil(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_ceil)
    }

    fn visit_f64_floor(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_floor)
    }

    fn visit_f64_trunc(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_trunc)
    }

    fn visit_f64_nearest(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_nearest)
    }

    fn visit_f64_sqrt(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_sqrt)
    }

    fn visit_f64_add(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_add)
    }

    fn visit_f64_sub(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_sub)
    }

    fn visit_f64_mul(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_mul)
    }

    fn visit_f64_div(&mut self) -> Self::Outcome {
        self.try_execute_binary(UntypedValue::f64_div)
    }

    fn visit_f64_min(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_min)
    }

    fn visit_f64_max(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_max)
    }

    fn visit_f64_copysign(&mut self) -> Self::Outcome {
        self.execute_binary(UntypedValue::f64_copysign)
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_wrap_i64)
    }

    fn visit_i32_trunc_f32(&mut self) -> Self::Outcome {
        self.try_execute_unary(UntypedValue::i32_trunc_f32_s)
    }

    fn visit_u32_trunc_f32(&mut self) -> Self::Outcome {
        self.try_execute_unary(UntypedValue::i32_trunc_f32_u)
    }

    fn visit_i32_trunc_f64(&mut self) -> Self::Outcome {
        self.try_execute_unary(UntypedValue::i32_trunc_f64_s)
    }

    fn visit_u32_trunc_f64(&mut self) -> Self::Outcome {
        self.try_execute_unary(UntypedValue::i32_trunc_f64_u)
    }

    fn visit_i64_extend_i32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_extend_i32_s)
    }

    fn visit_i64_extend_u32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_extend_i32_u)
    }

    fn visit_i64_trunc_f32(&mut self) -> Self::Outcome {
        self.try_execute_unary(UntypedValue::i64_trunc_f32_s)
    }

    fn visit_u64_trunc_f32(&mut self) -> Self::Outcome {
        self.try_execute_unary(UntypedValue::i64_trunc_f32_u)
    }

    fn visit_i64_trunc_f64(&mut self) -> Self::Outcome {
        self.try_execute_unary(UntypedValue::i64_trunc_f64_s)
    }

    fn visit_u64_trunc_f64(&mut self) -> Self::Outcome {
        self.try_execute_unary(UntypedValue::i64_trunc_f64_u)
    }

    fn visit_f32_convert_i32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_convert_i32_s)
    }

    fn visit_f32_convert_u32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_convert_i32_u)
    }

    fn visit_f32_convert_i64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_convert_i64_s)
    }

    fn visit_f32_convert_u64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_convert_i64_u)
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f32_demote_f64)
    }

    fn visit_f64_convert_i32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_convert_i32_s)
    }

    fn visit_f64_convert_u32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_convert_i32_u)
    }

    fn visit_f64_convert_i64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_convert_i64_s)
    }

    fn visit_f64_convert_u64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_convert_i64_u)
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::f64_promote_f32)
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Outcome {
        self.execute_reinterpret::<F32, i32>()
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Outcome {
        self.execute_reinterpret::<F64, i64>()
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Outcome {
        self.execute_reinterpret::<i32, F32>()
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Outcome {
        self.execute_reinterpret::<i64, F64>()
    }

    fn visit_i32_sign_extend8(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_extend8_s)
    }

    fn visit_i32_sign_extend16(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_extend16_s)
    }

    fn visit_i64_sign_extend8(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_extend8_s)
    }

    fn visit_i64_sign_extend16(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_extend16_s)
    }

    fn visit_i64_sign_extend32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_extend32_s)
    }

    fn visit_i32_trunc_sat_f32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_trunc_sat_f32_s)
    }

    fn visit_u32_trunc_sat_f32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_trunc_sat_f32_u)
    }

    fn visit_i32_trunc_sat_f64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_trunc_sat_f64_s)
    }

    fn visit_u32_trunc_sat_f64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i32_trunc_sat_f64_u)
    }

    fn visit_i64_trunc_sat_f32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_trunc_sat_f32_s)
    }

    fn visit_u64_trunc_sat_f32(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_trunc_sat_f32_u)
    }

    fn visit_i64_trunc_sat_f64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_trunc_sat_f64_s)
    }

    fn visit_u64_trunc_sat_f64(&mut self) -> Self::Outcome {
        self.execute_unary(UntypedValue::i64_trunc_sat_f64_u)
    }
}
