mod visit;

use super::{utils::Wrap, FuncTranslator};
use crate::{
    core::{TypedVal, V128},
    engine::{translator::provider::Provider, FuelCosts},
    ir::{Instruction, Reg},
    Error,
};

impl FuncTranslator {
    /// Generically translate any of the Wasm `simd` splat instruction.
    fn translate_simd_splat<T, Wrapped>(
        &mut self,
        make_instr: fn(result: Reg, value: Reg) -> Instruction,
        const_eval: fn(Wrapped) -> V128,
    ) -> Result<(), Error>
    where
        T: From<TypedVal> + Wrap<Wrapped>,
    {
        bail_unreachable!(self);
        let value = self.alloc.stack.pop();
        let value = match value {
            Provider::Register(value) => value,
            Provider::Const(value) => {
                let value = T::from(value).wrap();
                let result = const_eval(value);
                self.alloc.stack.push_const(result);
                return Ok(());
            }
        };
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, value), FuelCosts::base)?;
        Ok(())
    }

    /// Generically translate a Wasm binary instruction.
    fn translate_simd_binary(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        const_eval: fn(lhs: V128, rhs: V128) -> V128,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (lhs, rhs) = self.alloc.stack.pop2();
        if let (Provider::Const(lhs), Provider::Const(rhs)) = (lhs, rhs) {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.into(), rhs.into());
            self.alloc.stack.push_const(result);
            return Ok(());
        }
        let result = self.alloc.stack.push_dynamic()?;
        let lhs = self.alloc.stack.provider2reg(&lhs)?;
        let rhs = self.alloc.stack.provider2reg(&rhs)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCosts::base)?;
        Ok(())
    }
}
