use core::array;

use super::Executor;
use crate::{
    core::TrapCode,
    engine::{
        bytecode::{FuncIdx, Instruction, Register, RegisterSpan, SignatureIdx, TableIdx},
        code_map::InstructionPtr,
        executor::stack::{CallFrame, FrameRegisters, Stack},
        CompiledFunc,
        CompiledFuncEntity,
    },
    func::FuncEntity,
    Error,
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

trait CallContext {
    const KIND: CallKind;
    const HAS_PARAMS: bool;
}
trait ReturnCallContext: CallContext {}

mod marker {
    use super::{CallContext, CallKind, ReturnCallContext};

    pub enum ReturnCall0 {}
    impl CallContext for ReturnCall0 {
        const KIND: CallKind = CallKind::Tail;
        const HAS_PARAMS: bool = false;
    }
    impl ReturnCallContext for ReturnCall0 {}

    pub enum ReturnCall {}
    impl CallContext for ReturnCall {
        const KIND: CallKind = CallKind::Tail;
        const HAS_PARAMS: bool = true;
    }
    impl ReturnCallContext for ReturnCall {}

    pub enum NestedCall0 {}
    impl CallContext for NestedCall0 {
        const KIND: CallKind = CallKind::Nested;
        const HAS_PARAMS: bool = false;
    }

    pub enum NestedCall {}
    impl CallContext for NestedCall {
        const KIND: CallKind = CallKind::Nested;
        const HAS_PARAMS: bool = true;
    }
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Updates the [`InstructionPtr`] of the caller [`CallFrame`] before dispatching a call.
    ///
    /// # Note
    ///
    /// The `offset` denotes how many [`Instruction`] words make up the call instruction.
    #[inline]
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
    #[inline(always)]
    fn dispatch_compiled_func(
        &mut self,
        results: RegisterSpan,
        func: &CompiledFuncEntity,
    ) -> Result<CallFrame, Error> {
        let (base_ptr, frame_ptr) = self.value_stack.alloc_call_frame(func)?;
        // We have to reinstantiate the `self.sp` [`FrameRegisters`] since we just called
        // [`ValueStack::alloc_call_frame`] which might invalidate all live [`FrameRegisters`].
        let caller = self
            .call_stack
            .peek()
            .expect("need to have a caller on the call stack");
        // Safety: We use the base offset of a live call frame on the call stack.
        self.sp = unsafe { self.value_stack.stack_ptr_at(caller.base_offset()) };
        let instance = caller.instance();
        let instr_ptr = InstructionPtr::new(func.instrs().as_ptr());
        let frame = CallFrame::new(instr_ptr, frame_ptr, base_ptr, results, *instance);
        Ok(frame)
    }

    /// Copies the parameters from caller for the callee [`CallFrame`].
    ///
    /// This will also adjust the instruction pointer to point to the
    /// last call parameter [`Instruction`] if any.
    #[inline(always)]
    fn copy_call_params(&mut self, mut callee_regs: FrameRegisters) {
        let mut dst = Register::from_i16(0);
        self.ip.add(1);
        if let Instruction::RegisterList(_) = self.ip.get() {
            self.copy_call_params_list(&mut dst, &mut callee_regs);
        }
        match self.ip.get() {
            Instruction::Register(value) => {
                self.copy_regs(&mut dst, &mut callee_regs, array::from_ref(value));
            }
            Instruction::Register2(values) => {
                self.copy_regs(&mut dst, &mut callee_regs, values);
            }
            Instruction::Register3(values) => {
                self.copy_regs(&mut dst, &mut callee_regs, values);
            }
            unexpected => {
                unreachable!(
                    "unexpected Instruction found while copying call parameters: {unexpected:?}"
                )
            }
        }
    }

    /// Copies an array of [`Register`] to the `dst` [`Register`] span.
    #[inline(always)]
    fn copy_regs<const N: usize>(
        &self,
        dst: &mut Register,
        callee_regs: &mut FrameRegisters,
        regs: &[Register; N],
    ) {
        for value in regs {
            let value = self.get_register(*value);
            // Safety: The `callee.results()` always refer to a span of valid
            //         registers of the `caller` that does not overlap with the
            //         registers of the callee since they reside in different
            //         call frames. Therefore this access is safe.
            unsafe { callee_regs.set(*dst, value) }
            *dst = dst.next();
        }
    }

    /// Copies a list of [`Instruction::RegisterList`] to the `dst` [`Register`] span.
    /// Copies the parameters from `src` for the called [`CallFrame`].
    ///
    /// This will make the [`InstructionPtr`] point to the [`Instruction`] following the
    /// last [`Instruction::RegisterList`] if any.
    #[inline]
    #[cold]
    fn copy_call_params_list(&mut self, dst: &mut Register, callee_regs: &mut FrameRegisters) {
        while let Instruction::RegisterList(values) = self.ip.get() {
            self.copy_regs(dst, callee_regs, values);
            self.ip.add(1);
        }
    }

    /// Prepares a [`CompiledFunc`] call with optional call parameters.
    #[inline(always)]
    fn prepare_compiled_func_call<C: CallContext>(
        &mut self,
        results: RegisterSpan,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        let func = self.code_map.get(Some(self.ctx.fuel_mut()), func)?;
        let mut called = self.dispatch_compiled_func(results, func)?;
        if <C as CallContext>::HAS_PARAMS {
            let called_sp = self.frame_stack_ptr(&called);
            self.copy_call_params(called_sp);
        }
        match <C as CallContext>::KIND {
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
    pub fn execute_return_call_internal_0(&mut self, func: CompiledFunc) -> Result<(), Error> {
        self.execute_return_call_internal_impl::<marker::ReturnCall0>(func)
    }

    /// Executes an [`Instruction::ReturnCallInternal`].
    #[inline(always)]
    pub fn execute_return_call_internal(&mut self, func: CompiledFunc) -> Result<(), Error> {
        self.execute_return_call_internal_impl::<marker::ReturnCall>(func)
    }

    /// Executes an [`Instruction::ReturnCallInternal`] or [`Instruction::ReturnCallInternal0`].
    fn execute_return_call_internal_impl<C: CallContext>(
        &mut self,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        let results = self.caller_results();
        self.prepare_compiled_func_call::<C>(results, func)
    }

    /// Returns the `results` [`RegisterSpan`] of the top-most [`CallFrame`] on the [`CallStack`].
    ///
    /// # Note
    ///
    /// We refer to the top-most [`CallFrame`] as the `caller` since this method is used for
    /// tail call instructions for which the top-most [`CallFrame`] is the caller.
    ///
    /// [`CallStack`]: crate::engine::executor::stack::CallStack
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
    ) -> Result<(), Error> {
        self.prepare_compiled_func_call::<marker::NestedCall0>(results, func)
    }

    /// Executes an [`Instruction::CallInternal`].
    #[inline(always)]
    pub fn execute_call_internal(
        &mut self,
        results: RegisterSpan,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        self.prepare_compiled_func_call::<marker::NestedCall>(results, func)
    }

    /// Executes an [`Instruction::ReturnCallImported0`].
    #[inline(always)]
    pub fn execute_return_call_imported_0(&mut self, func: FuncIdx) -> Result<CallOutcome, Error> {
        self.execute_return_call_imported_impl::<marker::ReturnCall0>(func)
    }

    /// Executes an [`Instruction::ReturnCallImported`].
    #[inline(always)]
    pub fn execute_return_call_imported(&mut self, func: FuncIdx) -> Result<CallOutcome, Error> {
        self.execute_return_call_imported_impl::<marker::ReturnCall>(func)
    }

    /// Executes an [`Instruction::ReturnCallImported`] or [`Instruction::ReturnCallImported0`].
    fn execute_return_call_imported_impl<C: ReturnCallContext>(
        &mut self,
        func: FuncIdx,
    ) -> Result<CallOutcome, Error> {
        let func = self.cache.get_func(self.ctx, func);
        let results = self.caller_results();
        self.execute_call_imported_impl::<C>(results, &func)
    }

    /// Executes an [`Instruction::CallImported0`].
    #[inline(never)]
    pub fn execute_call_imported_0(
        &mut self,
        results: RegisterSpan,
        func: FuncIdx,
    ) -> Result<CallOutcome, Error> {
        let func = self.cache.get_func(self.ctx, func);
        self.execute_call_imported_impl::<marker::NestedCall0>(results, &func)
    }

    /// Executes an [`Instruction::CallImported`].
    #[inline(never)]
    pub fn execute_call_imported(
        &mut self,
        results: RegisterSpan,
        func: FuncIdx,
    ) -> Result<CallOutcome, Error> {
        let func = self.cache.get_func(self.ctx, func);
        self.execute_call_imported_impl::<marker::NestedCall>(results, &func)
    }

    /// Executes an imported or indirect (tail) call instruction.
    fn execute_call_imported_impl<C: CallContext>(
        &mut self,
        results: RegisterSpan,
        func: &Func,
    ) -> Result<CallOutcome, Error> {
        match self.ctx.resolve_func(func) {
            FuncEntity::Wasm(func) => {
                let instance = *func.instance();
                self.prepare_compiled_func_call::<C>(results, func.func_body())?;
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
                // We have to reinstantiate the `self.sp` [`FrameRegisters`] since we just called
                // [`ValueStack::reserve`] which might invalidate all live [`FrameRegisters`].
                let caller = self
                    .call_stack
                    .peek()
                    .expect("need to have a caller on the call stack");
                // Safety: we use the base offset of a live call frame on the call stack.
                self.sp = unsafe { self.value_stack.stack_ptr_at(caller.base_offset()) };
                // Safety: we just called reserve to fit the new values.
                let offset = unsafe { self.value_stack.extend_zeros(max_inout) };
                let offset_sp = unsafe { self.value_stack.stack_ptr_at(offset) };
                if <C as CallContext>::HAS_PARAMS {
                    self.copy_call_params(offset_sp);
                }
                if matches!(<C as CallContext>::KIND, CallKind::Nested) {
                    self.update_instr_ptr_at(1);
                }
                self.cache.reset();
                Ok(CallOutcome::Call {
                    results,
                    host_func: *func,
                    call_kind: <C as CallContext>::KIND,
                })
            }
        }
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(never)]
    pub fn execute_return_call_indirect_0(
        &mut self,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, Error> {
        let (index, table) = self.pull_call_indirect_params();
        let results = self.caller_results();
        self.execute_call_indirect_impl::<marker::ReturnCall0>(results, func_type, index, table)
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(never)]
    pub fn execute_return_call_indirect(
        &mut self,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, Error> {
        let (index, table) = self.pull_call_indirect_params();
        let results = self.caller_results();
        self.execute_call_indirect_impl::<marker::ReturnCall>(results, func_type, index, table)
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(never)]
    pub fn execute_call_indirect_0(
        &mut self,
        results: RegisterSpan,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, Error> {
        let (index, table) = self.pull_call_indirect_params();
        self.execute_call_indirect_impl::<marker::NestedCall0>(results, func_type, index, table)
    }

    /// Executes an [`Instruction::CallIndirect`].
    #[inline(never)]
    pub fn execute_call_indirect(
        &mut self,
        results: RegisterSpan,
        func_type: SignatureIdx,
    ) -> Result<CallOutcome, Error> {
        let (index, table) = self.pull_call_indirect_params();
        self.execute_call_indirect_impl::<marker::NestedCall>(results, func_type, index, table)
    }

    /// Executes an [`Instruction::CallIndirect`] and [`Instruction::CallIndirect0`].
    fn execute_call_indirect_impl<C: CallContext>(
        &mut self,
        results: RegisterSpan,
        func_type: SignatureIdx,
        index: u32,
        table: TableIdx,
    ) -> Result<CallOutcome, Error> {
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
            return Err(Error::from(TrapCode::BadSignature));
        }
        self.execute_call_imported_impl::<C>(results, func)
    }
}
