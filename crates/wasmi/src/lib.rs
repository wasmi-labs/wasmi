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
//! fn main() -> anyhow::Result<()> {
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

mod engine;
mod error;
mod externref;
mod func;
mod global;
mod instance;
mod limits;
mod linker;
mod memory;
mod module;
mod store;
mod table;
mod value;

/// Definitions from the `wasmi_core` crate.
#[doc(inline)]
pub use wasmi_core as core;

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
        global::GlobalError,
        ir::Error as IrError,
        linker::LinkerError,
        memory::MemoryError,
        module::{InstantiationError, ReadError},
        store::FuelError,
        table::TableError,
    };
}

pub use self::{
    engine::{
        CompilationMode,
        Config,
        EnforcedLimits,
        Engine,
        EngineWeak,
        ResumableCall,
        ResumableInvocation,
        StackLimits,
        TypedResumableCall,
        TypedResumableInvocation,
    },
    error::Error,
    externref::ExternRef,
    func::{
        Caller,
        Func,
        FuncRef,
        FuncType,
        IntoFunc,
        TypedFunc,
        WasmParams,
        WasmResults,
        WasmRet,
        WasmTy,
        WasmTyList,
    },
    global::{Global, GlobalType, Mutability},
    instance::{Export, ExportsIter, Extern, ExternType, Instance},
    limits::{ResourceLimiter, StoreLimits, StoreLimitsBuilder},
    linker::{state, Linker, LinkerBuilder},
    memory::{Memory, MemoryType},
    module::{
        CustomSection,
        CustomSectionsIter,
        ExportType,
        ImportType,
        InstancePre,
        Module,
        ModuleExportsIter,
        ModuleImportsIter,
        Read,
    },
    store::{AsContext, AsContextMut, CallHook, Store, StoreContext, StoreContextMut},
    table::{Table, TableType},
    value::Val,
};
use self::{
    func::{FuncEntity, FuncIdx},
    global::{GlobalEntity, GlobalIdx},
    instance::{InstanceEntity, InstanceEntityBuilder, InstanceIdx},
    memory::{DataSegmentEntity, DataSegmentIdx, MemoryEntity, MemoryIdx},
    store::Stored,
    table::{ElementSegment, ElementSegmentEntity, ElementSegmentIdx, TableEntity, TableIdx},
};
