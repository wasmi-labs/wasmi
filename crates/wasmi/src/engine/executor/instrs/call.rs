use super::Executor;
use crate::{
    core::TrapCode,
    engine::{
        bytecode::{FuncIdx, Instruction, Register, RegisterSpan, SignatureIdx, TableIdx},
        code_map::InstructionPtr,
        executor::stack::{CallFrame, FrameRegisters, Stack, ValueStack},
        func_types::FuncTypeRegistry,
        CompiledFunc,
        CompiledFuncEntity,
        FuncParams,
    },
    func::{FuncEntity, HostFuncEntity},
    store::StoreInner,
    Error,
    Func,
    FuncRef,
    Instance,
    Store,
};
use core::array;
use std::fmt;

/// Dispatches and executes the host function.
///
/// Returns the number of parameters and results of the called host function.
///
/// # Errors
///
/// Returns the error of the host function if an error occurred.
pub fn dispatch_host_func<T>(
    store: &mut Store<T>,
    func_types: &FuncTypeRegistry,
    value_stack: &mut ValueStack,
    host_func: HostFuncEntity,
    instance: Option<&Instance>,
) -> Result<(usize, usize), Error> {
    let (input_types, output_types) = func_types
        .resolve_func_type(host_func.ty_dedup())
        .params_results();
    let len_inputs = input_types.len();
    let len_outputs = output_types.len();
    let max_inout = len_inputs.max(len_outputs);
    let values = value_stack.as_slice_mut();
    let params_results = FuncParams::new(
        values.split_at_mut(values.len() - max_inout).1,
        len_inputs,
        len_outputs,
    );
    let trampoline = store.resolve_trampoline(host_func.trampoline()).clone();
    trampoline
        .call(store, instance, params_results)
        .map_err(|error| {
            // Note: We drop the values that have been temporarily added to
            //       the stack to act as parameter and result buffer for the
            //       called host function. Since the host function failed we
            //       need to clean up the temporary buffer values here.
            //       This is required for resumable calls to work properly.
            value_stack.drop(max_inout);
            error
        })?;
    Ok((len_inputs, len_outputs))
}

/// The kind of a function call.
#[derive(Debug, Copy, Clone)]
pub enum CallKind {
    /// A nested function call.
    Nested,
    /// A tailing function call.
    Tail,
}

/// Error returned from a called host function in a resumable state.
#[derive(Debug)]
pub struct ResumableHostError {
    host_error: Error,
    host_func: Func,
    caller_results: RegisterSpan,
}

impl fmt::Display for ResumableHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.host_error.fmt(f)
    }
}

impl ResumableHostError {
    /// Creates a new [`ResumableHostError`].
    #[cold]
    pub(crate) fn new(host_error: Error, host_func: Func, caller_results: RegisterSpan) -> Self {
        Self {
            host_error,
            host_func,
            caller_results,
        }
    }

    /// Consumes `self` to return the underlying [`Error`].
    pub(crate) fn into_error(self) -> Error {
        self.host_error
    }

    /// Returns the [`Func`] of the [`ResumableHostError`].
    pub(crate) fn host_func(&self) -> &Func {
        &self.host_func
    }

