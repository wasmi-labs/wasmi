use super::{
    builder::ModuleHeaderBuilder,
    export::ExternIdx,
    global::Global,
    import::{FuncTypeIdx, Import},
    DataSegment,
    ElementSegment,
    FuncIdx,
    Module,
    ModuleBuilder,
    ModuleHeader,
    Read,
};
use crate::{
    engine::{CompiledFunc, EngineLimitsError},
    Engine,
    Error,
    FuncType,
    MemoryType,
    TableType,
};
use core::ops::Range;
use std::{boxed::Box, vec::Vec};
use wasmparser::{
    Chunk,
    DataSectionReader,
    ElementSectionReader,
    Encoding,
    ExportSectionReader,
    FunctionBody,
    FunctionSectionReader,
    GlobalSectionReader,
    ImportSectionReader,
    MemorySectionReader,
    Parser as WasmParser,
    Payload,
    TableSectionReader,
    TypeSectionReader,
    Validator,
};

/// Parse, validate and translate the Wasm bytecode stream into Wasm IR bytecode.
///
/// - Returns the fully compiled and validated Wasm [`Module`] upon success.
/// - Uses the given [`Engine`] as the translation target of the process.
///
/// # Errors
///
/// If the Wasm bytecode stream fails to parse, validate or translate.
pub fn parse(engine: &Engine, stream: impl Read) -> Result<Module, Error> {
    ModuleParser::new(engine).parse(stream)
}

/// Parse and translate the Wasm bytecode stream into Wasm IR bytecode.
///
/// - Returns the fully compiled Wasm [`Module`] upon success.
/// - Uses the given [`Engine`] as the translation target of the process.
///
/// # Errors
///
/// If the Wasm bytecode stream fails to parse or translate.
pub unsafe fn parse_unchecked(engine: &Engine, stream: impl Read) -> Result<Module, Error> {
    unsafe { ModuleParser::new(engine).parse_unchecked(stream) }
}

/// Context used to construct a WebAssembly module from a stream of bytes.
pub struct ModuleParser {
    /// The engine used for translation.
    engine: Engine,
    /// The Wasm validator used throughout stream parsing.
    validator: Validator,
    /// The underlying Wasm parser.
    parser: WasmParser,
    /// The number of compiled or processed functions.
    compiled_funcs: u32,
    /// Flag, `true` when `stream` is at the end.
    eof: bool,
}

/// The mode of Wasm validation when parsing a Wasm module.
#[derive(Debug, Copy, Clone)]
pub enum ValidationMode {
    /// Perform Wasm validation on the entire Wasm module including Wasm function bodies.
    All,
    /// Perform Wasm validation only on the Wasm header but not on Wasm function bodies.
    HeaderOnly,
}

impl ModuleParser {
    /// Creates a new [`ModuleParser`] for the given [`Engine`].
    fn new(engine: &Engine) -> Self {
        let validator = Validator::new_with_features(engine.config().wasm_features());
        let parser = WasmParser::new(0);
        Self {
            engine: engine.clone(),
            validator,
            parser,
            compiled_funcs: 0,
            eof: false,
        }
    }

    /// Starts parsing and validating the Wasm bytecode stream.
    ///
    /// Returns the compiled and validated Wasm [`Module`] upon success.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to validate.
    pub fn parse(self, stream: impl Read) -> Result<Module, Error> {
        self.parse_impl(ValidationMode::All, stream)
    }

    /// Starts parsing and validating the Wasm bytecode stream.
    ///
    /// Returns the compiled and validated Wasm [`Module`] upon success.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to validate.
    pub unsafe fn parse_unchecked(self, stream: impl Read) -> Result<Module, Error> {
        self.parse_impl(ValidationMode::HeaderOnly, stream)
    }

