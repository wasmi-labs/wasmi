use std::rc::{Rc, Weak};
use std::fmt;
use std::collections::HashMap;
use parity_wasm::elements::{Local, Opcodes};
use {Trap, TrapKind, Signature};
use host::Externals;
use runner::{check_function_args, Interpreter};
use value::RuntimeValue;
use module::ModuleInstance;

/// Reference to a function (See [`FuncInstance`] for details).
///
/// This reference has a reference-counting semantics.
///
/// [`FuncInstance`]: struct.FuncInstance.html
#[derive(Clone, Debug)]
pub struct FuncRef(Rc<FuncInstance>);

impl ::std::ops::Deref for FuncRef {
	type Target = FuncInstance;
	fn deref(&self) -> &FuncInstance {
		&self.0
	}
}

/// Runtime representation of a function.
///
/// Functions are the unit of orgianization of code in WebAssembly. Each function takes a sequence of values
/// as parameters and either optionally return a value or trap.
/// Functions can call other function including itself (i.e recursive calls are allowed) and imported functions
/// (i.e functions defined in another module or by the host environment).
///
/// Functions can be defined either:
///
/// - by a wasm module,
/// - by the host environment and passed to a wasm module as an import.
///   See more in [`Externals`].
///
/// [`Externals`]: trait.Externals.html
pub struct FuncInstance(FuncInstanceInternal);

#[derive(Clone)]
pub(crate) enum FuncInstanceInternal {
	Internal {
		signature: Rc<Signature>,
		module: Weak<ModuleInstance>,
		body: Rc<FuncBody>,
	},
	Host {
		signature: Signature,
		host_func_index: usize,
	},
}

impl fmt::Debug for FuncInstance {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.as_internal() {
			&FuncInstanceInternal::Internal {
				ref signature,
				..
			} => {
				// We can't write description of self.module here, because it generate
				// debug string for function instances and this will lead to infinite loop.
				write!(
					f,
					"Internal {{ signature={:?} }}",
					signature,
				)
			}
			&FuncInstanceInternal::Host { ref signature, .. } => {
				write!(f, "Host {{ signature={:?} }}", signature)
			}
		}
	}
}

impl FuncInstance {
	/// Allocate a function instance for a host function.
	///
	/// When this function instance will be called by the wasm code,
	/// the instance of [`Externals`] will be invoked by calling `invoke_index`
	/// with specified `host_func_index` here.
	/// This call will be made with the `signature` provided here.
	///
	/// [`Externals`]: trait.Externals.html
	pub fn alloc_host(signature: Signature, host_func_index: usize) -> FuncRef {
		let func = FuncInstanceInternal::Host {
			signature,
			host_func_index,
		};
		FuncRef(Rc::new(FuncInstance(func)))
	}

	/// Returns [signature] of this function instance.
	///
	/// This function instance can only be called with matching signatures.
	///
	/// [signature]: struct.Signature.html
	pub fn signature(&self) -> &Signature {
		match *self.as_internal() {
			FuncInstanceInternal::Internal { ref signature, .. } => signature,
			FuncInstanceInternal::Host { ref signature, .. } => signature,
		}
	}

	pub(crate) fn as_internal(&self) -> &FuncInstanceInternal {
		&self.0
	}

	pub(crate) fn alloc_internal(
		module: Weak<ModuleInstance>,
		signature: Rc<Signature>,
		body: FuncBody,
	) -> FuncRef {
		let func = FuncInstanceInternal::Internal {
			signature,
			module: module,
			body: Rc::new(body),
		};
		FuncRef(Rc::new(FuncInstance(func)))
	}

	pub(crate) fn body(&self) -> Option<Rc<FuncBody>> {
		match *self.as_internal() {
			FuncInstanceInternal::Internal { ref body, .. } => Some(Rc::clone(body)),
			FuncInstanceInternal::Host { .. } => None,
		}
	}

	/// Invoke this function.
	///
	/// # Errors
	///
	/// Returns `Err` if `args` types is not match function [`signature`] or
	/// if [`Trap`] at execution time occured.
	///
	/// [`signature`]: #method.signature
	/// [`Trap`]: #enum.Trap.html
	pub fn invoke<E: Externals>(
		func: &FuncRef,
		args: &[RuntimeValue],
		externals: &mut E,
	) -> Result<Option<RuntimeValue>, Trap> {
		check_function_args(func.signature(), &args).map_err(|_| TrapKind::UnexpectedSignature)?;
		match *func.as_internal() {
			FuncInstanceInternal::Internal { .. } => {
				let mut interpreter = Interpreter::new(externals);
				interpreter.start_execution(func, args)
			}
			FuncInstanceInternal::Host {
				ref host_func_index,
				..
			} => externals.invoke_index(*host_func_index, args.into()),
		}
	}
}

#[derive(Clone, Debug)]
pub struct FuncBody {
	pub locals: Vec<Local>,
	pub opcodes: Opcodes,
	pub labels: HashMap<usize, usize>,
}
