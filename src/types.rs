use parity_wasm::elements::{
	FunctionType, ValueType as EValueType, GlobalType, TableType, MemoryType};

/// Signature of a function.
///
/// Signature of a function consists of zero or more parameter [types][type] and zero or one return [type].
///
/// Two signatures are considered equal if they have equal list of parameters and equal return types.
///
/// [type]: enum.ValueType.html
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Type of a value.
///
/// Wasm code manipulate values of the four basic value types:
/// integers and floating-point (IEEE 754-2008) data of 32 or 64 bit width each, respectively.
///
/// There is no distinction between signed and unsigned integer types. Instead, integers are
/// interpreted by respective operations as either unsigned or signed in two’s complement representation.
///
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
	pub(crate) fn from_elements(global_type: &GlobalType) -> GlobalDescriptor {
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

pub struct TableDescriptor {
	initial: u32,
	maximum: Option<u32>,
}

impl TableDescriptor {
	pub(crate) fn from_elements(table_type: &TableType) -> TableDescriptor {
		TableDescriptor {
			initial: table_type.limits().initial(),
			maximum: table_type.limits().maximum(),
		}
	}

	pub fn initial(&self) -> u32 {
		self.initial
	}

	pub fn maximum(&self) -> Option<u32> {
		self.maximum
	}
}

pub struct MemoryDescriptor {
	initial: u32,
	maximum: Option<u32>,
}

impl MemoryDescriptor {
	pub(crate) fn from_elements(memory_type: &MemoryType) -> MemoryDescriptor {
		MemoryDescriptor {
			initial: memory_type.limits().initial(),
			maximum: memory_type.limits().maximum(),
		}
	}

	pub fn initial(&self) -> u32 {
		self.initial
	}

	pub fn maximum(&self) -> Option<u32> {
		self.maximum
	}
}
