use super::{Executor, ReturnOutcome};
use crate::{
    core::UntypedValue,
    engine::bytecode2::{AnyConst32, Const32, Register, RegisterSpanIter},
};

#[cfg(doc)]
use crate::engine::bytecode2::Instruction;

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
        // Note: We pop instead of peek to avoid borrow checker errors.
        // This might be slow and we might want to fix this in that case.
        match self.call_stack.pop() {
            Some(caller) => {
                self.init_call_frame(&caller);
                self.call_stack
                    .push(caller)
                    .expect("pushing a single call frame after popping 2 cannot fail");
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

    /// Execute a generic return [`Instruction`] returning a single value.
    fn execute_return_value<T>(
        &mut self,
        value: T,
        f: fn(&mut Self, T) -> UntypedValue,
    ) -> ReturnOutcome {
        match self.call_stack.peek() {
            Some(caller) => unsafe {
                // Case: we need to return the `value` back to the caller frame.
                // Safety: TODO
                let mut caller_sp = self.value_stack.stack_ptr_at(caller.base_offset());
                let result = caller_sp.get_mut(caller.results().head());
                *result = f(self, value);
            },
            None => {
                // Case: the root call frame is returning.
                todo!()
            }
        }
        self.return_impl()
    }

    /// Execute an [`Instruction::ReturnReg`] returning a single [`Register`] value.
    #[inline(always)]
    pub fn execute_return_reg(&mut self, value: Register) -> ReturnOutcome {
        self.execute_return_value(value, |this, value| this.get_register(value))
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

    /// Execute an [`Instruction::ReturnMany`] returning many values.
    #[inline(always)]
    pub fn execute_return_many(&mut self, values: RegisterSpanIter) -> ReturnOutcome {
        match self.call_stack.peek() {
            Some(caller) => unsafe {
                // Case: we need to return the `value` back to the caller frame.
                // Safety: TODO
                let mut caller_sp = self.value_stack.stack_ptr_at(caller.base_offset());
                let results = caller.results().iter(values.len());
                for (result, value) in results.zip(values) {
                    *caller_sp.get_mut(result) = self.get_register(value);
                }
            },
            None => {
                // Case: the root call frame is returning.
                todo!()
            }
        }
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

    /// Execute an [`Instruction::ReturnNezMany`] returning many values.
    #[inline(always)]
    pub fn execute_return_nez_many(
        &mut self,
        condition: Register,
        values: RegisterSpanIter,
    ) -> ReturnOutcome {
        self.execute_return_nez_impl(condition, values, Self::execute_return_many)
    }
}
