use super::Executor;
use crate::{
    core::UntypedValue,
    engine::regmach::{
        bytecode::{AnyConst32, Const32, Instruction, Register, RegisterSpan, RegisterSpanIter},
        stack::ValueStackPtr,
    },
};
use core::slice;

/// The outcome of a Wasm return statement.
#[derive(Debug, Copy, Clone)]
pub enum ReturnOutcome {
    /// The call returns to a nested Wasm caller.
    Wasm,
    /// The call returns back to the host.
    Host,
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Returns the execution to the caller.
    ///
    /// Any return values are expected to already have been transferred
    /// from the returning callee to the caller.
    #[inline(always)]
    fn return_impl(&mut self) -> ReturnOutcome {
        let returned = self
            .call_stack
            .pop()
            .expect("the executing call frame is always on the stack");
        self.value_stack.truncate(returned.frame_offset());
        match self.call_stack.peek() {
            Some(caller) => {
                Self::init_call_frame_impl(
                    self.value_stack,
                    &mut self.sp,
                    &mut self.ip,
                    self.cache,
                    caller,
                );
                ReturnOutcome::Wasm
            }
            None => ReturnOutcome::Host,
        }
    }

    /// Execute an [`Instruction::Return`].
    #[inline(always)]
    pub fn execute_return(&mut self) -> ReturnOutcome {
        self.return_impl()
    }

