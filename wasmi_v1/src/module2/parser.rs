use crate::TableType;

use super::{import::FuncTypeIdx, Module, ModuleBuilder, ModuleError, Read};
use wasmparser::{
    Chunk,
    DataSectionReader,
    ElementSectionReader,
    ExportSectionReader,
    FunctionSectionReader,
    GlobalSectionReader,
    ImportSectionReader,
    MemorySectionReader,
    Parser as WasmParser,
    Payload,
    Range,
    TableSectionReader,
    TypeSectionReader,
    Validator,
    WasmFeatures,
};

/// Context used to construct a WebAssembly module from a stream of bytes.
pub struct ModuleParser {
    /// The module builder used throughout stream parsing.
    builder: ModuleBuilder,
    /// The Wasm validator used throughout stream parsing.
    validator: Validator,
    /// The underlying Wasm parser.
    parser: WasmParser,
}

impl Default for ModuleParser {
    fn default() -> Self {
        let builder = ModuleBuilder::default();
        let mut validator = Validator::default();
        validator.wasm_features(Self::features());
        let parser = WasmParser::new(0);
        Self {
            builder,
            validator,
            parser,
        }
    }
}

impl ModuleParser {
    /// Returns the Wasm features supported by `wasmi`.
    fn features() -> WasmFeatures {
        WasmFeatures {
            reference_types: false,
            multi_value: false,
            bulk_memory: false,
            module_linking: false,
            simd: false,
            relaxed_simd: false,
            threads: false,
            tail_call: false,
            deterministic_only: true,
            multi_memory: false,
            exceptions: false,
            memory64: false,
            extended_const: false,
        }
    }

