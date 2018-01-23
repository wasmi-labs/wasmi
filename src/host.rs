use std::any::TypeId;
use value::RuntimeValue;
use Error;

pub struct RuntimeArgs<'a>(&'a [RuntimeValue]);

impl<'a> From<&'a [RuntimeValue]> for RuntimeArgs<'a> {
	fn from(inner: &'a [RuntimeValue]) -> Self {
		RuntimeArgs(inner)
	}
}

impl<'a> RuntimeArgs<'a> {
	fn nth<T>(&self, idx: usize) -> Result<T, Error> where RuntimeValue: From<T> {
		Err(Error::Value("Invalid cast".to_owned()))
	}
}

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

#[cfg(test)]
mod tests {

	use value::RuntimeValue;
	use super::RuntimeArgs;

	#[test]
	fn i32_runtime_args() {
		let args: RuntimeArgs = (&[RuntimeValue::I32(0)][..]).into();
		let val: i32 = args.nth(0).unwrap();
		assert_eq!(val, 0);
	}

}