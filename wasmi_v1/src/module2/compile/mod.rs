use super::ModuleResources;
use crate::{Engine, ModuleError};
use wasmparser::{FuncValidator, Parser, ValidatorResources};

// mod block_type;
// mod control_frame;

/// Translates the Wasm bytecode into `wasmi` bytecode.
///
/// # Note
///
/// - Uses the given `engine` as target for the translation.
/// - Uses the given `parser` and `validator` for parsing and validation of
///   the incoming Wasm bytecode stream.
/// - Uses the given module resources `res` as shared immutable data of the
///   already parsed and validated module parts required for the translation.
///
/// # Errors
///
/// If the function body fails to validate.
pub fn translate<'parser>(
    engine: &Engine,
    parser: &'parser mut Parser,
    validator: FuncValidator<ValidatorResources>,
    res: ModuleResources<'parser>,
) -> Result<(), ModuleError> {
    FunctionTranslator::new(engine, parser, validator, res).translate()
}

/// Translates Wasm bytecode into `wasmi` bytecode for a single Wasm function.
struct FunctionTranslator<'parser> {
    /// The target `wasmi` engine for `wasmi` bytecode translation.
    engine: Engine,
    /// The Wasm parser.
    parser: &'parser mut Parser,
    /// The Wasm validator.
    validator: FuncValidator<ValidatorResources>,
    /// The `wasmi` module resources.
    ///
    /// Provides immutable information about the translated Wasm module
    /// required for function translation to `wasmi` bytecode.
    res: ModuleResources<'parser>,
}

impl<'parser> FunctionTranslator<'parser> {
    /// Creates a new Wasm to `wasmi` bytecode function translator.
    fn new(
        engine: &Engine,
        parser: &'parser mut Parser,
        validator: FuncValidator<ValidatorResources>,
        res: ModuleResources<'parser>,
    ) -> Self {
        Self {
            engine: engine.clone(),
            parser,
            validator,
            res,
        }
    }

    /// Starts translation of the Wasm stream into `wasmi` bytecode.
    fn translate(&mut self) -> Result<(), ModuleError> {
        todo!()
    }
}