    /// Starts parsing and validating the Wasm bytecode stream.
    ///
    /// Returns the compiled and validated Wasm [`Module`] upon success.
    ///
    /// # Errors
    ///
    /// If the Wasm bytecode stream fails to validate.
    fn parse_impl(
        mut self,
        validation_mode: ValidationMode,
        mut stream: impl Read,
    ) -> Result<Module, Error> {
        let mut buffer = Vec::new();
        let header = Self::parse_header(&mut self, &mut stream, &mut buffer)?;
        let builder =
            Self::parse_code(&mut self, validation_mode, &mut stream, &mut buffer, header)?;
        let module = Self::parse_data(&mut self, &mut stream, &mut buffer, builder)?;
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
    fn parse_header(
        &mut self,
        stream: &mut impl Read,
        buffer: &mut Vec<u8>,
    ) -> Result<ModuleHeader, Error> {
        let mut header = ModuleHeaderBuilder::new(&self.engine);
        loop {
            match self.parser.parse(&buffer[..], self.eof)? {
                Chunk::NeedMoreData(hint) => {
                    self.eof = Self::pull_bytes(buffer, hint, stream)?;
                    if self.eof {
                        break;
                    }
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
                        Payload::InstanceSection(section) => self.process_instances(section),
                        Payload::FunctionSection(section) => {
                            self.process_functions(section, &mut header)
                        }
                        Payload::TableSection(section) => self.process_tables(section, &mut header),
                        Payload::MemorySection(section) => {
                            self.process_memories(section, &mut header)
                        }
                        Payload::TagSection(section) => self.process_tags(section),
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
                            buffer.drain(..consumed);
                            break;
                        }
                        Payload::DataSection(_) => break,
                        Payload::End(_) => break,
                        Payload::CustomSection { .. } => Ok(()),
                        Payload::UnknownSection { id, range, .. } => {
                            self.process_unknown(id, range)
                        }
                        unexpected => {
                            unreachable!("encountered unexpected Wasm section: {unexpected:?}")
                        }
                    }?;
                    // Cut away the parts from the intermediate buffer that have already been parsed.
                    buffer.drain(..consumed);
                }
            }
        }
        Ok(header.finish())
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
    fn parse_code(
        &mut self,
        validation_mode: ValidationMode,
        stream: &mut impl Read,
        buffer: &mut Vec<u8>,
        header: ModuleHeader,
    ) -> Result<ModuleBuilder, Error> {
        loop {
            match self.parser.parse(&buffer[..], self.eof)? {
                Chunk::NeedMoreData(hint) => {
                    self.eof = Self::pull_bytes(buffer, hint, stream)?;
                }
                Chunk::Parsed { consumed, payload } => {
                    match payload {
                        Payload::CodeSectionEntry(func_body) => {
                            // Note: Unfortunately the `wasmparser` crate is missing an API
                            //       to return the byte slice for the respective code section
                            //       entry payload. Please remove this work around as soon as
                            //       such an API becomes available.
                            let remaining = func_body.get_binary_reader().bytes_remaining();
                            let start = consumed - remaining;
                            let bytes = &buffer[start..consumed];
                            self.process_code_entry(func_body, validation_mode, bytes, &header)?;
                        }
                        Payload::CustomSection { .. } => {}
                        Payload::UnknownSection { id, range, .. } => {
                            self.process_unknown(id, range)?
                        }
                        _ => break,
                    }
                    // Cut away the parts from the intermediate buffer that have already been parsed.
                    buffer.drain(..consumed);
                }
            }
        }
        Ok(ModuleBuilder::new(header))
    }

    fn parse_data(
        &mut self,
        stream: &mut impl Read,
        buffer: &mut Vec<u8>,
        mut builder: ModuleBuilder,
    ) -> Result<Module, Error> {
        loop {
            match self.parser.parse(&buffer[..], self.eof)? {
                Chunk::NeedMoreData(hint) => {
                    self.eof = Self::pull_bytes(buffer, hint, stream)?;
                }
                Chunk::Parsed { consumed, payload } => {
                    match payload {
                        Payload::DataSection(section) => {
                            self.process_data(section, &mut builder)?;
                        }
                        Payload::End(offset) => {
                            self.process_end(offset)?;
                            buffer.drain(..consumed);
                            break;
                        }
                        Payload::CustomSection { .. } => {}
                        Payload::UnknownSection { id, range, .. } => {
                            self.process_unknown(id, range)?
                        }
                        unexpected => {
                            unreachable!("encountered unexpected Wasm section: {unexpected:?}")
                        }
                    }
                    // Cut away the parts from the intermediate buffer that have already been parsed.
                    buffer.drain(..consumed);
                }
            }
        }
        Ok(builder.finish(&self.engine))
    }

