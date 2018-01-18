//! WebAssembly interpreter module.

// TODO(pepyakin): Fix these asap
#![allow(missing_docs)]

#[cfg(test)]
extern crate wabt;
extern crate parity_wasm;
extern crate byteorder;

use std::fmt;
use std::error;
use parity_wasm::elements::{FunctionType, ValueType};

/// Internal interpreter error.
#[derive(Debug)]
pub enum Error {
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

impl From<::common::stack::Error> for Error {
	fn from(e: ::common::stack::Error) -> Self {
		Error::Stack(e.to_string())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
	func_type: FunctionType,
}

impl Signature {
	pub fn new(params: &[ValueType], return_type: Option<ValueType>) -> Signature {
		Signature {
			func_type: FunctionType::new(params.to_vec(), return_type),
		}
	}

	pub fn params(&self) -> &[ValueType] {
		self.func_type.params()
	}

	pub fn return_type(&self) -> Option<ValueType> {
		self.func_type.return_type()
	}
}

impl From<FunctionType> for Signature {
	fn from(func_type: FunctionType) -> Signature {
		Signature { func_type }
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

#[cfg(test)]
mod tests;

pub use self::memory::{MemoryInstance, MemoryRef};
pub use self::table::{TableInstance, TableRef};
pub use self::value::{RuntimeValue, TryInto};
pub use self::host::{Externals, NopExternals, HostError};
pub use self::imports::{ModuleImportResolver, ImportResolver, ImportsBuilder};
pub use self::module::{ModuleInstance, ModuleRef, ExternVal, NotStartedModuleRef};
pub use self::global::{GlobalInstance, GlobalRef};
pub use self::func::{FuncInstance, FuncRef};
