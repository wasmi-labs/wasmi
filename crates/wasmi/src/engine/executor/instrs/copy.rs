use super::{Executor, InstructionPtr};
use crate::{
    core::UntypedVal,
    engine::utils::unreachable_unchecked,
    ir::{AnyConst32, Const32, FixedRegSpan, Instruction, Reg, RegSpan},
};
use core::slice;

impl Executor<'_> {
    /// Executes a generic `copy` [`Instruction`].
    fn execute_copy_impl<T>(&mut self, result: Reg, value: T, f: fn(&mut Self, T) -> UntypedVal) {
        let value = f(self, value);
        self.set_register(result, value);
        self.next_instr()
    }

    /// Executes an [`Instruction::Copy`].
    pub fn execute_copy(&mut self, result: Reg, value: Reg) {
        self.execute_copy_impl(result, value, |this, value| this.get_register(value))
    }

    /// Executes an [`Instruction::Copy2`].
    pub fn execute_copy_2(&mut self, results: FixedRegSpan<2>, values: [Reg; 2]) {
        self.execute_copy_2_impl(results, values);
        self.next_instr()
    }

    /// Internal implementation of [`Instruction::Copy2`] execution.
    fn execute_copy_2_impl(&mut self, results: FixedRegSpan<2>, values: [Reg; 2]) {
        let result0 = results.span().head();
        let result1 = result0.next();
        // We need `tmp` in case `results[0] == values[1]` to avoid overwriting `values[1]` before reading it.
        let tmp = self.get_register(values[1]);
        self.set_register(result0, self.get_register(values[0]));
        self.set_register(result1, tmp);
    }

    /// Executes an [`Instruction::CopyImm32`].
    pub fn execute_copy_imm32(&mut self, result: Reg, value: AnyConst32) {
        self.execute_copy_impl(result, value, |_, value| UntypedVal::from(u32::from(value)))
    }

    /// Executes an [`Instruction::CopyI64Imm32`].
    pub fn execute_copy_i64imm32(&mut self, result: Reg, value: Const32<i64>) {
        self.execute_copy_impl(result, value, |_, value| UntypedVal::from(i64::from(value)))
    }

    /// Executes an [`Instruction::CopyF64Imm32`].
    pub fn execute_copy_f64imm32(&mut self, result: Reg, value: Const32<f64>) {
        self.execute_copy_impl(result, value, |_, value| UntypedVal::from(f64::from(value)))
    }

    /// Executes an [`Instruction::CopySpan`].
    ///
    /// # Note
    ///
    /// - This instruction assumes that `results` and `values` do _not_ overlap
    ///   and thus can copy all the elements without a costly temporary buffer.
    /// - If `results` and `values` _do_ overlap [`Instruction::CopySpan`] is used.
    pub fn execute_copy_span(&mut self, results: RegSpan, values: RegSpan, len: u16) {
        self.execute_copy_span_impl(results, values, len);
        self.next_instr();
    }

    /// Internal implementation of [`Instruction::CopySpan`] execution.
    pub fn execute_copy_span_impl(&mut self, results: RegSpan, values: RegSpan, len: u16) {
        let results = results.iter(len);
        let values = values.iter(len);
        for (result, value) in results.into_iter().zip(values.into_iter()) {
            let value = self.get_register(value);
            self.set_register(result, value);
        }
    }

    /// Executes an [`Instruction::CopyMany`].
    pub fn execute_copy_many(&mut self, results: RegSpan, values: [Reg; 2]) {
        self.ip.add(1);
        self.ip = self.execute_copy_many_impl(self.ip, results, &values);
        self.next_instr()
    }

    /// Internal implementation of [`Instruction::CopyMany`] execution.
    pub fn execute_copy_many_impl(
        &mut self,
        ip: InstructionPtr,
        results: RegSpan,
        values: &[Reg],
    ) -> InstructionPtr {
        let mut ip = ip;
        let mut result = results.head();
        let mut copy_values = |values: &[Reg]| {
            for &value in values {
                let value = self.get_register(value);
                self.set_register(result, value);
                result = result.next();
            }
        };
        copy_values(values);
        while let Instruction::RegisterList { regs } = ip.get() {
            copy_values(regs);
            ip.add(1);
        }
        let values = match ip.get() {
            Instruction::Register { reg } => slice::from_ref(reg),
            Instruction::Register2 { regs } => regs,
            Instruction::Register3 { regs } => regs,
            unexpected => {
                // Safety: Wasmi translator guarantees that register-list finalizer exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected register-list finalizer but found: {unexpected:?}"
                    )
                }
            }
        };
        copy_values(values);
        ip
    }
}
