#![expect(dead_code, unused_imports, unused_variables)]

#[macro_use]
mod utils;
mod instrs;
mod layout;
#[cfg(feature = "simd")]
mod simd;
mod stack;
mod visit;

use self::{
    instrs::InstrEncoder,
    layout::{StackLayout, StackSpace},
    stack::{LocalIdx, Operand, OperandIdx, Stack},
    utils::Reset,
};
use crate::{
    core::FuelCostsProvider,
    engine::{
        translator::{Instr, LabelRegistry, WasmTranslator},
        BlockType,
        CompiledFuncEntity,
    },
    ir::Instruction,
    module::{FuncIdx, ModuleHeader, WasmiValueType},
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
    /// Wasm value and control stack.
    stack: Stack,
    /// Wasm layout to map stack slots to Wasmi registers.
    layout: StackLayout,
    /// Registers and pins labels and tracks their users.
    labels: LabelRegistry,
    /// Constructs and encodes function instructions.
    instrs: InstrEncoder,
}

/// Heap allocated data structured used by the [`FuncTranslator`].
#[derive(Debug, Default)]
pub struct FuncTranslatorAllocations {
    /// Wasm value and control stack.
    stack: Stack,
    /// Wasm layout to map stack slots to Wasmi registers.
    layout: StackLayout,
    /// Registers and pins labels and tracks their users.
    labels: LabelRegistry,
    /// Constructs and encodes function instructions.
    instrs: InstrEncoder,
}

impl Reset for FuncTranslatorAllocations {
    fn reset(&mut self) {
        self.stack.reset();
        self.layout.reset();
        self.labels.reset();
        self.instrs.reset();
    }
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
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        let ty = WasmiValueType::from(value_type).into_inner();
        self.stack.register_locals(amount, ty)?;
        self.layout.register_locals(amount, ty)?;
        Ok(())
    }

    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn update_pos(&mut self, _pos: usize) {}

    fn finish(
        self,
        _finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        todo!()
    }
}

impl FuncTranslator {
    /// Creates a new [`FuncTranslator`].
    pub fn new(
        func: FuncIdx,
        module: ModuleHeader,
        alloc: FuncTranslatorAllocations,
    ) -> Result<Self, Error> {
        let Some(engine) = module.engine().upgrade() else {
            panic!(
                "cannot compile function since engine does no longer exist: {:?}",
                module.engine()
            )
        };
        let config = engine.config();
        let fuel_costs = config
            .get_consume_fuel()
            .then(|| config.fuel_costs())
            .cloned();
        let FuncTranslatorAllocations {
            mut stack,
            layout,
            mut labels,
            mut instrs,
        } = {
            let mut alloc = alloc;
            alloc.reset();
            alloc
        };
        let func_ty = module.get_type_of_func(func);
        let block_ty = BlockType::func_type(func_ty);
        let end_label = labels.new_label();
        let consume_fuel = fuel_costs
            .as_ref()
            .map(|_| instrs.push_instr(Instruction::consume_fuel(1)));
        stack.push_block(block_ty, end_label, consume_fuel)?;
        Ok(Self {
            func,
            engine,
            module,
            reachable: true,
            fuel_costs,
            stack,
            layout,
            labels,
            instrs,
        })
    }

    /// Consumes `self` and returns the underlying reusable [`FuncTranslatorAllocations`].
    fn into_allocations(self) -> FuncTranslatorAllocations {
        FuncTranslatorAllocations {
            stack: self.stack,
            layout: self.layout,
            labels: self.labels,
            instrs: self.instrs,
        }
    }

    /// Returns the [`Engine`] for which the function is compiled.
    fn engine(&self) -> &Engine {
        &self.engine
    }
}
