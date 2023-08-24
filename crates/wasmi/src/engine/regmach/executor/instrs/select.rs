use super::Executor;
use crate::engine::{
    bytecode2::{AnyConst32, Const32, Instruction, Register},
    code_map::InstructionPtr2 as InstructionPtr,
};
use wasmi_core::UntypedValue;

/// Fetches the parameters for a `select` instruction with immutable `lhs` and `rhs`.
macro_rules! fetch_select_imm_param {
    ( $this:expr, $variant:ident ) => {{
        let mut addr: InstructionPtr = $this.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::$variant {
                result_or_condition,
                lhs_or_rhs,
            } => (result_or_condition, lhs_or_rhs),
            unexpected => ::core::unreachable!(
                "expected {} but found {unexpected:?}",
                ::core::stringify!($variant)
            ),
        }
    }};
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Returns the parameter of [`Instruction::Select`] or [`Instruction::SelectRev`] as [`UntypedValue`].
    fn fetch_select_param(&self) -> UntypedValue {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::Register(register) => self.get_register(register),
            Instruction::Const32(value) => UntypedValue::from(value.to_u32()),
            Instruction::I64Const32(value) => UntypedValue::from(i64::from(value)),
            Instruction::F64Const32(value) => UntypedValue::from(f64::from(value)),
            unexpected => unreachable!(
                "expected a select parameter instruction word but found {unexpected:?}"
            ),
        }
    }

    /// Executes a `select` instruction generically.
    fn execute_select_impl<L, R>(
        &mut self,
        result: Register,
        condition: Register,
        lhs: impl FnOnce(&Self) -> L,
        rhs: impl FnOnce(&Self) -> R,
    ) where
        L: Into<UntypedValue>,
        R: Into<UntypedValue>,
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
    pub fn execute_select(&mut self, result: Register, condition: Register, lhs: Register) {
        self.execute_select_impl(
            result,
            condition,
            |this| this.get_register(lhs),
            Self::fetch_select_param,
        )
    }

    /// Executes an [`Instruction::SelectRev`].
    pub fn execute_select_rev(&mut self, result: Register, condition: Register, rhs: Register) {
        self.execute_select_impl(result, condition, Self::fetch_select_param, |this| {
            this.get_register(rhs)
        })
    }

    /// Executes an [`Instruction::SelectImm32`].
    pub fn execute_select_imm32(&mut self, result: Register, lhs: AnyConst32) {
        let (condition, rhs) = fetch_select_imm_param!(self, SelectImm32);
        self.execute_select_impl(result, condition, |_| lhs.to_u32(), |_| rhs.to_u32())
    }

    /// Executes an [`Instruction::SelectI64Imm32`].
    pub fn execute_select_i64imm32(&mut self, result: Register, lhs: Const32<i64>) {
        let (condition, rhs) = fetch_select_imm_param!(self, SelectI64Imm32);
        self.execute_select_impl(result, condition, |_| i64::from(lhs), |_| i64::from(rhs))
    }

    /// Executes an [`Instruction::SelectF64Imm32`].
    pub fn execute_select_f64imm32(&mut self, result: Register, lhs: Const32<f64>) {
        let (condition, rhs) = fetch_select_imm_param!(self, SelectF64Imm32);
        self.execute_select_impl(result, condition, |_| f64::from(lhs), |_| f64::from(rhs))
    }
}