    /// Returns the caller results [`RegisterSpan`] of the [`ResumableHostError`].
    pub(crate) fn caller_results(&self) -> &RegisterSpan {
        &self.caller_results
    }
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

impl<'engine> Executor<'engine> {
    /// Updates the [`InstructionPtr`] of the caller [`CallFrame`] before dispatching a call.
    ///
    /// # Note
    ///
    /// The `offset` denotes how many [`Instruction`] words make up the call instruction.
    #[inline]
    fn update_instr_ptr_at(&mut self, offset: usize) {
        // Note: we explicitly do not mutate `self.ip` since that would make
        // other parts of the code more fragile with respect to instruction ordering.
        self.ip.add(offset);
        let caller = self
            .call_stack
            .peek_mut()
            .expect("caller call frame must be on the stack");
        caller.update_instr_ptr(self.ip);
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
    fn dispatch_compiled_func<C: CallContext>(
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
        if <C as CallContext>::HAS_PARAMS {
            let called_sp = self.frame_stack_ptr(&frame);
            self.copy_call_params(called_sp);
        }
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
        store: &mut StoreInner,
        results: RegisterSpan,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        let func = self.code_map.get(Some(store.fuel_mut()), func)?;
        let mut called = self.dispatch_compiled_func::<C>(results, func)?;
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
    pub fn execute_return_call_internal_0(
        &mut self,
        store: &mut StoreInner,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        self.execute_return_call_internal_impl::<marker::ReturnCall0>(store, func)
    }

    /// Executes an [`Instruction::ReturnCallInternal`].
    #[inline(always)]
    pub fn execute_return_call_internal(
        &mut self,
        store: &mut StoreInner,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        self.execute_return_call_internal_impl::<marker::ReturnCall>(store, func)
    }

    /// Executes an [`Instruction::ReturnCallInternal`] or [`Instruction::ReturnCallInternal0`].
    fn execute_return_call_internal_impl<C: CallContext>(
        &mut self,
        store: &mut StoreInner,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        let results = self.caller_results();
        self.prepare_compiled_func_call::<C>(store, results, func)
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
        store: &mut StoreInner,
        results: RegisterSpan,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        self.prepare_compiled_func_call::<marker::NestedCall0>(store, results, func)
    }

    /// Executes an [`Instruction::CallInternal`].
    #[inline(always)]
    pub fn execute_call_internal(
        &mut self,
        store: &mut StoreInner,
        results: RegisterSpan,
        func: CompiledFunc,
    ) -> Result<(), Error> {
        self.prepare_compiled_func_call::<marker::NestedCall>(store, results, func)
    }

    /// Executes an [`Instruction::ReturnCallImported0`].
    #[inline(always)]
    pub fn execute_return_call_imported_0<T>(
        &mut self,
        store: &mut Store<T>,
        func: FuncIdx,
    ) -> Result<(), Error> {
        self.execute_return_call_imported_impl::<marker::ReturnCall0, T>(store, func)
    }

    /// Executes an [`Instruction::ReturnCallImported`].
    #[inline(always)]
    pub fn execute_return_call_imported<T>(
        &mut self,
        store: &mut Store<T>,
        func: FuncIdx,
    ) -> Result<(), Error> {
        self.execute_return_call_imported_impl::<marker::ReturnCall, T>(store, func)
    }

    /// Executes an [`Instruction::ReturnCallImported`] or [`Instruction::ReturnCallImported0`].
    fn execute_return_call_imported_impl<C: ReturnCallContext, T>(
        &mut self,
        store: &mut Store<T>,
        func: FuncIdx,
    ) -> Result<(), Error> {
        let func = self.cache.get_func(&store.inner, func);
        let results = self.caller_results();
        self.execute_call_imported_impl::<C, T>(store, results, &func)
    }

    /// Executes an [`Instruction::CallImported0`].
    #[inline(never)]
    pub fn execute_call_imported_0<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegisterSpan,
        func: FuncIdx,
    ) -> Result<(), Error> {
        let func = self.cache.get_func(&store.inner, func);
        self.execute_call_imported_impl::<marker::NestedCall0, T>(store, results, &func)
    }

    /// Executes an [`Instruction::CallImported`].
    #[inline(never)]
    pub fn execute_call_imported<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegisterSpan,
        func: FuncIdx,
    ) -> Result<(), Error> {
        let func = self.cache.get_func(&store.inner, func);
        self.execute_call_imported_impl::<marker::NestedCall, T>(store, results, &func)
    }

    /// Executes an imported or indirect (tail) call instruction.
    fn execute_call_imported_impl<C: CallContext, T>(
        &mut self,
        store: &mut Store<T>,
        results: RegisterSpan,
        func: &Func,
    ) -> Result<(), Error> {
        match store.inner.resolve_func(func) {
            FuncEntity::Wasm(func) => {
                let instance = *func.instance();
                let func_body = func.func_body();
                self.prepare_compiled_func_call::<C>(&mut store.inner, results, func_body)?;
                self.cache.update_instance(&instance);
                Ok(())
            }
            FuncEntity::Host(host_func) => {
                self.execute_host_func::<C, T>(store, results, func, *host_func)
            }
        }
    }

