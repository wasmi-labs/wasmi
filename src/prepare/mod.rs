use crate::{
    isa,
    validation::{validate_module2, Error, Validation},
};
use parity_wasm::elements::Module;

mod compile;

#[derive(Clone)]
pub struct CompiledModule {
    pub code_map: Vec<isa::Instructions>,
    pub module: Module,
}

pub struct WasmiValidation {
    code_map: Vec<isa::Instructions>,
}

impl Validation for WasmiValidation {
    type Output = Vec<isa::Instructions>;
    type FunctionValidator = compile::Compiler;
    fn new(_module: &Module) -> Self {
        WasmiValidation {
            // TODO: with capacity?
            code_map: Vec::new(),
        }
    }
    fn on_function_validated(&mut self, _index: u32, output: isa::Instructions) {
        self.code_map.push(output);
    }
    fn finish(self) -> Vec<isa::Instructions> {
        self.code_map
    }
}

/// Validate a module and compile it to the internal representation.
pub fn compile_module(module: Module) -> Result<CompiledModule, Error> {
    let code_map = validate_module2::<WasmiValidation>(&module)?;
    Ok(CompiledModule { module, code_map })
}
