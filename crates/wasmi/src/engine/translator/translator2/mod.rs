#![expect(dead_code)]

#[cfg(feature = "simd")]
mod simd;
mod stack;
mod visit;

use self::stack::{Operand, OperandIdx, Stack};
use crate::{
    core::FuelCostsProvider,
    engine::{translator::WasmTranslator, CompiledFuncEntity},
    module::{FuncIdx, ModuleHeader},
    Engine,
    Error,
};
use wasmparser::WasmFeatures;

/// Type concerned with translating from Wasm bytecode to Wasmi bytecode.
#[derive(Debug)]
pub struct FuncTranslator {
    /// The reference to the Wasm module function under construction.
    func: FuncIdx,
    /// The engine for which the function is compiled.
    ///
    /// # Note
    ///
    /// Technically this is not needed since the information is redundant given via
    /// the `module` field. However, this acts like a faster access since `module`
    /// only holds a weak reference to the engine.
    engine: Engine,
    /// The immutable Wasmi module resources.
    module: ModuleHeader,
    /// This represents the reachability of the currently translated code.
    ///
    /// - `true`: The currently translated code is reachable.
    /// - `false`: The currently translated code is unreachable and can be skipped.
    ///
    /// # Note
    ///
    /// Visiting the Wasm `Else` or `End` control flow operator resets
    /// reachability to `true` again.
    reachable: bool,
    /// Fuel costs for fuel metering.
    ///
    /// `None` if fuel metering is disabled.
    fuel_costs: Option<FuelCostsProvider>,
    /// The reusable data structures of the [`FuncTranslator`].
    alloc: FuncTranslatorAllocations,
}

#[derive(Debug, Default)]
pub struct FuncTranslatorAllocations {
    stack: Stack,
}

impl WasmTranslator<'_> for FuncTranslator {
    type Allocations = FuncTranslatorAllocations;

    fn setup(&mut self, _bytes: &[u8]) -> Result<bool, Error> {
        Ok(false)
    }

    #[inline]
    fn features(&self) -> WasmFeatures {
        self.engine.config().wasm_features()
    }

    fn translate_locals(
        &mut self,
        _amount: u32,
        _value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        todo!()
    }

    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        todo!()
    }

    fn update_pos(&mut self, _pos: usize) {}

    fn finish(
        self,
        _finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        todo!()
    }
}
