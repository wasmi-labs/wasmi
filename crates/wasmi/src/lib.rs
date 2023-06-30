//! The `wasmi` virtual machine definitions.
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
//! use anyhow::{anyhow, Result};
//! use wasmi::*;
//!
//! fn main() -> Result<()> {
//!     // First step is to create the Wasm execution engine with some config.
//!     // In this example we are using the default configuration.
//!     let engine = Engine::default();
//!     let wat = r#"
//!         (module
//!             (import "host" "hello" (func $host_hello (param i32)))
//!             (func (export "hello")
//!                 (call $host_hello (i32.const 3))
//!             )
//!         )
//!     "#;
//!     // Wasmi does not yet support parsing `.wat` so we have to convert
//!     // out `.wat` into `.wasm` before we compile and validate it.
//!     let wasm = wat::parse_str(&wat)?;
//!     let module = Module::new(&engine, &mut &wasm[..])?;
//!
//!     // All Wasm objects operate within the context of a `Store`.
//!     // Each `Store` has a type parameter to store host-specific data,
//!     // which in this case we are using `42` for.
//!     type HostState = u32;
//!     let mut store = Store::new(&engine, 42);
//!     let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
//!         println!("Got {param} from WebAssembly");
//!         println!("My host state is: {}", caller.data());
//!     });
//!
//!     // In order to create Wasm module instances and link their imports
//!     // and exports we require a `Linker`.
//!     let mut linker = <Linker<HostState>>::new(&engine);
//!     // Instantiation of a Wasm module requires defining its imports and then
//!     // afterwards we can fetch exports by name, as well as asserting the
//!     // type signature of the function with `get_typed_func`.
//!     //
//!     // Also before using an instance created this way we need to start it.
//!     linker.define("host", "hello", host_hello)?;
//!     let instance = linker
//!         .instantiate(&mut store, &module)?
//!         .start(&mut store)?;
//!     let hello = instance.get_typed_func::<(), ()>(&store, "hello")?;
//!
//!     // And finally we can call the wasm!
//!     hello.call(&mut store, ())?;
//!
//!     Ok(())
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
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
#![recursion_limit = "750"]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

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

/// Defines some errors that may occur upon interaction with `wasmi`.
pub mod errors {
    pub use super::{
        func::FuncError,
        global::GlobalError,
        linker::LinkerError,
        memory::MemoryError,
        module::{InstantiationError, ModuleError},
        store::FuelError,
        table::TableError,
    };
}

pub use self::{
    engine::{
        Config,
        Engine,
        FuelConsumptionMode,
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
        WasmType,
        WasmTypeList,
    },
    global::{Global, GlobalType, Mutability},
    instance::{Export, ExportsIter, Extern, ExternType, Instance},
    limits::{ResourceLimiter, StoreLimits, StoreLimitsBuilder},
    linker::Linker,
    memory::{Memory, MemoryType},
    module::{
        ExportType,
        ImportType,
        InstancePre,
        Module,
        ModuleExportsIter,
        ModuleImportsIter,
        Read,
    },
    store::{AsContext, AsContextMut, Store, StoreContext, StoreContextMut},
    table::{Table, TableType},
    value::Value,
};
use self::{
    func::{FuncEntity, FuncIdx},
    global::{GlobalEntity, GlobalIdx},
    instance::{InstanceEntity, InstanceEntityBuilder, InstanceIdx},
    memory::{DataSegmentEntity, DataSegmentIdx, MemoryEntity, MemoryIdx},
    store::{StoreInner, Stored},
    table::{ElementSegment, ElementSegmentEntity, ElementSegmentIdx, TableEntity, TableIdx},
};
