#![allow(missing_docs, dead_code)] // TODO: remove

mod compile;
mod error;

use self::compile::FuncBodyTranslator;
use self::error::TranslationError;
use super::interpreter::InstructionsBuilder;
use super::Engine;
use core::mem;
use parity_wasm::elements as pwasm;
use validation::{validate_module, FuncValidator, Validator};

/// A compiled and validated WebAssembly module.
///
/// Can be used to create new [`Instance`] instantiations.
///
/// [`Instance`]: [`super::Instance`]
#[derive(Debug)]
pub struct Module {
    module: pwasm::Module,
}

#[derive(Debug)]
pub struct ModuleValidation {
    engine: Engine,
    inst_builder: InstructionsBuilder,
}

impl Validator for ModuleValidation {
    type Input = Engine;
    type Output = Self;
    type FuncValidator = FuncBodyTranslator;

    fn new(_module: &pwasm::Module, engine: Self::Input) -> Self {
        ModuleValidation {
            engine,
            inst_builder: InstructionsBuilder::default(),
        }
    }

    fn on_function_validated(
        &mut self,
        _index: u32,
        inst_builder: <Self::FuncValidator as FuncValidator>::Output,
    ) {
        self.inst_builder = inst_builder;
    }

    fn finish(self) -> Self::Output {
        self
    }

    fn func_validator_input(
        &mut self,
    ) -> <Self::FuncValidator as validation::FuncValidator>::Input {
        let inst_builder = mem::take(&mut self.inst_builder);
        (self.engine.clone(), inst_builder)
    }
}

impl Module {
    /// Create a new module from the binary Wasm encoded bytes.
    pub fn new(engine: &Engine, bytes: impl AsRef<[u8]>) -> Result<Module, TranslationError> {
        let module = pwasm::deserialize_buffer(bytes.as_ref())?;
        validate_module::<ModuleValidation>(&module, engine.clone())?;
        Ok(Self { module })
    }
}
