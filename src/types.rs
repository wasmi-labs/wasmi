use parity_wasm::elements::{FunctionType, ValueType};

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
