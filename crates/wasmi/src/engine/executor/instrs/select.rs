use super::{Executor, InstructionPtr};
use crate::{
    core::UntypedVal,
    engine::utils::unreachable_unchecked,
    ir::{AnyConst32, Const32, Instruction, Reg},
};

impl<'engine> Executor<'engine> {
    /// Fetches two [`Reg`]s.
    fn fetch_register_2(&self) -> (Reg, Reg) {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::Register2 { regs: [reg0, reg1] } => (reg0, reg1),
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::Register2`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::Register2` but found {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Fetches a [`Reg`] and a 32-bit immediate value of type `T`.
    fn fetch_register_and_imm32<T>(&self) -> (Reg, T)
    where
        T: From<AnyConst32>,
    {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::RegisterAndImm32 { reg, imm } => (reg, T::from(imm)),
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::RegisterAndImm32`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::RegisterAndImm32` but found {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Executes a `select` instruction generically.
    fn execute_select_impl<L, R>(
        &mut self,
        result: Reg,
        condition: Reg,
        lhs: impl FnOnce(&Self) -> L,
        rhs: impl FnOnce(&Self) -> R,
    ) where
        L: Into<UntypedVal>,
        R: Into<UntypedVal>,
    {
        let condition: bool = self.get_register_as(condition);
        let selected = match condition {
            true => lhs(self).into(),
            false => rhs(self).into(),
        };
        self.set_register(result, selected);
        self.next_instr_at(2);
    }

    /// Executes an [`Instruction::Select`].
    pub fn execute_select(&mut self, result: Reg, lhs: Reg) {
        let (condition, rhs) = self.fetch_register_2();
        self.execute_select_impl(
            result,
            condition,
            |this| this.get_register(lhs),
            |this| this.get_register(rhs),
        )
    }

    /// Executes an [`Instruction::SelectImm32Rhs`].
    pub fn execute_select_imm32_rhs(&mut self, result: Reg, lhs: Reg) {
        let (condition, rhs) = self.fetch_register_and_imm32::<AnyConst32>();
        self.execute_select_impl(
            result,
            condition,
            |this: &Executor<'engine>| this.get_register(lhs),
            |_| u32::from(rhs),
        )
    }

    /// Executes an [`Instruction::SelectImm32Lhs`].
    pub fn execute_select_imm32_lhs(&mut self, result: Reg, lhs: AnyConst32) {
        let (condition, rhs) = self.fetch_register_2();
        self.execute_select_impl(
            result,
            condition,
            |_| u32::from(lhs),
            |this| this.get_register(rhs),
        )
    }

    /// Executes an [`Instruction::SelectImm32`].
    pub fn execute_select_imm32(&mut self, result: Reg, lhs: AnyConst32) {
        let (condition, rhs) = self.fetch_register_and_imm32::<AnyConst32>();
        self.execute_select_impl(result, condition, |_| u32::from(lhs), |_| u32::from(rhs))
    }

    /// Executes an [`Instruction::SelectI64Imm32Rhs`].
    pub fn execute_select_i64imm32_rhs(&mut self, result: Reg, lhs: Reg) {
        let (condition, rhs) = self.fetch_register_and_imm32::<i32>();
        self.execute_select_impl(
            result,
            condition,
            |this| this.get_register(lhs),
            |_| i64::from(rhs),
        )
    }

    /// Executes an [`Instruction::SelectI64Imm32Lhs`].
    pub fn execute_select_i64imm32_lhs(&mut self, result: Reg, lhs: Const32<i64>) {
        let (condition, rhs) = self.fetch_register_2();
        self.execute_select_impl(
            result,
            condition,
            |_| i64::from(lhs),
            |this| this.get_register(rhs),
        )
    }

    /// Executes an [`Instruction::SelectI64Imm32`].
    pub fn execute_select_i64imm32(&mut self, result: Reg, lhs: Const32<i64>) {
        let (condition, rhs) = self.fetch_register_and_imm32::<i32>();
        self.execute_select_impl(result, condition, |_| i64::from(lhs), |_| i64::from(rhs))
    }

    /// Executes an [`Instruction::SelectF64Imm32Rhs`].
    pub fn execute_select_f64imm32_rhs(&mut self, result: Reg, lhs: Reg) {
        let (condition, rhs) = self.fetch_register_and_imm32::<f32>();
        self.execute_select_impl(
            result,
            condition,
            |this| this.get_register(lhs),
            |_| f64::from(rhs),
        )
    }

    /// Executes an [`Instruction::SelectF64Imm32Lhs`].
    pub fn execute_select_f64imm32_lhs(&mut self, result: Reg, lhs: Const32<f64>) {
        let (condition, rhs) = self.fetch_register_2();
        self.execute_select_impl(
            result,
            condition,
            |_| f64::from(lhs),
            |this| this.get_register(rhs),
        )
    }

    /// Executes an [`Instruction::SelectF64Imm32`].
    pub fn execute_select_f64imm32(&mut self, result: Reg, lhs: Const32<f64>) {
        let (condition, rhs) = self.fetch_register_and_imm32::<f32>();
        self.execute_select_impl(result, condition, |_| f64::from(lhs), |_| f64::from(rhs))
    }
}
