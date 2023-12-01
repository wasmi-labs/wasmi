use super::Executor;
use crate::{
    core::TrapCode,
    engine::regmach::{
        bytecode::{FuncIdx, Instruction, Register, RegisterSpan, SignatureIdx, TableIdx},
        code_map::{CompiledFuncEntity, InstructionPtr},
        stack::{CallFrame, Stack, ValueStackPtr},
        CompiledFunc,
    },
    func::FuncEntity,
    Func,
    FuncRef,
};
use core::slice;

/// Describes whether a `call` instruction has at least one parameter or none.
#[derive(Debug, Copy, Clone)]
pub enum CallParams {
    /// The function call has no parameters.
    None,
    /// The function call has at least one parameter.
    Some,
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
    Call {
        results: RegisterSpan,
        host_func: Func,
        call_kind: CallKind,
    },
}

/// The kind of a function call.
#[derive(Debug, Copy, Clone)]
pub enum CallKind {
    /// A nested function call.
    Nested,
    /// A tailing function call.
    Tail,
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Updates the [`InstructionPtr`] of the caller [`CallFrame`] before dispatching a call.
    ///
    /// # Note
    ///
    /// The `offset` denotes how many [`Instruction`] words make up the call instruction.
    fn update_instr_ptr_at(&mut self, offset: usize) {
        // Note: we explicitly do not mutate `self.ip` since that would make
        // other parts of the code more fragile with respect to instruction ordering.
        let mut ip = self.ip;
        ip.add(offset);
        let caller = self
            .call_stack
            .peek_mut()
            .expect("caller call frame must be on the stack");
        caller.update_instr_ptr(ip);
    }

    /// Fetches the [`Instruction::CallIndirectParams`] parameter for a call [`Instruction`].
    ///
    /// # Note
    ///
    /// - This advances the [`InstructionPtr`] to the next [`Instruction`].
    /// - This is done by encoding an [`Instruction::TableGet`] instruction
    ///   word following the actual instruction where the [`TableIdx`]
    ///   paremeter belongs to.
    /// - This is required for some instructions that do not fit into
    ///   a single instruction word and store a [`TableIdx`] value in
    ///   another instruction word.
    fn pull_call_indirect_params(&mut self) -> (u32, TableIdx) {
        self.ip.add(1);
        match self.ip.get() {
            Instruction::CallIndirectParams(call_params) => {
                let index = u32::from(self.get_register(call_params.index));
                let table = call_params.table;
                (index, table)
            }
            Instruction::CallIndirectParamsImm16(call_params) => {
                let index = u32::from(call_params.index);
                let table = call_params.table;
                (index, table)
            }
            unexpected => unreachable!(
                "expected `Instruction::CallIndirectParams[Imm16]` but found {unexpected:?}"
            ),
        }
    }

    /// Creates a [`CallFrame`] for calling the [`CompiledFunc`].
    fn dispatch_compiled_func(
        &mut self,
        results: RegisterSpan,
        func: &CompiledFuncEntity,
    ) -> Result<CallFrame, TrapCode> {
        let instrs = func.instrs();
        let instr_ptr = InstructionPtr::new(instrs.as_ptr());
        let (base_ptr, frame_ptr) = self.value_stack.alloc_call_frame(func)?;
        // We have to reinstantiate the `self.sp` [`ValueStackPtr`] since we just called
        // [`ValueStack::alloc_call_frame`] which might invalidate all live [`ValueStackPtr`].
        let caller = self
            .call_stack
            .peek()
            .expect("need to have a caller on the call stack");
        // Safety: We use the base offset of a live call frame on the call stack.
        self.sp = unsafe { self.value_stack.stack_ptr_at(caller.base_offset()) };
        let instance = caller.instance();
        let frame = CallFrame::new(instr_ptr, frame_ptr, base_ptr, results, *instance);
        Ok(frame)
    }

    /// Copies the parameters from `src` for the called [`CallFrame`].
    ///
    /// This will also adjust the instruction pointer to point to the
    /// last call parameter [`Instruction`] if any.
    #[must_use]
    fn copy_call_params(&mut self, mut called_regs: ValueStackPtr) -> InstructionPtr {
        let mut dst = Register::from_i16(0);
        let mut ip = self.ip;
        let mut copy_params = |values: &[Register]| {
            for value in values {
                let value = self.get_register(*value);
                // Safety: The `callee.results()` always refer to a span of valid
                //         registers of the `caller` that does not overlap with the
                //         registers of the callee since they reside in different
                //         call frames. Therefore this access is safe.
                let cell = unsafe { called_regs.get_mut(dst) };
                *cell = value;
                dst = dst.next();
            }
        };
        ip.add(1);
        while let Instruction::RegisterList(values) = ip.get() {
            copy_params(values);
            ip.add(1);
        }
        let values = match ip.get() {
            Instruction::Register(value) => slice::from_ref(value),
            Instruction::Register2(values) => values,
            Instruction::Register3(values) => values,
            unexpected => {
                unreachable!(
                    "unexpected Instruction found while copying call parameters: {unexpected:?}"
                )
            }
        };
        copy_params(values);
        // Finally return the instruction pointer to the last call parameter [`Instruction`] if any.
        ip
    }

