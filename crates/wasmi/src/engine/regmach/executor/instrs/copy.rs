use super::Executor;
use crate::{
    core::UntypedValue,
    engine::regmach::bytecode::{AnyConst32, Const32, Register, RegisterSpan},
};

#[cfg(doc)]
use crate::engine::regmach::bytecode::Instruction;

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Executes a generic `copy` [`Instruction`].
    fn execute_copy_impl<T>(
        &mut self,
        result: Register,
        value: T,
        f: fn(&mut Self, T) -> UntypedValue,
    ) {
        let value = f(self, value);
        self.set_register(result, value);
        self.next_instr()
    }

    /// Executes an [`Instruction::Copy`].
    #[inline(always)]
    pub fn execute_copy(&mut self, result: Register, value: Register) {
        self.execute_copy_impl(result, value, |this, value| this.get_register(value))
    }

    /// Executes an [`Instruction::CopyImm32`].
    #[inline(always)]
    pub fn execute_copy_imm32(&mut self, result: Register, value: AnyConst32) {
        self.execute_copy_impl(result, value, |_, value| UntypedValue::from(value.to_u32()))
    }

    /// Executes an [`Instruction::CopyI64Imm32`].
    #[inline(always)]
    pub fn execute_copy_i64imm32(&mut self, result: Register, value: Const32<i64>) {
        self.execute_copy_impl(result, value, |_, value| {
            UntypedValue::from(i64::from(value))
        })
    }

    /// Executes an [`Instruction::CopyF64Imm32`].
    #[inline(always)]
    pub fn execute_copy_f64imm32(&mut self, result: Register, value: Const32<f64>) {
        self.execute_copy_impl(result, value, |_, value| {
            UntypedValue::from(f64::from(value))
        })
    }

    /// Executes an [`Instruction::CopySpan`].
    #[inline(always)]
    pub fn execute_copy_span(&mut self, results: RegisterSpan, values: RegisterSpan, len: u16) {
        let results = results.iter_u16(len);
        let values = values.iter_u16(len);
        self.execute_copy_span_impl(results, values)
    }

    /// Executes a [`Instruction::CopySpan`] generically.
    fn execute_copy_span_impl(
        &mut self,
        results: impl IntoIterator<Item = Register>,
        values: impl IntoIterator<Item = Register>,
    ) {
        for (result, value) in results.into_iter().zip(values.into_iter()) {
            let value = self.get_register(value);
            self.set_register(result, value);
        }
        self.next_instr();
    }
}
