use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;
use std::borrow::Cow;
use parity_wasm::elements::{FunctionType, Local, Opcodes};
use Error;
use host::Externals;
use runner::{prepare_function_args, FunctionContext, Interpreter};
use value::RuntimeValue;
use module::ModuleRef;
use common::stack::StackWithLimit;
use common::{DEFAULT_FRAME_STACK_LIMIT, DEFAULT_VALUE_STACK_LIMIT};

#[derive(Clone, Debug)]
pub struct FuncRef(Rc<FuncInstance>);

impl ::std::ops::Deref for FuncRef {
	type Target = FuncInstance;
	fn deref(&self) -> &FuncInstance {
		&self.0
	}
}

#[derive(Clone)]
pub enum FuncInstance {
	Internal {
		func_type: Rc<FunctionType>,
		module: ModuleRef,
		body: Rc<FuncBody>,
	},
	Host {
		func_type: FunctionType,
		host_func_index: usize,
	},
}

impl fmt::Debug for FuncInstance {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&FuncInstance::Internal {
				ref func_type,
				ref module,
				..
			} => {
				write!(
					f,
					"Internal {{ type={:?}, module={:?} }}",
					func_type,
					module
				)
			}
			&FuncInstance::Host { ref func_type, .. } => {
				write!(f, "Host {{ type={:?} }}", func_type)
			}
		}
	}
}

impl FuncInstance {
	pub(crate) fn alloc_internal(
		module: ModuleRef,
		func_type: Rc<FunctionType>,
		body: FuncBody,
	) -> FuncRef {
		let func = FuncInstance::Internal {
			func_type,
			module: module,
			body: Rc::new(body),
		};
		FuncRef(Rc::new(func))
	}

	pub fn alloc_host(func_type: FunctionType, host_func_index: usize) -> FuncRef {
		let func = FuncInstance::Host {
			func_type,
			host_func_index,
		};
		FuncRef(Rc::new(func))
	}

	pub fn func_type(&self) -> &FunctionType {
		match *self {
			FuncInstance::Internal { ref func_type, .. } => func_type,
			FuncInstance::Host { ref func_type, .. } => func_type,
		}
	}

	pub(crate) fn body(&self) -> Option<Rc<FuncBody>> {
		match *self {
			FuncInstance::Internal { ref body, .. } => Some(Rc::clone(body)),
			FuncInstance::Host { .. } => None,
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

		let result = match *func {
			FuncInstance::Internal { ref func_type, .. } => {
				let mut stack =
					StackWithLimit::with_data(args.into_iter().cloned(), DEFAULT_VALUE_STACK_LIMIT);
				let args = prepare_function_args(func_type, &mut stack)?;
				let context = FunctionContext::new(
					func.clone(),
					DEFAULT_VALUE_STACK_LIMIT,
					DEFAULT_FRAME_STACK_LIMIT,
					func_type,
					args,
				);
				InvokeKind::Internal(context)
			}
			FuncInstance::Host { ref host_func_index, .. } => {
				InvokeKind::Host(*host_func_index, &*args)
			}
		};

		match result {
			InvokeKind::Internal(ctx) => {
				let mut interpreter = Interpreter::new(externals);
				interpreter.run_function(ctx)
			}
			InvokeKind::Host(host_func, args) => externals.invoke_index(host_func, args),
		}
	}
}

#[derive(Clone, Debug)]
pub struct FuncBody {
	pub locals: Vec<Local>,
	pub opcodes: Opcodes,
	pub labels: HashMap<usize, usize>,
}
