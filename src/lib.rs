//! WebAssembly interpreter module.

// TODO(pepyakin): Fix these asap
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
pub use self::host::{Externals, NopExternals, HostError};
pub use self::imports::{ModuleImportResolver, ImportResolver, ImportsBuilder};
pub use self::module::{ModuleInstance, ModuleRef, ExternVal, NotStartedModuleRef};
pub use self::global::{GlobalInstance, GlobalRef};
pub use self::func::{FuncRef};
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
