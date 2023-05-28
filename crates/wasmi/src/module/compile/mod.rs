pub use self::block_type::BlockType;
use super::{parser::ReusableAllocations, FuncIdx, ModuleResources};
use crate::{
    engine::{CompiledFunc, FuncBuilder, FuncTranslatorAllocations},
    errors::ModuleError,
};
use wasmparser::{FuncValidator, FunctionBody, ValidatorResources};

mod block_type;

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
    func: FuncIdx,
    compiled_func: CompiledFunc,
    func_body: FunctionBody<'parser>,
    validator: FuncValidator<ValidatorResources>,
    res: ModuleResources<'parser>,
    allocations: FuncTranslatorAllocations,
) -> Result<ReusableAllocations, ModuleError> {
    FunctionTranslator::new(func, compiled_func, func_body, validator, res, allocations).translate()
}

/// Translates Wasm bytecode into `wasmi` bytecode for a single Wasm function.
struct FunctionTranslator<'parser> {
    /// The function body that shall be translated.
    func_body: FunctionBody<'parser>,
    /// The interface to incrementally build up the `wasmi` bytecode function.
    func_builder: FuncBuilder<'parser>,
}

impl<'parser> FunctionTranslator<'parser> {
    /// Creates a new Wasm to `wasmi` bytecode function translator.
    fn new(
        func: FuncIdx,
        compiled_func: CompiledFunc,
        func_body: FunctionBody<'parser>,
        validator: FuncValidator<ValidatorResources>,
        res: ModuleResources<'parser>,
        allocations: FuncTranslatorAllocations,
    ) -> Self {
        let func_builder = FuncBuilder::new(func, compiled_func, res, validator, allocations);
        Self {
            func_body,
            func_builder,
        }
    }

    /// Starts translation of the Wasm stream into `wasmi` bytecode.
    fn translate(mut self) -> Result<ReusableAllocations, ModuleError> {
        self.translate_locals()?;
        let offset = self.translate_operators()?;
        let allocations = self.finish(offset)?;
        Ok(allocations)
    }

    /// Finishes construction of the function and returns its [`CompiledFunc`].
    fn finish(self, offset: usize) -> Result<ReusableAllocations, ModuleError> {
        self.func_builder.finish(offset).map_err(Into::into)
    }

    /// Translates local variables of the Wasm function.
    fn translate_locals(&mut self) -> Result<(), ModuleError> {
        let mut reader = self.func_body.get_locals_reader()?;
        let len_locals = reader.get_count();
        for _ in 0..len_locals {
            let offset = reader.original_position();
            let (amount, value_type) = reader.read()?;
            self.func_builder
                .translate_locals(offset, amount, value_type)?;
        }
        self.func_builder.finish_translate_locals()?;
        Ok(())
    }

    /// Translates the Wasm operators of the Wasm function.
    ///
    /// Returns the offset of the `End` Wasm operator.
    fn translate_operators(&mut self) -> Result<usize, ModuleError> {
        let mut reader = self.func_body.get_operators_reader()?;
        while !reader.eof() {
            let pos = reader.original_position();
            self.func_builder.update_pos(pos);
            reader.visit_operator(&mut self.func_builder)??;
        }
        reader.ensure_end()?;
        Ok(reader.original_position())
    }
}
