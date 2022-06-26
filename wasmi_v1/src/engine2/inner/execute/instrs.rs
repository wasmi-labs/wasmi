use super::stack::StackFrameView;
use crate::{
    engine2::{
        bytecode::{self, ExecRegister, ExecuteTypes, VisitInstruction},
        inner::EngineResources,
        ExecProvider,
        ExecProviderSlice,
        ExecRegisterSlice,
        InstructionTypes,
        ResolvedFuncBody,
        Target,
    },
    AsContext,
    AsContextMut,
    Func,
    Global,
    Instance,
    Memory,
    StoreContextMut,
    Table,
};
use wasmi_core::{ExtendInto, LittleEndianConvert, Trap, TrapCode, UntypedValue};

/// The possible outcomes of an instruction execution.
#[derive(Debug, Copy, Clone)]
pub enum ExecOutcome {
    /// Continues execution at the next instruction.
    Continue,
    /// Branch to the target instruction.
    Branch {
        /// The target instruction to branch to.
        target: Target,
    },
    /// Returns the result of the function execution.
    Return {
        /// The returned result values.
        results: ExecProviderSlice,
    },
    /// Persons a nested function call.
    Call {
        /// The results of the function call.
        results: ExecRegisterSlice,
        /// The called function.
        callee: Func,
    },
}

/// State that is used during Wasm function execution.
#[derive(Debug)]
pub struct ExecContext<'engine, 'func, 'ctx, T> {
    /// The function frame that is being executed.
    frame: StackFrameView<'func>,
    /// The resolved function body of the executed function frame.
    func_body: ResolvedFuncBody<'engine>,
    /// The read-only engine resources.
    res: &'engine EngineResources,
    /// The associated store context.
    ctx: StoreContextMut<'ctx, T>,
}

impl<'engine, 'func, 'ctx, T> ExecContext<'engine, 'func, 'ctx, T> {
    /// Returns the [`ExecOutcome`] to continue to the next instruction.
    ///
    /// # Note
    ///
    /// This is a convenience function with the purpose to simplify
    /// the process to change the behavior of the dispatch once required
    /// for optimization purposes.
    fn next_instr(&self) -> Result<ExecOutcome, Trap> {
        Ok(ExecOutcome::Continue)
    }

    /// Returns the default linear memory.
    ///
    /// # Panics
    ///
    /// If there exists is no linear memory for the instance.
    fn default_memory(&mut self) -> Memory {
        self.frame.default_memory(&self.ctx)
    }

    /// Returns the default table.
    ///
    /// # Panics
    ///
    /// If there exists is no table for the instance.
    fn default_table(&mut self) -> Table {
        self.frame.default_table(&self.ctx)
    }

    /// Returns the global variable at the given global variable index.
    ///
    /// # Panics
    ///
    /// If there is no global variable at the given index.
    fn resolve_global(&self, global_index: bytecode::Global) -> Global {
        self.frame
            .instance
            .get_global(self.ctx.as_context(), global_index.into_inner())
            .unwrap_or_else(|| {
                panic!(
                    "missing global at index {:?} for instance {:?}",
                    global_index, self.frame.instance
                )
            })
    }

    /// Calculates the effective address of a linear memory access.
    ///
    /// # Errors
    ///
    /// If the resulting effective address overflows.
    fn effective_address(offset: bytecode::Offset, ptr: UntypedValue) -> Result<usize, Trap> {
        offset
            .into_inner()
            .checked_add(u32::from(ptr))
            .map(|address| address as usize)
            .ok_or(TrapCode::MemoryAccessOutOfBounds)
            .map_err(Into::into)
    }

    /// Loads bytes from the default memory into the given `buffer`.
    ///
    /// # Errors
    ///
    /// If the memory access is out of bounds.
    ///
    /// # Panics
    ///
    /// If there exists is no linear memory for the instance.
    fn load_bytes(
        &mut self,
        ptr: ExecRegister,
        offset: bytecode::Offset,
        buffer: &mut [u8],
    ) -> Result<(), Trap> {
        let memory = self.default_memory();
        let ptr = self.frame.regs.get(ptr);
        let address = Self::effective_address(offset, ptr)?;
        memory
            .read(&self.ctx, address, buffer.as_mut())
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        Ok(())
    }

