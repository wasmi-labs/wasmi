use std::any::TypeId;
use value::RuntimeValue;
use Error;

/// Custom user error.
pub trait HostError: 'static + ::std::fmt::Display + ::std::fmt::Debug {
	#[doc(hidden)]
	fn __private_get_type_id__(&self) -> TypeId {
		TypeId::of::<Self>()
	}
}

impl HostError {
	/// Attempt to downcast this `HostError` to a concrete type by reference.
	pub fn downcast_ref<T: HostError>(&self) -> Option<&T> {
		if self.__private_get_type_id__() == TypeId::of::<T>() {
			unsafe { Some(&*(self as *const HostError as *const T)) }
		} else {
			None
		}
	}

	/// Attempt to downcast this `HostError` to a concrete type by mutable
	/// reference.
	pub fn downcast_mut<T: HostError>(&mut self) -> Option<&mut T> {
		if self.__private_get_type_id__() == TypeId::of::<T>() {
			unsafe { Some(&mut *(self as *mut HostError as *mut T)) }
		} else {
			None
		}
	}
}

pub trait Externals {
	fn invoke_index(
		&mut self,
		index: usize,
		args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error>;
}

pub struct NopExternals;

impl Externals for NopExternals {
	fn invoke_index(
		&mut self,
		_index: usize,
		_args: &[RuntimeValue],
	) -> Result<Option<RuntimeValue>, Error> {
		Err(Error::Trap("invoke index on no-op externals".into()))
	}
}
