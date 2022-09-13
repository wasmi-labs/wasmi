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
//!         println!("My host state is: {}", caller.host_data());
//!     });
//!
//!     // In order to create Wasm module instances and link their imports
//!     // and exports we require a `Linker`.
//!     let mut linker = <Linker<HostState>>::new();
//!     // Instantiation of a Wasm module requires defning its imports and then
//!     // afterwards we can fetch exports by name, as well as asserting the
//!     // type signature of the function with `get_typed_func`.
//!     //
//!     // Also before using an instance created this way we need to start it.
//!     linker.define("host", "hello", host_hello)?;
//!     let instance = linker
//!         .instantiate(&mut store, &module)?
//!         .start(&mut store)?;
//!     let hello = instance
//!         .get_export(&store, "hello")
//!         .and_then(Extern::into_func)
//!         .ok_or_else(|| anyhow!("could not find function \"hello\""))?
//!         .typed::<(), (), _>(&mut store)?;
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
    clippy::map_unwrap_or,
    clippy::items_after_statements
)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

#[macro_use]
mod foreach_tuple;

mod arena;
mod engine;
mod error;
mod external;
mod func;
mod func_type;
mod global;
mod instance;
mod linker;
mod memory;
mod module;
mod store;
mod table;

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
        table::TableError,
    };
}

use self::{
    arena::{GuardedEntity, Index},
    func::{FuncEntity, FuncIdx},
    global::{GlobalEntity, GlobalIdx},
    instance::{InstanceEntity, InstanceEntityBuilder, InstanceIdx},
    memory::{MemoryEntity, MemoryIdx},
    store::Stored,
    table::{TableEntity, TableIdx},
};
pub use self::{
    engine::{Config, Engine, StackLimits},
    error::Error,
    external::Extern,
    func::{Caller, Func, TypedFunc, WasmParams, WasmResults},
    func_type::FuncType,
    global::{Global, GlobalType, Mutability},
    instance::{ExportsIter, Instance},
    linker::Linker,
    memory::{Memory, MemoryType},
    module::{
        ExportItem,
        ExportItemKind,
        InstancePre,
        Module,
        ModuleError,
        ModuleExportsIter,
        Read,
    },
    store::{AsContext, AsContextMut, Store, StoreContext, StoreContextMut},
    table::{Table, TableType},
};
