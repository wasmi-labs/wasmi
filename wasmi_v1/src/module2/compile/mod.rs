use super::{utils::value_type_from_wasmparser, ModuleResources};
use crate::{Engine, ModuleError};
use wasmparser::{FuncValidator, FunctionBody, Operator, Parser, ValidatorResources};

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
    func_body: FunctionBody<'parser>,
    parser: &'parser mut Parser,
    validator: FuncValidator<ValidatorResources>,
    res: ModuleResources<'parser>,
) -> Result<(), ModuleError> {
    FunctionTranslator::new(engine, func_body, parser, validator, res).translate()
}

/// Translates Wasm bytecode into `wasmi` bytecode for a single Wasm function.
struct FunctionTranslator<'parser> {
    /// The target `wasmi` engine for `wasmi` bytecode translation.
    engine: Engine,
    /// The function body that shall be translated.
    func_body: FunctionBody<'parser>,
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
        func_body: FunctionBody<'parser>,
        parser: &'parser mut Parser,
        validator: FuncValidator<ValidatorResources>,
        res: ModuleResources<'parser>,
    ) -> Self {
        Self {
            engine: engine.clone(),
            func_body,
            parser,
            validator,
            res,
        }
    }

    /// Starts translation of the Wasm stream into `wasmi` bytecode.
    fn translate(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates local variables of the Wasm function.
    fn translate_locals(&mut self) -> Result<(), ModuleError> {
        let mut reader = self.func_body.get_locals_reader()?;
        let len_locals = reader.get_count();
        for _ in 0..len_locals {
            let offset = reader.original_position();
            let (amount, value_type) = reader.read()?;
            self.validator.define_locals(offset, amount, value_type)?;
            let value_type = value_type_from_wasmparser(&value_type)?;
            todo!() // TODO: inform backend about local variables
        }
        Ok(())
    }

    /// Translates the Wasm operators of the Wasm function.
    fn translate_operators(&mut self) -> Result<(), ModuleError> {
        let mut reader = self.func_body.get_operators_reader()?;
        while !reader.eof() {
            let (operator, offset) = reader.read_with_offset()?;
            self.validator.op(offset, &operator)?;
            self.translate_operator(operator, offset)?;
        }
        reader.ensure_end()?;
        Ok(())
    }

    /// Translate a single Wasm operator of the Wasm function.
    fn translate_operator(&mut self, operator: Operator, offset: usize) -> Result<(), ModuleError> {
        // TODO: inform backend about Wasm operator
        Ok(())
    }
}