    /// Executes a host function.
    ///
    /// # Note
    ///
    /// This uses the value stack to store paramters and results of the host function call.
    /// Returns an [`Error::ResumableHost`] variant if the host function returned an error
    /// and there are still call frames on the call stack making it possible to resume the
    /// execution at a later point in time.
    fn execute_host_func<C: CallContext, T>(
        &mut self,
        store: &mut Store<T>,
        results: RegisterSpan,
        func: &Func,
        host_func: HostFuncEntity,
    ) -> Result<(), Error> {
        let (input_types, output_types) = self
            .func_types
            .resolve_func_type(host_func.ty_dedup())
            .params_results();
        let len_params = input_types.len();
        let len_results = output_types.len();
        let max_inout = len_params.max(len_results);
        self.value_stack.reserve(max_inout)?;
        // Safety: we just called reserve to fit the new values.
        let offset = unsafe { self.value_stack.extend_zeros(max_inout) };
        let offset_sp = unsafe { self.value_stack.stack_ptr_at(offset) };
        // We have to reinstantiate the `self.sp` [`FrameRegisters`] since we just called
        // [`ValueStack::reserve`] which might invalidate all live [`FrameRegisters`].
        let caller = match <C as CallContext>::KIND {
            CallKind::Nested => self.call_stack.peek().copied(),
            CallKind::Tail => self.call_stack.pop(),
        }
        .expect("need to have a caller on the call stack");
        // Safety: we use the base offset of a live call frame on the call stack.
        self.sp = unsafe { self.value_stack.stack_ptr_at(caller.base_offset()) };
        if <C as CallContext>::HAS_PARAMS {
            self.copy_call_params(offset_sp);
        }
        if matches!(<C as CallContext>::KIND, CallKind::Nested) {
            self.update_instr_ptr_at(1);
        }
        self.cache.reset();
        let (len_inputs, len_outputs) = self
            .dispatch_host_func::<T>(store, host_func, caller)
            .map_err(|error| match self.call_stack.is_empty() {
                true => error,
                false => ResumableHostError::new(error, *func, results).into(),
            })?;
        // # Safety (1)
        //
        // We can safely acquire the stack pointer to the caller's and callee's (host)
        // call frames because we just allocated the host call frame and can be sure that
        // they are different.
        // In the following we make sure to not access registers out of bounds of each
        // call frame since we rely on Wasm validation and proper Wasm translation to
        // provide us with valid result registers.
        let mut caller_sp = unsafe { self.value_stack.stack_ptr_at(caller.base_offset()) };
        // # Safety: See Safety (1) above.
        let callee_sp = unsafe {
            self.value_stack
                .stack_ptr_last_n(len_inputs.max(len_outputs))
        };
        let results = results.iter(len_outputs);
        let values = RegisterSpan::new(Register::from_i16(0)).iter(len_outputs);
        for (result, value) in results.zip(values) {
            // # Safety: See Safety (1) above.
            unsafe { caller_sp.set(result, callee_sp.get(value)) };
        }
        // Finally, the value stack needs to be truncated to its original size.
        self.value_stack.drop(max_inout);
        Ok(())
    }

    /// Convenience forwarder to [`Executor::dispatch_host_func_impl`].
    fn dispatch_host_func<T>(
        &mut self,
        store: &mut Store<T>,
        host_func: HostFuncEntity,
        caller: CallFrame,
    ) -> Result<(usize, usize), Error> {
        dispatch_host_func(
            store,
            self.func_types,
            self.value_stack,
            host_func,
            Some(caller.instance()),
        )
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(never)]
    pub fn execute_return_call_indirect_0<T>(
        &mut self,
        store: &mut Store<T>,
        func_type: SignatureIdx,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params();
        let results = self.caller_results();
        self.execute_call_indirect_impl::<marker::ReturnCall0, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(never)]
    pub fn execute_return_call_indirect<T>(
        &mut self,
        store: &mut Store<T>,
        func_type: SignatureIdx,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params();
        let results = self.caller_results();
        self.execute_call_indirect_impl::<marker::ReturnCall, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect0`].
    #[inline(never)]
    pub fn execute_call_indirect_0<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegisterSpan,
        func_type: SignatureIdx,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params();
        self.execute_call_indirect_impl::<marker::NestedCall0, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect`].
    #[inline(never)]
    pub fn execute_call_indirect<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegisterSpan,
        func_type: SignatureIdx,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params();
        self.execute_call_indirect_impl::<marker::NestedCall, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect`] and [`Instruction::CallIndirect0`].
    fn execute_call_indirect_impl<C: CallContext, T>(
        &mut self,
        store: &mut Store<T>,
        results: RegisterSpan,
        func_type: SignatureIdx,
        index: u32,
        table: TableIdx,
    ) -> Result<(), Error> {
        let table = self.cache.get_table(&store.inner, table);
        let funcref = store
            .inner
            .resolve_table(&table)
            .get_untyped(index)
            .map(FuncRef::from)
            .ok_or(TrapCode::TableOutOfBounds)?;
        let func = funcref.func().ok_or(TrapCode::IndirectCallToNull)?;
        let actual_signature = store.inner.resolve_func(func).ty_dedup();
        let expected_signature = store
            .inner
            .resolve_instance(self.cache.instance())
            .get_signature(func_type.to_u32())
            .unwrap_or_else(|| {
                panic!("missing signature for call_indirect at index: {func_type:?}")
            });
        if actual_signature != expected_signature {
            return Err(Error::from(TrapCode::BadSignature));
        }
        self.execute_call_imported_impl::<C, T>(store, results, func)
    }
}
