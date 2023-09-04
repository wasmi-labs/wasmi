use super::Executor;
use crate::{
    core::TrapCode,
    engine::{
        bytecode::{FuncIdx, SignatureIdx, TableIdx},
        bytecode2::{CallParams, Instruction, Register, RegisterSpan, RegisterSpanIter},
        code_map::{CompiledFuncEntity, InstructionPtr2 as InstructionPtr},
        regmach::stack::{CallFrame, Stack},
        CompiledFunc,
    },
    func::FuncEntity,
    Func,
    FuncRef,
};

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

/// Resolved [`Instruction::CallIndirectParams`] or [`Instruction::CallIndirectParamsImm16`].
pub struct ResolvedCallIndirectParams {
    /// The index of the called function in the table.
    pub index: u32,
    /// The table which holds the called function at the index.
    pub table: TableIdx,
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

    /// Fetches the [`Instruction::CallParams`] parameter for a call [`Instruction`].
    ///
    /// # Note
    ///
    ///
    /// - This is done by encoding an [`Instruction::TableGet`] instruction
    ///   word following the actual instruction where the [`TableIdx`]
    ///   paremeter belongs to.
    /// - This is required for some instructions that do not fit into
    ///   a single instruction word and store a [`TableIdx`] value in
    ///   another instruction word.
    fn fetch_call_params(&self, offset: usize) -> CallParams {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match addr.get() {
            Instruction::CallParams(call_params) => *call_params,
            unexpected => unreachable!(
                "expected Instruction::CallParams at this address but found {unexpected:?}"
            ),
        }
    }

