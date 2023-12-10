pub use self::block_type::BlockType;
use crate::{engine::WasmTranslator, errors::ModuleError};
use wasmparser::FunctionBody;

mod block_type;

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
pub fn translate<'a, T>(
    func_body: FunctionBody<'a>,
    bytes: &'a [u8],
    translator: T,
) -> Result<T::Allocations, ModuleError>
where
    T: WasmTranslator<'a>,
{
    <FuncTranslationDriver<'a, T>>::new(func_body, bytes, translator)?.translate()
}

/// Translates Wasm bytecode into `wasmi` bytecode for a single Wasm function.
struct FuncTranslationDriver<'parser, T> {
    /// The function body that shall be translated.
    func_body: FunctionBody<'parser>,
    /// The bytes that make up the entirety of the function body.
    bytes: &'parser [u8],
    /// The underlying translator used for the translation (and validation) process.
    translator: T,
}

impl<'parser, T> FuncTranslationDriver<'parser, T> {
    /// Creates a new Wasm to `wasmi` bytecode function translator.
    fn new(
        func_body: FunctionBody<'parser>,
        bytes: &'parser [u8],
        translator: T,
    ) -> Result<Self, ModuleError> {
        Ok(Self {
            func_body,
            bytes,
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
        if self.translator.setup(self.bytes)? {
            let allocations = self.translator.finish()?;
            return Ok(allocations);
        }
        self.translate_locals()?;
        let offset = self.translate_operators()?;
        let allocations = self.finish(offset)?;
        Ok(allocations)
    }

    /// Finishes construction of the function and returns its reusable allocations.
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
