use crate::{
    engine::{code_map::CompiledFuncEntity, WasmTranslator},
    Error,
};
use wasmparser::FunctionBody;

// /// Translates the Wasm `bytes` of a Wasm function into `wasmi` IR bytecode.
// ///
// /// # Note
// ///
// /// - `bytes` resemble the Wasm function body bytes.
// /// - `offset` represents the global offset of `bytes` in the Wasm module.
// ///   `offset` is used for Wasm validation and thus not required.
// /// - `translator` is responsible for Wasm validation and translation.
// ///
// /// # Errors
// ///
// /// If the function body fails to translate the Wasm function body.
// pub fn translate_wasm_func<'a, T>(
//     offset: impl Into<Option<usize>>,
//     bytes: &'a [u8],
//     translator: T,
//     finalize: impl FnOnce(CompiledFuncEntity),
// ) -> Result<T::Allocations, Error>
// where
//     T: WasmTranslator<'a>,
// {
//     <FuncTranslationDriver<'a, T>>::new(offset.into(), bytes, translator)?.translate(finalize)
// }

/// Translates Wasm bytecode into `wasmi` bytecode for a single Wasm function.
pub struct FuncTranslationDriver<'parser, T> {
    /// The function body that shall be translated.
    func_body: FunctionBody<'parser>,
    /// The bytes that make up the entirety of the function body.
    bytes: &'parser [u8],
    /// The underlying translator used for the translation (and validation) process.
    translator: T,
}

impl<'parser, T> FuncTranslationDriver<'parser, T> {
    /// Creates a new Wasm to `wasmi` bytecode function translator.
    pub fn new(
        offset: impl Into<Option<usize>>,
        bytes: &'parser [u8],
        translator: T,
    ) -> Result<Self, Error> {
        let offset = offset.into().unwrap_or(0);
        let func_body = FunctionBody::new(offset, bytes);
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
    pub fn translate(
        mut self,
        finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<T::Allocations, Error> {
        if self.translator.setup(self.bytes)? {
            let allocations = self.translator.finish(finalize)?;
            return Ok(allocations);
        }
        self.translate_locals()?;
        let offset = self.translate_operators()?;
        let allocations = self.finish(offset, finalize)?;
        Ok(allocations)
    }

    /// Finishes construction of the function and returns its reusable allocations.
    fn finish(
        mut self,
        offset: usize,
        finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<T::Allocations, Error> {
        self.translator.update_pos(offset);
        self.translator.finish(finalize).map_err(Into::into)
    }

    /// Translates local variables of the Wasm function.
    fn translate_locals(&mut self) -> Result<(), Error> {
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
    fn translate_operators(&mut self) -> Result<usize, Error> {
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