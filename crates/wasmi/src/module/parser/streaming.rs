use super::{
    CustomSectionsBuilder,
    ModuleBuilder,
    ModuleHeader,
    ModuleHeaderBuilder,
    ModuleParser,
};
use crate::{Error, Module, Read};
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};
use wasmparser::{Chunk, Payload, Validator};

/// A buffer for holding parsed payloads in bytes.
#[derive(Debug, Default, Clone)]
struct ParseBuffer {
    buffer: Vec<u8>,
}

impl ParseBuffer {
    /// Drops the first `amount` bytes from the [`ParseBuffer`] as they have been consumed.
    #[inline]
    fn consume(buffer: &mut Self, amount: usize) {
        buffer.drain(..amount);
    }

    /// Pulls more bytes from the `stream` in order to produce Wasm payload.
    ///
    /// Returns `true` if the parser reached the end of the stream.
    ///
    /// # Note
    ///
    /// Uses `hint` to efficiently preallocate enough space for the next payload.
    #[inline]
    fn pull_bytes(buffer: &mut Self, hint: u64, stream: &mut impl Read) -> Result<bool, Error> {
        // Use the hint to preallocate more space, then read
        // some more data into the buffer.
        //
        // Note that the buffer management here is not ideal,
        // but it's compact enough to fit in an example!
        let len = buffer.len();
        let new_len = len + hint as usize;
        buffer.resize(new_len, 0x0_u8);
        let read_bytes = stream.read(&mut buffer[len..])?;
        buffer.truncate(len + read_bytes);
        let reached_end = read_bytes == 0;
        Ok(reached_end)
    }
}

impl Deref for ParseBuffer {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for ParseBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl ModuleParser {
    /// Parses and validates the Wasm bytecode `stream`.
    ///
    /// Returns the compiled and validated Wasm [`Module`] upon success.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to validate.
    pub fn parse_streaming(mut self, stream: impl Read) -> Result<Module, Error> {
        let features = self.engine.config().wasm_features();
        self.validator = Some(Validator::new_with_features(features));
        // SAFETY: we just pre-populated the Wasm module parser with a validator
        //         thus calling this method is safe.
        unsafe { self.parse_streaming_impl(stream) }
    }

