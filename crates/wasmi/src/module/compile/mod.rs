pub use self::block_type::BlockType;
use super::{FuncIdx, ModuleHeader};
use crate::{
    engine::{
        CompiledFunc,
        FuncTranslator,
        FuncTranslatorAllocations,
        ReusableAllocations,
        ValidatingFuncTranslator,
        WasmTranslator,
    },
    errors::ModuleError,
};
use wasmparser::{FuncValidator, FunctionBody, ValidatorResources};

mod block_type;

/// Validates and translates the Wasm bytecode into `wasmi` IR bytecode.
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
/// If the function body fails to validate or translate the Wasm function body.
pub fn translate(
    func: FuncIdx,
    compiled_func: CompiledFunc,
    func_body: FunctionBody,
    validator: FuncValidator<ValidatorResources>,
    res: ModuleHeader,
    allocations: FuncTranslatorAllocations,
) -> Result<ReusableAllocations, ModuleError> {
    <FuncTranslationDriver<'_, ValidatingFuncTranslator>>::new(
        func,
        compiled_func,
        func_body,
        validator,
        res,
        allocations,
    )?
    .translate()
}

/// Translates the Wasm bytecode into `wasmi` IR bytecode.
///
/// # Note
///
/// - Uses the given `engine` as target for the translation.
/// - Uses the given module resources `res` as shared immutable data of the
///   already parsed and validated module parts required for the translation.
/// - Does _not_ validate the Wasm input.
///
/// # Errors
///
/// If the function body fails to translate the Wasm function body.
pub fn translate_unchecked(
    func: FuncIdx,
    compiled_func: CompiledFunc,
    func_body: FunctionBody,
    res: ModuleHeader,
    allocations: FuncTranslatorAllocations,
) -> Result<FuncTranslatorAllocations, ModuleError> {
    <FuncTranslationDriver<'_, FuncTranslator>>::new(
        func,
        compiled_func,
        func_body,
        res,
        allocations,
    )?
    .translate()
}

/// Translates Wasm bytecode into `wasmi` bytecode for a single Wasm function.
struct FuncTranslationDriver<'parser, T> {
    /// The function body that shall be translated.
    func_body: FunctionBody<'parser>,
    /// The underlying translator used for the translation (and validation) process.
    translator: T,
}

impl<'parser> FuncTranslationDriver<'parser, ValidatingFuncTranslator> {
    /// Creates a new Wasm to `wasmi` bytecode function translator.
    fn new(
        func: FuncIdx,
        compiled_func: CompiledFunc,
        func_body: FunctionBody<'parser>,
        validator: FuncValidator<ValidatorResources>,
        res: ModuleHeader,
        allocations: FuncTranslatorAllocations,
    ) -> Result<Self, ModuleError> {
        let translator =
            ValidatingFuncTranslator::new(func, compiled_func, res, validator, allocations)?;
        Ok(Self {
            func_body,
            translator,
        })
    }
}

impl<'parser> FuncTranslationDriver<'parser, FuncTranslator> {
    /// Creates a new Wasm to `wasmi` bytecode function translator.
    fn new(
        func: FuncIdx,
        compiled_func: CompiledFunc,
        func_body: FunctionBody<'parser>,
        res: ModuleHeader,
        allocations: FuncTranslatorAllocations,
    ) -> Result<Self, ModuleError> {
        let translator = FuncTranslator::new(func, compiled_func, res, allocations)?;
        Ok(Self {
            func_body,
            translator,
        })
    }
}

impl<'parser, T> FuncTranslationDriver<'parser, T>
where
    T: WasmTranslator<'parser>,
{
    /// Starts translation of the Wasm stream into `wasmi` bytecode.
    fn translate(mut self) -> Result<T::Allocations, ModuleError> {
        self.translate_locals()?;
        let offset = self.translate_operators()?;
        let allocations = self.finish(offset)?;
        Ok(allocations)
    }

    /// Finishes construction of the function and returns its [`CompiledFunc`].
    fn finish(mut self, offset: usize) -> Result<T::Allocations, ModuleError> {
        self.translator.update_pos(offset);
        self.translator.finish().map_err(Into::into)
    }

    /// Translates local variables of the Wasm function.
    fn translate_locals(&mut self) -> Result<(), ModuleError> {
        let mut reader = self.func_body.get_locals_reader()?;
        let len_locals = reader.get_count();
        for _ in 0..len_locals {
            let offset = reader.original_position();
            let (amount, value_type) = reader.read()?;
            self.translator.update_pos(offset);
            self.translator.translate_locals(amount, value_type)?;
        }
        self.translator.finish_translate_locals()?;
        Ok(())
    }

    /// Translates the Wasm operators of the Wasm function.
    ///
    /// Returns the offset of the `End` Wasm operator.
    fn translate_operators(&mut self) -> Result<usize, ModuleError> {
        let mut reader = self.func_body.get_operators_reader()?;
        while !reader.eof() {
            let pos = reader.original_position();
            self.translator.update_pos(pos);
            reader.visit_operator(&mut self.translator)??;
        }
        reader.ensure_end()?;
        Ok(reader.original_position())
    }
}
