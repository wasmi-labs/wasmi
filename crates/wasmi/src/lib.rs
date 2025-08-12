//! The Wasmi virtual machine definitions.
//!
//! These closely mirror the WebAssembly specification definitions.
//! The overall structure is heavily inspired by the `wasmtime` virtual
//! machine architecture.
//!
//! # Example
//!
//! The following example shows a "Hello, World!"-like example of creating
//! a Wasm module from some initial `.wat` contents, defining a simple host
//! function and calling the exported Wasm function.
//!
//! The example was inspired by
//! [Wasmtime's API example](https://docs.rs/wasmtime/0.39.1/wasmtime/).
//!
//! ```
//! use wasmi::*;
//!
//! // In this simple example we are going to compile the below Wasm source,
//! // instantiate a Wasm module from it and call its exported "hello" function.
//! fn main() -> Result<(), wasmi::Error> {
//!     let wasm = r#"
//!         (module
//!             (import "host" "hello" (func $host_hello (param i32)))
//!             (func (export "hello")
//!                 (call $host_hello (i32.const 3))
//!             )
//!         )
//!     "#;
//!     // First step is to create the Wasm execution engine with some config.
//!     //
//!     // In this example we are using the default configuration.
//!     let engine = Engine::default();
//!     // Now we can compile the above Wasm module with the given Wasm source.
//!     let module = Module::new(&engine, wasm)?;
//!
//!     // Wasm objects operate within the context of a Wasm `Store`.
//!     //
//!     // Each `Store` has a type parameter to store host specific data.
//!     // In this example the host state is a simple `u32` type with value `42`.
//!     type HostState = u32;
//!     let mut store = Store::new(&engine, 42);
//!
//!     // A linker can be used to instantiate Wasm modules.
//!     // The job of a linker is to satisfy the Wasm module's imports.
//!     let mut linker = <Linker<HostState>>::new(&engine);
//!     // We are required to define all imports before instantiating a Wasm module.
//!     linker.func_wrap("host", "hello", |caller: Caller<'_, HostState>, param: i32| {
//!         println!("Got {param} from WebAssembly and my host state is: {}", caller.data());
//!     });
//!     let instance = linker
//!         .instantiate(&mut store, &module)?
//!         .start(&mut store)?;
//!     // Now we can finally query the exported "hello" function and call it.
//!     instance
//!         .get_typed_func::<(), ()>(&store, "hello")?
//!         .call(&mut store, ())?;
//!     Ok(())
//! }
//! ```
//!
//! # Crate Features
//!
//! | Feature | Crates | Description |
//! |:-:|:--|:--|
//! | `std` | `wasmi`<br>`wasmi_core`<br>`wasmi_ir`<br>`wasmi_collections` | Enables usage of Rust's standard library. This may have some performance advantages when enabled. Disabling this feature makes Wasmi compile on platforms that do not provide Rust's standard library such as many embedded platforms. <br><br> Enabled by default. |
//! | `wat` | `wasmi` | Enables support to parse Wat encoded Wasm modules. <br><br> Enabled by default. |
//! | `simd` | `wasmi`<br>`wasmi_core`<br>`wasmi_ir`<br>`wasmi_cli` | Enables support for the Wasm `simd` and `relaxed-simd` proposals. Note that this may introduce execution overhead and increased memory consumption for Wasm executions that do not need Wasm `simd` functionality. <br><br> Disabled by default. |
//! | `hash-collections` | `wasmi`<br>`wasmi_collections` | Enables use of hash-map based collections in Wasmi internals. This might yield performance improvements in some use cases. <br><br> Disabled by default. |
//! | `prefer-btree-collections` | `wasmi`<br>`wasmi_collections` | Enforces use of btree-map based collections in Wasmi internals. This may yield performance improvements and memory consumption decreases in some use cases. Also it enables Wasmi to run on platforms that have no random source. <br><br> Disabled by default. |
//! | `extra-checks` | `wasmi` | Enables extra runtime checks in the Wasmi executor. Expected execution overhead is ~20%. Enable this if your focus is on safety. Disable this for maximum execution performance. <br><br> Disabled by default. |

