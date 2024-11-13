use super::{Executor, InstructionPtr};
use crate::{
    core::TrapCode,
    engine::{
        code_map::CompiledFuncRef,
        executor::stack::{CallFrame, FrameParams, ValueStack},
        utils::unreachable_unchecked,
        EngineFunc,
        FuncParams,
    },
    func::{FuncEntity, HostFuncEntity},
    ir::{index, Instruction, Reg, RegSpan},
    store::StoreInner,
    CallHook,
    Error,
    Func,
    FuncRef,
    Instance,
    Store,
};
use core::array;
use core::fmt;

/// Dispatches and executes the host function.
///
/// Returns the number of parameters and results of the called host function.
///
/// # Errors
///
/// Returns the error of the host function if an error occurred.
pub fn dispatch_host_func<T>(
    store: &mut Store<T>,
    value_stack: &mut ValueStack,
    host_func: HostFuncEntity,
    instance: Option<&Instance>,
) -> Result<(u16, u16), Error> {
    let len_params = host_func.len_params();
    let len_results = host_func.len_results();
    let max_inout = len_params.max(len_results);
    let values = value_stack.as_slice_mut();
    let params_results = FuncParams::new(
        values.split_at_mut(values.len() - usize::from(max_inout)).1,
        usize::from(len_params),
        usize::from(len_results),
    );
    let trampoline = store.resolve_trampoline(host_func.trampoline()).clone();
    trampoline
        .call(store, instance, params_results)
        .inspect_err(|_error| {
            // Note: We drop the values that have been temporarily added to
            //       the stack to act as parameter and result buffer for the
            //       called host function. Since the host function failed we
            //       need to clean up the temporary buffer values here.
            //       This is required for resumable calls to work properly.
            value_stack.drop(usize::from(max_inout));
        })?;
    Ok((len_params, len_results))
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
    /// The error returned by the called host function.
    host_error: Error,
    /// The host function that returned the error.
    host_func: Func,
    /// The result registers of the caller of the host function.
    caller_results: RegSpan,
}

#[cfg(feature = "std")]
impl std::error::Error for ResumableHostError {}

impl fmt::Display for ResumableHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.host_error.fmt(f)
    }
}

