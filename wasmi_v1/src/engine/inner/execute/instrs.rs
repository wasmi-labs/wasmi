use super::{stack::StackFrameView, CallOutcome};
use crate::{
    engine::{
        bytecode::{self, visit_instr, ExecRegister, ExecuteTypes, VisitInstruction},
        code_map::CodeMap,
        inner::EngineResources,
        ExecProvider,
        ExecProviderSlice,
        ExecRegisterSlice,
        InstructionTypes,
        Target,
    },
    module::{FuncIdx, FuncTypeIdx},
    AsContext,
    AsContextMut,
    Func,
    Global,
    Memory,
    StoreContextMut,
    Table,
};
use core::cmp;
use wasmi_core::{
    memory_units::Pages,
    ExtendInto,
    LittleEndianConvert,
    Trap,
    TrapCode,
    UntypedValue,
    WrapInto,
    F32,
    F64,
};

/// The possible outcomes of an instruction execution.
#[derive(Debug, Copy, Clone)]
pub enum ExecOutcome {
    /// Continues execution at the next instruction.
    Continue,
    /// Branch to the target instruction.
    Branch {
        /// The program counter to the branched to instruction.
        next_pc: usize,
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
        /// The parameters of the function call.
        params: ExecProviderSlice,
    },
}

/// Executes the given [`StackFrameView`].
///
/// Returns the outcome of the execution.
///
/// # Errors
///
/// If the execution traps.
///
/// # Panics
///
/// If resources are missing unexpectedly.
/// For example, a linear memory instance, global variable, etc.
pub(super) fn execute_frame(
    mut ctx: impl AsContextMut,
    code_map: &CodeMap,
    res: &EngineResources,
    frame: &mut StackFrameView,
) -> Result<CallOutcome, Trap> {
    let func_body = code_map.resolve(frame.func_body);
    let mut exec_ctx = ExecContext {
        frame,
        res,
        ctx: ctx.as_context_mut(),
    };
    loop {
        // # Safety
        //
        // Since the Wasm and `wasmi` bytecode has already been validated the
        // indices passed at this point can be assumed to be valid always.
        let instr = unsafe { func_body.get_release_unchecked(*exec_ctx.frame.pc) };
        match visit_instr(&mut exec_ctx, *instr)? {
            ExecOutcome::Continue => {
                // Advance program counter.
                *exec_ctx.frame.pc += 1;
            }
            ExecOutcome::Branch { next_pc } => {
                // Set program counter to the branch target.
                *exec_ctx.frame.pc = next_pc;
            }
            ExecOutcome::Call {
                results,
                callee,
                params,
            } => {
                // Advance program counter before calling.
                *exec_ctx.frame.pc += 1;
                return Ok(CallOutcome::Call {
                    results,
                    callee,
                    params,
                });
            }
            ExecOutcome::Return { results } => {
                return Ok(CallOutcome::Return { returned: results });
            }
        }
    }
}

/// State that is used during Wasm function execution.
#[derive(Debug)]
pub struct ExecContext<'engine, 'func1, 'func2, 'ctx, T> {
    /// The function frame that is being executed.
    frame: &'func1 mut StackFrameView<'func2>,
    /// The read-only engine resources.
    res: &'engine EngineResources,
    /// The associated store context.
    ctx: StoreContextMut<'ctx, T>,
}

impl<'engine, 'func1, 'func2, 'ctx, T> ExecContext<'engine, 'func1, 'func2, 'ctx, T> {
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

    /// Returns the [`ExecOutcome`] to branch to the given `target`.
    ///
    /// # Note
    ///
    /// This is a convenience function with the purpose to simplify
    /// the process to change the behavior of the dispatch once required
    /// for optimization purposes.
    fn branch_to_target(&self, target: Target) -> Result<ExecOutcome, Trap> {
        Ok(ExecOutcome::Branch {
            next_pc: target.destination().into_inner() as usize,
        })
    }

