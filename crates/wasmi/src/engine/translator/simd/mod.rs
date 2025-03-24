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
}
