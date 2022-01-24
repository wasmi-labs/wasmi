use super::{Module, ModuleBuilder, ModuleError, Read};
use crate::FuncType;
use wasmi_core::ValueType;
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
        self.builder.reserve_func_types(len_types);
        for _ in 0..len_types {
            match section.read()? {
                wasmparser::TypeDef::Func(func_type) => match func_type_from_wasmparser(&func_type)
                {
                    Some(func_type) => {
                        self.builder.push_func_type(func_type);
                    }
                    None => return Err(ModuleError::unsupported(func_type)),
                },
                wasmparser::TypeDef::Instance(instance_type) => {
                    return Err(ModuleError::unsupported(instance_type))
                }
                wasmparser::TypeDef::Module(module_type) => {
                    return Err(ModuleError::unsupported(module_type))
                }
            };
        }
        Ok(())
    }

    fn process_imports(&mut self, section: ImportSectionReader) -> Result<(), ModuleError> {
        self.validator.import_section(&section)?;
        Ok(())
    }

    fn process_aliases(
        &mut self,
        section: wasmparser::AliasSectionReader,
    ) -> Result<(), ModuleError> {
        self.validator.alias_section(&section).map_err(Into::into)
    }

    fn process_instances(
        &mut self,
        section: wasmparser::InstanceSectionReader,
    ) -> Result<(), ModuleError> {
        self.validator
            .instance_section(&section)
            .map_err(Into::into)
    }

    fn process_functions(&mut self, section: FunctionSectionReader) -> Result<(), ModuleError> {
        self.validator.function_section(&section)?;
        Ok(())
    }

    fn process_tables(&mut self, section: TableSectionReader) -> Result<(), ModuleError> {
        self.validator.table_section(&section)?;
        Ok(())
    }

    fn process_memories(&mut self, section: MemorySectionReader) -> Result<(), ModuleError> {
        self.validator.memory_section(&section)?;
        Ok(())
    }

    fn process_tags(&mut self, section: wasmparser::TagSectionReader) -> Result<(), ModuleError> {
        self.validator.tag_section(&section).map_err(Into::into)
    }

    fn process_globals(&mut self, section: GlobalSectionReader) -> Result<(), ModuleError> {
        self.validator.global_section(&section)?;
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

    fn process_module_entry(&mut self, range: Range) -> Result<(), ModuleError> {
        Err(ModuleError::unsupported(format!(
            "module linking entry at {:?}",
            range
        )))
    }

    fn process_unknown(&mut self, id: u8, range: Range) -> Result<(), ModuleError> {
        self.validator
            .unknown_section(id, &range)
            .map_err(Into::into)
    }
}

/// Creates a [`FuncType`] from the given [`wasmparser::FuncType`].
///
/// Returns `None` if the [`wasmparser::FuncType`] has parameter or result types
/// that are not supported by `wasmi`.
fn func_type_from_wasmparser(func: &wasmparser::FuncType) -> Option<FuncType> {
    /// Returns `true` if the given [`wasmparser::Type`] is supported by `wasmi`.
    fn is_supported_value_type(value_type: &wasmparser::Type) -> bool {
        value_type_from_wasmparser(value_type).is_some()
    }
    if !func.params.iter().all(is_supported_value_type)
        || !func.returns.iter().all(is_supported_value_type)
    {
        // One of more function parameter or result types are not supported by `wasmi`.
        return None;
    }
    /// Returns the [`ValueType`] from the given [`wasmparser::Type`].
    ///
    /// # Panics
    ///
    /// If the [`wasmparser::Type`] is not supported by `wasmi`.
    fn extract_value_type(value_type: &wasmparser::Type) -> ValueType {
        match value_type_from_wasmparser(value_type) {
            Some(ty) => ty,
            None => {
                // This is unreachable since we already filtered out unsupported
                // types in the preconditions above.
                unreachable!("encountered unsupported wasmparser type: {:?}", value_type)
            }
        }
    }
    let params = func.params.iter().map(extract_value_type);
    let results = func.returns.iter().map(extract_value_type);
    let func_type = FuncType::new(params, results);
    Some(func_type)
}

/// Creates a [`ValueType`] from the given [`wasmparser::Type`].
///
/// Returns `None` if the given [`wasmparser::Type`] is not supported by `wasmi`.
fn value_type_from_wasmparser(value_type: &wasmparser::Type) -> Option<ValueType> {
    match value_type {
        wasmparser::Type::I32 => Some(ValueType::I32),
        wasmparser::Type::I64 => Some(ValueType::I64),
        wasmparser::Type::F32 => Some(ValueType::F32),
        wasmparser::Type::F64 => Some(ValueType::F64),
        wasmparser::Type::V128
        | wasmparser::Type::FuncRef
        | wasmparser::Type::ExternRef
        | wasmparser::Type::ExnRef
        | wasmparser::Type::Func
        | wasmparser::Type::EmptyBlockType => None,
    }
}
