use parity_wasm::elements::{FunctionType, ValueType as EValueType, GlobalType};

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
	params: Vec<ValueType>,
	return_type: Option<ValueType>,
}

impl Signature {
	pub fn new(params: &[ValueType], return_type: Option<ValueType>) -> Signature {
		Signature {
			params: params.to_vec(),
			return_type: return_type,
		}
	}

	pub fn params(&self) -> &[ValueType] {
		&self.params
	}

	pub fn return_type(&self) -> Option<ValueType> {
		self.return_type
	}

	pub(crate) fn from_elements(func_type: FunctionType) -> Signature {
		Signature {
			params: func_type.params().iter().cloned().map(ValueType::from_elements).collect(),
			return_type: func_type.return_type().map(ValueType::from_elements),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ValueType {
	I32,
	I64,
	F32,
	F64,
}

impl ValueType {
	pub(crate) fn from_elements(value_type: EValueType) -> ValueType {
		match value_type {
			EValueType::I32 => ValueType::I32,
			EValueType::I64 => ValueType::I64,
			EValueType::F32 => ValueType::F32,
			EValueType::F64 => ValueType::F64,
		}
	}

	pub(crate) fn into_elements(self) -> EValueType {
		match self {
			ValueType::I32 => EValueType::I32,
			ValueType::I64 => EValueType::I64,
			ValueType::F32 => EValueType::F32,
			ValueType::F64 => EValueType::F64,
		}
	}
}

pub struct GlobalDescriptor {
	value_type: ValueType,
	mutable: bool,
}

impl GlobalDescriptor {
	pub(crate) fn from_global_type(global_type: &GlobalType) -> GlobalDescriptor {
		GlobalDescriptor {
			value_type: ValueType::from_elements(global_type.content_type()),
			mutable: global_type.is_mutable(),
		}
	}

	pub fn value_type(&self) -> ValueType {
		self.value_type
	}

	pub fn is_mutable(&self) -> bool {
		self.mutable
	}
}