#![no_std]
#![warn(
    clippy::cast_lossless,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::redundant_closure_for_method_calls,
    clippy::type_repetition_in_bounds,
    clippy::inconsistent_struct_constructor,
    clippy::default_trait_access,
    clippy::items_after_statements
)]
#![recursion_limit = "1000"]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[macro_use]
mod foreach_tuple;

#[cfg(test)]
pub mod tests;

mod engine;
mod error;
mod func;
mod global;
mod instance;
mod limits;
mod linker;
mod memory;
mod module;
mod reftype;
mod store;
mod table;
mod value;

/// Definitions from the `wasmi_core` crate.
#[deprecated(since = "0.49.0", note = "use root `wasmi` definitions instead")]
pub mod core {
    #[cfg(feature = "simd")]
    pub(crate) use wasmi_core::simd;
    pub(crate) use wasmi_core::{
        hint,
        wasm,
        DecodeUntypedSlice,
        ElementSegment as CoreElementSegment,
        EncodeUntypedSlice,
        Fuel,
        FuelCostsProvider,
        FuncType as CoreFuncType,
        Global as CoreGlobal,
        IndexType,
        LimiterError,
        Memory as CoreMemory,
        MemoryType as CoreMemoryType,
        MemoryTypeBuilder as CoreMemoryTypeBuilder,
        ReadAs,
        ResourceLimiterRef,
        Table as CoreTable,
        TableType as CoreTableType,
        Typed,
        TypedVal,
        UntypedError,
        UntypedVal,
        WriteAs,
    };
    pub use wasmi_core::{
        GlobalType,
        Mutability,
        ResourceLimiter,
        TrapCode,
        ValType,
        F32,
        F64,
        V128,
    };
}

/// Definitions from the `wasmi_collections` crate.
#[doc(inline)]
use wasmi_collections as collections;

/// Definitions from the `wasmi_collections` crate.
#[doc(inline)]
use wasmi_ir as ir;

/// Defines some errors that may occur upon interaction with Wasmi.
pub mod errors {
    pub use super::{
        engine::EnforcedLimitsError,
        error::ErrorKind,
        func::FuncError,
        ir::Error as IrError,
        linker::LinkerError,
        module::{InstantiationError, ReadError},
    };
    pub use wasmi_core::{FuelError, GlobalError, HostError, MemoryError, TableError};
}

#[expect(deprecated)]
pub use self::linker::{state, LinkerBuilder};
#[expect(deprecated)]
pub use self::module::InstancePre;
pub use self::{
    engine::{
        CompilationMode,
        Config,
        EnforcedLimits,
        Engine,
        EngineWeak,
        ResumableCall,
        ResumableCallHostTrap,
        ResumableCallOutOfFuel,
        TypedResumableCall,
        TypedResumableCallHostTrap,
        TypedResumableCallOutOfFuel,
    },
    error::Error,
    func::{
        Caller,
        Func,
        FuncType,
        IntoFunc,
        TypedFunc,
        WasmParams,
        WasmResults,
        WasmRet,
        WasmTy,
        WasmTyList,
    },
    global::Global,
    instance::{Export, ExportsIter, Extern, ExternType, Instance},
    limits::{StoreLimits, StoreLimitsBuilder},
    linker::Linker,
    memory::{Memory, MemoryType, MemoryTypeBuilder},
    module::{
        CustomSection,
        CustomSectionsIter,
        ExportType,
        ImportType,
        Module,
        ModuleExportsIter,
        ModuleImportsIter,
        Read,
    },
    reftype::{ExternRef, Ref},
    store::{AsContext, AsContextMut, CallHook, Store, StoreContext, StoreContextMut},
    table::{Table, TableType},
    value::Val,
};
use self::{
    func::{FuncEntity, FuncIdx},
    global::GlobalIdx,
    instance::{InstanceEntity, InstanceEntityBuilder, InstanceIdx},
    memory::{DataSegmentEntity, DataSegmentIdx, MemoryIdx},
    store::Stored,
    table::{ElementSegment, ElementSegmentIdx, TableIdx},
};
pub use wasmi_core::{GlobalType, Mutability, ResourceLimiter, TrapCode, ValType, F32, F64, V128};