    /// Copys values from `src` to `dst`.
    ///
    /// # Panics (Debug)
    ///
    /// If both slices do not have the same length.
    fn copy_many(&mut self, dst: ExecRegisterSlice, src: ExecProviderSlice) {
        debug_assert_eq!(dst.len(), src.len());
        let src = self.res.provider_pool.resolve(src);
        dst.into_iter().zip(src).for_each(|(dst, src)| {
            let src = self.load_provider(*src);
            self.frame.regs.set(dst, src);
        });
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
    fn exec_store_wrap<V, U>(
        &mut self,
        ptr: ExecRegister,
        offset: bytecode::Offset,
        new_value: ExecProvider,
    ) -> Result<ExecOutcome, Trap>
    where
        V: From<UntypedValue> + WrapInto<U>,
        U: LittleEndianConvert,
    {
        let new_value = V::from(self.load_provider(new_value)).wrap_into();
        let bytes = <U as LittleEndianConvert>::into_le_bytes(new_value);
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
        results: ExecRegisterSlice,
        returned: ExecProviderSlice,
        op: fn(UntypedValue) -> bool,
    ) -> Result<ExecOutcome, Trap> {
        let condition = self.frame.regs.get(condition);
        if op(condition) {
            self.copy_many(results, returned);
            return self.branch_to_target(target);
        }
        self.next_instr()
    }
}

impl<'engine, 'func1, 'func2, 'ctx, T> VisitInstruction<ExecuteTypes>
    for ExecContext<'engine, 'func1, 'func2, 'ctx, T>
{
    type Outcome = Result<ExecOutcome, Trap>;

    fn visit_br(
        &mut self,
        target: Target,
        results: <ExecuteTypes as InstructionTypes>::RegisterSlice,
        returned: <ExecuteTypes as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome {
        self.copy_many(results, returned);
        self.branch_to_target(target)
    }

    fn visit_br_eqz(
        &mut self,
        target: Target,
        condition: <ExecuteTypes as InstructionTypes>::Register,
        results: <ExecuteTypes as InstructionTypes>::RegisterSlice,
        returned: <ExecuteTypes as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome {
        self.exec_branch_conditionally(target, condition, results, returned, |condition| {
            condition == UntypedValue::from(0_i32)
        })
    }

    fn visit_br_nez(
        &mut self,
        target: Target,
        condition: <ExecuteTypes as InstructionTypes>::Register,
        results: <ExecuteTypes as InstructionTypes>::RegisterSlice,
        returned: <ExecuteTypes as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome {
        self.exec_branch_conditionally(target, condition, results, returned, |condition| {
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
        let index = u32::from(self.frame.regs.get(case)) as usize;
        // The index of the default target is the last target of the `br_table`.
        let max_index = len_targets - 1;
        // A normalized index will always yield a target without panicking.
        let normalized_index = cmp::min(index, max_index);
        // Simply branch to the selected instruction which is going to be either
        // a `br` or a `return` instruction as demanded by the `wasmi` bytecode.
        let next_pc = *self.frame.pc + normalized_index + 1;
        Ok(ExecOutcome::Branch { next_pc })
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
        func: FuncIdx,
        results: <ExecuteTypes as InstructionTypes>::RegisterSlice,
        params: <ExecuteTypes as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome {
        let callee = self
            .frame
            .instance
            .get_func(&mut self.ctx, func.into_u32())
            .unwrap_or_else(|| {
                panic!(
                    "unexpected missing function at index {:?} for instance {:?}",
                    func, self.frame.instance
                )
            });
        Ok(ExecOutcome::Call {
            results,
            callee,
            params,
        })
    }

    fn visit_call_indirect(
        &mut self,
        func_type: FuncTypeIdx,
        results: <ExecuteTypes as InstructionTypes>::RegisterSlice,
        index: <ExecuteTypes as InstructionTypes>::Provider,
        params: <ExecuteTypes as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome {
        let index = u32::from(self.load_provider(index));
        let table = self.default_table();
        let callee = table
            .get(&self.ctx, index as usize)
            .map_err(|_| TrapCode::TableAccessOutOfBounds)?
            .ok_or(TrapCode::ElemUninitialized)?;
        let actual_signature = callee.signature(&self.ctx);
        let expected_signature = self
            .frame
            .instance
            .get_signature(&self.ctx, func_type.into_u32())
            .unwrap_or_else(|| {
                panic!(
                    "missing signature for `call_indirect` at index {:?} for instance {:?}",
                    func_type, self.frame.instance
                )
            });
        if actual_signature != expected_signature {
            return Err(Trap::from(TrapCode::UnexpectedSignature));
        }
        Ok(ExecOutcome::Call {
            results,
            callee,
            params,
        })
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
        let value = self.resolve_global(global).get(&self.ctx);
        self.frame.regs.set(result, value.into());
        self.next_instr()
    }

    fn visit_global_set(
        &mut self,
        global: bytecode::Global,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        let global_var = self.resolve_global(global);
        let value = self
            .load_provider(value)
            .with_type(global_var.value_type(&self.ctx));
        global_var
            .set(&mut self.ctx, value)
            .unwrap_or_else(|error| {
                panic!(
                    "unexpected type mismatch upon `global.set` for global {:?}: {}",
                    global_var, error
                )
            });
        self.next_instr()
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
        self.exec_load::<F32>(result, ptr, offset)
    }

    fn visit_f64_load(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
    ) -> Self::Outcome {
        self.exec_load::<F64>(result, ptr, offset)
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
        self.exec_store::<F32>(ptr, offset, value)
    }

    fn visit_f64_store(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store::<F64>(ptr, offset, value)
    }

    fn visit_i32_store_8(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store_wrap::<i32, i8>(ptr, offset, value)
    }

    fn visit_i32_store_16(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store_wrap::<i32, i16>(ptr, offset, value)
    }

    fn visit_i64_store_8(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store_wrap::<i64, i8>(ptr, offset, value)
    }

    fn visit_i64_store_16(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store_wrap::<i64, i16>(ptr, offset, value)
    }

    fn visit_i64_store_32(
        &mut self,
        ptr: <ExecuteTypes as InstructionTypes>::Register,
        offset: bytecode::Offset,
        value: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        self.exec_store_wrap::<i64, i32>(ptr, offset, value)
    }

    fn visit_memory_size(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
    ) -> Self::Outcome {
        let memory = self.default_memory();
        let size = memory.current_pages(&self.ctx).0 as u32;
        self.frame.regs.set(result, size.into());
        self.next_instr()
    }

    fn visit_memory_grow(
        &mut self,
        result: <ExecuteTypes as InstructionTypes>::Register,
        amount: <ExecuteTypes as InstructionTypes>::Provider,
    ) -> Self::Outcome {
        let amount = u32::from(self.load_provider(amount));
        let memory = self.default_memory();
        let old_size = match memory.grow(self.ctx.as_context_mut(), Pages(amount as usize)) {
            Ok(Pages(old_size)) => old_size as u32,
            Err(_) => {
                // Note: The WebAssembly specification demands to return
                //       `0xFFFF_FFFF` for the failure case of this instruction.
                u32::MAX
            }
        };
        self.frame.regs.set(result, old_size.into());
        self.next_instr()
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
