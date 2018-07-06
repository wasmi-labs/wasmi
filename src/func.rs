use std::rc::{Rc, Weak};
use std::fmt;
use parity_wasm::elements::Local;
use {Trap, TrapKind, Signature};
use host::Externals;
use runner::{check_function_args, Interpreter, InterpreterState};
use value::RuntimeValue;
use types::ValueType;
use module::ModuleInstance;
use isa;

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
				let mut interpreter = Interpreter::new(func, args, externals)?;
				interpreter.start_execution()
			}
			FuncInstanceInternal::Host {
				ref host_func_index,
				..
			} => externals.invoke_index(*host_func_index, args.into()),
		}
	}

	/// Invoke the function, get a resumable handle.  This handle can then be used to actually start the execution. If a
	/// Host trap happens, caller can use `resume_execution` to feed the expected return value back in, and then
	/// continue the execution.
	///
	/// # Errors
	///
	/// Returns `Err` if `args` types is not match function [`signature`].
	///
	/// [`signature`]: #method.signature
	/// [`Trap`]: #enum.Trap.html
	pub fn invoke_resumable<'args, 'externals, E: Externals + 'externals>(
		func: &FuncRef,
		args: &'args [RuntimeValue],
		externals: &'externals mut E,
	) -> Result<FuncInvocation<'args, 'externals, E>, Trap> {
		check_function_args(func.signature(), &args).map_err(|_| TrapKind::UnexpectedSignature)?;
		match *func.as_internal() {
			FuncInstanceInternal::Internal { .. } => {
				let interpreter = Interpreter::new(func, args, externals)?;
				Ok(FuncInvocation {
					kind: FuncInvocationKind::Internal(interpreter),
				})
			}
			FuncInstanceInternal::Host {
				ref host_func_index,
				..
			} => {
				Ok(FuncInvocation {
					kind: FuncInvocationKind::Host {
						args, externals,
						host_func_index: *host_func_index,
						finished: false,
					},
				})
			},
		}
	}
}

/// A resumable invocation error.
pub enum ResumableError {
	/// Trap happened.
	Trap(Trap),
	/// The invocation is not resumable.
	NotResumable,
	/// The invocation has already been started.
	AlreadyStarted,
}

impl From<Trap> for ResumableError {
	fn from(trap: Trap) -> Self {
		ResumableError::Trap(trap)
	}
}

/// A resumable invocation handle. This struct is returned by `FuncInstance::invoke_resumable`.
pub struct FuncInvocation<'args, 'externals, E: Externals + 'externals> {
	kind: FuncInvocationKind<'args, 'externals, E>,
}

enum FuncInvocationKind<'args, 'externals, E: Externals + 'externals> {
	Internal(Interpreter<'externals, E>),
	Host {
		args: &'args [RuntimeValue],
		externals: &'externals mut E,
		host_func_index: usize,
		finished: bool
	},
}

impl<'args, 'externals, E: Externals + 'externals> FuncInvocation<'args, 'externals, E> {
	/// Whether this invocation is currently resumable.
	pub fn is_resumable(&self) -> bool {
		match &self.kind {
			&FuncInvocationKind::Internal(ref interpreter) => interpreter.state().is_resumable(),
			&FuncInvocationKind::Host { .. } => false,
		}
	}

	/// If the invocation is resumable, the expected return value type to be feed back in.
	pub fn resumable_value_type(&self) -> Option<ValueType> {
		match &self.kind {
			&FuncInvocationKind::Internal(ref interpreter) => {
				match interpreter.state() {
					&InterpreterState::Resumable(ref value_type) => value_type.clone(),
					_ => None,
				}
			},
			&FuncInvocationKind::Host { .. } => None,
		}
	}

	/// Start the invocation execution.
	pub fn start_execution(&mut self) -> Result<Option<RuntimeValue>, ResumableError> {
		match self.kind {
			FuncInvocationKind::Internal(ref mut interpreter) => {
				if interpreter.state() != &InterpreterState::Initialized {
					return Err(ResumableError::AlreadyStarted);
				}
				Ok(interpreter.start_execution()?)
			},
			FuncInvocationKind::Host { ref args, ref mut externals, ref mut finished, ref host_func_index } => {
				if *finished {
					return Err(ResumableError::AlreadyStarted);
				}
				*finished = true;
				Ok(externals.invoke_index(*host_func_index, args.clone().into())?)
			},
		}
	}

	/// Resume an execution if a previous trap of Host kind happened.
	pub fn resume_execution(&mut self, return_val: Option<RuntimeValue>) -> Result<Option<RuntimeValue>, ResumableError> {
		match self.kind {
			FuncInvocationKind::Internal(ref mut interpreter) => {
				if !interpreter.state().is_resumable() {
					return Err(ResumableError::AlreadyStarted);
				}
				Ok(interpreter.resume_execution(return_val)?)
			},
			FuncInvocationKind::Host { .. } => {
				return Err(ResumableError::NotResumable);
			},
		}
	}
}

#[derive(Clone, Debug)]
pub struct FuncBody {
	pub locals: Vec<Local>,
	pub code: isa::Instructions,
}