    /// Prepares a [`CompiledFunc`] call with optional [`CallParams`].
    fn prepare_compiled_func_call(
        &mut self,
        results: RegisterSpan,
        func: CompiledFunc,
        params: CallParams,
        call_kind: CallKind,
    ) -> Result<(), TrapCode> {
        let func = self.code_map.get(func);
        let mut called = self.dispatch_compiled_func(results, func)?;
        if let CallParams::Some = params {
            let called_sp = self.frame_stack_ptr(&called);
            self.ip = self.copy_call_params(called_sp);
        }
        match call_kind {
            CallKind::Nested => {
                // We need to update the instruction pointer of the caller call frame.
                self.update_instr_ptr_at(1);
            }
            CallKind::Tail => {
                // In case of a tail call we have to remove the caller call frame after
                // allocating the callee call frame. This moves all cells of the callee frame
                // and may invalidate pointers to it.
                //
                // Safety:
                //
                // We provide `merge_call_frames` properly with `frame` that has just been allocated
                // on the value stack which is what the function expects. After this operation we ensure
                // that `self.sp` is adjusted via a call to `init_call_frame` since it may have been
                // invalidated by this method.
                unsafe { Stack::merge_call_frames(self.call_stack, self.value_stack, &mut called) };
            }
        }
        self.init_call_frame(&called);
        self.call_stack.push(called)?;
        Ok(())
    }

    /// Executes an [`Instruction::ReturnCallInternal0`].
    #[inline(always)]
    pub fn execute_return_call_internal_0(&mut self, func: CompiledFunc) -> Result<(), TrapCode> {
        self.execute_return_call_internal_impl(func, CallParams::None)
    }

    /// Executes an [`Instruction::ReturnCallInternal`].
    #[inline(always)]
    pub fn execute_return_call_internal(&mut self, func: CompiledFunc) -> Result<(), TrapCode> {
        self.execute_return_call_internal_impl(func, CallParams::Some)
    }

    /// Executes an [`Instruction::ReturnCallInternal`] or [`Instruction::ReturnCallInternal0`].
    fn execute_return_call_internal_impl(
        &mut self,
        func: CompiledFunc,
        params: CallParams,
    ) -> Result<(), TrapCode> {
        let results = self.caller_results();
        self.prepare_compiled_func_call(results, func, params, CallKind::Tail)
    }

    /// Returns the `results` [`RegisterSpan`] of the top-most [`CallFrame`] on the [`CallStack`].
    ///
    /// # Note
    ///
    /// We refer to the top-most [`CallFrame`] as the `caller` since this method is used for
    /// tail call instructions for which the top-most [`CallFrame`] is the caller.
    ///
    /// [`CallStack`]: crate::engine::regmach::stack::CallStack
    fn caller_results(&self) -> RegisterSpan {
        self.call_stack
            .peek()
            .expect("must have caller on the stack")
            .results()
    }

    /// Executes an [`Instruction::CallInternal0`].
    #[inline(always)]
    pub fn execute_call_internal_0(
        &mut self,
        results: RegisterSpan,
        func: CompiledFunc,
    ) -> Result<(), TrapCode> {
        self.prepare_compiled_func_call(results, func, CallParams::None, CallKind::Nested)
    }

    /// Executes an [`Instruction::CallInternal`].
    #[inline(always)]
    pub fn execute_call_internal(
        &mut self,
        results: RegisterSpan,
        func: CompiledFunc,
    ) -> Result<(), TrapCode> {
        self.prepare_compiled_func_call(results, func, CallParams::Some, CallKind::Nested)
    }

    /// Executes an [`Instruction::ReturnCallImported0`].
    #[inline(always)]
    pub fn execute_return_call_imported_0(
        &mut self,
        func: FuncIdx,
    ) -> Result<CallOutcome, TrapCode> {
        self.execute_return_call_imported_impl(func, CallParams::None)
    }

    /// Executes an [`Instruction::ReturnCallImported`].
    #[inline(always)]
    pub fn execute_return_call_imported(&mut self, func: FuncIdx) -> Result<CallOutcome, TrapCode> {
        self.execute_return_call_imported_impl(func, CallParams::Some)
    }

    /// Executes an [`Instruction::ReturnCallImported`] or [`Instruction::ReturnCallImported0`].
    fn execute_return_call_imported_impl(
        &mut self,
        func: FuncIdx,
        params: CallParams,
    ) -> Result<CallOutcome, TrapCode> {
        let func = self.cache.get_func(self.ctx, func);
        let results = self.caller_results();
        self.execute_call_imported_impl(results, &func, params, CallKind::Tail)
    }

