use crate::value::{FromRuntimeValue, RuntimeValue};
use crate::{Trap, TrapKind};

use downcast_rs::{impl_downcast, DowncastSync};

/// Wrapper around slice of [`RuntimeValue`] for using it
/// as an argument list conveniently.
///
/// [`RuntimeValue`]: enum.RuntimeValue.html
#[derive(Debug)]
pub struct RuntimeArgs<'a>(&'a [RuntimeValue]);

impl<'a> From<&'a [RuntimeValue]> for RuntimeArgs<'a> {
    fn from(inner: &'a [RuntimeValue]) -> Self {
        RuntimeArgs(inner)
    }
}

impl<'a> AsRef<[RuntimeValue]> for RuntimeArgs<'a> {
    fn as_ref(&self) -> &[RuntimeValue] {
        self.0
    }
}

impl<'a> RuntimeArgs<'a> {
    /// Extract argument by index `idx`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if cast is invalid or not enough arguments.
    pub fn nth_checked<T>(&self, idx: usize) -> Result<T, Trap>
    where
        T: FromRuntimeValue,
    {
        self.nth_value_checked(idx)?
            .try_into()
            .ok_or(TrapKind::UnexpectedSignature)
            .map_err(Into::into)
    }

    /// Extract argument as a [`RuntimeValue`] by index `idx`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if this list has not enough arguments.
    ///
    /// [`RuntimeValue`]: enum.RuntimeValue.html
    pub fn nth_value_checked(&self, idx: usize) -> Result<RuntimeValue, Trap> {
        if self.0.len() <= idx {
            return Err(TrapKind::UnexpectedSignature.into());
        }
        Ok(self.0[idx])
    }

    /// Extract argument by index `idx`.
    ///
    /// # Panics
    ///
    /// Panics if cast is invalid or not enough arguments.
    pub fn nth<T>(&self, idx: usize) -> T
    where
        T: FromRuntimeValue,
    {
        let value = self.nth_value_checked(idx).expect("Invalid argument index");
        value.try_into().expect("Unexpected argument type")
    }

    /// Total number of arguments
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Trait that allows the host to return custom error.
///
/// It should be useful for representing custom traps,
/// troubles at instantiation time or other host specific conditions.
///
/// Types that implement this trait can automatically be converted to `wasmi::Error` and `wasmi::Trap`
/// and will be represented as a boxed `HostError`. You can then use the various methods on `wasmi::Error`
/// to get your custom error type back
///
/// # Examples
///
/// ```rust
/// use std::fmt;
/// use wasmi::{Error, HostError};
///
/// #[derive(Debug)]
/// struct MyError {
///     code: u32,
/// }
///
/// impl fmt::Display for MyError {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         write!(f, "MyError, code={}", self.code)
///     }
/// }
///
/// impl HostError for MyError { }
///
/// fn failable_fn() -> Result<(), Error> {
///     let my_error = MyError { code: 1312 };
///     // Note how you can just convert your errors to `wasmi::Error`
///     Err(my_error.into())
/// }
///
/// // Get a reference to the concrete error
/// match failable_fn() {
///     Err(Error::Host(host_error)) => {
///         let my_error: &MyError = host_error.downcast_ref::<MyError>().unwrap();
///         assert_eq!(my_error.code, 1312);
///     }
///     _ => panic!(),
/// }
///
/// // get the concrete error itself
/// match failable_fn() {
///     Err(err) => {
///         let my_error: Box<MyError> = err.try_into_host_error().unwrap().downcast::<MyError>().unwrap();
///         assert_eq!(my_error.code, 1312);
///     }
///     _ => panic!(),
/// }
///
/// ```
pub trait HostError: 'static + ::core::fmt::Display + ::core::fmt::Debug + DowncastSync {}
impl_downcast!(HostError);

/// Trait that allows to implement host functions.
///
/// # Examples
///
/// ```rust
/// use wasmi::{
///     Externals, RuntimeValue, RuntimeArgs, Error, ModuleImportResolver,
///     FuncRef, ValueType, Signature, FuncInstance, Trap,
/// };
///
/// struct HostExternals {
///     counter: usize,
/// }
///
/// const ADD_FUNC_INDEX: usize = 0;
///
/// impl Externals for HostExternals {
///     fn invoke_index(
///         &mut self,
///         index: usize,
///         args: RuntimeArgs,
///     ) -> Result<Option<RuntimeValue>, Trap> {
///         match index {
///             ADD_FUNC_INDEX => {
///                 let a: u32 = args.nth_checked(0)?;
///                 let b: u32 = args.nth_checked(1)?;
///                 let result = a + b;
///
///                 Ok(Some(RuntimeValue::I32(result as i32)))
///             }
///             _ => panic!("Unimplemented function at {}", index),
///         }
///     }
/// }
///
/// impl HostExternals {
///     fn check_signature(
///         &self,
///         index: usize,
///         signature: &Signature
///     ) -> bool {
///         let (params, ret_ty): (&[ValueType], Option<ValueType>) = match index {
///             ADD_FUNC_INDEX => (&[ValueType::I32, ValueType::I32], Some(ValueType::I32)),
///             _ => return false,
///         };
///         signature.params() == params && signature.return_type() == ret_ty
///     }
/// }
///
/// impl ModuleImportResolver for HostExternals {
///     fn resolve_func(
///         &self,
///         field_name: &str,
///         signature: &Signature
///     ) -> Result<FuncRef, Error> {
///         let index = match field_name {
///             "add" => ADD_FUNC_INDEX,
///             _ => {
///                 return Err(Error::Instantiation(
///                     format!("Export {} not found", field_name),
///                 ))
///             }
///         };
///
///         if !self.check_signature(index, signature) {
///             return Err(Error::Instantiation(
///                 format!("Export {} has a bad signature", field_name)
///             ));
///         }
///
///         Ok(FuncInstance::alloc_host(
///             Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
///             index,
///         ))
///     }
/// }
/// ```
pub trait Externals {
    /// Perform invoke of a host function by specified `index`.
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap>;
}

/// Implementation of [`Externals`] that just traps on [`invoke_index`].
///
/// [`Externals`]: trait.Externals.html
/// [`invoke_index`]: trait.Externals.html#tymethod.invoke_index
pub struct NopExternals;

impl Externals for NopExternals {
    fn invoke_index(
        &mut self,
        _index: usize,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        Err(TrapKind::Unreachable.into())
    }
}

#[cfg(test)]
mod tests {

    use super::{HostError, RuntimeArgs};
    use crate::value::RuntimeValue;

    #[test]
    fn i32_runtime_args() {
        let args: RuntimeArgs = (&[RuntimeValue::I32(0)][..]).into();
        let val: i32 = args.nth_checked(0).unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn i64_invalid_arg_cast() {
        let args: RuntimeArgs = (&[RuntimeValue::I64(90534534545322)][..]).into();
        assert!(args.nth_checked::<i32>(0).is_err());
    }

    // Tests that `HostError` trait is object safe.
    fn _host_error_is_object_safe(_: &dyn HostError) {}
}
