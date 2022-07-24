//! # wasmi
//!
//! This library allows WebAssembly modules to be loaded in binary format and their functions invoked.
//!
//! # Introduction
//!
//! WebAssembly (wasm) is a safe, portable and compact format that is designed for efficient execution.
//!
//! Wasm code is distributed in the form of modules that contains definitions of:
//!
//! - functions,
//! - global variables,
//! - linear memory instances and
//! - tables.
//!
//! Each of these definitions can be imported and exported.
//!
//! In addition to these definitions, modules can define initialization data for their memory or tables. This initialization data can take the
//! form of segments, copied to given offsets. They can also define a `start` function that is automatically executed when the module is loaded.
//!
//! ## Loading and Validation
//!
//! Before execution, a module must be validated. This process checks that the module is well-formed
//! and makes only allowed operations.
//!
//! A valid module can't access memory outside its sandbox, can't cause stack underflows
//! and can only call functions with correct signatures.
//!
//! ## Instantiation
//!
//! In order to execute code from a wasm module, it must be instantiated.
//! Instantiation includes the following steps:
//!
//! 1. Creating an empty module instance.
//! 2. Resolving the definition instances for each declared import in the module.
//! 3. Instantiating definitions declared in the module (e.g. allocate global variables, allocate linear memory, etc.).
//! 4. Initializing memory and table contents by copying segments into them.
//! 5. Executing the `start` function, if any.
//!
//! After these steps, the module instance is ready to execute functions.
//!
//! ## Execution
//!
//! It only is allowed to call functions which are exported by the module.
//! Functions can either return a result or trap (e.g. there can't be linking error in the middle of the function execution).
//! This property is ensured by the validation process.
//!
//! # Examples
//!
//! ```rust
//! extern crate wasmi;
//! extern crate wat;
//!
//! use wasmi::{ModuleInstance, ImportsBuilder, NopExternals, RuntimeValue};
//!
//! fn main() {
//!     // Parse WAT (WebAssembly Text format) into wasm bytecode.
//!     let wasm_binary: Vec<u8> =
//!         wat::parse_str(
//!             r#"
//!             (module
//!                 (func (export "test") (result i32)
//!                     i32.const 1337
//!                 )
//!             )
//!             "#,
//!         )
//!         .expect("failed to parse wat");
//!
//!     // Load wasm binary and prepare it for instantiation.
//!     let module = wasmi::Module::from_buffer(&wasm_binary)
//!         .expect("failed to load wasm");
//!
//!     // Instantiate a module with empty imports and
//!     // assert that there is no `start` function.
//!     let instance =
//!         ModuleInstance::new(
//!             &module,
//!             &ImportsBuilder::default()
//!         )
//!         .expect("failed to instantiate wasm module")
//!         .assert_no_start();
//!
//!     // Finally, invoke the exported function "test" with no parameters
//!     // and empty external function executor.
//!     assert_eq!(
//!         instance.invoke_export(
//!             "test",
//!             &[],
//!             &mut NopExternals,
//!         ).expect("failed to execute export"),
//!         Some(RuntimeValue::I32(1337)),
//!     );
//! }
//! ```

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::len_without_is_empty)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt;
#[cfg(feature = "std")]
use std::error;

/// Internal interpreter error.
#[derive(Debug)]
pub enum Error {
    /// Module validation error. Might occur only at load time.
    Validation(String),
    /// Error while instantiating a module. Might occur when provided
    /// with incorrect exports (i.e. linkage failure).
    Instantiation(String),
    /// Function-level error.
    Function(String),
    /// Table-level error.
    Table(String),
    /// Memory-level error.
    Memory(String),
    /// Global-level error.
    Global(String),
    /// Value-level error.
    Value(String),
    /// Trap.
    Trap(Trap),
    /// Custom embedder error.
    Host(Box<dyn HostError>),
}

impl Error {
    /// Creates a new host error.
    pub fn host<T>(host_error: T) -> Self
    where
        T: HostError + Sized,
    {
        Self::Host(Box::new(host_error))
    }

    /// Returns a reference to a [`HostError`] if this `Error` represents some host error.
    ///
    /// I.e. if this error have variant [`Host`] or [`Trap`][`Trap`] with [host][`TrapKind::Host`] error.
    ///
    /// [`HostError`]: trait.HostError.html
    /// [`Host`]: enum.Error.html#variant.Host
    /// [`Trap`]: enum.Error.html#variant.Trap
    /// [`TrapKind::Host`]: enum.TrapKind.html#variant.Host
    pub fn as_host_error(&self) -> Option<&dyn HostError> {
        match self {
            Self::Host(host_error) => Some(&**host_error),
            Self::Trap(Trap::Host(host_error)) => Some(&**host_error),
            _ => None,
        }
    }