    /// Executes an [`Instruction::CallImported0`].
    #[inline(always)]
    pub fn execute_call_imported_0(
        &mut self,
        results: RegisterSpan,
        func: FuncIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let func = self.cache.get_func(self.ctx, func);
        self.execute_call_imported_impl(results, &func, CallParams::None, CallKind::Nested)
    }

    /// Executes an [`Instruction::CallImported`].
    #[inline(always)]
    pub fn execute_call_imported(
        &mut self,
        results: RegisterSpan,
        func: FuncIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let func = self.cache.get_func(self.ctx, func);
        self.execute_call_imported_impl(results, &func, CallParams::Some, CallKind::Nested)
    }

    /// Executes an imported or indirect (tail) call instruction.
    fn execute_call_imported_impl(
        &mut self,
        results: RegisterSpan,
        func: &Func,
        params: CallParams,
        call_kind: CallKind,
    ) -> Result<CallOutcome, TrapCode> {
        match self.ctx.resolve_func(func) {
            FuncEntity::Wasm(func) => {
                let instance = *func.instance();
                self.prepare_compiled_func_call(results, func.func_body(), params, call_kind)?;
                self.cache.update_instance(&instance);
                Ok(CallOutcome::Continue)
            }
            FuncEntity::Host(host_func) => {
                let (input_types, output_types) = self
                    .func_types
                    .resolve_func_type(host_func.ty_dedup())
                    .params_results();
                let len_params = input_types.len();
                let len_results = output_types.len();
                let max_inout = len_params.max(len_results);
                self.value_stack.reserve(max_inout)?;
                // We have to reinstantiate the `self.sp` [`ValueStackPtr`] since we just called
                // [`ValueStack::reserve`] which might invalidate all live [`ValueStackPtr`].
                let caller = self
                    .call_stack
                    .peek()
                    .expect("need to have a caller on the call stack");
                // Safety: We use the base offset of a live call frame on the call stack.
                self.sp = unsafe { self.value_stack.stack_ptr_at(caller.base_offset()) };
                let offset = self.value_stack.extend_zeros(max_inout);
                let offset_sp = unsafe { self.value_stack.stack_ptr_at(offset) };
                if matches!(params, CallParams::Some) {
                    let new_ip = self.copy_call_params(offset_sp);
                    if matches!(call_kind, CallKind::Nested) {
                        self.ip = new_ip;
                    }
                }
                if matches!(call_kind, CallKind::Nested) {
                    self.update_instr_ptr_at(1);
                }
                self.cache.reset();
                Ok(CallOutcome::Call {
                    results,
                    host_func: *func,
                    call_kind,
                })
            }
        }
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(always)]
    pub fn execute_return_call_indirect_0(
        &mut self,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let (index, table) = self.pull_call_indirect_params();
        let results = self.caller_results();
        self.execute_call_indirect_impl(
            results,
            func_type,
            index,
            table,
            CallParams::None,
            CallKind::Tail,
        )
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(always)]
    pub fn execute_return_call_indirect(
        &mut self,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let (index, table) = self.pull_call_indirect_params();
        let results = self.caller_results();
        self.execute_call_indirect_impl(
            results,
            func_type,
            index,
            table,
            CallParams::Some,
            CallKind::Tail,
        )
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(always)]
    pub fn execute_call_indirect_0(
        &mut self,
        results: RegisterSpan,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let (index, table) = self.pull_call_indirect_params();
        self.execute_call_indirect_impl(
            results,
            func_type,
            index,
            table,
            CallParams::None,
            CallKind::Nested,
        )
    }

    /// Executes an [`Instruction::CallIndirect`].
    #[inline(always)]
    pub fn execute_call_indirect(
        &mut self,
        results: RegisterSpan,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let (index, table) = self.pull_call_indirect_params();
        self.execute_call_indirect_impl(
            results,
            func_type,
            index,
            table,
            CallParams::Some,
            CallKind::Nested,
        )
    }

    /// Executes an [`Instruction::CallIndirect`] and [`Instruction::CallIndirect0`].
    fn execute_call_indirect_impl(
        &mut self,
        results: RegisterSpan,
        func_type: SignatureIdx,
        index: u32,
        table: TableIdx,
        params: CallParams,
        call_kind: CallKind,
    ) -> Result<CallOutcome, TrapCode> {
        let table = self.cache.get_table(self.ctx, table);
        let funcref = self
            .ctx
            .resolve_table(&table)
            .get_untyped(index)
            .map(FuncRef::from)
            .ok_or(TrapCode::TableOutOfBounds)?;
        let func = funcref.func().ok_or(TrapCode::IndirectCallToNull)?;
        let actual_signature = self.ctx.resolve_func(func).ty_dedup();
        let expected_signature = self
            .ctx
            .resolve_instance(self.cache.instance())
            .get_signature(func_type.to_u32())
            .unwrap_or_else(|| {
                panic!("missing signature for call_indirect at index: {func_type:?}")
            });
        if actual_signature != expected_signature {
            return Err(TrapCode::BadSignature);
        }
        self.execute_call_imported_impl(results, func, params, call_kind)
    }
}
