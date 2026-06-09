use super::{ModuleBuilder, ModuleParser};
use crate::{Error, Module};
use wasmparser::{Chunk, Payload, Validator};

impl ModuleParser {
    /// Starts parsing and validating the Wasm bytecode stream.
    ///
    /// Returns the compiled and validated Wasm [`Module`] upon success.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to validate.
    pub fn parse_buffered(mut self, buffer: &[u8]) -> Result<Module, Error> {
        let features = self.engine.config().wasm_features();
        self.validator = Some(Validator::new_with_features(features));
        // SAFETY: we just pre-populated the Wasm module parser with a validator
        //         thus calling this method is safe.
        unsafe { self.parse_buffered_impl(buffer) }
    }

    /// Starts parsing and validating the Wasm bytecode stream.
    ///
    /// Returns the compiled and validated Wasm [`Module`] upon success.
    ///
    /// # Safety
    ///
    /// The caller is responsible to make sure that the provided
    /// `stream` yields valid WebAssembly bytecode.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to validate.
    pub unsafe fn parse_buffered_unchecked(self, buffer: &[u8]) -> Result<Module, Error> {
        unsafe { self.parse_buffered_impl(buffer) }
    }

    /// Starts parsing and validating the Wasm bytecode stream.
    ///
    /// Returns the compiled and validated Wasm [`Module`] upon success.
    ///
    /// # Safety
    ///
    /// The caller is responsible to either
    ///
    /// 1) Populate the [`ModuleParser`] with a [`Validator`] prior to calling this method, OR;
    /// 2) Make sure that the provided `stream` yields valid WebAssembly bytecode.
    ///
    /// Otherwise this method has undefined behavior.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to validate.
    unsafe fn parse_buffered_impl(mut self, mut buffer: &[u8]) -> Result<Module, Error> {
        let mut builder = ModuleBuilder::new(&self.engine);
        self.parse_module(&mut buffer, &mut builder)?;
        Ok(builder.finish())
    }

    /// Fetch next Wasm module payload and adust the `buffer`.
    ///
    /// # Errors
    ///
    /// If the parsed Wasm is malformed.
    fn next_payload<'a>(&mut self, buffer: &mut &'a [u8]) -> Result<(usize, Payload<'a>), Error> {
        match self.parser.parse(&buffer[..], true)? {
            Chunk::Parsed { consumed, payload } => Ok((consumed, payload)),
            Chunk::NeedMoreData(_hint) => {
                // This is not possible since `eof` is always true.
                unreachable!()
            }
        }
    }

    /// Consumes the parts of the buffer that have been processed.
    fn consume_buffer<'a>(consumed: usize, buffer: &mut &'a [u8]) -> &'a [u8] {
        let (consumed, remaining) = buffer.split_at(consumed);
        *buffer = remaining;
        consumed
    }

    /// Parse the Wasm module header.
    ///
    /// - The Wasm module header is the set of all sections that appear before
    ///   the Wasm code section.
    /// - We separate parsing of the Wasm module header since the information of
    ///   the Wasm module header is required for translating the Wasm code section.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to parse or validate.
    fn parse_module(
        &mut self,
        buffer: &mut &[u8],
        module: &mut ModuleBuilder,
    ) -> Result<(), Error> {
        loop {
            let (consumed, payload) = self.next_payload(buffer)?;
            match payload {
                Payload::Version {
                    num,
                    encoding,
                    range,
                } => self.process_version(num, encoding, range),
                Payload::TypeSection(section) => self.process_types(section, module),
                Payload::ImportSection(section) => self.process_imports(section, module),
                Payload::FunctionSection(section) => self.process_functions(section, module),
                Payload::TableSection(section) => self.process_tables(section, module),
                Payload::MemorySection(section) => self.process_memories(section, module),
                Payload::GlobalSection(section) => self.process_globals(section, module),
                Payload::ExportSection(section) => self.process_exports(section, module),
                Payload::StartSection { func, range } => self.process_start(func, range, module),
                Payload::ElementSection(section) => self.process_element(section, module),
                Payload::DataCountSection { count, range } => self.process_data_count(count, range),
                Payload::CodeSectionStart { count, range, size } => {
                    self.process_code_start(count, range, size)
                }
                Payload::CodeSectionEntry(func_body) => {
                    let bytes = func_body.as_bytes();
                    self.process_code_entry(func_body, bytes, module)
                }
                Payload::DataSection(section) => self.process_data(section, module),
                Payload::End(offset) => {
                    self.process_end(offset)?;
                    break;
                }
                Payload::CustomSection(reader) => self.process_custom_section(reader, module),
                unexpected => self.process_invalid_payload(unexpected),
            }?;
            Self::consume_buffer(consumed, buffer);
        }
        Ok(())
    }
}