    /// Returns [`HostError`] if this `Error` represents some host error.
    ///
    /// I.e. if this error have variant [`Host`] or [`Trap`][`Trap`] with [host][`TrapKind::Host`] error.
    ///
    /// [`HostError`]: trait.HostError.html
    /// [`Host`]: enum.Error.html#variant.Host
    /// [`Trap`]: enum.Error.html#variant.Trap
    /// [`TrapKind::Host`]: enum.TrapKind.html#variant.Host
    pub fn into_host_error(self) -> Option<Box<dyn HostError>> {
        match self {
            Error::Host(host_error) => Some(host_error),
            Self::Trap(Trap::Host(host_error)) => Some(host_error),
            _ => None,
        }
    }

    /// Returns [`HostError`] if this `Error` represents some host error, otherwise returns the original error.
    ///
    /// I.e. if this error have variant [`Host`] or [`Trap`][`Trap`] with [host][`TrapKind::Host`] error.
    ///
    /// [`HostError`]: trait.HostError.html
    /// [`Host`]: enum.Error.html#variant.Host
    /// [`Trap`]: enum.Error.html#variant.Trap
    /// [`TrapKind::Host`]: enum.TrapKind.html#variant.Host
    pub fn try_into_host_error(self) -> Result<Box<dyn HostError>, Self> {
        match self {
            Error::Host(host_error) => Ok(host_error),
            Self::Trap(Trap::Host(host_error)) => Ok(host_error),
            other => Err(other),
        }
    }
}

