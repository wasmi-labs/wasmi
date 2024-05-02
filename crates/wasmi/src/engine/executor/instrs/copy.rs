use super::Executor;
use crate::{
    core::UntypedVal,
    engine::bytecode::{AnyConst32, Const32, Instruction, Register, RegisterSpan},
};
use core::slice;
use smallvec::SmallVec;

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Executes a generic `copy` [`Instruction`].
    fn execute_copy_impl<T>(
        &mut self,
        result: Register,
        value: T,
        f: fn(&mut Self, T) -> UntypedVal,
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
        self.execute_copy_impl(result, value, |_, value| UntypedVal::from(u32::from(value)))
    }

    /// Executes an [`Instruction::CopyI64Imm32`].
    #[inline(always)]
    pub fn execute_copy_i64imm32(&mut self, result: Register, value: Const32<i64>) {
        self.execute_copy_impl(result, value, |_, value| UntypedVal::from(i64::from(value)))
    }

    /// Executes an [`Instruction::CopyF64Imm32`].
    #[inline(always)]
    pub fn execute_copy_f64imm32(&mut self, result: Register, value: Const32<f64>) {
        self.execute_copy_impl(result, value, |_, value| UntypedVal::from(f64::from(value)))
    }

    /// Executes an [`Instruction::CopySpan`].
    ///
    /// # Note
    ///
    /// - This instruction assumes that `results` and `values` _do_ overlap
    ///   and thus requires a costly temporary buffer to avoid overwriting
    ///   intermediate copy results.
    /// - If `results` and `values` do _not_ overlap [`Instruction::CopySpanNonOverlapping`] is used.
    #[inline(always)]
    pub fn execute_copy_span(&mut self, results: RegisterSpan, values: RegisterSpan, len: u16) {
        let results = results.iter_u16(len);
        let values = values.iter_u16(len);
        let mut tmp = <SmallVec<[UntypedVal; 8]>>::default();
        tmp.extend(values.into_iter().map(|value| self.get_register(value)));
        for (result, value) in results.into_iter().zip(tmp) {
            self.set_register(result, value);
        }
        self.next_instr();
    }

    /// Executes an [`Instruction::CopySpanNonOverlapping`].
    ///
    /// # Note
    ///
    /// - This instruction assumes that `results` and `values` do _not_ overlap
    ///   and thus can copy all the elements without a costly temporary buffer.
    /// - If `results` and `values` _do_ overlap [`Instruction::CopySpan`] is used.
    #[inline(always)]
    pub fn execute_copy_span_non_overlapping(
        &mut self,
        results: RegisterSpan,
        values: RegisterSpan,
        len: u16,
    ) {
        let results = results.iter_u16(len);
        let values = values.iter_u16(len);
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
        let mut tmp = <SmallVec<[UntypedVal; 8]>>::default();
        let mut ip = self.ip;
        tmp.extend(values.into_iter().map(|value| self.get_register(value)));
        ip.add(1);
        while let Instruction::RegisterList(values) = ip.get() {
            tmp.extend(values.iter().map(|value| self.get_register(*value)));
            ip.add(1);
        }
        let values = match ip.get() {
            Instruction::Register(value) => slice::from_ref(value),
            Instruction::Register2(values) => values,
            Instruction::Register3(values) => values,
            unexpected => unreachable!("unexpected Instruction found while executing Instruction::CopyMany: {unexpected:?}"),
        };
        tmp.extend(values.iter().map(|value| self.get_register(*value)));
        for (result, value) in results.iter(tmp.len()).zip(tmp) {
            self.set_register(result, value);
        }
        self.ip = ip;
        self.next_instr()
    }

    /// Executes an [`Instruction::CopyManyNonOverlapping`].
    #[inline(always)]
    pub fn execute_copy_many_non_overlapping(
        &mut self,
        results: RegisterSpan,
        values: [Register; 2],
    ) {
        let mut ip = self.ip;
        let mut result = results.head();
        let mut copy_values = |values: &[Register]| {
            for &value in values {
                let value = self.get_register(value);
                self.set_register(result, value);
                result = result.next();
            }
        };
        copy_values(&values);
        ip.add(1);
        while let Instruction::RegisterList(values) = ip.get() {
            copy_values(values);
            ip.add(1);
        }
        let values = match ip.get() {
            Instruction::Register(value) => slice::from_ref(value),
            Instruction::Register2(values) => values,
            Instruction::Register3(values) => values,
            unexpected => unreachable!("unexpected Instruction found while executing Instruction::CopyManyNonOverlapping: {unexpected:?}"),
        };
        copy_values(values);
        self.ip = ip;
        self.next_instr()
    }
}