    /// Pulls more bytes from the `stream` in order to produce Wasm payload.
    ///
    /// Returns `true` if the parser reached the end of the stream.
    ///
    /// # Note
    ///
    /// Uses `hint` to efficiently preallocate enough space for the next payload.
    fn pull_bytes(buffer: &mut Vec<u8>, hint: u64, stream: &mut impl Read) -> Result<bool, Error> {
        // Use the hint to preallocate more space, then read
        // some more data into the buffer.
        //
        // Note that the buffer management here is not ideal,
        // but it's compact enough to fit in an example!
        let len = buffer.len();
        buffer.extend((0..hint).map(|_| 0u8));
        let read_bytes = stream.read(&mut buffer[len..])?;
        buffer.truncate(len + read_bytes);
        let reached_end = read_bytes == 0;
        Ok(reached_end)
    }

    /// Processes the end of the Wasm binary.
    fn process_end(&mut self, offset: usize) -> Result<(), Error> {
        self.validator.end(offset)?;
        Ok(())
    }

    /// Validates the Wasm version section.
    fn process_version(
        &mut self,
        num: u16,
        encoding: Encoding,
        range: Range<usize>,
    ) -> Result<(), Error> {
        self.validator
            .version(num, encoding, &range)
            .map_err(Into::into)
    }

    /// Processes the Wasm type section.
    ///
    /// # Note
    ///
    /// This extracts all function types into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If an unsupported function type is encountered.
    fn process_types(
        &mut self,
        section: TypeSectionReader,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        self.validator.type_section(&section)?;
        let limits = self.engine.config().get_engine_limits();
        let func_types = section.into_iter().map(|result| {
            let wasmparser::Type::Func(ty) = result?;
            if let Some(limit) = limits.max_params {
                if ty.params().len() > limit {
                    return Err(Error::from(EngineLimitsError::TooManyParameters { limit }));
                }
            }
            if let Some(limit) = limits.max_results {
                if ty.results().len() > limit {
                    return Err(Error::from(EngineLimitsError::TooManyResults { limit }));
                }
            }
            Ok(FuncType::from_wasmparser(ty))
        });
        header.push_func_types(func_types)?;
        Ok(())
    }

    /// Processes the Wasm import section.
    ///
    /// # Note
    ///
    /// This extracts all imports into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// - If an import fails to validate.
    /// - If an unsupported import declaration is encountered.
    fn process_imports(
        &mut self,
        section: ImportSectionReader,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        self.validator.import_section(&section)?;
        let imports = section
            .into_iter()
            .map(|import| import.map(Import::from).map_err(Error::from));
        header.push_imports(imports)?;
        Ok(())
    }

    /// Process module instances.
    ///
    /// # Note
    ///
    /// This is part of the module linking Wasm proposal and not yet supported
    /// by Wasmi.
    fn process_instances(
        &mut self,
        section: wasmparser::InstanceSectionReader,
    ) -> Result<(), Error> {
        self.validator
            .instance_section(&section)
            .map_err(Into::into)
    }

    /// Process module function declarations.
    ///
    /// # Note
    ///
    /// This extracts all function declarations into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a function declaration fails to validate.
    fn process_functions(
        &mut self,
        section: FunctionSectionReader,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        if let Some(limit) = self.engine.config().get_engine_limits().max_functions {
            if section.count() > limit {
                return Err(Error::from(EngineLimitsError::TooManyFunctions { limit }));
            }
        }
        self.validator.function_section(&section)?;
        let funcs = section
            .into_iter()
            .map(|func| func.map(FuncTypeIdx::from).map_err(Error::from));
        header.push_funcs(funcs)?;
        Ok(())
    }