impl From<Error> for String {
    fn from(error: Error) -> Self {
        match error {
            Error::Validation(message) => message,
            Error::Instantiation(message) => message,
            Error::Function(message) => message,
            Error::Table(message) => message,
            Error::Memory(message) => message,
            Error::Global(message) => message,
            Error::Value(message) => message,
            Error::Trap(trap) => format!("trap: {trap:?}"),
            Error::Host(error) => format!("user: {error}"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Validation(ref s) => write!(f, "Validation: {}", s),
            Error::Instantiation(ref s) => write!(f, "Instantiation: {}", s),
            Error::Function(ref s) => write!(f, "Function: {}", s),
            Error::Table(ref s) => write!(f, "Table: {}", s),
            Error::Memory(ref s) => write!(f, "Memory: {}", s),
            Error::Global(ref s) => write!(f, "Global: {}", s),
            Error::Value(ref s) => write!(f, "Value: {}", s),
            Error::Trap(ref s) => write!(f, "Trap: {:?}", s),
            Error::Host(ref e) => write!(f, "User: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Validation(ref s) => s,
            Error::Instantiation(ref s) => s,
            Error::Function(ref s) => s,
            Error::Table(ref s) => s,
            Error::Memory(ref s) => s,
            Error::Global(ref s) => s,
            Error::Value(ref s) => s,
            Error::Trap(_) => "Trap",
            Error::Host(_) => "Host error",
        }
    }
}

impl From<Trap> for Error {
    fn from(e: Trap) -> Error {
        Error::Trap(e)
    }
}

impl From<validation::Error> for Error {
    fn from(e: validation::Error) -> Error {
        Error::Validation(e.to_string())
    }
}

mod func;
mod global;
mod host;
mod imports;
mod isa;
mod memory;
mod module;
mod prepare;
mod pwasm;
mod runner;
mod table;
mod types;

pub use self::{
    func::{FuncInstance, FuncInvocation, FuncRef, ResumableError},
    global::{GlobalInstance, GlobalRef},
    host::{Externals, NopExternals, RuntimeArgs},
    imports::{ImportResolver, ImportsBuilder, ModuleImportResolver},
    memory::{MemoryInstance, MemoryRef, LINEAR_MEMORY_PAGE_SIZE},
    module::{ExternVal, ModuleInstance, ModuleRef, NotStartedModuleRef},
    runner::{StackRecycler, DEFAULT_CALL_STACK_LIMIT, DEFAULT_VALUE_STACK_LIMIT},
    table::{TableInstance, TableRef},
    types::{GlobalDescriptor, MemoryDescriptor, Signature, TableDescriptor},
};
#[doc(inline)]
pub use wasmi_core::Value as RuntimeValue;
pub use wasmi_core::{
    memory_units,
    FromValue,
    HostError,
    LittleEndianConvert,
    Trap,
    TrapCode,
    ValueType,
};

/// Mirrors the old value module.
pub(crate) mod value {
    pub use wasmi_core::{
        ArithmeticOps,
        ExtendInto,
        Float,
        FromValue,
        Integer,
        LittleEndianConvert,
        TransmuteInto,
        TryTruncateInto,
        Value as RuntimeValue,
        ValueType,
        WrapInto,
    };
}

/// Floating point types that preserve NaN values.
pub mod nan_preserving_float {
    pub use wasmi_core::{F32, F64};
}

/// Deserialized module prepared for instantiation.
pub struct Module {
    code_map: Vec<isa::Instructions>,
    module: parity_wasm::elements::Module,
}

impl Module {
    /// Create `Module` from `parity_wasm::elements::Module`.
    ///
    /// This function will load, validate and prepare a `parity_wasm`'s `Module`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if provided `Module` is not valid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate parity_wasm;
    /// extern crate wasmi;
    ///
    /// use parity_wasm::builder;
    /// use parity_wasm::elements;
    ///
    /// fn main() {
    ///     let parity_module =
    ///         builder::module()
    ///             .function()
    ///                 .signature().with_param(elements::ValueType::I32).build()
    ///                 .body().build()
    ///             .build()
    ///         .build();
    ///
    ///     let module = wasmi::Module::from_parity_wasm_module(parity_module)
    ///         .expect("parity-wasm builder generated invalid module!");
    ///
    ///     // Instantiate `module`, etc...
    /// }
    /// ```
    pub fn from_parity_wasm_module(module: parity_wasm::elements::Module) -> Result<Module, Error> {
        let prepare::CompiledModule { code_map, module } = prepare::compile_module(module)?;

        Ok(Module { code_map, module })
    }

    /// Fail if the module contains any floating-point operations
    ///
    /// # Errors
    ///
    /// Returns `Err` if provided `Module` is not valid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate wasmi;
    /// # extern crate wat;
    ///
    /// let wasm_binary: Vec<u8> =
    ///     wat::parse_str(
    ///         r#"
    ///         (module
    ///          (func $add (param $lhs i32) (param $rhs i32) (result i32)
    ///                get_local $lhs
    ///                get_local $rhs
    ///                i32.add))
    ///         "#,
    ///     )
    ///     .expect("failed to parse wat");
    ///
    /// // Load wasm binary and prepare it for instantiation.
    /// let module = wasmi::Module::from_buffer(&wasm_binary).expect("Parsing failed");
    /// assert!(module.deny_floating_point().is_ok());
    ///
    /// let wasm_binary: Vec<u8> =
    ///     wat::parse_str(
    ///         r#"
    ///         (module
    ///          (func $add (param $lhs f32) (param $rhs f32) (result f32)
    ///                get_local $lhs
    ///                get_local $rhs
    ///                f32.add))
    ///         "#,
    ///     )
    ///     .expect("failed to parse wat");
    ///
    /// let module = wasmi::Module::from_buffer(&wasm_binary).expect("Parsing failed");
    /// assert!(module.deny_floating_point().is_err());
    ///
    /// let wasm_binary: Vec<u8> =
    ///     wat::parse_str(
    ///         r#"
    ///         (module
    ///          (func $add (param $lhs f32) (param $rhs f32) (result f32)
    ///                get_local $lhs))
    ///         "#,
    ///     )
    ///     .expect("failed to parse wat");
    ///
    /// let module = wasmi::Module::from_buffer(&wasm_binary).expect("Parsing failed");
    /// assert!(module.deny_floating_point().is_err());
    /// ```
    pub fn deny_floating_point(&self) -> Result<(), Error> {
        prepare::deny_floating_point(&self.module).map_err(Into::into)
    }

    /// Create `Module` from a given buffer.
    ///
    /// This function will deserialize wasm module from a given module,
    /// validate and prepare it for instantiation.
    ///
    /// # Errors
    ///
    /// Returns `Err` if wasm binary in provided `buffer` is not valid wasm binary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate wasmi;
    ///
    /// fn main() {
    ///     let module =
    ///         wasmi::Module::from_buffer(
    ///             // Minimal module:
    ///             //   \0asm - magic
    ///             //    0x01 - version (in little-endian)
    ///             &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
    ///         ).expect("Failed to load minimal module");
    ///
    ///     // Instantiate `module`, etc...
    /// }
    /// ```
    pub fn from_buffer<B: AsRef<[u8]>>(buffer: B) -> Result<Module, Error> {
        let module = parity_wasm::elements::deserialize_buffer(buffer.as_ref())
            .map_err(|e: parity_wasm::elements::Error| Error::Validation(e.to_string()))?;
        Module::from_parity_wasm_module(module)
    }

    pub(crate) fn module(&self) -> &parity_wasm::elements::Module {
        &self.module
    }

    pub(crate) fn code(&self) -> &Vec<isa::Instructions> {
        &self.code_map
    }
}
