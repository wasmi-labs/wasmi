#![allow(dead_code)] // TODO: remove

use super::{CallFrame, CallStack, Stack, ValueStack, ValueStackPtr, ValueStackPtrIter};
use crate::{
    core::UntypedValue,
    engine::{
        bytecode::{Register, RegisterSpan},
        cache::InstanceCache,
        code_map::{CompiledFuncEntity, InstructionPtr},
    },
    Error,
};

/// A wrapper for [`Stack`] optimized for execution.
///
/// # Note
///
/// The currently executed call frame is always on top of `calls`.
pub struct ExecStack<'a> {
    /// The current stack pointer.
    ///
    /// This provides an efficient register access for the currently executed call frame.
    sp: ValueStackPtr,
    /// The current instruction pointer within the currently executed frame.
    ip: InstructionPtr,
    /// The entire value stack.
    values: &'a mut ValueStack,
    /// The entire call stack.
    calls: &'a mut CallStack,
}

impl<'a> ExecStack<'a> {
    /// Create a new [`ExecStack`] from the given [`Stack`].
    pub fn new(stack: &'a mut Stack) -> Self {
        let Some(frame) = stack.calls.peek() else {
            panic!("unexpected empty call stack")
        };
        let ip = frame.instr_ptr();
        // SAFETY: We created the `ExecStack` from a single `Stack` which must
        //         have valid values for each of its call frames by invariant.
        let sp = unsafe { stack.values.stack_ptr_at(frame.base_offset()) };
        Self {
            ip,
            sp,
            values: &mut stack.values,
            calls: &mut stack.calls,
        }
    }

    /// Returns the [`Register`] value of the currently executed frame.
    pub fn get_register(&self, register: Register) -> UntypedValue {
        // Safety: While it is the callers responsibility to feed in valid `register`
        //         for this function we are guaranteed by Wasmi bytecode construction
        //         that each function only contains valid registers for its frame.
        unsafe { self.sp.get(register) }
    }

    /// Returns the [`Register`] value of the currently executed frame decoded as `T`.
    ///
    pub fn get_register_as<T>(&self, register: Register) -> T
    where
        T: From<UntypedValue>,
    {
        T::from(self.get_register(register))
    }

    /// Sets the [`Register`] value of the currently executed frame to `value`.
    pub fn set_register(&mut self, register: Register, value: impl Into<UntypedValue>) {
        // Safety: While it is the callers responsibility to feed in valid `register`
        //         for this function we are guaranteed by Wasmi bytecode construction
        //         that each function only contains valid registers for its frame.
        unsafe { self.sp.set(register, value.into()) }
    }

    /// Offsets the [`InstructionPtr`] by the given `amount`.
    ///
    /// # Note
    ///
    /// This is used by `wasmi` instructions that have a fixed
    /// encoding size of two instruction words such as [`Instruction::Branch`].
    #[inline(always)]
    pub fn offset_instr_ptr(&mut self, amount: isize) {
        self.ip.offset(amount)
    }

    fn return_(&mut self, cache: &mut InstanceCache) -> CallOrigin {
        let callee = self.calls.pop().expect("missing callee call frame");
        self.values.truncate(callee.frame_offset());
        match self.calls.peek() {
            Some(caller) => {
                // SAFETY:
                // Safety: We are using the frame's own base offset as input because it is
                //         guaranteed by the Wasm validation and translation phase to be
                //         valid for all register indices used by the associated function body.
                self.sp = unsafe { self.values.stack_ptr_at(caller.base_offset()) };
                self.ip = caller.instr_ptr();
                cache.update_instance(caller.instance());
                CallOrigin::Wasm
            }
            None => CallOrigin::Host,
        }
    }

    /// Returns the [`ValueStackPtrIter`] of the caller results.
    fn return_caller_results(&mut self) -> ValueStackPtrIter {
        let (callee, caller) = self
            .calls
            .peek_2()
            .expect("the callee must exist on the call stack");
        match caller {
            Some(caller) => {
                // Case: we need to return the `value` back to the caller frame.
                //
                // In this case we transfer the single return `value` to the `results`
                // register span of the caller's call frame.
                //
                // Safety: The caller call frame is still live on the value stack
                //         and therefore it is safe to acquire its value stack pointer.
                let caller_sp = unsafe { self.values.stack_ptr_at(caller.base_offset()) };
                let results = callee.results();
                ValueStackPtrIter::new(caller_sp, results.head())
            }
            None => {
                // Case: the root call frame is returning.
                //
                // In this case we transfer the single return `value` to the root
                // register span of the entire value stack which is simply its zero index.
                let dst_sp = self.values.root_stack_ptr();
                let results = RegisterSpan::new(Register::from_i16(0));
                ValueStackPtrIter::new(dst_sp, results.head())
            }
        }
    }

    /// Creates a [`CallFrame`] for calling the [`CompiledFuncEntity`].
    #[inline(always)]
    fn dispatch_compiled_func(
        &mut self,
        results: RegisterSpan,
        func: &CompiledFuncEntity,
    ) -> Result<(), Error> {
        let instr_ptr = InstructionPtr::new(func.instrs().as_ptr());
        let (base_ptr, frame_ptr) = self.values.alloc_call_frame(func)?;
        // We have to reinstantiate the `self.sp` [`ValueStackPtr`] since we just called
        // [`ValueStack::alloc_call_frame`] which might invalidate all live [`ValueStackPtr`].
        let caller = self.calls.peek().expect("missing caller call frame");
        // Safety: We use the base offset of a live call frame on the call stack.
        self.sp = unsafe { self.values.stack_ptr_at(caller.base_offset()) };
        let instance = caller.instance();
        let frame = CallFrame::new(instr_ptr, frame_ptr, base_ptr, results, *instance);
        self.calls.push(frame)?;
        Ok(())
    }

    /// Merge the two top-most [`CallFrame`] with respect to a tail call.
    ///
    /// # Panics (Debug)
    ///
    /// - If the two top-most [`CallFrame`] do not have matching `results`.
    /// - If there are not at least two [`CallFrame`] on the [`CallStack`].
    ///
    /// # Safety
    ///
    /// Any [`ValueStackPtr`] allocated within the range `from..to` on the [`ValueStack`]
    /// may be invalidated by this operation. It is the caller's responsibility to reinstantiate
    /// all [`ValueStackPtr`] affected by this.
    pub fn merge_call_frames(&mut self) {
        let caller = self.calls.pop_caller();
        let callee = self.calls.peek_mut().expect("missing callee call frame");
        debug_assert_eq!(callee.results(), caller.results());
        debug_assert!(caller.base_offset() <= callee.base_offset());
        // Safety:
        //
        // We only drain cells of the second top-most call frame on the value stack.
        // Therefore only value stack offsets of the top-most call frame on the
        // value stack are going to be invalidated which we ensure to adjust and
        // reinstantiate after this operation.
        let len_drained = self
            .values
            .drain(caller.frame_offset(), callee.frame_offset());
        callee.move_down(len_drained);
    }
}

/// The origin of a Wasm function call.
#[derive(Debug, Copy, Clone)]
pub enum CallOrigin {
    /// The call originated from within another Wasm function.
    Wasm,
    /// The call originated from the host side.
    Host,
}
