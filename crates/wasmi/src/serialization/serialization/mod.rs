use crate::serialization::types::{SerializedActiveDataSegment, SerializedPassiveDataSegment};
use crate::serialization::{
    EngineConfig, SerializationError, SerializedDataSegment, SerializedElementSegment,
    SerializedMemoryType, SerializedModule, SerializedTableType, SERIALIZATION_VERSION,
};
use crate::Engine;

use wasmi_core::ReadAs;
extern crate alloc;
use crate::serialization::serialized_module::types::{
    SerializedConstExpr, SerializedExport, SerializedFeatures, SerializedFuncType,
    SerializedGlobal, SerializedGlobalType, SerializedImport, SerializedInternalFunc,
};
use crate::{serialization::RequiredFeatures, ExternType, Module};
use alloc::vec::Vec;

#[cfg(test)]
mod tests;

/// Serializes a Wasmi module to a compact binary format.
///
/// # Arguments
///
/// * `module` - The Wasmi module to serialize
/// * `features` - The required features configuration
///
/// # Returns
///
/// Returns a `Vec<u8>` containing the serialized module data.
///
/// # Errors
///
/// Returns a `SerializationError` if serialization fails.
pub fn serialize_module(
    module: &crate::Module,
    features: &RequiredFeatures,
    engine: &Engine,
) -> Result<alloc::vec::Vec<u8>, SerializationError> {
    let ser = SerializedModule::from_module(module, features, engine)?;
    postcard::to_allocvec(&ser).map_err(|_e| SerializationError::SerializationFailed {
        cause: "postcard serialization failed",
    })
}