impl ResumableHostError {
    /// Creates a new [`ResumableHostError`].
    #[cold]
    pub(crate) fn new(host_error: Error, host_func: Func, caller_results: RegSpan) -> Self {
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

    /// Returns the caller results [`RegSpan`] of the [`ResumableHostError`].
    pub(crate) fn caller_results(&self) -> &RegSpan {
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

impl Executor<'_> {
    /// Updates the [`InstructionPtr`] of the caller [`CallFrame`] before dispatching a call.
    ///
    /// # Note
    ///
    /// The `offset` denotes how many [`Instruction`] words make up the call instruction.
    fn update_instr_ptr_at(&mut self, offset: usize) {
        // Note: we explicitly do not mutate `self.ip` since that would make
        // other parts of the code more fragile with respect to instruction ordering.
        self.ip.add(offset);
        let caller = self
            .stack
            .calls
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
    ///   word following the actual instruction where the [`index::Table`]
    ///   paremeter belongs to.
    /// - This is required for some instructions that do not fit into
    ///   a single instruction word and store a [`index::Table`] value in
    ///   another instruction word.
    fn pull_call_indirect_params(&mut self) -> (u32, index::Table) {
        self.ip.add(1);
        match *self.ip.get() {
            Instruction::CallIndirectParams { index, table } => {
                let index = u32::from(self.get_register(index));
                (index, table)
            }
            unexpected => {
                // Safety: Wasmi translation guarantees that correct instruction parameter follows.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::CallIndirectParams` but found {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Fetches the [`Instruction::CallIndirectParamsImm16`] parameter for a call [`Instruction`].
    ///
    /// # Note
    ///
    /// - This advances the [`InstructionPtr`] to the next [`Instruction`].
    /// - This is done by encoding an [`Instruction::TableGet`] instruction
    ///   word following the actual instruction where the [`index::Table`]
    ///   paremeter belongs to.
    /// - This is required for some instructions that do not fit into
    ///   a single instruction word and store a [`index::Table`] value in
    ///   another instruction word.
    fn pull_call_indirect_params_imm16(&mut self) -> (u32, index::Table) {
        self.ip.add(1);
        match *self.ip.get() {
            Instruction::CallIndirectParamsImm16 { index, table } => {
                let index = u32::from(index);
                (index, table)
            }
            unexpected => {
                // Safety: Wasmi translation guarantees that correct instruction parameter follows.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::CallIndirectParamsImm16` but found {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Creates a [`CallFrame`] for calling the [`EngineFunc`].
    fn dispatch_compiled_func<C: CallContext>(
        &mut self,
        results: RegSpan,
        func: CompiledFuncRef,
    ) -> Result<CallFrame, Error> {
        // We have to reinstantiate the `self.sp` [`FrameRegisters`] since we just called
        // [`ValueStack::alloc_call_frame`] which might invalidate all live [`FrameRegisters`].
        let caller = self
            .stack
            .calls
            .peek()
            .expect("need to have a caller on the call stack");
        let (mut uninit_params, offsets) = self.stack.values.alloc_call_frame(func, |this| {
            // Safety: We use the base offset of a live call frame on the call stack.
            self.sp = unsafe { this.stack_ptr_at(caller.base_offset()) };
        })?;
        let instr_ptr = InstructionPtr::new(func.instrs().as_ptr());
        let frame = CallFrame::new(instr_ptr, offsets, results);
        if <C as CallContext>::HAS_PARAMS {
            self.copy_call_params(&mut uninit_params);
        }
        uninit_params.init_zeroes();
        Ok(frame)
    }

    /// Copies the parameters from caller for the callee [`CallFrame`].
    ///
    /// This will also adjust the instruction pointer to point to the
    /// last call parameter [`Instruction`] if any.
    fn copy_call_params(&mut self, uninit_params: &mut FrameParams) {
        self.ip.add(1);
        if let Instruction::RegisterList { .. } = self.ip.get() {
            self.copy_call_params_list(uninit_params);
        }
        match self.ip.get() {
            Instruction::Register { reg } => {
                self.copy_regs(uninit_params, array::from_ref(reg));
            }
            Instruction::Register2 { regs } => {
                self.copy_regs(uninit_params, regs);
            }
            Instruction::Register3 { regs } => {
                self.copy_regs(uninit_params, regs);
            }
            unexpected => {
                // Safety: Wasmi translation guarantees that register list finalizer exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected register-list finalizer but found: {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Copies an array of [`Reg`] to the `dst` [`Reg`] span.
    fn copy_regs<const N: usize>(&self, uninit_params: &mut FrameParams, regs: &[Reg; N]) {
        for value in regs {
            let value = self.get_register(*value);
            // Safety: The `callee.results()` always refer to a span of valid
            //         registers of the `caller` that does not overlap with the
            //         registers of the callee since they reside in different
            //         call frames. Therefore this access is safe.
            unsafe { uninit_params.init_next(value) }
        }
    }

    /// Copies a list of [`Instruction::RegisterList`] to the `dst` [`Reg`] span.
    /// Copies the parameters from `src` for the called [`CallFrame`].
    ///
    /// This will make the [`InstructionPtr`] point to the [`Instruction`] following the
    /// last [`Instruction::RegisterList`] if any.
    #[cold]
    fn copy_call_params_list(&mut self, uninit_params: &mut FrameParams) {
        while let Instruction::RegisterList { regs } = self.ip.get() {
            self.copy_regs(uninit_params, regs);
            self.ip.add(1);
        }
    }

    /// Prepares a [`EngineFunc`] call with optional call parameters.
    fn prepare_compiled_func_call<C: CallContext>(
        &mut self,
        store: &mut StoreInner,
        results: RegSpan,
        func: EngineFunc,
        mut instance: Option<Instance>,
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
                let caller_instance = unsafe { self.stack.merge_call_frames(&mut called) };
                if let Some(caller_instance) = caller_instance {
                    instance.get_or_insert(caller_instance);
                }
            }
        }
        self.init_call_frame(&called);
        self.stack.calls.push(called, instance)?;
        Ok(())
    }

    /// Executes an [`Instruction::ReturnCallInternal0`].
    pub fn execute_return_call_internal_0(
        &mut self,
        store: &mut StoreInner,
        func: EngineFunc,
    ) -> Result<(), Error> {
        self.execute_return_call_internal_impl::<marker::ReturnCall0>(store, func)
    }

    /// Executes an [`Instruction::ReturnCallInternal`].
    pub fn execute_return_call_internal(
        &mut self,
        store: &mut StoreInner,
        func: EngineFunc,
    ) -> Result<(), Error> {
        self.execute_return_call_internal_impl::<marker::ReturnCall>(store, func)
    }

    /// Executes an [`Instruction::ReturnCallInternal`] or [`Instruction::ReturnCallInternal0`].
    fn execute_return_call_internal_impl<C: CallContext>(
        &mut self,
        store: &mut StoreInner,
        func: EngineFunc,
    ) -> Result<(), Error> {
        let results = self.caller_results();
        self.prepare_compiled_func_call::<C>(store, results, func, None)
    }

    /// Returns the `results` [`RegSpan`] of the top-most [`CallFrame`] on the [`CallStack`].
    ///
    /// # Note
    ///
    /// We refer to the top-most [`CallFrame`] as the `caller` since this method is used for
    /// tail call instructions for which the top-most [`CallFrame`] is the caller.
    ///
    /// [`CallStack`]: crate::engine::executor::stack::CallStack
    fn caller_results(&self) -> RegSpan {
        self.stack
            .calls
            .peek()
            .expect("must have caller on the stack")
            .results()
    }

    /// Executes an [`Instruction::CallInternal0`].
    pub fn execute_call_internal_0(
        &mut self,
        store: &mut StoreInner,
        results: RegSpan,
        func: EngineFunc,
    ) -> Result<(), Error> {
        self.prepare_compiled_func_call::<marker::NestedCall0>(store, results, func, None)
    }

    /// Executes an [`Instruction::CallInternal`].
    pub fn execute_call_internal(
        &mut self,
        store: &mut StoreInner,
        results: RegSpan,
        func: EngineFunc,
    ) -> Result<(), Error> {
        self.prepare_compiled_func_call::<marker::NestedCall>(store, results, func, None)
    }

    /// Executes an [`Instruction::ReturnCallImported0`].
    pub fn execute_return_call_imported_0<T>(
        &mut self,
        store: &mut Store<T>,
        func: index::Func,
    ) -> Result<(), Error> {
        self.execute_return_call_imported_impl::<marker::ReturnCall0, T>(store, func)
    }

    /// Executes an [`Instruction::ReturnCallImported`].
    pub fn execute_return_call_imported<T>(
        &mut self,
        store: &mut Store<T>,
        func: index::Func,
    ) -> Result<(), Error> {
        self.execute_return_call_imported_impl::<marker::ReturnCall, T>(store, func)
    }

    /// Executes an [`Instruction::ReturnCallImported`] or [`Instruction::ReturnCallImported0`].
    fn execute_return_call_imported_impl<C: ReturnCallContext, T>(
        &mut self,
        store: &mut Store<T>,
        func: index::Func,
    ) -> Result<(), Error> {
        let func = self.get_func(func);
        let results = self.caller_results();
        self.execute_call_imported_impl::<C, T>(store, results, &func)
    }

    /// Executes an [`Instruction::CallImported0`].
    pub fn execute_call_imported_0<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func: index::Func,
    ) -> Result<(), Error> {
        let func = self.get_func(func);
        self.execute_call_imported_impl::<marker::NestedCall0, T>(store, results, &func)
    }

    /// Executes an [`Instruction::CallImported`].
    pub fn execute_call_imported<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func: index::Func,
    ) -> Result<(), Error> {
        let func = self.get_func(func);
        self.execute_call_imported_impl::<marker::NestedCall, T>(store, results, &func)
    }

    /// Executes an imported or indirect (tail) call instruction.
    fn execute_call_imported_impl<C: CallContext, T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func: &Func,
    ) -> Result<(), Error> {
        match store.inner.resolve_func(func) {
            FuncEntity::Wasm(func) => {
                let instance = *func.instance();
                let func_body = func.func_body();
                self.prepare_compiled_func_call::<C>(
                    &mut store.inner,
                    results,
                    func_body,
                    Some(instance),
                )?;
                self.cache.update(&mut store.inner, &instance);
                Ok(())
            }
            FuncEntity::Host(host_func) => {
                let host_func = *host_func;

                store.invoke_call_hook(CallHook::CallingHost)?;
                self.execute_host_func::<C, T>(store, results, func, host_func)?;
                store.invoke_call_hook(CallHook::ReturningFromHost)?;

                Ok(())
            }
        }
    }

    /// Executes a host function.
    ///
    /// # Note
    ///
    /// This uses the value stack to store paramters and results of the host function call.
    /// Returns an [`ErrorKind::ResumableHost`] variant if the host function returned an error
    /// and there are still call frames on the call stack making it possible to resume the
    /// execution at a later point in time.
    ///
    /// [`ErrorKind::ResumableHost`]: crate::error::ErrorKind::ResumableHost
    fn execute_host_func<C: CallContext, T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func: &Func,
        host_func: HostFuncEntity,
    ) -> Result<(), Error> {
        let len_params = host_func.len_params();
        let len_results = host_func.len_results();
        let max_inout = usize::from(len_params.max(len_results));
        let instance = *self.stack.calls.instance_expect();
        // We have to reinstantiate the `self.sp` [`FrameRegisters`] since we just called
        // [`ValueStack::reserve`] which might invalidate all live [`FrameRegisters`].
        let caller = match <C as CallContext>::KIND {
            CallKind::Nested => self.stack.calls.peek().copied(),
            CallKind::Tail => self.stack.calls.pop().map(|(frame, _instance)| frame),
        }
        .expect("need to have a caller on the call stack");
        let buffer = self.stack.values.extend_by(max_inout, |this| {
            // Safety: we use the base offset of a live call frame on the call stack.
            self.sp = unsafe { this.stack_ptr_at(caller.base_offset()) };
        })?;
        if <C as CallContext>::HAS_PARAMS {
            let mut uninit_params = FrameParams::new(buffer);
            self.copy_call_params(&mut uninit_params);
        }
        if matches!(<C as CallContext>::KIND, CallKind::Nested) {
            self.update_instr_ptr_at(1);
        }
        self.dispatch_host_func::<T>(store, host_func, &instance)
            .map_err(|error| match self.stack.calls.is_empty() {
                true => error,
                false => ResumableHostError::new(error, *func, results).into(),
            })?;
        self.cache.update(&mut store.inner, &instance);
        let results = results.iter(len_results);
        let returned = self.stack.values.drop_return(max_inout);
        for (result, value) in results.zip(returned) {
            // # Safety (1)
            //
            // We can safely acquire the stack pointer to the caller's and callee's (host)
            // call frames because we just allocated the host call frame and can be sure that
            // they are different.
            // In the following we make sure to not access registers out of bounds of each
            // call frame since we rely on Wasm validation and proper Wasm translation to
            // provide us with valid result registers.
            unsafe { self.sp.set(result, *value) };
        }
        Ok(())
    }

    /// Convenience forwarder to [`dispatch_host_func`].
    fn dispatch_host_func<T>(
        &mut self,
        store: &mut Store<T>,
        host_func: HostFuncEntity,
        instance: &Instance,
    ) -> Result<(u16, u16), Error> {
        dispatch_host_func(store, &mut self.stack.values, host_func, Some(instance))
    }

    /// Executes an [`Instruction::CallIndirect0`].
    pub fn execute_return_call_indirect_0<T>(
        &mut self,
        store: &mut Store<T>,
        func_type: index::FuncType,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params();
        let results = self.caller_results();
        self.execute_call_indirect_impl::<marker::ReturnCall0, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect0Imm16`].
    pub fn execute_return_call_indirect_0_imm16<T>(
        &mut self,
        store: &mut Store<T>,
        func_type: index::FuncType,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params_imm16();
        let results = self.caller_results();
        self.execute_call_indirect_impl::<marker::ReturnCall0, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect0`].
    pub fn execute_return_call_indirect<T>(
        &mut self,
        store: &mut Store<T>,
        func_type: index::FuncType,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params();
        let results = self.caller_results();
        self.execute_call_indirect_impl::<marker::ReturnCall, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect0Imm16`].
    pub fn execute_return_call_indirect_imm16<T>(
        &mut self,
        store: &mut Store<T>,
        func_type: index::FuncType,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params_imm16();
        let results = self.caller_results();
        self.execute_call_indirect_impl::<marker::ReturnCall, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect0`].
    pub fn execute_call_indirect_0<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func_type: index::FuncType,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params();
        self.execute_call_indirect_impl::<marker::NestedCall0, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect0Imm16`].
    pub fn execute_call_indirect_0_imm16<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func_type: index::FuncType,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params_imm16();
        self.execute_call_indirect_impl::<marker::NestedCall0, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect`].
    pub fn execute_call_indirect<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func_type: index::FuncType,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params();
        self.execute_call_indirect_impl::<marker::NestedCall, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirectImm16`].
    pub fn execute_call_indirect_imm16<T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func_type: index::FuncType,
    ) -> Result<(), Error> {
        let (index, table) = self.pull_call_indirect_params_imm16();
        self.execute_call_indirect_impl::<marker::NestedCall, T>(
            store, results, func_type, index, table,
        )
    }

    /// Executes an [`Instruction::CallIndirect`] and [`Instruction::CallIndirect0`].
    fn execute_call_indirect_impl<C: CallContext, T>(
        &mut self,
        store: &mut Store<T>,
        results: RegSpan,
        func_type: index::FuncType,
        index: u32,
        table: index::Table,
    ) -> Result<(), Error> {
        let table = self.get_table(table);
        let funcref = store
            .inner
            .resolve_table(&table)
            .get_untyped(index)
            .map(FuncRef::from)
            .ok_or(TrapCode::TableOutOfBounds)?;
        let func = funcref.func().ok_or(TrapCode::IndirectCallToNull)?;
        let actual_signature = store.inner.resolve_func(func).ty_dedup();
        let expected_signature = &self.get_func_type_dedup(func_type);
        if actual_signature != expected_signature {
            return Err(Error::from(TrapCode::BadSignature));
        }
        self.execute_call_imported_impl::<C, T>(store, results, func)
    }
}