    /// Stores bytes to the default memory from the given `buffer`.
    ///
    /// # Errors
    ///
    /// If the memory access is out of bounds.
    ///
    /// # Panics
    ///
    /// If there exists is no linear memory for the instance.
    fn store_bytes(
        &mut self,
        ptr: ExecRegister,
        offset: bytecode::Offset,
        bytes: &[u8],
    ) -> Result<(), Trap> {
        let memory = self.default_memory();
        let ptr = self.frame.regs.get(ptr);
        let address = Self::effective_address(offset, ptr)?;
        memory
            .write(&mut self.ctx, address, bytes)
            .map_err(|_| TrapCode::MemoryAccessOutOfBounds)?;
        Ok(())
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
    fn exec_load<V>(
        &mut self,
        result: ExecRegister,
        ptr: ExecRegister,
        offset: bytecode::Offset,
    ) -> Result<ExecOutcome, Trap>
    where
        V: LittleEndianConvert + Into<UntypedValue>,
    {
        let mut buffer = <<V as LittleEndianConvert>::Bytes as Default>::default();
        self.load_bytes(ptr, offset, buffer.as_mut())?;
        let value = <V as LittleEndianConvert>::from_le_bytes(buffer);
        self.frame.regs.set(result, value.into());
        self.next_instr()
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
    fn exec_load_extend<V, U>(
        &mut self,
        result: ExecRegister,
        ptr: ExecRegister,
        offset: bytecode::Offset,
    ) -> Result<ExecOutcome, Trap>
    where
        V: ExtendInto<U> + LittleEndianConvert,
        U: Into<UntypedValue>,
    {
        let mut buffer = <<V as LittleEndianConvert>::Bytes as Default>::default();
        self.load_bytes(ptr, offset, buffer.as_mut())?;
        let extended = <V as LittleEndianConvert>::from_le_bytes(buffer).extend_into();
        self.frame.regs.set(result, extended.into());
        self.next_instr()
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
    fn exec_store<V>(
        &mut self,
        ptr: ExecRegister,
        offset: bytecode::Offset,
        new_value: ExecProvider,
    ) -> Result<ExecOutcome, Trap>
    where
        V: LittleEndianConvert + From<UntypedValue>,
    {
        let new_value = V::from(self.load_provider(new_value));
        let bytes = <V as LittleEndianConvert>::into_le_bytes(new_value);
        self.store_bytes(ptr, offset, bytes.as_ref())?;
        self.next_instr()
    }

    /// Executes the given unary `wasmi` operation.
    ///
    /// # Note
    ///
    /// Loads from the given `input` register,
    /// performs the given operation `op` and stores the
    /// result back into the `result` register.
    ///
    /// # Errors
    ///
    /// Returns `Result::Ok` for convenience.
    fn exec_unary_op(
        &mut self,
        result: ExecRegister,
        input: ExecRegister,
        op: fn(UntypedValue) -> UntypedValue,
    ) -> Result<ExecOutcome, Trap> {
        let input = self.frame.regs.get(input);
        self.frame.regs.set(result, op(input));
        self.next_instr()
    }

    /// Executes the given fallible unary `wasmi` operation.
    ///
    /// # Note
    ///
    /// Loads from the given `input` register,
    /// performs the given operation `op` and stores the
    /// result back into the `result` register.
    ///
    /// # Errors
    ///
    /// Returns an error if the given operation `op` fails.
    fn exec_fallible_unary_op(
        &mut self,
        result: ExecRegister,
        input: ExecRegister,
        op: fn(UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<ExecOutcome, Trap> {
        let input = self.frame.regs.get(input);
        self.frame.regs.set(result, op(input)?);
        self.next_instr()
    }

    /// Loads the value of the given `provider`.
    ///
    /// # Panics
    ///
    /// If the provider refers to an non-existing immediate value.
    /// Note that reaching this case reflects a bug in the interpreter.
    fn load_provider(&self, provider: ExecProvider) -> UntypedValue {
        provider.decode_using(
            |rhs| self.frame.regs.get(rhs),
            |imm| {
                self.res.const_pool.resolve(imm).unwrap_or_else(|| {
                    panic!("unexpectedly failed to resolve immediate at {:?}", imm)
                })
            },
        )
    }

    /// Executes the given binary `wasmi` operation.
    ///
    /// # Note
    ///
    /// Loads from the given `lhs` and `rhs` registers,
    /// performs the given operation `op` and stores the
    /// result back into the `result` register.
    ///
    /// # Errors
    ///
    /// Returns `Result::Ok` for convenience.
    fn exec_binary_op(
        &mut self,
        result: ExecRegister,
        lhs: ExecRegister,
        rhs: ExecProvider,
        op: fn(UntypedValue, UntypedValue) -> UntypedValue,
    ) -> Result<ExecOutcome, Trap> {
        let lhs = self.frame.regs.get(lhs);
        let rhs = self.load_provider(rhs);
        self.frame.regs.set(result, op(lhs, rhs));
        self.next_instr()
    }

    /// Executes the given fallible binary `wasmi` operation.
    ///
    /// # Note
    ///
    /// Loads from the given `lhs` and `rhs` registers,
    /// performs the given operation `op` and stores the
    /// result back into the `result` register.
    ///
    /// # Errors
    ///
    /// Returns an error if the given operation `op` fails.
    fn exec_fallible_binary_op(
        &mut self,
        result: ExecRegister,
        lhs: ExecRegister,
        rhs: ExecProvider,
        op: fn(UntypedValue, UntypedValue) -> Result<UntypedValue, TrapCode>,
    ) -> Result<ExecOutcome, Trap> {
        let lhs = self.frame.regs.get(lhs);
        let rhs = self.load_provider(rhs);
        self.frame.regs.set(result, op(lhs, rhs)?);
        self.next_instr()
    }

    /// Executes a conditional branch.
    ///
    /// Only branches when `op(condition)` evaluates to `true`.
    ///
    /// # Errors
    ///
    /// Returns `Result::Ok` for convenience.
    fn exec_branch_conditionally(
        &mut self,
        target: Target,
        condition: ExecRegister,
        op: fn(UntypedValue) -> bool,
    ) -> Result<ExecOutcome, Trap> {
        let condition = self.frame.regs.get(condition);
        if op(condition) {
            return Ok(ExecOutcome::Branch { target });
        }
        self.next_instr()
    }
}

impl<'engine, 'func, 'ctx, T> VisitInstruction<ExecuteTypes>
    for ExecContext<'engine, 'func, 'ctx, T>
{
    type Outcome = Result<ExecOutcome, Trap>;

    fn visit_br(&mut self, target: Target) -> Self::Outcome {
        Ok(ExecOutcome::Branch { target })
    }

    fn visit_br_eqz(
        &mut self,
        target: Target,
        condition: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_branch_conditionally(target, condition, |condition| {
            condition == UntypedValue::from(0_i32)
        })
    }

    fn visit_br_nez(
        &mut self,
        target: Target,
        condition: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_branch_conditionally(target, condition, |condition| {
            condition != UntypedValue::from(0_i32)
        })
    }

    fn visit_return_nez(
        &mut self,
        results: <ExecuteTypes as InstructionTypes>::ProviderSlice,
        condition: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        let condition = self.frame.regs.get(condition);
        let zero = UntypedValue::from(0_i32);
        if condition != zero {
            return Ok(ExecOutcome::Return { results });
        }
        self.next_instr()
    }

    fn visit_br_table(
        &mut self,
        case: <ExecuteTypes as InstructionTypes>::Register,
        len_targets: usize,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_trap(&mut self, trap_code: TrapCode) -> Self::Outcome {
        Err(Trap::from(trap_code))
    }

    fn visit_return(
        &mut self,
        results: <ExecuteTypes as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome {
        Ok(ExecOutcome::Return { results })
    }

    fn visit_call(
        &mut self,
        func: crate::module::FuncIdx,
        results: <ExecuteTypes as InstructionTypes>::RegisterSlice,
        params: <ExecuteTypes as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_call_indirect(
        &mut self,
        func_type: crate::module::FuncTypeIdx,
        results: <ExecuteTypes as InstructionTypes>::RegisterSlice,
        index: <ExecuteTypes as InstructionTypes>::Provider,
        params: <ExecuteTypes as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_copy(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        let input = self.load_provider(input);
        self.frame.regs.set(result, input);
        self.next_instr()
    }

    fn visit_select(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        condition: <ExecuteTypes as InstructionTypes>::Register,
        if_true: <ExecuteTypes as InstructionTypes>::Provider,
        if_false: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        let condition = self.frame.regs.get(condition);
        let zero = UntypedValue::from(0_i32);
        let case = if condition != zero {
            self.load_provider(if_true)
        } else {
            self.load_provider(if_false)
        };
        self.frame.regs.set(result, case);
        self.next_instr()
    }

    fn visit_global_get(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        global: bytecode::Global,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_global_set(
        &mut self,
        global: bytecode::Global,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_i32_load(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load::<i32>(result, ptr, offset)
    }

    fn visit_i64_load(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load::<i64>(result, ptr, offset)
    }

    fn visit_f32_load(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load::<f32>(result, ptr, offset)
    }

    fn visit_f64_load(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load::<f64>(result, ptr, offset)
    }

    fn visit_i32_load_8_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<i8, i32>(result, ptr, offset)
    }

    fn visit_i32_load_8_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<u8, i32>(result, ptr, offset)
    }

    fn visit_i32_load_16_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<i16, i32>(result, ptr, offset)
    }

    fn visit_i32_load_16_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<u16, i32>(result, ptr, offset)
    }

    fn visit_i64_load_8_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<i8, i64>(result, ptr, offset)
    }

    fn visit_i64_load_8_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<u8, i64>(result, ptr, offset)
    }

    fn visit_i64_load_16_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<i16, i64>(result, ptr, offset)
    }

    fn visit_i64_load_16_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<u16, i64>(result, ptr, offset)
    }

    fn visit_i64_load_32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<i32, i64>(result, ptr, offset)
    }

    fn visit_i64_load_32_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load_extend::<u32, i64>(result, ptr, offset)
    }

    fn visit_i32_store(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store::<i32>(ptr, offset, value)
    }

    fn visit_i64_store(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store::<i64>(ptr, offset, value)
    }

    fn visit_f32_store(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store::<f32>(ptr, offset, value)
    }

    fn visit_f64_store(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store::<f64>(ptr, offset, value)
    }

    fn visit_i32_store_8(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_i32_store_16(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_i64_store_8(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_i64_store_16(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_i64_store_32(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_memory_size(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_memory_grow(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        amount: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        todo!()
    }

    fn visit_i32_eq(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_eq)
    }

    fn visit_i32_ne(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_ne)
    }

    fn visit_i32_lt_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_lt_s)
    }

    fn visit_i32_lt_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_lt_u)
    }

    fn visit_i32_gt_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_gt_s)
    }

    fn visit_i32_gt_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_gt_u)
    }

    fn visit_i32_le_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_le_s)
    }

    fn visit_i32_le_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_le_u)
    }

    fn visit_i32_ge_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_ge_s)
    }

    fn visit_i32_ge_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_ge_u)
    }

    fn visit_i64_eq(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_eq)
    }

    fn visit_i64_ne(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_ne)
    }

    fn visit_i64_lt_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_lt_s)
    }

    fn visit_i64_lt_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_lt_u)
    }

    fn visit_i64_gt_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_gt_s)
    }

    fn visit_i64_gt_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_gt_u)
    }

    fn visit_i64_le_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_le_s)
    }

    fn visit_i64_le_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_le_u)
    }

    fn visit_i64_ge_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_ge_s)
    }

    fn visit_i64_ge_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_ge_u)
    }

    fn visit_f32_eq(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_eq)
    }

    fn visit_f32_ne(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_ne)
    }

    fn visit_f32_lt(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_lt)
    }

    fn visit_f32_gt(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_gt)
    }

    fn visit_f32_le(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_le)
    }

    fn visit_f32_ge(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_ge)
    }

    fn visit_f64_eq(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_eq)
    }

    fn visit_f64_ne(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_ne)
    }

    fn visit_f64_lt(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_lt)
    }

    fn visit_f64_gt(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_gt)
    }

    fn visit_f64_le(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_le)
    }

    fn visit_f64_ge(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_ge)
    }

    fn visit_i32_clz(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_clz)
    }

    fn visit_i32_ctz(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_ctz)
    }

    fn visit_i32_popcnt(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_popcnt)
    }

    fn visit_i32_add(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_add)
    }

    fn visit_i32_sub(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_sub)
    }

    fn visit_i32_mul(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_mul)
    }

    fn visit_i32_div_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::i32_div_s)
    }

    fn visit_i32_div_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::i32_div_u)
    }

    fn visit_i32_rem_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::i32_rem_s)
    }

    fn visit_i32_rem_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::i32_rem_u)
    }

    fn visit_i32_and(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_and)
    }

    fn visit_i32_or(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_or)
    }

    fn visit_i32_xor(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_xor)
    }

    fn visit_i32_shl(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_shl)
    }

    fn visit_i32_shr_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_shr_s)
    }

    fn visit_i32_shr_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_shr_u)
    }

    fn visit_i32_rotl(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_rotl)
    }

    fn visit_i32_rotr(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i32_rotr)
    }

    fn visit_i64_clz(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_clz)
    }

    fn visit_i64_ctz(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_ctz)
    }

    fn visit_i64_popcnt(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_popcnt)
    }

    fn visit_i64_add(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_add)
    }

    fn visit_i64_sub(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_sub)
    }

    fn visit_i64_mul(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_mul)
    }

    fn visit_i64_div_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::i64_div_s)
    }

    fn visit_i64_div_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::i64_div_u)
    }

    fn visit_i64_rem_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::i64_rem_s)
    }

    fn visit_i64_rem_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::i64_rem_u)
    }

    fn visit_i64_and(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_and)
    }

    fn visit_i64_or(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_or)
    }

    fn visit_i64_xor(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_xor)
    }

    fn visit_i64_shl(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_shl)
    }

    fn visit_i64_shr_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_shr_s)
    }

    fn visit_i64_shr_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_shr_u)
    }

    fn visit_i64_rotl(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_rotl)
    }

    fn visit_i64_rotr(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::i64_rotr)
    }

    fn visit_f32_abs(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_abs)
    }

    fn visit_f32_neg(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_neg)
    }

    fn visit_f32_ceil(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_ceil)
    }

    fn visit_f32_floor(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_floor)
    }

    fn visit_f32_trunc(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_trunc)
    }

    fn visit_f32_nearest(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_nearest)
    }

    fn visit_f32_sqrt(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_sqrt)
    }

    fn visit_f32_add(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_add)
    }

    fn visit_f32_sub(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_sub)
    }

    fn visit_f32_mul(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_mul)
    }

    fn visit_f32_div(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::f32_div)
    }

    fn visit_f32_min(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_min)
    }

    fn visit_f32_max(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_max)
    }

    fn visit_f32_copysign(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f32_copysign)
    }

    fn visit_f64_abs(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_abs)
    }

    fn visit_f64_neg(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_neg)
    }

    fn visit_f64_ceil(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_ceil)
    }

    fn visit_f64_floor(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_floor)
    }

    fn visit_f64_trunc(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_trunc)
    }

    fn visit_f64_nearest(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_nearest)
    }

    fn visit_f64_sqrt(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_sqrt)
    }

    fn visit_f64_add(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_add)
    }

    fn visit_f64_sub(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_sub)
    }

    fn visit_f64_mul(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_mul)
    }

    fn visit_f64_div(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_fallible_binary_op(result, lhs, rhs, UntypedValue::f64_div)
    }

    fn visit_f64_min(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_min)
    }

    fn visit_f64_max(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_max)
    }

    fn visit_f64_copysign(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        lhs: <ExecuteTypes as InstructionTypes>::Register,
        rhs: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_binary_op(result, lhs, rhs, UntypedValue::f64_copysign)
    }

    fn visit_i32_wrap_i64(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_wrap_i64)
    }

    fn visit_i32_trunc_f32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_fallible_unary_op(result, input, UntypedValue::i32_trunc_f32_s)
    }

    fn visit_i32_trunc_f32_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_fallible_unary_op(result, input, UntypedValue::i32_trunc_f32_u)
    }

    fn visit_i32_trunc_f64_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_fallible_unary_op(result, input, UntypedValue::i32_trunc_f64_s)
    }

    fn visit_i32_trunc_f64_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_fallible_unary_op(result, input, UntypedValue::i32_trunc_f64_u)
    }

    fn visit_i64_extend_i32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_extend_i32_s)
    }

    fn visit_i64_extend_i32_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_extend_i32_u)
    }

    fn visit_i64_trunc_f32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_fallible_unary_op(result, input, UntypedValue::i64_trunc_f32_s)
    }

    fn visit_i64_trunc_f32_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_fallible_unary_op(result, input, UntypedValue::i64_trunc_f32_u)
    }

    fn visit_i64_trunc_f64_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_fallible_unary_op(result, input, UntypedValue::i64_trunc_f64_s)
    }

    fn visit_i64_trunc_f64_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_fallible_unary_op(result, input, UntypedValue::i64_trunc_f64_u)
    }

    fn visit_f32_convert_i32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_convert_i32_s)
    }

    fn visit_f32_convert_i32_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_convert_i32_u)
    }

    fn visit_f32_convert_i64_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_convert_i64_s)
    }

    fn visit_f32_convert_i64_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_convert_i64_u)
    }

    fn visit_f32_demote_f64(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f32_demote_f64)
    }

    fn visit_f64_convert_i32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_convert_i32_s)
    }

    fn visit_f64_convert_i32_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_convert_i32_u)
    }

    fn visit_f64_convert_i64_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_convert_i64_s)
    }

    fn visit_f64_convert_i64_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_convert_i64_u)
    }

    fn visit_f64_promote_f32(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::f64_promote_f32)
    }

    fn visit_i32_extend8_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_extend8_s)
    }

    fn visit_i32_extend16_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_extend16_s)
    }

    fn visit_i64_extend8_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_extend8_s)
    }

    fn visit_i64_extend16_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_extend16_s)
    }

    fn visit_i64_extend32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_extend32_s)
    }

    fn visit_i32_trunc_sat_f32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_trunc_sat_f32_s)
    }

    fn visit_i32_trunc_sat_f32_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_trunc_sat_f32_u)
    }

    fn visit_i32_trunc_sat_f64_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_trunc_sat_f64_s)
    }

    fn visit_i32_trunc_sat_f64_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i32_trunc_sat_f64_u)
    }

    fn visit_i64_trunc_sat_f32_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_trunc_sat_f32_s)
    }

    fn visit_i64_trunc_sat_f32_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_trunc_sat_f32_u)
    }

    fn visit_i64_trunc_sat_f64_s(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_trunc_sat_f64_s)
    }

    fn visit_i64_trunc_sat_f64_u(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        input: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        self.exec_unary_op(result, input, UntypedValue::i64_trunc_sat_f64_u)
    }
}
