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

    /// Generically translate any of the Wasm `simd` extract lane instructions.
    fn translate_extract_lane<T: IntoLaneIdx, R>(
        &mut self,
        lane: u8,
        make_instr: fn(result: Reg, input: Reg, lane: T::LaneIdx) -> Instruction,
        const_eval: fn(input: V128, lane: T::LaneIdx) -> R,
    ) -> Result<(), Error>
    where
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        let Ok(lane) = <T::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}")
        };
        let input = self.stack.pop();
        if let Operand::Immediate(input) = input {
            let result = const_eval(input.val().into(), lane);
            self.stack.push_immediate(result)?;
            return Ok(());
        };
        let input = self.layout.operand_to_reg(input)?;
        self.push_instr_with_result(
            <R as Typed>::TY,
            |result| make_instr(result, input, lane),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }
}