impl SerializedModule {
    pub(crate) fn from_module(
        module: &Module,
        features: &RequiredFeatures,
        engine: &Engine,
    ) -> Result<Self, crate::serialization::error::SerializationError> {
        let engine_config = engine.config();
        let use_fuel = engine_config.get_consume_fuel();
        let serialized_engine_config = EngineConfig { use_fuel };

        // serialize the module
        let func_types: Vec<SerializedFuncType> = extract_func_types(module);
        let imports = extract_imports(module, &func_types);
        let mut internal_functions = Vec::new();
        let engine = module.engine();
        let num_imports = module.imports().count();
        for (internal_idx, (dedup_fn, engine_func)) in module.internal_funcs().enumerate() {
            let actual_func_idx = num_imports + internal_idx;
            // Find the function type index as before
            let func_type = engine.resolve_func_type(&dedup_fn, Clone::clone);
            let ser_fn_type = SerializedFuncType::from(&func_type);
            let type_index = func_types
                .iter()
                .position(|fn_ty| *fn_ty == ser_fn_type)
                .expect("internal function type not found") as u32;

            // Extract function metadata from the engine
            let compiled_func = engine.get_compiled_func(engine_func).map_err(|_| {
                crate::serialization::error::SerializationError::SerializationFailed {
                    cause: "failed to get compiled function",
                }
            })?;

            // Extract all instructions for this function using the new helper
            let instrs = engine
                .get_instructions(engine_func)
                .map_err(|_| {
                    crate::serialization::error::SerializationError::SerializationFailed {
                        cause: "failed to extract instructions",
                    }
                })?
                .to_vec();

            // Build the serialized function struct
            let serialized_func = SerializedInternalFunc {
                type_idx: type_index,
                func_idx: actual_func_idx as u32,
                len_registers: compiled_func.len_registers(),
                consts: compiled_func.consts().to_vec(),
                instructions: instrs,
            };
            internal_functions.push(serialized_func);
        }

        // Extract internal tables
        let tables: Vec<SerializedTableType> = module
            .internal_tables()
            .map(SerializedTableType::from)
            .collect();

        // Extract internal memories
        let memories: Vec<SerializedMemoryType> = module
            .internal_memories()
            .map(SerializedMemoryType::from)
            .collect();

        // Extract internal globals
        let globals: Vec<SerializedGlobal> = module
            .internal_globals()
            .map(|(global_type, global_init)| SerializedGlobal {
                ty: SerializedGlobalType::from(global_type),
                init: SerializedConstExpr::from(global_init),
            })
            .collect();

        // Extract exports with actual indices
        let mut exports = Vec::new();
        for (export_name, extern_idx) in module.exports_with_indices() {
            match extern_idx {
                crate::module::export::ExternIdx::Func(func_idx) => {
                    exports.push(SerializedExport::from_export_with_func_idx(
                        export_name,
                        func_idx.into_u32(),
                    ));
                }
                crate::module::export::ExternIdx::Table(_table_idx) => {
                    unimplemented!("table exports are not supported yet");
                }
                crate::module::export::ExternIdx::Memory(memory_idx) => {
                    // Get the memory type from the module
                    let memory_type =
                        &module.header().inner.memories[memory_idx.into_u32() as usize];
                    exports.push(SerializedExport::from_export_with_memory_type(
                        export_name,
                        memory_type,
                    ));
                }
                crate::module::export::ExternIdx::Global(global_idx) => {
                    // Store the global index directly
                    exports.push(SerializedExport::from_export_with_global_idx(
                        export_name,
                        global_idx.into_u32(),
                    ));
                }
            }
        }

        // Extract start function index
        let start = module.start_func_index();

        // Extract element segments
        let element_segments = module
            .element_segments()
            .map(|segment| {
                // For now, only handle active element segments with function references
                match segment.kind() {
                    crate::module::ElementSegmentKind::Active(active) => {
                        // Extract function indices from items
                        let function_indices: Vec<u32> = segment
                            .items()
                            .iter()
                            .map(|item| {
                                item.funcref().map(super::super::module::export::FuncIdx::into_u32).expect(
                                    "Only function references are supported in element segments",
                                )
                            })
                            .collect();

                        SerializedElementSegment {
                            table_index: active.table_index().into_u32(),
                            offset: SerializedConstExpr::from(active.offset()),
                            function_indices,
                        }
                    }
                    _ => panic!("Only active element segments are supported for serialization"),
                }
            })
            .collect();

        // Extract data segments
        let data_segments = module
            .all_init_data_segments()
            .map(|segment| match segment {
                crate::module::InitDataSegment::Active { memory_index, offset, bytes } => {
                    // Only support i32.const offsets
                    use crate::module::init_expr::{Op};
                    match &offset.op {
                        Op::Const(op) => {


                            // Try to extract i32 value
                            if let Some(val) = op.value.into() {
                                SerializedDataSegment::Active(SerializedActiveDataSegment {
                                    memory_index: memory_index.into_u32(),
                                    offset: val.read_as(),
                                    bytes: bytes.to_vec(),
                                })
                            } else {
                                panic!("Only i32.const offsets are supported for data segment serialization");
                            }
                        }
                        _ => panic!("Only i32.const offsets are supported for data segment serialization"),
                    }
                }
                crate::module::InitDataSegment::Passive { bytes } => {
                    SerializedDataSegment::Passive(SerializedPassiveDataSegment {
                        bytes: bytes.as_ref().to_vec(),
                    })
                }
            })
            .collect();

        Ok(SerializedModule {
            version: SERIALIZATION_VERSION,
            required_features: SerializedFeatures {
                simd: features.simd,
                bulk_memory: features.bulk_memory,
                reference_types: features.reference_types,
                tail_calls: features.tail_calls,
            },
            func_types,
            imports,
            internal_functions,
            tables,
            memories,
            globals,
            exports,
            start,
            data_segments,
            element_segments,
            engine_config: serialized_engine_config,
        })
    }
}

fn extract_func_types(module: &Module) -> Vec<SerializedFuncType> {
    let header = module.func_types_cloned();
    header
        .iter()
        .map(|dedup_ty| {
            let func_type = module.engine().resolve_func_type(dedup_ty, Clone::clone);
            SerializedFuncType::from(&func_type)
        })
        .collect()
}

fn extract_imports(
    module: &Module,
    ser_func_types: &[SerializedFuncType],
) -> Vec<SerializedImport> {
    let mut func_idx = 0;
    let mut global_idx = 0;

    module
        .imports()
        .map(|import| {
            let result = match import.ty() {
                ExternType::Func(_) => {
                    let idx = func_idx;
                    func_idx += 1;
                    SerializedImport::from_import(&import, ser_func_types, idx, 0)
                }
                ExternType::Global(_) => {
                    let idx = global_idx;
                    global_idx += 1;
                    SerializedImport::from_import(&import, ser_func_types, 0, idx)
                }
                _ => SerializedImport::from_import(&import, ser_func_types, 0, 0),
            };
            result
        })
        .collect()
}