    /// Returns the [`ValueStackPtr`] of the caller and the [`RegisterSpan`] of the results.
    ///
    /// The returned [`ValueStackPtr`] is valid for all [`Register`] in the returned [`RegisterSpan`].
    fn return_caller_results(&mut self) -> (ValueStackPtr, RegisterSpan) {
        let (callee, caller) = self
            .call_stack
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
                let caller_sp = unsafe { self.value_stack.stack_ptr_at(caller.base_offset()) };
                let results = callee.results();
                (caller_sp, results)
            }
            None => {
                // Case: the root call frame is returning.
                //
                // In this case we transfer the single return `value` to the root
                // register span of the entire value stack which is simply its zero index.
                let dst_sp = self.value_stack.root_stack_ptr();
                let results = RegisterSpan::new(Register::from_i16(0));
                (dst_sp, results)
            }
        }
    }

    /// Execute a generic return [`Instruction`] returning a single value.
    fn execute_return_value<T>(
        &mut self,
        value: T,
        f: fn(&Self, T) -> UntypedValue,
    ) -> ReturnOutcome {
        let (mut caller_sp, results) = self.return_caller_results();
        let value = f(self, value);
        // Safety: The `callee.results()` always refer to a span of valid
        //         registers of the `caller` that does not overlap with the
        //         registers of the callee since they reside in different
        //         call frames. Therefore this access is safe.
        let result = unsafe { caller_sp.get_mut(results.head()) };
        *result = value;
        self.return_impl()
    }

    /// Execute an [`Instruction::ReturnReg`] returning a single [`Register`] value.
    #[inline(always)]
    pub fn execute_return_reg(&mut self, value: Register) -> ReturnOutcome {
        self.execute_return_value(value, Self::get_register)
    }

    /// Execute an [`Instruction::ReturnReg2`] returning two [`Register`] values.
    #[inline(always)]
    pub fn execute_return_reg2(&mut self, values: [Register; 2]) -> ReturnOutcome {
        self.execute_return_reg_n_impl::<2>(values)
    }

    /// Execute an [`Instruction::ReturnReg3`] returning three [`Register`] values.
    #[inline(always)]
    pub fn execute_return_reg3(&mut self, values: [Register; 3]) -> ReturnOutcome {
        self.execute_return_reg_n_impl::<3>(values)
    }

    /// Executes an [`Instruction::ReturnReg2`] or [`Instruction::ReturnReg3`] generically.
    fn execute_return_reg_n_impl<const N: usize>(
        &mut self,
        values: [Register; N],
    ) -> ReturnOutcome {
        let (mut caller_sp, results) = self.return_caller_results();
        debug_assert!(u16::try_from(N).is_ok());
        for (result, value) in results.iter_u16(N as u16).zip(values) {
            let value = self.get_register(value);
            // Safety: The `callee.results()` always refer to a span of valid
            //         registers of the `caller` that does not overlap with the
            //         registers of the callee since they reside in different
            //         call frames. Therefore this access is safe.
            let cell = unsafe { caller_sp.get_mut(result) };
            *cell = value;
        }
        self.return_impl()
    }

    /// Execute an [`Instruction::ReturnImm32`] returning a single 32-bit value.
    #[inline(always)]
    pub fn execute_return_imm32(&mut self, value: AnyConst32) -> ReturnOutcome {
        self.execute_return_value(value, |_, value| value.to_u32().into())
    }

    /// Execute an [`Instruction::ReturnI64Imm32`] returning a single 32-bit encoded `i64` value.
    #[inline(always)]
    pub fn execute_return_i64imm32(&mut self, value: Const32<i64>) -> ReturnOutcome {
        self.execute_return_value(value, |_, value| i64::from(value).into())
    }

    /// Execute an [`Instruction::ReturnF64Imm32`] returning a single 32-bit encoded `f64` value.
    #[inline(always)]
    pub fn execute_return_f64imm32(&mut self, value: Const32<f64>) -> ReturnOutcome {
        self.execute_return_value(value, |_, value| f64::from(value).into())
    }

    /// Execute an [`Instruction::ReturnSpan`] returning many values.
    #[inline(always)]
    pub fn execute_return_span(&mut self, values: RegisterSpanIter) -> ReturnOutcome {
        let (mut caller_sp, results) = self.return_caller_results();
        let results = results.iter(values.len());
        for (result, value) in results.zip(values) {
            // Safety: The `callee.results()` always refer to a span of valid
            //         registers of the `caller` that does not overlap with the
            //         registers of the callee since they reside in different
            //         call frames. Therefore this access is safe.
            let cell = unsafe { caller_sp.get_mut(result) };
            *cell = self.get_register(value);
        }
        self.return_impl()
    }

    /// Execute an [`Instruction::ReturnMany`] returning many values.
    #[inline(always)]
    pub fn execute_return_many(&mut self, values: [Register; 3]) -> ReturnOutcome {
        self.execute_return_many_impl(&values)
    }

    /// Executes [`Instruction::ReturnMany`] or parts of [`Instruction::ReturnNezMany`] generically.
    fn execute_return_many_impl(&mut self, values: &[Register]) -> ReturnOutcome {
        let (mut caller_sp, results) = self.return_caller_results();
        let mut result = results.head();
        let mut copy_results = |values: &[Register]| {
            for value in values {
                let value = self.get_register(*value);
                // Safety: The `callee.results()` always refer to a span of valid
                //         registers of the `caller` that does not overlap with the
                //         registers of the callee since they reside in different
                //         call frames. Therefore this access is safe.
                let cell = unsafe { caller_sp.get_mut(result) };
                *cell = value;
                result = result.next();
            }
        };
        copy_results(values);
        let mut ip = self.ip;
        ip.add(1);
        while let Instruction::RegisterList(values) = ip.get() {
            copy_results(values);
            ip.add(1);
        }
        let values = match ip.get() {
            Instruction::Register(value) => slice::from_ref(value),
            Instruction::Register2(values) => values,
            Instruction::Register3(values) => values,
            unexpected => unreachable!("unexpected Instruction found while executing Instruction::ReturnMany: {unexpected:?}"),
        };
        copy_results(values);
        self.return_impl()
    }

    /// Execute a generic conditional return [`Instruction`].
    fn execute_return_nez_impl<T>(
        &mut self,
        condition: Register,
        value: T,
        f: fn(&mut Self, T) -> ReturnOutcome,
    ) -> ReturnOutcome {
        let condition = self.get_register(condition);
        match bool::from(condition) {
            true => f(self, value),
            false => {
                self.next_instr();
                ReturnOutcome::Wasm
            }
        }
    }

    /// Execute an [`Instruction::Return`].
    #[inline(always)]
    pub fn execute_return_nez(&mut self, condition: Register) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, (), |this, _| this.execute_return())
    }

    /// Execute an [`Instruction::ReturnNezReg`] returning a single [`Register`] value.
    #[inline(always)]
    pub fn execute_return_nez_reg(
        &mut self,
        condition: Register,
        value: Register,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_reg)
    }

    /// Execute an [`Instruction::ReturnNezReg`] returning a single [`Register`] value.
    #[inline(always)]
    pub fn execute_return_nez_reg2(
        &mut self,
        condition: Register,
        value: [Register; 2],
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_reg2)
    }

    /// Execute an [`Instruction::ReturnNezImm32`] returning a single 32-bit constant value.
    #[inline(always)]
    pub fn execute_return_nez_imm32(
        &mut self,
        condition: Register,
        value: AnyConst32,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_imm32)
    }

    /// Execute an [`Instruction::ReturnNezI64Imm32`] returning a single 32-bit encoded constant `i64` value.
    #[inline(always)]
    pub fn execute_return_nez_i64imm32(
        &mut self,
        condition: Register,
        value: Const32<i64>,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_i64imm32)
    }

    /// Execute an [`Instruction::ReturnNezF64Imm32`] returning a single 32-bit encoded constant `f64` value.
    #[inline(always)]
    pub fn execute_return_nez_f64imm32(
        &mut self,
        condition: Register,
        value: Const32<f64>,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, value, Self::execute_return_f64imm32)
    }

    /// Execute an [`Instruction::ReturnNezSpan`] returning many values.
    #[inline(always)]
    pub fn execute_return_nez_span(
        &mut self,
        condition: Register,
        values: RegisterSpanIter,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, values, Self::execute_return_span)
    }

    /// Execute an [`Instruction::ReturnNezMany`] returning many values.
    #[inline(always)]
    pub fn execute_return_nez_many(
        &mut self,
        condition: Register,
        values: [Register; 2],
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, &values[..], Self::execute_return_many_impl)
    }
}
