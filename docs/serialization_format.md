# Wasmi Serialization Format

## Overview

This document describes the serialization format used by Wasmi to convert parsed and validated WebAssembly modules into a compact binary representation. The serialized format is designed for:

- **Compactness**: Minimal binary size for embedded targets
- **Cross-platform compatibility**: No platform-specific data
- **Execution efficiency**: All data needed for execution is preserved
- **Versioning**: Forward/backward compatibility support

## Format Version

Current version: `1`

## Data Mapping

### Module → SerializedModule

The following table maps Wasmi's internal `Module` structure to the serialized format:

| Module Component | SerializedModule Field | Description |
|------------------|------------------------|-------------|
| `ModuleHeader::func_types` | `func_types` | Deduplicated function type signatures |
| `ModuleHeader::imports` | `imports` | Import declarations with names and types |
| `ModuleHeader::funcs` | `internal_funcs` | Internal function type indices |
| `ModuleHeader::tables` | `tables` | Table type definitions |
| `ModuleHeader::memories` | `memories` | Memory type definitions |
| `ModuleHeader::globals` + `globals_init` | `globals` | Global types and initial values |
| `ModuleHeader::exports` | `exports` | Export declarations |
| `ModuleHeader::start` | `start` | Start function index |
| `ModuleHeader::element_segments` | `element_segments` | Table element segments |
| `Module::data_segments` | `data_segments` | Memory data segments |
| `EngineFunc` bodies | `compiled_funcs` | Compiled function bytecode |

### Excluded Data

The following data is **not** serialized:

- **Custom sections**: Debug info, source maps, etc.
- **Runtime data**: Engine references, weak pointers
- **Validation state**: Parser/validator internal state
- **Allocation metadata**: Arena indices, memory layout info
- **Lazy function data**: Raw Wasm bytes for lazy compilation

## Serialized Data Structures

### SerializedModule

```rust
pub struct SerializedModule {
    pub version: u32,                    // Format version
    pub required_features: SerializedFeatures,
    pub func_types: Vec<SerializedFuncType>,
    pub imports: Vec<SerializedImport>,
    pub internal_funcs: Vec<u32>,        // Indices into func_types
    pub tables: Vec<SerializedTableType>,
    pub memories: Vec<SerializedMemoryType>,
    pub globals: Vec<SerializedGlobal>,
    pub exports: Vec<SerializedExport>,
    pub start: Option<u32>,
    pub element_segments: Vec<SerializedElementSegment>,
    pub data_segments: Vec<SerializedDataSegment>,
    pub compiled_funcs: Vec<SerializedCompiledFunc>,
}
```

### SerializedFeatures

```rust
pub struct SerializedFeatures {
    pub simd: bool,
    pub bulk_memory: bool,
    pub reference_types: bool,
    pub tail_calls: bool,
    pub function_references: bool,
}
```

### SerializedFuncType

```rust
pub struct SerializedFuncType {
    pub params: Vec<SerializedValType>,
    pub results: Vec<SerializedValType>,
}
```

### SerializedValType

```rust
pub enum SerializedValType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}
```

### SerializedImport

```rust
pub struct SerializedImport {
    pub module: String,
    pub name: String,
    pub ty: SerializedExternType,
}
```

### SerializedExternType

```rust
pub enum SerializedExternType {
    Func(u32),                    // Index into func_types
    Table(SerializedTableType),
    Memory(SerializedMemoryType),
    Global(SerializedGlobalType),
}
```

### SerializedTableType

```rust
pub struct SerializedTableType {
    pub element: SerializedValType,
    pub min: u32,
    pub max: Option<u32>,
}
```

### SerializedMemoryType

```rust
pub struct SerializedMemoryType {
    pub min: u32,                 // Pages
    pub max: Option<u32>,         // Pages
    pub shared: bool,
    pub page_size_log2: u8,       // 0=1B, 16=64KB
}
```

### SerializedGlobal

```rust
pub struct SerializedGlobal {
    pub ty: SerializedGlobalType,
    pub init: SerializedConstExpr,
}
```

### SerializedGlobalType

```rust
pub struct SerializedGlobalType {
    pub val_type: SerializedValType,
    pub mutable: bool,
}
```

### SerializedConstExpr

```rust
pub enum SerializedConstExpr {
    I32Const(i32),
    I64Const(i64),
    F32Const(u32),               // Bit pattern
    F64Const(u64),               // Bit pattern
    V128Const([u8; 16]),
    GlobalGet(u32),              // Index into globals
    RefNull(SerializedValType),
    RefFunc(u32),                // Index into functions
}
```

### SerializedExport

```rust
pub struct SerializedExport {
    pub name: String,
    pub ty: SerializedExternType,
}
```

### SerializedElementSegment

```rust
pub struct SerializedElementSegment {
    pub table: u32,
    pub offset: SerializedConstExpr,
    pub elements: Vec<u32>,       // Function indices
}
```

### SerializedDataSegment

```rust
pub struct SerializedDataSegment {
    pub memory: u32,
    pub offset: SerializedConstExpr,
    pub data: Vec<u8>,
}
```

### SerializedCompiledFunc

```rust
pub struct SerializedCompiledFunc {
    pub index: u32,               // Function index
    pub code: Vec<u8>,            // Compiled bytecode
    pub locals: Vec<SerializedValType>,
}
```

## Binary Format

The serialized module is encoded using the [Postcard](https://github.com/jamesmunns/postcard) format for maximum compactness:

1. **Postcard serialization** of `SerializedModule` struct
2. **No additional framing** or metadata
3. **Endian-neutral** representation
4. **Deterministic output** for identical modules

## Version Compatibility

### Version 1
- Initial format
- Supports all current Wasm features
- Includes feature flags for future compatibility

### Future Versions
- New versions will add fields with default values
- Old versions will be rejected with clear error messages
- Feature flags allow graceful degradation

## Security Considerations

1. **No arbitrary code execution** during deserialization
2. **Bounded memory allocation** with size limits
3. **Validation of all indices** and references
4. **Feature compatibility checking** before execution
5. **No pointer serialization** or platform-specific data

## Performance Characteristics

- **Serialization**: O(n) where n is module size
- **Deserialization**: O(n) with minimal allocations
- **Memory overhead**: ~10-20% of original module size
- **Binary size**: Typically 30-50% smaller than Wasm binary

## Example Usage

```rust
use wasmi::{Engine, Module};
use wasmi_serialization::{serialize_module, deserialize_module};

// Serialize a module
let engine = Engine::default();
let module = Module::new(&engine, wasm_bytes)?;
let serialized = serialize_module(&module)?;

// Deserialize on target device
let target_engine = Engine::default();
let deserialized = deserialize_module(&target_engine, &serialized)?;

// Use the module
let mut store = wasmi::Store::new(&target_engine, ());
let instance = wasmi::Instance::new(&mut store, &deserialized, &[])?;
``` 