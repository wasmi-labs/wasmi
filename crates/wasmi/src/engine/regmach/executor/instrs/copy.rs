use super::Executor;
use crate::{
    core::UntypedValue,
    engine::regmach::bytecode::{AnyConst32, Const32, Instruction, Register, RegisterSpan},
};
use smallvec::SmallVec;

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

    /// Executes an [`Instruction::Copy2`].
    #[inline(always)]
    pub fn execute_copy_2(&mut self, results: RegisterSpan, values: [Register; 2]) {
        let result0 = results.head();
        let result1 = result0.next();
        // We need `tmp` in case `results[0] == values[1]` to avoid overwriting `values[1]` before reading it.
        let tmp = self.get_register(values[1]);
        self.set_register(result0, self.get_register(values[0]));
        self.set_register(result1, tmp);
        self.next_instr()
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

    /// Executes an [`Instruction::CopyMany`].
    #[inline(always)]
    pub fn execute_copy_many(&mut self, results: RegisterSpan, values: [Register; 2]) {
        // We need `tmp` since `values[n]` might be overwritten by previous copies.
        let mut tmp = <SmallVec<[UntypedValue; 8]>>::default();
        let mut ip = self.ip;
        ip.add(1);
        tmp.push(self.get_register(values[0]));
        tmp.push(self.get_register(values[1]));
        while let Instruction::RegisterList(values) = ip.get() {
            for value in values {
                tmp.push(self.get_register(*value));
            }
            ip.add(1);
        }
        let values = match ip.get() {
            Instruction::Register(value) => core::slice::from_ref(value),
            Instruction::Register2(values) => values,
            Instruction::Register3(values) => values,
            unexpected => unreachable!("unexpected Instruction found while executing Instruction::CopyMany: {unexpected:?}"),
        };
        for value in values {
            tmp.push(self.get_register(*value));
        }
        for (result, value) in results.iter(tmp.len()).zip(tmp) {
            self.set_register(result, value);
        }
        ip.add(1);
        self.ip = ip;
    }
}
