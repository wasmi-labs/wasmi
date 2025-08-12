use wasmi_core::GlobalType;

use crate::module::export::{ExternIdx, FuncIdx, MemoryIdx};
use crate::module::GlobalIdx;

use crate::Config;
use crate::{
    module::{
        builder::{ModuleBuilder, ModuleHeaderBuilder, ModuleImportsBuilder},
        ImportName,
    },
    serialization::{
        DeserializationError, SerializedExternType, SerializedModule, SERIALIZATION_VERSION,
    },
    Engine, FuncType, MemoryType, Module, TableType,
};
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::module::data::{
    ActiveDataSegment, DataSegment, DataSegmentInner, DataSegments, PassiveDataSegmentBytes,
};
use crate::module::element::{ActiveElementSegment, ElementSegment, ElementSegmentKind};
use crate::module::export::TableIdx;
use crate::module::init_expr::{ConstExpr, Op};
use wasmi_core::ValType;

#[cfg(all(test, feature = "parser"))]
mod tests;

/// Deserializes a Wasmi module from a compact binary format.
///
/// # Arguments
///
/// * `engine` - The Wasmi engine to use for deserialization
/// * `data` - The serialized module data
///
/// # Returns
///
/// Returns a `Module` instance.
///
/// # Errors
///
/// Returns a `DeserializationError` if deserialization fails.
pub fn deserialize_module(data: &[u8]) -> Result<(Module, Engine), DeserializationError> {
    let ser_mod: SerializedModule = postcard::from_bytes(data).map_err(|_e| {
        crate::serialization::error::DeserializationError::CorruptedData {
            reason: "postcard deserialization failed",
        }
    })?;

    if ser_mod.version != SERIALIZATION_VERSION {
        return Err(
            crate::serialization::error::DeserializationError::UnsupportedVersion {
                version: ser_mod.version,
                supported: SERIALIZATION_VERSION,
            },
        );
    }

    let engine = set_engine_features(&ser_mod)?;
    let module = ser_mod.deserialize(&engine)?;
    Ok((module, engine))
}

pub(super) fn set_engine_features(
    ser_mod: &SerializedModule,
) -> Result<Engine, DeserializationError> {
    let mut engine_config = Config::default();

    engine_config.consume_fuel(ser_mod.engine_config.use_fuel);
    engine_config.compilation_mode(crate::CompilationMode::Eager); // just always setting this for now
    let engine = Engine::new(&engine_config);
    Ok(engine)
}

