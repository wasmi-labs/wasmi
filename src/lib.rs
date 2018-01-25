//! # wasmi
//!
//! This library allows to load WebAssembly modules in binary format and invoke functions on them.
//!
//! # Introduction
//!
//! WebAssembly (wasm) is a safe, portable, compact format that designed for efficient execution.
//!
//! Wasm code is distributed in a form of modules, that contains definitions of:
//!
//! - functions,
//! - global variables,
//! - linear memories,
//! - tables.
//!
//! and this definitions can be imported. Also, each definition can be exported.
//!
//! In addition to definitions, modules can define initialization data for their memories or tables that takes the
//! form of segments copied to given offsets. They can also define a `start` function that is automatically executed.
//!
//! ## Loading and Validation
//!
//! Before execution a module should be validated. This process checks that module is well-formed
//! and makes only allowed operations.
//!
//! Valid modules can't access memory out of it's sandbox, can't cause stack underflow
//! and can call functions only with correct signatures.
//!
//! ## Instantiatiation
//!
//! In order to execute code in wasm module it should be instatiated.
//! Instantiation includes the following steps:
//!
//! 1. Create an empty module instance,
//! 2. Resolve definition instances for each declared import in the module,
//! 3. Instantiate definitions declared in the module (e.g. allocate global variables, allocate linear memory, etc),
//! 4. Initialize memory and table contents by copiying segments into them,
//! 5. Execute `start` function, if any.
//!
//! After these steps, module instance are ready to execute functions.
//!
//! ## Execution
//!
//! It is allowed to only execute functions which are exported by a module.
//! Functions can either return a result or trap (e.g. there can't be linking-error at the middle of execution).
//! This property is ensured by the validation process.
//!
//! # Examples
//!
//! ```rust
//! extern crate wasmi;
//! extern crate wabt;
//!
//! use wasmi::{ModuleInstance, ImportsBuilder, NopExternals, RuntimeValue};
//!
//! fn main() {
//!     // Parse WAT (WebAssembly Text format) into wasm bytecode.
//!     let wasm_binary: Vec<u8> =
//!         wabt::wat2wasm(
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
//!     let module = wasmi::load_from_buffer(&wasm_binary)
//!         .expect("failed to load wasm");
//!
//!     // Instantiate a module with empty imports and
//!     // asserting that there is no `start` function.
//!     let instance =
//!         ModuleInstance::new(
//!             &module,
//!             &ImportsBuilder::default()
//!         )
//!         .expect("failed to instantiate wasm module")
//!         .assert_no_start();
//!
//!     // Finally, invoke exported function "test" with no parameters
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

// TODO(pepyakin): Fix this asap https://github.com/pepyakin/wasmi/issues/3
#![allow(missing_docs)]

#[cfg(test)]
extern crate wabt;
extern crate parity_wasm;
extern crate byteorder;

use std::fmt;
use std::error;
use std::collections::HashMap;
use parity_wasm::elements::Module;

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
	/// Stack-level error.
	Stack(String),
	/// Value-level error.
	Value(String),
	/// Trap.
	Trap(String),
	/// Custom embedder error.
	Host(Box<host::HostError>),
}

impl Into<String> for Error {
	fn into(self) -> String {
		match self {
			Error::Validation(s) => s,
			Error::Instantiation(s) => s,
			Error::Function(s) => s,
			Error::Table(s) => s,
			Error::Memory(s) => s,
			Error::Global(s) => s,
			Error::Stack(s) => s,
			Error::Value(s) => s,
			Error::Trap(s) => format!("trap: {}", s),
			Error::Host(e) => format!("user: {}", e),
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
			Error::Stack(ref s) => write!(f, "Stack: {}", s),
			Error::Value(ref s) => write!(f, "Value: {}", s),
			Error::Trap(ref s) => write!(f, "Trap: {}", s),
			Error::Host(ref e) => write!(f, "User: {}", e),
		}
	}
}



impl error::Error for Error {
	fn description(&self) -> &str {
		match *self {
			Error::Validation(ref s) => s,
			Error::Instantiation(ref s) => s,
			Error::Function(ref s) => s,
			Error::Table(ref s) => s,
			Error::Memory(ref s) => s,
			Error::Global(ref s) => s,
			Error::Stack(ref s) => s,
			Error::Value(ref s) => s,
			Error::Trap(ref s) => s,
			Error::Host(_) => "Host error",
		}
	}
}


impl<U> From<U> for Error where U: host::HostError + Sized {
	fn from(e: U) -> Self {
		Error::Host(Box::new(e))
	}
}

impl From<validation::Error> for Error {
	fn from(e: validation::Error) -> Error {
		Error::Validation(e.to_string())
	}
}

impl From<::common::stack::Error> for Error {
	fn from(e: ::common::stack::Error) -> Self {
		Error::Stack(e.to_string())
	}
}

mod validation;
mod common;
mod memory;
mod module;
mod runner;
mod table;
mod value;
mod host;
mod imports;
mod global;
mod func;
mod types;

#[cfg(test)]
mod tests;

pub use self::memory::{MemoryInstance, MemoryRef};
pub use self::table::{TableInstance, TableRef};
pub use self::value::{RuntimeValue, TryInto};
pub use self::host::{Externals, NopExternals, HostError, RuntimeArgs};
pub use self::imports::{ModuleImportResolver, ImportResolver, ImportsBuilder};
pub use self::module::{ModuleInstance, ModuleRef, ExternVal, NotStartedModuleRef};
pub use self::global::{GlobalInstance, GlobalRef};
pub use self::func::{FuncInstance, FuncRef};
pub use self::types::{Signature, ValueType, GlobalDescriptor, TableDescriptor, MemoryDescriptor};

pub struct LoadedModule {
	labels: HashMap<usize, HashMap<usize, usize>>,
	module: Module,
}

impl LoadedModule {
	pub(crate) fn module(&self) -> &Module {
		&self.module
	}

	pub(crate) fn labels(&self) -> &HashMap<usize, HashMap<usize, usize>> {
		&self.labels
	}

	pub fn into_module(self) -> Module {
		self.module
	}
}

pub fn load_from_module(module: Module) -> Result<LoadedModule, Error> {
	use validation::{validate_module, ValidatedModule};
	let ValidatedModule {
		labels,
		module,
	} = validate_module(module)?;

	Ok(LoadedModule {
		labels,
		module,
	})
}

pub fn load_from_buffer<B: AsRef<[u8]>>(buffer: B) -> Result<LoadedModule, Error> {
	let module = parity_wasm::elements::deserialize_buffer(buffer.as_ref())
		.map_err(|e: parity_wasm::elements::Error| Error::Validation(e.to_string()))?;
	load_from_module(module)
}
