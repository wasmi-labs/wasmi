use std::rc::{Rc, Weak};
use std::fmt;
use std::collections::HashMap;
use std::borrow::Cow;
use parity_wasm::elements::{Local, Opcodes};
use {Error, Signature};
use host::Externals;
use runner::{prepare_function_args, FunctionContext, Interpreter};
use value::RuntimeValue;
use module::ModuleInstance;
use common::stack::StackWithLimit;
use common::{DEFAULT_FRAME_STACK_LIMIT, DEFAULT_VALUE_STACK_LIMIT};

/// Reference to a [`FuncInstance`].
///
/// This reference has a reference-counting semantics.
///
/// [`FuncInstance`]: struct.FuncInstance.html
///
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
/// Functions can call other function (including itself, i.e recursively) and imported functions
/// (i.e defined in another module).
///
/// Functions can be defined either:
///
/// - by a wasm module,
/// - by the host environment and passed to a wasm module as an import.
///   See more in [`Externals`].
///
/// [`Externals`]: trait.Externals.html
///
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
	pub fn alloc_host(signature: Signature, host_func_index: usize) -> FuncRef {
		let func = FuncInstanceInternal::Host {
			signature,
			host_func_index,
		};
		FuncRef(Rc::new(FuncInstance(func)))
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

	pub fn signature(&self) -> &Signature {
		match *self.as_internal() {
			FuncInstanceInternal::Internal { ref signature, .. } => signature,
			FuncInstanceInternal::Host { ref signature, .. } => signature,
		}
	}

	pub(crate) fn body(&self) -> Option<Rc<FuncBody>> {
		match *self.as_internal() {
			FuncInstanceInternal::Internal { ref body, .. } => Some(Rc::clone(body)),
			FuncInstanceInternal::Host { .. } => None,
		}
	}

	pub(crate) fn invoke<E: Externals>(
		func: FuncRef,
		args: Cow<[RuntimeValue]>,
		externals: &mut E,
	) -> Result<Option<RuntimeValue>, Error> {
		enum InvokeKind<'a> {
			Internal(FunctionContext),
			Host(usize, &'a [RuntimeValue]),
		}

		let result = match *func.as_internal() {
			FuncInstanceInternal::Internal { ref signature, .. } => {
				let mut stack =
					StackWithLimit::with_data(args.into_iter().cloned(), DEFAULT_VALUE_STACK_LIMIT);
				let args = prepare_function_args(signature, &mut stack)?;
				let context = FunctionContext::new(
					func.clone(),
					DEFAULT_VALUE_STACK_LIMIT,
					DEFAULT_FRAME_STACK_LIMIT,
					signature,
					args,
				);
				InvokeKind::Internal(context)
			}
			FuncInstanceInternal::Host { ref host_func_index, .. } => {
				InvokeKind::Host(*host_func_index, &*args)
			}
		};

		match result {
			InvokeKind::Internal(ctx) => {
				let mut interpreter = Interpreter::new(externals);
				interpreter.run_function(ctx)
			}
			InvokeKind::Host(host_func, args) => externals.invoke_index(host_func, args.into()),
		}
	}
}

#[derive(Clone, Debug)]
pub struct FuncBody {
	pub locals: Vec<Local>,
	pub opcodes: Opcodes,
	pub labels: HashMap<usize, usize>,
}