    pub fn parse(&mut self, mut stream: impl Read) -> Result<Module, ModuleError> {
        let mut buffer = Vec::new();
        let mut eof = false;
        'outer: loop {
            match self.parser.parse(&buffer[..], eof)? {
                Chunk::NeedMoreData(hint) => {
                    eof = Self::pull_bytes(&mut buffer, hint, &mut stream)?;
                    continue 'outer;
                }
                Chunk::Parsed { consumed, payload } => {
                    eof = self.process_payload(payload)?;
                    // Cut away the parts from the intermediate buffer that have already been parsed.
                    buffer.drain(..consumed);
                    if eof {
                        break 'outer;
                    }
                }
            }
        }
        todo!()
    }

    /// Pulls more bytes from the `stream` in order to produce Wasm payload.
    ///
    /// Returns `true` if the parser reached the end of the stream.
    ///
    /// # Note
    ///
    /// Uses `hint` to efficiently preallocate enough space for the next payload.
    fn pull_bytes(
        buffer: &mut Vec<u8>,
        hint: u64,
        stream: &mut impl Read,
    ) -> Result<bool, ModuleError> {
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

    /// Processes the `wasmparser` payload.
    ///
    /// # Errors
    ///
    /// - If Wasm validation of the payload fails.
    /// - If some unsupported Wasm proposal definition is encountered.
    /// - If `wasmi` limits are exceeded.
    fn process_payload(&mut self, payload: Payload) -> Result<bool, ModuleError> {
        match payload {
            Payload::Version { num, range } => self.process_version(num, range),
            Payload::TypeSection(section) => self.process_types(section),
            Payload::ImportSection(section) => self.process_imports(section),
            Payload::AliasSection(section) => self.process_aliases(section),
            Payload::InstanceSection(section) => self.process_instances(section),
            Payload::FunctionSection(section) => self.process_functions(section),
            Payload::TableSection(section) => self.process_tables(section),
            Payload::MemorySection(section) => self.process_memories(section),
            Payload::TagSection(section) => self.process_tags(section),
            Payload::GlobalSection(section) => self.process_globals(section),
            Payload::ExportSection(section) => self.process_exports(section),
            Payload::StartSection { func, range } => self.process_start(func, range),
            Payload::ElementSection(section) => self.process_element(section),
            Payload::DataCountSection { count, range } => self.process_data_count(count, range),
            Payload::DataSection(section) => self.process_data(section),
            Payload::CustomSection { .. } => Ok(()),
            Payload::CodeSectionStart { count, range, size } => {
                self.process_code_start(count, range)
            }
            Payload::CodeSectionEntry(_section) => self.process_code_entry(),
            Payload::ModuleSectionStart { count, range, .. } => {
                self.process_module_start(count, range)
            }
            Payload::ModuleSectionEntry { parser, range } => self.process_module_entry(range),
            Payload::UnknownSection { id, range, .. } => self.process_unknown(id, range),
            Payload::End => return Ok(false),
        }?;
        Ok(true)
    }

    /// Validates the Wasm version section.
    fn process_version(&mut self, num: u32, range: Range) -> Result<(), ModuleError> {
        self.validator.version(num, &range).map_err(Into::into)
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
    fn process_types(&mut self, mut section: TypeSectionReader) -> Result<(), ModuleError> {
        self.validator.type_section(&section)?;
        let len_types = section.get_count();
        let func_types = (0..len_types).map(|_| match section.read()? {
            wasmparser::TypeDef::Func(ty) => ty.try_into(),
            wasmparser::TypeDef::Instance(ty) => Err(ModuleError::unsupported(ty)),
            wasmparser::TypeDef::Module(ty) => Err(ModuleError::unsupported(ty)),
        });
        self.builder.push_func_types(func_types)?;
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
    fn process_imports(&mut self, mut section: ImportSectionReader) -> Result<(), ModuleError> {
        self.validator.import_section(&section)?;
        let len_imports = section.get_count();
        let imports = (0..len_imports).map(|_| section.read()?.try_into());
        self.builder.push_imports(imports)?;
        Ok(())
    }

    /// Process module aliases.
    ///
    /// # Note
    ///
    /// This is part of the module linking Wasm proposal and not yet supported
    /// by `wasmi`.
    fn process_aliases(
        &mut self,
        section: wasmparser::AliasSectionReader,
    ) -> Result<(), ModuleError> {
        self.validator.alias_section(&section).map_err(Into::into)
    }

    /// Process module instances.
    ///
    /// # Note
    ///
    /// This is part of the module linking Wasm proposal and not yet supported
    /// by `wasmi`.
    fn process_instances(
        &mut self,
        section: wasmparser::InstanceSectionReader,
    ) -> Result<(), ModuleError> {
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
    /// - If a function declaration fails to validate.
    fn process_functions(&mut self, mut section: FunctionSectionReader) -> Result<(), ModuleError> {
        self.validator.function_section(&section)?;
        let len_funcs = section.get_count();
        let funcs = (0..len_funcs).map(|_| section.read().map(FuncTypeIdx).map_err(Into::into));
        self.builder.push_funcs(funcs)?;
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
    /// - If a table declaration fails to validate.
    fn process_tables(&mut self, mut section: TableSectionReader) -> Result<(), ModuleError> {
        self.validator.table_section(&section)?;
        let len_tables = section.get_count();
        let tables = (0..len_tables).map(|_| section.read()?.try_into());
        self.builder.push_tables(tables)?;
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
    /// - If a linear memory declaration fails to validate.
    fn process_memories(&mut self, mut section: MemorySectionReader) -> Result<(), ModuleError> {
        self.validator.memory_section(&section)?;
        let len_memories = section.get_count();
        let memories = (0..len_memories).map(|_| section.read()?.try_into());
        self.builder.push_memories(memories)?;
        Ok(())
    }

    /// Process module tags.
    ///
    /// # Note
    ///
    /// This is part of the module linking Wasm proposal and not yet supported
    /// by `wasmi`.
    fn process_tags(&mut self, section: wasmparser::TagSectionReader) -> Result<(), ModuleError> {
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
    /// - If a global variable declaration fails to validate.
    fn process_globals(&mut self, mut section: GlobalSectionReader) -> Result<(), ModuleError> {
        self.validator.global_section(&section)?;
        let len_globals = section.get_count();
        let globals = (0..len_globals).map(|_| section.read()?.try_into());
        self.builder.push_globals(globals)?;
        Ok(())
    }

    fn process_exports(&mut self, section: ExportSectionReader) -> Result<(), ModuleError> {
        self.validator.export_section(&section)?;
        Ok(())
    }

    fn process_start(&mut self, func: u32, range: Range) -> Result<(), ModuleError> {
        self.validator.start_section(func, &range)?;
        Ok(())
    }

    fn process_element(&mut self, section: ElementSectionReader) -> Result<(), ModuleError> {
        self.validator.element_section(&section)?;
        Ok(())
    }

    fn process_data_count(&mut self, count: u32, range: Range) -> Result<(), ModuleError> {
        self.validator
            .data_count_section(count, &range)
            .map_err(Into::into)
    }

    fn process_data(&mut self, section: DataSectionReader) -> Result<(), ModuleError> {
        self.validator.data_section(&section)?;
        Ok(())
    }

    fn process_code_start(&mut self, count: u32, range: Range) -> Result<(), ModuleError> {
        self.validator
            .code_section_start(count, &range)
            .map_err(Into::into)
    }

    fn process_code_entry(&mut self) -> Result<(), ModuleError> {
        let _fn_validator = self.validator.code_section_entry()?;
        Ok(())
    }

    fn process_module_start(&mut self, count: u32, range: Range) -> Result<(), ModuleError> {
        self.validator
            .module_section_start(count, &range)
            .map_err(Into::into)
    }

    /// Process a module entry.
    ///
    /// # Note
    ///
    /// This is part of the module linking Wasm proposal and not yet supported
    /// by `wasmi`.
    fn process_module_entry(&mut self, range: Range) -> Result<(), ModuleError> {
        Err(ModuleError::unsupported(format!(
            "module linking entry at {:?}",
            range
        )))
    }

    /// Process an unknown Wasm module section.
    ///
    /// # Note
    ///
    /// This generally will be treated as an error for now.
    fn process_unknown(&mut self, id: u8, range: Range) -> Result<(), ModuleError> {
        self.validator
            .unknown_section(id, &range)
            .map_err(Into::into)
    }
}
