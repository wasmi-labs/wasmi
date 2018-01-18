use std::rc::Rc;
use std::cell::Cell;
use value::RuntimeValue;
use Error;
use types::ValueType;
use parity_wasm::elements::{ValueType as EValueType};

#[derive(Clone, Debug)]
pub struct GlobalRef(Rc<GlobalInstance>);

impl ::std::ops::Deref for GlobalRef {
	type Target = GlobalInstance;
	fn deref(&self) -> &GlobalInstance {
		&self.0
	}
}

#[derive(Debug)]
pub struct GlobalInstance {
	val: Cell<RuntimeValue>,
	mutable: bool,
}

impl GlobalInstance {

	pub fn alloc(val: RuntimeValue, mutable: bool) -> GlobalRef {
		let global = GlobalInstance::new(val, mutable);
		GlobalRef(Rc::new(global))
	}

	fn new(val: RuntimeValue, mutable: bool) -> GlobalInstance {
		GlobalInstance {
			val: Cell::new(val),
			mutable,
		}
	}

	pub fn set(&self, val: RuntimeValue) -> Result<(), Error> {
		if !self.mutable {
			return Err(Error::Global("Attempt to change an immutable variable".into()));
		}
		if self.value_type() != val.value_type() {
			return Err(Error::Global("Attempt to change variable type".into()));
		}
		self.val.set(val);
		Ok(())
	}

	pub fn get(&self) -> RuntimeValue {
		self.val.get()
	}

	pub fn is_mutable(&self) -> bool {
		self.mutable
	}

	pub fn value_type(&self) -> ValueType {
		self.val.get().value_type()
	}

	pub(crate) fn elements_value_type(&self) -> EValueType {
		self.value_type().into_elements()
	}
}
