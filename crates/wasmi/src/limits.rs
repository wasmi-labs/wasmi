// These limits mirror the WebAssembly JS-API implementation-defined limits
// (https://webassembly.github.io/spec/js-api/#limits).

use core::fmt;

const MAX_MODULE_SIZE: usize = 1_073_741_824; // 1 GiB
const MAX_TYPE_COUNT: usize = 1_000_000;
const MAX_IMPORT_COUNT: usize = 1_000_000;
const MAX_FUNC_COUNT: usize = 1_000_000;
const MAX_FUNC_PARAM_COUNT: usize = 1_000;
const MAX_FUNC_RESULT_COUNT: usize = 1_000;
const MAX_TABLE_COUNT: usize = 100_000;
const MAX_TABLE_SIZE: usize = 10_000_000;
const MAX_TABLE_INIT_SIZE: usize = 10_000_000;
const MAX_MEMORY_COUNT: usize = 100;
const MAX_GLOBAL_COUNT: usize = 1_000_000;
const MAX_EXPORT_COUNT: usize = 1_000_000;
const MAX_ELEM_COUNT: usize = 100_000;
const MAX_ELEM_SIZE: usize = 10_000_000;
const MAX_FUNC_LOCAL_COUNT: usize = 50_000;
const MAX_FUNC_BODY_SIZE: usize = 7_654_321;
const MAX_DATA_COUNT: usize = 1_000_000;
const MAX_DATA_SIZE: usize = 1_000_000;

#[derive(Debug, Copy, Clone)]
pub enum LimitsError {
    /// Wasm module contains too many bytes.
    ModuleTooBig,
    /// Wasm module uses too many types.
    TooManyTypes,
    /// Wasm module has too many imports.
    TooManyImports,
    /// Wasm module declares too many functions.
    TooManyFunctions,
    /// Function has too many parameters.
    TooManyFuncParams,
    /// Function has too many results.
    TooManyFuncResults,
    /// Wasm module has too many tables.
    TooManyTables,
    /// Table has too many elements.
    TableTooBig,
    /// Table initializer has too many elements.
    TableInitTooBig,
    /// Wasm module has too many linear memories.
    TooManyMemories,
    /// Wasm module has too many global variables.
    TooManyGlobals,
    /// Wasm module has too many exports.
    TooManyExports,
    /// Wasm module has too many element segments.
    TooManyElementSegments,
    /// Element segment has too many elements.
    ElementSegmentTooBig,
    /// Wasm module has too many data segments.
    TooManyDataSegments,
    /// Data segment contains too many bytes.
    DataSegmentTooBig,
    /// Function defines too many local variables (including parameters).
    TooManyFunctionLocals,
    /// Function body contains too many bytes.
    FunctionBodyTooBig,
    /// Instance has too many handles.
    TooManyInstanceHandles,
}

impl core::error::Error for LimitsError {}

macro_rules! impl_limits_error {
    (
        $(
            pub fn $name:ident(len: usize) -> Result<(), Self> = ($limit:expr, Self::$err:ident);
        )*
        $(;)?
    ) => {
        $(
            #[doc = concat!("Returns `Ok` if `len` does not exceed `", stringify!($limit), "`.")]
            pub fn $name(len: usize) -> Result<(), Self> {
                if len > $limit {
                    return Err(Self::$err)
                }
                Ok(())
            }
        )*
    };
}
impl LimitsError {
    impl_limits_error! {
        pub fn max_module_size(len: usize) -> Result<(), Self> = (MAX_MODULE_SIZE, Self::ModuleTooBig);
        pub fn max_type_count(len: usize) -> Result<(), Self> = (MAX_TYPE_COUNT, Self::TooManyTypes);
        pub fn max_import_count(len: usize) -> Result<(), Self> = (MAX_IMPORT_COUNT, Self::TooManyImports);
        pub fn max_func_count(len: usize) -> Result<(), Self> = (MAX_FUNC_COUNT, Self::TooManyFunctions);
        pub fn max_func_param_count(len: usize) -> Result<(), Self> = (MAX_FUNC_PARAM_COUNT, Self::TooManyFuncParams);
        pub fn max_func_result_count(len: usize) -> Result<(), Self> = (MAX_FUNC_RESULT_COUNT, Self::TooManyFuncResults);
        pub fn max_table_count(len: usize) -> Result<(), Self> = (MAX_TABLE_COUNT, Self::TooManyTables);
        pub fn max_table_size(len: usize) -> Result<(), Self> = (MAX_TABLE_SIZE, Self::TableTooBig);
        pub fn max_table_init_size(len: usize) -> Result<(), Self> = (MAX_TABLE_INIT_SIZE, Self::TableInitTooBig);
        pub fn max_memory_count(len: usize) -> Result<(), Self> = (MAX_MEMORY_COUNT, Self::TooManyMemories);
        pub fn max_global_count(len: usize) -> Result<(), Self> = (MAX_GLOBAL_COUNT, Self::TooManyGlobals);
        pub fn max_export_count(len: usize) -> Result<(), Self> = (MAX_EXPORT_COUNT, Self::TooManyExports);
        pub fn max_elem_count(len: usize) -> Result<(), Self> = (MAX_ELEM_COUNT, Self::TooManyElementSegments);
        pub fn max_elem_size(len: usize) -> Result<(), Self> = (MAX_ELEM_SIZE, Self::ElementSegmentTooBig);
        pub fn max_func_local_count(len: usize) -> Result<(), Self> = (MAX_FUNC_LOCAL_COUNT, Self::TooManyFunctionLocals);
        pub fn max_func_body_size(len: usize) -> Result<(), Self> = (MAX_FUNC_BODY_SIZE, Self::FunctionBodyTooBig);
        pub fn max_data_count(len: usize) -> Result<(), Self> = (MAX_DATA_COUNT, Self::TooManyDataSegments);
        pub fn max_data_size(len: usize) -> Result<(), Self> = (MAX_DATA_SIZE, Self::DataSegmentTooBig);
    }
}

impl fmt::Display for LimitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ModuleTooBig => "Wasm module contains too many bytes",
            Self::TooManyTypes => "Wasm module uses too many types",
            Self::TooManyImports => "Wasm module has too many imports",
            Self::TooManyFunctions => "Wasm module declares too many functions",
            Self::TooManyFuncParams => "function has too many parameters",
            Self::TooManyFuncResults => "function has too many results",
            Self::TooManyTables => "Wasm module has too many tables",
            Self::TableTooBig => "table has too many elements",
            Self::TableInitTooBig => "table initializer has too many elements",
            Self::TooManyMemories => "Wasm module has too many linear memories",
            Self::TooManyGlobals => "Wasm module has too many global variables",
            Self::TooManyExports => "Wasm module has too many exports",
            Self::TooManyElementSegments => "Wasm module has too many element segments",
            Self::ElementSegmentTooBig => "element segment has too many elements",
            Self::TooManyDataSegments => "Wasm module has too many data segments",
            Self::DataSegmentTooBig => "data segment contains too many bytes",
            Self::TooManyFunctionLocals => {
                "function defines too many local variables (including parameters)"
            }
            Self::FunctionBodyTooBig => "function body contains too many bytes",
            Self::TooManyInstanceHandles => "instance has too many handles",
        };
        f.write_str(s)
    }
}