    /// Fetches the [`Instruction::CallIndirectParams`] parameter for a call [`Instruction`].
    ///
    /// # Note
    ///
    ///
    /// - This is done by encoding an [`Instruction::TableGet`] instruction
    ///   word following the actual instruction where the [`TableIdx`]
    ///   paremeter belongs to.
    /// - This is required for some instructions that do not fit into
    ///   a single instruction word and store a [`TableIdx`] value in
    ///   another instruction word.
    fn fetch_call_indirect_params(&self, offset: usize) -> ResolvedCallIndirectParams {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match addr.get() {
            Instruction::CallIndirectParams(call_params) => {
                let index = u32::from(self.get_register(call_params.index));
                let table = call_params.table;
                ResolvedCallIndirectParams { index, table }
            }
            Instruction::CallIndirectParamsImm16(call_params) => {
                let index = u32::from(call_params.index);
                let table = call_params.table;
                ResolvedCallIndirectParams { index, table }
            }
            unexpected => unreachable!(
                "expected Instruction::CallIndirectParams at this address but found {unexpected:?}"
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
    fn copy_call_params(&mut self, called: &CallFrame, src: RegisterSpan, len_params: usize) {
        let mut frame_sp = self.frame_stack_ptr(called);
        let src: RegisterSpanIter = src.iter(len_params);
        let dst = RegisterSpan::new(Register::from_i16(0)).iter(len_params);
        for (dst, src) in dst.zip(src) {
            // Safety: TODO
            let cell = unsafe { frame_sp.get_mut(dst) };
            *cell = self.get_register(src);
        }
    }

    /// Prepares a [`CompiledFunc`] call with optional [`CallParams`].
    fn prepare_compiled_func_call(
        &mut self,
        results: RegisterSpan,
        func: CompiledFunc,
        call_params: Option<&CallParams>,
        call_kind: CallKind,
    ) -> Result<(), TrapCode> {
        let func = self.code_map.get(func);
        let mut frame = self.dispatch_compiled_func(results, func)?;
        if let Some(call_params) = call_params {
            let len_params = call_params.len_params as usize;
            let src = call_params.params;
            self.copy_call_params(&frame, src, len_params);
        }
        if matches!(call_kind, CallKind::Tail) {
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
            unsafe { Stack::merge_call_frames(self.call_stack, self.value_stack, &mut frame) };
        }
        self.init_call_frame(&frame);
        self.call_stack.push(frame)?;
        Ok(())
    }

    /// Executes an [`Instruction::ReturnCallInternal0`].
    #[inline(always)]
    pub fn execute_return_call_internal_0(&mut self, func: CompiledFunc) -> Result<(), TrapCode> {
        self.update_instr_ptr_at(1);
        self.execute_return_call_internal_impl(func, None)?;
        Ok(())
    }

    /// Executes an [`Instruction::ReturnCallInternal`].
    #[inline(always)]
    pub fn execute_return_call_internal(&mut self, func: CompiledFunc) -> Result<(), TrapCode> {
        let call_params = self.fetch_call_params(1);
        self.update_instr_ptr_at(2);
        self.execute_return_call_internal_impl(func, Some(&call_params))?;
        Ok(())
    }

    /// Executes an [`Instruction::ReturnCallInternal`] or [`Instruction::ReturnCallInternal0`].
    fn execute_return_call_internal_impl(
        &mut self,
        func: CompiledFunc,
        call_params: Option<&CallParams>,
    ) -> Result<(), TrapCode> {
        let results = self.caller_results();
        self.prepare_compiled_func_call(results, func, call_params, CallKind::Tail)?;
        Ok(())
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
        self.update_instr_ptr_at(1);
        self.prepare_compiled_func_call(results, func, None, CallKind::Nested)?;
        Ok(())
    }

    /// Executes an [`Instruction::CallInternal`].
    #[inline(always)]
    pub fn execute_call_internal(
        &mut self,
        results: RegisterSpan,
        func: CompiledFunc,
    ) -> Result<(), TrapCode> {
        let call_params = self.fetch_call_params(1);
        self.update_instr_ptr_at(2);
        self.prepare_compiled_func_call(results, func, Some(&call_params), CallKind::Nested)?;
        Ok(())
    }

    /// Executes an [`Instruction::ReturnCallImported0`].
    #[inline(always)]
    pub fn execute_return_call_imported_0(
        &mut self,
        func: FuncIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let func = self.cache.get_func(self.ctx, func);
        self.update_instr_ptr_at(1);
        self.execute_return_call_imported_impl(&func, None)
    }

    /// Executes an [`Instruction::ReturnCallImported`].
    #[inline(always)]
    pub fn execute_return_call_imported(&mut self, func: FuncIdx) -> Result<CallOutcome, TrapCode> {
        let call_params = self.fetch_call_params(1);
        let func = self.cache.get_func(self.ctx, func);
        self.update_instr_ptr_at(2);
        self.execute_return_call_imported_impl(&func, Some(&call_params))
    }

    /// Executes an [`Instruction::ReturnCallImported`] or [`Instruction::ReturnCallImported0`].
    fn execute_return_call_imported_impl(
        &mut self,
        func: &Func,
        call_params: Option<&CallParams>,
    ) -> Result<CallOutcome, TrapCode> {
        let results = self.caller_results();
        self.execute_call_imported_impl(results, func, call_params, CallKind::Tail)
    }

    /// Executes an [`Instruction::CallImported0`].
    #[inline(always)]
    pub fn execute_call_imported_0(
        &mut self,
        results: RegisterSpan,
        func: FuncIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let func = self.cache.get_func(self.ctx, func);
        self.update_instr_ptr_at(1);
        self.execute_call_imported_impl(results, &func, None, CallKind::Nested)
    }

    /// Executes an [`Instruction::CallImported`].
    #[inline(always)]
    pub fn execute_call_imported(
        &mut self,
        results: RegisterSpan,
        func: FuncIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let call_params = self.fetch_call_params(1);
        let func = self.cache.get_func(self.ctx, func);
        self.update_instr_ptr_at(2);
        self.execute_call_imported_impl(results, &func, Some(&call_params), CallKind::Nested)
    }

    /// Executes an imported or indirect (tail) call instruction.
    fn execute_call_imported_impl(
        &mut self,
        results: RegisterSpan,
        func: &Func,
        call_params: Option<&CallParams>,
        call_kind: CallKind,
    ) -> Result<CallOutcome, TrapCode> {
        match self.ctx.resolve_func(func) {
            FuncEntity::Wasm(func) => {
                let instance = *func.instance();
                self.prepare_compiled_func_call(results, func.func_body(), call_params, call_kind)?;
                self.cache.update_instance(&instance);
                Ok(CallOutcome::Continue)
            }
            FuncEntity::Host(_host_func) => {
                // Note: host function calls cannot be implemented as tail calls.
                //       The Wasm spec is not mandating tail behavior for host calls.
                //
                // TODO: copy parameters for the host function call
                self.cache.reset();
                Ok(CallOutcome::Call(*func))
            }
        }
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(always)]
    pub fn execute_return_call_indirect_0(
        &mut self,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let call_indirect_params = self.fetch_call_indirect_params(1);
        self.update_instr_ptr_at(2);
        let results = self.caller_results();
        self.execute_call_indirect_impl(
            results,
            func_type,
            &call_indirect_params,
            None,
            CallKind::Tail,
        )
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(always)]
    pub fn execute_return_call_indirect(
        &mut self,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, TrapCode> {
        let call_indirect_params = self.fetch_call_indirect_params(1);
        let call_params = self.fetch_call_params(2);
        self.update_instr_ptr_at(3);
        let results = self.caller_results();
        self.execute_call_indirect_impl(
            results,
            func_type,
            &call_indirect_params,
            Some(&call_params),
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
        let call_indirect_params = self.fetch_call_indirect_params(1);
        self.update_instr_ptr_at(2);
        self.execute_call_indirect_impl(
            results,
            func_type,
            &call_indirect_params,
            None,
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
        let call_indirect_params = self.fetch_call_indirect_params(1);
        let call_params = self.fetch_call_params(2);
        self.update_instr_ptr_at(3);
        self.execute_call_indirect_impl(
            results,
            func_type,
            &call_indirect_params,
            Some(&call_params),
            CallKind::Nested,
        )
    }

    /// Executes an [`Instruction::CallIndirect`] and [`Instruction::CallIndirect0`].
    fn execute_call_indirect_impl(
        &mut self,
        results: RegisterSpan,
        func_type: SignatureIdx,
        call_indirect_params: &ResolvedCallIndirectParams,
        call_params: Option<&CallParams>,
        call_kind: CallKind,
    ) -> Result<CallOutcome, TrapCode> {
        let index = call_indirect_params.index;
        let table = call_indirect_params.table;
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
            return Err(TrapCode::BadSignature).map_err(Into::into);
        }
        self.execute_call_imported_impl(results, func, call_params, call_kind)
    }
}
