use super::FuncTranslator;

mod visit;

use crate::{
    core::{simd::IntoLaneIdx, FuelCostsProvider, Typed, TypedVal, ValType, V128},
    engine::translator::{func2::Operand, utils::Wrap},
    ir::{
        Instruction,
        Reg,
    },
    Error,
};

impl FuncTranslator {
    /// Generically translate any of the Wasm `simd` splat instructions.
    fn translate_simd_splat<T, Wrapped>(
        &mut self,
        make_instr: fn(result: Reg, value: Reg) -> Instruction,
        const_eval: fn(Wrapped) -> V128,
    ) -> Result<(), Error>
    where
        T: From<TypedVal> + Wrap<Wrapped>,
    {
        bail_unreachable!(self);
        let value = self.stack.pop();
        if let Operand::Immediate(value) = value {
            let value = T::from(value.val()).wrap();
            let result = const_eval(value);
            self.stack.push_immediate(result)?;
            return Ok(());
        };
        let value = self.layout.operand_to_reg(value)?;
        self.push_instr_with_result(
            ValType::V128,
            |result| make_instr(result, value),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }
}