impl SerializedModule {
    pub(crate) fn deserialize(self, engine: &Engine) -> Result<Module, DeserializationError> {
        // Step 1: Reconstruct deduplicated function types, allocating them in the engine used for deser
        let func_types: Vec<_> = self
            .func_types
            .iter()
            .map(|ser_func_type| {
                let func_type = FuncType::from(ser_func_type);
                engine.alloc_func_type(func_type)
            })
            .collect();

        // Step 2: Imports (names only, no types yet)
        let mut imports = ModuleImportsBuilder::default();
        for import in &self.imports {
            let name = ImportName::new(&import.module, &import.name);
            match &import.ty {
                SerializedExternType::Func(_) => imports.funcs.push(name),
                SerializedExternType::Table(_) => imports.tables.push(name),
                SerializedExternType::Memory(_) => imports.memories.push(name),
                SerializedExternType::Global(_) => imports.globals.push(name),
                SerializedExternType::GlobalIdx(_) => imports.globals.push(name),
            }
        }

        // Build header with only func_types and imports
        let mut header_builder = ModuleHeaderBuilder::new(engine);
        header_builder.func_types = func_types;
        header_builder.imports = imports;

        // Also populate the type vectors for imports
        for import in &self.imports {
            match &import.ty {
                SerializedExternType::Func(idx) => {
                    header_builder
                        .funcs
                        .push(header_builder.func_types[*idx as usize]);
                }
                SerializedExternType::Table(table) => {
                    header_builder.tables.push(TableType::from(table));
                }
                SerializedExternType::Memory(mem) => {
                    header_builder.memories.push(MemoryType::from(mem));
                }
                SerializedExternType::Global(global) => {
                    header_builder.globals.push(GlobalType::from(global));
                }
                SerializedExternType::GlobalIdx(_) => {
                    unreachable!("GlobalIdx should not be used for imports");
                }
            }
        }

        // Reconstruct the functions
        let num_internal_funcs = self.internal_functions.len();
        let engine_funcs = engine.inner.code_map.alloc_funcs(num_internal_funcs);

        for (serialized_func, engine_fn) in self.internal_functions.iter().zip(engine_funcs.iter())
        {
            let compiled = crate::engine::code_map::CompiledFuncEntity::new(
                serialized_func.len_registers,
                serialized_func.instructions.clone(),
                serialized_func.consts.clone(),
            );
            engine
                .inner
                .code_map
                .init_func_as_compiled(engine_fn, compiled);
            // Add the deduped type to the funcs vector
            let func_type = &header_builder.func_types[serialized_func.type_idx as usize];
            header_builder.funcs.push(*func_type);
        }

        header_builder.engine_funcs = engine_funcs;

        // Reconstruct exports (functions, tables, memories only)

        let num_imported_memories = header_builder.imports.memories.len();
        for export in &self.exports {
            let name: Box<str> = export.name.clone().into_boxed_str();
            let idx = match &export.ty {
                SerializedExternType::Func(func_idx) => {
                    // Now func_idx is the actual function index, not a function type index
                    ExternIdx::Func(FuncIdx::from(*func_idx))
                }
                SerializedExternType::Table(_serialized_table_type) => {
                    unimplemented!("table exports are not supported yet");
                }
                SerializedExternType::Memory(serialized_memory_type) => {
                    // Find the memory index by matching the type
                    let memory_type = MemoryType::from(serialized_memory_type);
                    let memory_idx = if let Some(idx) = header_builder
                        .memories
                        .iter()
                        .position(|m| *m == memory_type)
                    {
                        num_imported_memories as u32 + idx as u32
                    } else {
                        // Must be an imported memory - find it in imports
                        let import_idx = header_builder
                            .imports
                            .memories
                            .iter()
                            .position(|_| true)
                            .unwrap_or(0);
                        import_idx as u32
                    };
                    ExternIdx::Memory(MemoryIdx::from(memory_idx))
                }
                SerializedExternType::Global(_) => {
                    unreachable!("Global should not be used for exports, only GlobalIdx");
                }
                SerializedExternType::GlobalIdx(global_idx) => {
                    // Use the global index directly for exports
                    ExternIdx::Global(GlobalIdx::from(*global_idx))
                }
            };
            header_builder.exports.insert(name, idx);
        }

        // Add internal memories to the header
        for memory_type in &self.memories {
            header_builder.memories.push(MemoryType::from(memory_type));
        }

        // Add internal tables to the header
        for table_type in &self.tables {
            header_builder.tables.push(TableType::from(table_type));
        }

        // Add internal globals to the header
        for global in &self.globals {
            header_builder.globals.push(GlobalType::from(&global.ty));
            header_builder
                .globals_init
                .push(ConstExpr::from(&global.init));
        }

        // Add element segments to the header
        let element_segments: Vec<Result<ElementSegment, _>> = self
            .element_segments
            .iter()
            .map(|element_segment| {
                let offset = ConstExpr::from(&element_segment.offset);
                let items: Box<[ConstExpr]> = element_segment
                    .function_indices
                    .iter()
                    .map(|&func_idx| ConstExpr::new_funcref(func_idx))
                    .collect();

                Ok(ElementSegment {
                    kind: ElementSegmentKind::Active(ActiveElementSegment {
                        table_index: TableIdx::from(element_segment.table_index),
                        offset,
                    }),
                    ty: ValType::FuncRef,
                    items,
                })
            })
            .collect();
        header_builder.push_element_segments(element_segments)?;

        let header = header_builder.finish();

        // Build module with custom sections and data segments from the serialized module
        let custom_sections = crate::module::custom_section::CustomSectionsBuilder::default();

        let mut segments = Vec::new();
        let mut active_bytes = Vec::new();
        for seg in &self.data_segments {
            match seg {
                crate::serialization::serialized_module::SerializedDataSegment::Active(active) => {
                    let offset_expr = ConstExpr {
                        op: Op::Const(crate::module::init_expr::ConstOp {
                            value: active.offset.into(),
                        }),
                    };
                    let segment = DataSegment {
                        inner: DataSegmentInner::Active(ActiveDataSegment {
                            memory_index: MemoryIdx::from(active.memory_index),
                            offset: offset_expr,
                            len: active.bytes.len() as u32,
                        }),
                    };
                    active_bytes.extend_from_slice(&active.bytes);
                    segments.push(segment);
                }
                crate::serialization::serialized_module::SerializedDataSegment::Passive(
                    passive,
                ) => {
                    let segment = DataSegment {
                        inner: DataSegmentInner::Passive {
                            bytes: PassiveDataSegmentBytes::from_vec(passive.bytes.clone()),
                        },
                    };
                    segments.push(segment);
                }
            }
        }
        let data_segments = DataSegments {
            segments: segments.into_boxed_slice(),
            bytes: active_bytes,
        };
        let mut module_builder = ModuleBuilder::new(header, custom_sections);
        module_builder.set_data_segments(data_segments);
        let module = module_builder.finish(engine);
        Ok(module)
    }
}
