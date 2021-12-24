use crate::isa;
use alloc::vec::Vec;
use parity_wasm::elements::Module;
use validation::{validate_module, Error, Validator};

#[cfg(not(feature = "std"))]
use crate::alloc::string::ToString;

mod compile;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct CompiledModule {
    pub code_map: Vec<isa::Instructions>,
    pub module: Module,
}

pub struct WasmiValidation {
    code_map: Vec<isa::Instructions>,
}

// This implementation of `Validation` is compiling wasm code at the
// validation time.
impl Validator for WasmiValidation {
    type Input = ();
    type Output = Vec<isa::Instructions>;
    type FuncValidator = compile::Compiler;
    fn new(_module: &Module, _input: Self::Input) -> Self {
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

    fn func_validator_input(
        &mut self,
    ) -> <Self::FuncValidator as validation::FuncValidator>::Input {
    }
}

/// Validate a module and compile it to the internal representation.
pub fn compile_module(module: Module) -> Result<CompiledModule, Error> {
    let code_map = validate_module::<WasmiValidation>(&module, ())?;
    Ok(CompiledModule { code_map, module })
}

/// Verify that the module doesn't use floating point instructions or types.
///
/// Returns `Err` if
///
/// - Any of function bodies uses a floating pointer instruction (an instruction that
///   consumes or produces a value of a floating point type)
/// - If a floating point type used in a definition of a function.
pub fn deny_floating_point(module: &Module) -> Result<(), Error> {
    use parity_wasm::elements::{
        Instruction::{self, *},
        Type,
        ValueType,
    };

    if let Some(code) = module.code_section() {
        for op in code.bodies().iter().flat_map(|body| body.code().elements()) {
            macro_rules! match_eq {
                ($pattern:pat) => {
                    |val| if let $pattern = *val { true } else { false }
                };
            }

            const DENIED: &[fn(&Instruction) -> bool] = &[
                match_eq!(F32Load(_, _)),
                match_eq!(F64Load(_, _)),
                match_eq!(F32Store(_, _)),
                match_eq!(F64Store(_, _)),
                match_eq!(F32Const(_)),
                match_eq!(F64Const(_)),
                match_eq!(F32Eq),
                match_eq!(F32Ne),
                match_eq!(F32Lt),
                match_eq!(F32Gt),
                match_eq!(F32Le),
                match_eq!(F32Ge),
                match_eq!(F64Eq),
                match_eq!(F64Ne),
                match_eq!(F64Lt),
                match_eq!(F64Gt),
                match_eq!(F64Le),
                match_eq!(F64Ge),
                match_eq!(F32Abs),
                match_eq!(F32Neg),
                match_eq!(F32Ceil),
                match_eq!(F32Floor),
                match_eq!(F32Trunc),
                match_eq!(F32Nearest),
                match_eq!(F32Sqrt),
                match_eq!(F32Add),
                match_eq!(F32Sub),
                match_eq!(F32Mul),
                match_eq!(F32Div),
                match_eq!(F32Min),
                match_eq!(F32Max),
                match_eq!(F32Copysign),
                match_eq!(F64Abs),
                match_eq!(F64Neg),
                match_eq!(F64Ceil),
                match_eq!(F64Floor),
                match_eq!(F64Trunc),
                match_eq!(F64Nearest),
                match_eq!(F64Sqrt),
                match_eq!(F64Add),
                match_eq!(F64Sub),
                match_eq!(F64Mul),
                match_eq!(F64Div),
                match_eq!(F64Min),
                match_eq!(F64Max),
                match_eq!(F64Copysign),
                match_eq!(F32ConvertSI32),
                match_eq!(F32ConvertUI32),
                match_eq!(F32ConvertSI64),
                match_eq!(F32ConvertUI64),
                match_eq!(F32DemoteF64),
                match_eq!(F64ConvertSI32),
                match_eq!(F64ConvertUI32),
                match_eq!(F64ConvertSI64),
                match_eq!(F64ConvertUI64),
                match_eq!(F64PromoteF32),
                match_eq!(F32ReinterpretI32),
                match_eq!(F64ReinterpretI64),
                match_eq!(I32TruncSF32),
                match_eq!(I32TruncUF32),
                match_eq!(I32TruncSF64),
                match_eq!(I32TruncUF64),
                match_eq!(I64TruncSF32),
                match_eq!(I64TruncUF32),
                match_eq!(I64TruncSF64),
                match_eq!(I64TruncUF64),
                match_eq!(I32ReinterpretF32),
                match_eq!(I64ReinterpretF64),
            ];

            if DENIED.iter().any(|is_denied| is_denied(op)) {
                return Err(Error(format!("Floating point operation denied: {:?}", op)));
            }
        }
    }

    if let (Some(sec), Some(types)) = (module.function_section(), module.type_section()) {
        let types = types.types();

        for sig in sec.entries() {
            if let Some(typ) = types.get(sig.type_ref() as usize) {
                match *typ {
                    Type::Function(ref func) => {
                        if func
                            .params()
                            .iter()
                            .chain(func.results())
                            .any(|&typ| typ == ValueType::F32 || typ == ValueType::F64)
                        {
                            return Err(Error("Use of floating point types denied".to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