    /// Process module table declarations.
    ///
    /// # Note
    ///
    /// This extracts all table declarations into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a table declaration fails to validate.
    fn process_tables(
        &mut self,
        section: TableSectionReader,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        if let Some(limit) = self.engine.config().get_engine_limits().max_tables {
            if section.count() > limit {
                return Err(Error::from(EngineLimitsError::TooManyTables { limit }));
            }
        }
        self.validator.table_section(&section)?;
        let tables = section
            .into_iter()
            .map(|table| table.map(TableType::from_wasmparser).map_err(Error::from));
        header.push_tables(tables)?;
        Ok(())
    }

    /// Process module linear memory declarations.
    ///
    /// # Note
    ///
    /// This extracts all linear memory declarations into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a linear memory declaration fails to validate.
    fn process_memories(
        &mut self,
        section: MemorySectionReader,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        if let Some(limit) = self.engine.config().get_engine_limits().max_memories {
            if section.count() > limit {
                return Err(Error::from(EngineLimitsError::TooManyMemories { limit }));
            }
        }
        self.validator.memory_section(&section)?;
        let memories = section
            .into_iter()
            .map(|memory| memory.map(MemoryType::from_wasmparser).map_err(Error::from));
        header.push_memories(memories)?;
        Ok(())
    }

    /// Process module tags.
    ///
    /// # Note
    ///
    /// This is part of the module linking Wasm proposal and not yet supported
    /// by Wasmi.
    fn process_tags(&mut self, section: wasmparser::TagSectionReader) -> Result<(), Error> {
        self.validator.tag_section(&section).map_err(Into::into)
    }

    /// Process module global variable declarations.
    ///
    /// # Note
    ///
    /// This extracts all global variable declarations into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If a global variable declaration fails to validate.
    fn process_globals(
        &mut self,
        section: GlobalSectionReader,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        if let Some(limit) = self.engine.config().get_engine_limits().max_globals {
            if section.count() > limit {
                return Err(Error::from(EngineLimitsError::TooManyGlobals { limit }));
            }
        }
        self.validator.global_section(&section)?;
        let globals = section
            .into_iter()
            .map(|global| global.map(Global::from).map_err(Error::from));
        header.push_globals(globals)?;
        Ok(())
    }

    /// Process module export declarations.
    ///
    /// # Note
    ///
    /// This extracts all export declarations into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If an export declaration fails to validate.
    fn process_exports(
        &mut self,
        section: ExportSectionReader,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        self.validator.export_section(&section)?;
        let exports = section.into_iter().map(|export| {
            let export = export?;
            let field: Box<str> = export.name.into();
            let idx = ExternIdx::new(export.kind, export.index)?;
            Ok((field, idx))
        });
        header.push_exports(exports)?;
        Ok(())
    }

    /// Process module start section.
    ///
    /// # Note
    ///
    /// This sets the start function for the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If the start function declaration fails to validate.
    fn process_start(
        &mut self,
        func: u32,
        range: Range<usize>,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        self.validator.start_section(func, &range)?;
        header.set_start(FuncIdx::from(func));
        Ok(())
    }

    /// Process module table element segments.
    ///
    /// # Note
    ///
    /// This extracts all table element segments into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If any of the table element segments fail to validate.
    fn process_element(
        &mut self,
        section: ElementSectionReader,
        header: &mut ModuleHeaderBuilder,
    ) -> Result<(), Error> {
        if let Some(limit) = self
            .engine
            .config()
            .get_engine_limits()
            .max_element_segments
        {
            if section.count() > limit {
                return Err(Error::from(EngineLimitsError::TooManyElementSegments {
                    limit,
                }));
            }
        }
        self.validator.element_section(&section)?;
        let segments = section
            .into_iter()
            .map(|segment| segment.map(ElementSegment::from).map_err(Error::from));
        header.push_element_segments(segments)?;
        Ok(())
    }

    /// Process module data count section.
    ///
    /// # Note
    ///
    /// This is part of the bulk memory operations Wasm proposal and not yet supported
    /// by Wasmi.
    fn process_data_count(&mut self, count: u32, range: Range<usize>) -> Result<(), Error> {
        if let Some(limit) = self.engine.config().get_engine_limits().max_data_segments {
            if count > limit {
                return Err(Error::from(EngineLimitsError::TooManyDataSegments {
                    limit,
                }));
            }
        }
        self.validator
            .data_count_section(count, &range)
            .map_err(Into::into)
    }

