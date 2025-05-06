use super::{
    builder::ModuleHeaderBuilder,
    export::ExternIdx,
    global::Global,
    import::{FuncTypeIdx, Import},
    utils::FromWasmparser as _,
    CustomSectionsBuilder,
    ElementSegment,
    FuncIdx,
    ModuleBuilder,
    ModuleHeader,
};
use crate::{
    engine::{EnforcedLimitsError, EngineFunc},
    Engine,
    Error,
    FuncType,
    MemoryType,
    TableType,
};
use alloc::boxed::Box;
use core::ops::Range;
use wasmparser::{
    CustomSectionReader,
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
use crate::Module;

mod buffered;
mod streaming;

/// Context used to construct a WebAssembly module from a stream of bytes.
pub struct ModuleParser {
    /// The engine used for translation.
    engine: Engine,
    /// The Wasm validator used throughout stream parsing.
    validator: Option<Validator>,
    /// The underlying Wasm parser.
    parser: WasmParser,
    /// The number of compiled or processed functions.
    engine_funcs: u32,
    /// Flag, `true` when `stream` is at the end.
    eof: bool,
}

impl ModuleParser {
    /// Creates a new [`ModuleParser`] for the given [`Engine`].
    pub fn new(engine: &Engine) -> Self {
        let mut parser = WasmParser::new(0);
        parser.set_features(engine.config().wasm_features());
        Self {
            engine: engine.clone(),
            validator: None,
            parser,
            engine_funcs: 0,
            eof: false,
        }
    }

    /// Finish Wasm module parsing and returns the resulting [`Module`].
    fn finish(&mut self, offset: usize, builder: ModuleBuilder) -> Result<Module, Error> {
        self.process_end(offset)?;
        let module = builder.finish(&self.engine);
        Ok(module)
    }

    /// Processes the end of the Wasm binary.
    fn process_end(&mut self, offset: usize) -> Result<(), Error> {
        if let Some(validator) = &mut self.validator {
            // This only checks if the number of code section entries and data segments match
            // their expected numbers thus we must avoid this check in header-only mode because
            // otherwise we will receive errors for unmatched data section entries.
            validator.end(offset)?;
        }
        Ok(())
    }

    /// Validates the Wasm version section.
    fn process_version(
        &mut self,
        num: u16,
        encoding: Encoding,
        range: Range<usize>,
    ) -> Result<(), Error> {
        if let Some(validator) = &mut self.validator {
            validator.version(num, encoding, &range)?;
        }
        Ok(())
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
        if let Some(validator) = &mut self.validator {
            validator.type_section(&section)?;
        }
        let limits = self.engine.config().get_enforced_limits();
        let func_types = section.into_iter().map(|result| {
            let ty = result?.into_types().next().unwrap();
            let func_ty = ty.unwrap_func();
            if let Some(limit) = limits.max_params {
                if func_ty.params().len() > limit {
                    return Err(Error::from(EnforcedLimitsError::TooManyParameters {
                        limit,
                    }));
                }
            }
            if let Some(limit) = limits.max_results {
                if func_ty.results().len() > limit {
                    return Err(Error::from(EnforcedLimitsError::TooManyResults { limit }));
                }
            }
            Ok(FuncType::from_wasmparser(func_ty))
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
        if let Some(validator) = &mut self.validator {
            validator.import_section(&section)?;
        }
        let imports = section
            .into_iter()
            .map(|import| import.map(Import::from).map_err(Error::from));
        header.push_imports(imports)?;
        Ok(())
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
        if let Some(limit) = self.engine.config().get_enforced_limits().max_functions {
            if section.count() > limit {
                return Err(Error::from(EnforcedLimitsError::TooManyFunctions { limit }));
            }
        }
        if let Some(validator) = &mut self.validator {
            validator.function_section(&section)?;
        }
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
        if let Some(limit) = self.engine.config().get_enforced_limits().max_tables {
            if section.count() > limit {
                return Err(Error::from(EnforcedLimitsError::TooManyTables { limit }));
            }
        }
        if let Some(validator) = &mut self.validator {
            validator.table_section(&section)?;
        }
        let tables = section.into_iter().map(|table| match table {
            Ok(table) => {
                assert!(matches!(table.init, wasmparser::TableInit::RefNull));
                Ok(TableType::from_wasmparser(table.ty))
            }
            Err(err) => Err(err.into()),
        });
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
        if let Some(limit) = self.engine.config().get_enforced_limits().max_memories {
            if section.count() > limit {
                return Err(Error::from(EnforcedLimitsError::TooManyMemories { limit }));
            }
        }
        if let Some(validator) = &mut self.validator {
            validator.memory_section(&section)?;
        }
        let memories = section
            .into_iter()
            .map(|memory| memory.map(MemoryType::from_wasmparser).map_err(Error::from));
        header.push_memories(memories)?;
        Ok(())
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
        if let Some(limit) = self.engine.config().get_enforced_limits().max_globals {
            if section.count() > limit {
                return Err(Error::from(EnforcedLimitsError::TooManyGlobals { limit }));
            }
        }
        if let Some(validator) = &mut self.validator {
            validator.global_section(&section)?;
        }
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
        if let Some(validator) = &mut self.validator {
            validator.export_section(&section)?;
        }
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
        if let Some(validator) = &mut self.validator {
            validator.start_section(func, &range)?;
        }
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
            .get_enforced_limits()
            .max_element_segments
        {
            if section.count() > limit {
                return Err(Error::from(EnforcedLimitsError::TooManyElementSegments {
                    limit,
                }));
            }
        }
        if let Some(validator) = &mut self.validator {
            validator.element_section(&section)?;
        }
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
        if let Some(limit) = self.engine.config().get_enforced_limits().max_data_segments {
            if count > limit {
                return Err(Error::from(EnforcedLimitsError::TooManyDataSegments {
                    limit,
                }));
            }
        }
        if let Some(validator) = &mut self.validator {
            validator.data_count_section(count, &range)?;
        }
        Ok(())
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
        if let Some(limit) = self.engine.config().get_enforced_limits().max_data_segments {
            if section.count() > limit {
                return Err(Error::from(EnforcedLimitsError::TooManyDataSegments {
                    limit,
                }));
            }
        }
        if let Some(validator) = &mut self.validator {
            // Note: data section does not belong to the Wasm module header.
            //
            // Also benchmarks show that validation of the data section can be very costly.
            validator.data_section(&section)?;
        }
        builder.reserve_data_segments(section.count() as usize);
        for segment in section {
            builder.push_data_segment(segment?)?;
        }
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
        let enforced_limits = self.engine.config().get_enforced_limits();
        if let Some(limit) = enforced_limits.max_functions {
            if count > limit {
                return Err(Error::from(EnforcedLimitsError::TooManyFunctions { limit }));
            }
        }
        if let Some(limit) = enforced_limits.min_avg_bytes_per_function {
            if size >= limit.req_funcs_bytes {
                let limit = limit.min_avg_bytes_per_function;
                let avg = size / count;
                if avg < limit {
                    return Err(Error::from(EnforcedLimitsError::MinAvgBytesPerFunction {
                        limit,
                        avg,
                    }));
                }
            }
        }
        if let Some(validator) = &mut self.validator {
            validator.code_section_start(&range)?;
        }
        Ok(())
    }

    /// Returns the next `FuncIdx` for processing of its function body.
    fn next_func(&mut self, header: &ModuleHeader) -> (FuncIdx, EngineFunc) {
        let index = self.engine_funcs;
        let engine_func = header.inner.engine_funcs.get_or_panic(index);
        self.engine_funcs += 1;
        // We have to adjust the initial func reference to the first
        // internal function before we process any of the internal functions.
        let len_func_imports = u32::try_from(header.inner.imports.len_funcs())
            .unwrap_or_else(|_| panic!("too many imported functions"));
        let func_idx = FuncIdx::from(index + len_func_imports);
        (func_idx, engine_func)
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
        bytes: &[u8],
        header: &ModuleHeader,
    ) -> Result<(), Error> {
        let (func, engine_func) = self.next_func(header);
        let module = header.clone();
        let offset = func_body.get_binary_reader().original_position();
        let func_to_validate = match &mut self.validator {
            Some(validator) => Some(validator.code_section_entry(&func_body)?),
            None => None,
        };
        self.engine
            .translate_func(func, engine_func, offset, bytes, module, func_to_validate)?;
        Ok(())
    }

    /// Process a single Wasm custom section.
    fn process_custom_section(
        &mut self,
        custom_sections: &mut CustomSectionsBuilder,
        reader: CustomSectionReader,
    ) -> Result<(), Error> {
        if self.engine.config().get_ignore_custom_sections() {
            return Ok(());
        }
        custom_sections.push(reader.name(), reader.data());
        Ok(())
    }

    /// Process an unexpected, unsupported or malformed Wasm module section payload.
    fn process_invalid_payload(&mut self, payload: Payload<'_>) -> Result<(), Error> {
        if let Some(validator) = &mut self.validator {
            if let Err(error) = validator.payload(&payload) {
                return Err(Error::from(error));
            }
        }
        panic!("encountered unsupported, unexpected or malformed Wasm payload: {payload:?}")
    }
}