    /// Parses the Wasm bytecode `stream` without Wasm validation.
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
    pub unsafe fn parse_streaming_unchecked(self, stream: impl Read) -> Result<Module, Error> {
        unsafe { self.parse_streaming_impl(stream) }
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
    unsafe fn parse_streaming_impl(mut self, mut stream: impl Read) -> Result<Module, Error> {
        let custom_sections = CustomSectionsBuilder::default();
        let mut buffer = ParseBuffer::default();
        let module =
            Self::parse_streaming_module(&mut self, &mut stream, &mut buffer, custom_sections)?;
        Ok(module)
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
    fn parse_streaming_module(
        &mut self,
        stream: &mut impl Read,
        buffer: &mut ParseBuffer,
        mut custom_sections: CustomSectionsBuilder,
    ) -> Result<Module, Error> {
        let mut header = ModuleHeaderBuilder::new(&self.engine);
        loop {
            match self.parser.parse(&buffer[..], self.eof)? {
                Chunk::NeedMoreData(hint) => {
                    self.eof = ParseBuffer::pull_bytes(buffer, hint, stream)?;
                }
                Chunk::Parsed { consumed, payload } => {
                    match payload {
                        Payload::Version {
                            num,
                            encoding,
                            range,
                        } => self.process_version(num, encoding, range),
                        Payload::TypeSection(section) => self.process_types(section, &mut header),
                        Payload::ImportSection(section) => {
                            self.process_imports(section, &mut header)
                        }
                        Payload::FunctionSection(section) => {
                            self.process_functions(section, &mut header)
                        }
                        Payload::TableSection(section) => self.process_tables(section, &mut header),
                        Payload::MemorySection(section) => {
                            self.process_memories(section, &mut header)
                        }
                        Payload::GlobalSection(section) => {
                            self.process_globals(section, &mut header)
                        }
                        Payload::ExportSection(section) => {
                            self.process_exports(section, &mut header)
                        }
                        Payload::StartSection { func, range } => {
                            self.process_start(func, range, &mut header)
                        }
                        Payload::ElementSection(section) => {
                            self.process_element(section, &mut header)
                        }
                        Payload::DataCountSection { count, range } => {
                            self.process_data_count(count, range)
                        }
                        Payload::CodeSectionStart { count, range, size } => {
                            self.process_code_start(count, range, size)?;
                            ParseBuffer::consume(buffer, consumed);
                            return self.parse_streaming_code(
                                stream,
                                buffer,
                                header.finish(),
                                custom_sections,
                            );
                        }
                        Payload::DataSection(data_section) => {
                            let mut builder = ModuleBuilder::new(header.finish(), custom_sections);
                            self.process_data(data_section, &mut builder)?;
                            ParseBuffer::consume(buffer, consumed);
                            return self.parse_streaming_data(stream, buffer, builder);
                        }
                        Payload::End(offset) => {
                            ParseBuffer::consume(buffer, consumed);
                            return self.finish(
                                offset,
                                ModuleBuilder::new(header.finish(), custom_sections),
                            );
                        }
                        Payload::CustomSection(reader) => {
                            self.process_custom_section(&mut custom_sections, reader)
                        }
                        unexpected => self.process_invalid_payload(unexpected),
                    }?;
                    ParseBuffer::consume(buffer, consumed);
                }
            }
        }
    }

    /// Parse the Wasm code section entries.
    ///
    /// We separate parsing of the Wasm code section since most of a Wasm module
    /// is made up of code section entries which we can parse and validate more efficiently
    /// by serving them with a specialized routine.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to parse or validate.
    fn parse_streaming_code(
        &mut self,
        stream: &mut impl Read,
        buffer: &mut ParseBuffer,
        header: ModuleHeader,
        mut custom_sections: CustomSectionsBuilder,
    ) -> Result<Module, Error> {
        loop {
            match self.parser.parse(&buffer[..], self.eof)? {
                Chunk::NeedMoreData(hint) => {
                    self.eof = ParseBuffer::pull_bytes(buffer, hint, stream)?;
                }
                Chunk::Parsed { consumed, payload } => {
                    match payload {
                        Payload::CodeSectionEntry(func_body) => {
                            // Note: Unfortunately the `wasmparser` crate is missing an API
                            //       to return the byte slice for the respective code section
                            //       entry payload. Please remove this work around as soon as
                            //       such an API becomes available.
                            let bytes = func_body.as_bytes();
                            self.process_code_entry(func_body, bytes, &header)?;
                        }
                        Payload::CustomSection(reader) => {
                            self.process_custom_section(&mut custom_sections, reader)?;
                        }
                        Payload::DataSection(data_section) => {
                            let mut builder = ModuleBuilder::new(header, custom_sections);
                            self.process_data(data_section, &mut builder)?;
                            ParseBuffer::consume(buffer, consumed);
                            return self.parse_streaming_data(stream, buffer, builder);
                        }
                        Payload::End(offset) => {
                            ParseBuffer::consume(buffer, consumed);
                            return self.finish(offset, ModuleBuilder::new(header, custom_sections));
                        }
                        unexpected => self.process_invalid_payload(unexpected)?,
                    }
                    // Cut away the parts from the intermediate buffer that have already been parsed.
                    ParseBuffer::consume(buffer, consumed);
                }
            }
        }
    }

    /// Parse the Wasm data section and finalize parsing.
    ///
    /// We separate parsing of the Wasm data section since it is the only Wasm
    /// section that comes after the Wasm code section that we have to separate
    /// out for technical reasons.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to parse or validate.
    fn parse_streaming_data(
        &mut self,
        stream: &mut impl Read,
        buffer: &mut ParseBuffer,
        mut builder: ModuleBuilder,
    ) -> Result<Module, Error> {
        loop {
            match self.parser.parse(&buffer[..], self.eof)? {
                Chunk::NeedMoreData(hint) => {
                    self.eof = ParseBuffer::pull_bytes(buffer, hint, stream)?;
                }
                Chunk::Parsed { consumed, payload } => {
                    match payload {
                        Payload::End(offset) => {
                            ParseBuffer::consume(buffer, consumed);
                            return self.finish(offset, builder);
                        }
                        Payload::CustomSection(reader) => {
                            self.process_custom_section(&mut builder.custom_sections, reader)?
                        }
                        invalid => self.process_invalid_payload(invalid)?,
                    }
                    // Cut away the parts from the intermediate buffer that have already been parsed.
                    ParseBuffer::consume(buffer, consumed);
                }
            }
        }
    }
}
