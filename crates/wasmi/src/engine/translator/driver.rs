use crate::{
    Error,
    engine::{WasmTranslator, code_map::CompiledFuncEntity},
};
use wasmparser::{BinaryReader, FunctionBody};

/// Translates Wasm bytecode into Wasmi bytecode for a single Wasm function.
pub struct FuncTranslationDriver<'parser, T> {
    /// The function body that shall be translated.
    func_body: FunctionBody<'parser>,
    /// The bytes that make up the entirety of the function body.
    bytes: &'parser [u8],
    /// The underlying translator used for the translation (and validation) process.
    translator: T,
}

impl<'parser, T> FuncTranslationDriver<'parser, T>
where
    T: WasmTranslator<'parser>,
{
    /// Creates a new Wasm to Wasmi bytecode function translator.
    pub fn new(
        offset: impl Into<Option<usize>>,
        bytes: &'parser [u8],
        translator: T,
    ) -> Result<Self, Error> {
        let offset = offset.into().unwrap_or(0);
        let features = translator.features();
        let reader = BinaryReader::new_features(bytes, offset, features);
        let func_body = FunctionBody::new(reader);
        Ok(Self {
            func_body,
            bytes,
            translator,
        })
    }

    /// Starts translation of the Wasm stream into Wasmi bytecode.
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
        self.translator.finish(finalize)
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