    /// Process module linear memory data segments.
    ///
    /// # Note
    ///
    /// This extracts all table elements into the [`Module`] under construction.
    ///
    /// # Errors
    ///
    /// If any of the table elements fail to validate.
    fn process_data(
        &mut self,
        section: DataSectionReader,
        builder: &mut ModuleBuilder,
    ) -> Result<(), Error> {
        if let Some(limit) = self.engine.config().get_engine_limits().max_data_segments {
            if section.count() > limit {
                return Err(Error::from(EngineLimitsError::TooManyDataSegments {
                    limit,
                }));
            }
        }
        self.validator.data_section(&section)?;
        let segments = section
            .into_iter()
            .map(|segment| segment.map(DataSegment::from).map_err(Error::from));
        builder.push_data_segments(segments)?;
        Ok(())
    }

    /// Process module code section start.
    ///
    /// # Note
    ///
    /// This currently does not do a lot but it might become important in the
    /// future if we add parallel translation of function bodies to prepare for
    /// the translation.
    ///
    /// # Errors
    ///
    /// If the code start section fails to validate.
    fn process_code_start(
        &mut self,
        count: u32,
        range: Range<usize>,
        size: u32,
    ) -> Result<(), Error> {
        let engine_limits = self.engine.config().get_engine_limits();
        if let Some(limit) = engine_limits.max_functions {
            if count > limit {
                return Err(Error::from(EngineLimitsError::TooManyFunctions { limit }));
            }
        }
        if let Some(limit) = engine_limits.min_avg_bytes_per_function {
            if size >= limit.req_funcs_bytes {
                let limit = limit.min_avg_bytes_per_function;
                let avg = size / count;
                if avg < limit {
                    return Err(Error::from(EngineLimitsError::MinAvgBytesPerFunction {
                        limit,
                        avg,
                    }));
                }
            }
        }
        self.validator.code_section_start(count, &range)?;
        Ok(())
    }

    /// Returns the next `FuncIdx` for processing of its function body.
    fn next_func(&mut self, header: &ModuleHeader) -> (FuncIdx, CompiledFunc) {
        let index = self.compiled_funcs;
        let compiled_func = header.inner.compiled_funcs[index as usize];
        self.compiled_funcs += 1;
        // We have to adjust the initial func reference to the first
        // internal function before we process any of the internal functions.
        let len_func_imports = u32::try_from(header.inner.imports.len_funcs())
            .unwrap_or_else(|_| panic!("too many imported functions"));
        let func_idx = FuncIdx::from(index + len_func_imports);
        (func_idx, compiled_func)
    }

    /// Process a single module code section entry.
    ///
    /// # Note
    ///
    /// This contains the local variables and Wasm instructions of
    /// a single function body.
    /// This procedure is translating the Wasm bytecode into Wasmi bytecode.
    ///
    /// # Errors
    ///
    /// If the function body fails to validate.
    fn process_code_entry(
        &mut self,
        func_body: FunctionBody,
        validation_mode: ValidationMode,
        bytes: &[u8],
        header: &ModuleHeader,
    ) -> Result<(), Error> {
        let (func, compiled_func) = self.next_func(header);
        let module = header.clone();
        let offset = func_body.get_binary_reader().original_position();
        let func_to_validate = match validation_mode {
            ValidationMode::All => Some(self.validator.code_section_entry(&func_body)?),
            ValidationMode::HeaderOnly => None,
        };
        self.engine
            .translate_func(func, compiled_func, offset, bytes, module, func_to_validate)?;
        Ok(())
    }

    /// Process an unknown Wasm module section.
    ///
    /// # Note
    ///
    /// This generally will be treated as an error for now.
    fn process_unknown(&mut self, id: u8, range: Range<usize>) -> Result<(), Error> {
        self.validator
            .unknown_section(id, &range)
            .map_err(Into::into)
    }
}
