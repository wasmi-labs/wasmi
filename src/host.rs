use crate::{value::FromValue, RuntimeValue, Error};
use core::marker::PhantomData;
use wasmi_core::{CanResume, TrapCode};

/// Wrapper around slice of [`Value`] for using it
/// as an argument list conveniently.
///
/// [`Value`]: enum.Value.html
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
    pub fn nth_checked<T>(&self, idx: usize) -> Result<T, TrapCode>
    where
        T: FromValue,
    {
        self.nth_value_checked(idx)?
            .try_into()
            .ok_or(TrapCode::UnexpectedSignature)
    }

    /// Extract argument as a [`Value`] by index `idx`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if this list has not enough arguments.
    ///
    /// [`Value`]: enum.Value.html
    pub fn nth_value_checked(&self, idx: usize) -> Result<RuntimeValue, TrapCode> {
        if self.0.len() <= idx {
            return Err(TrapCode::UnexpectedSignature.into());
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
        T: FromValue,
    {
        let value = self.nth_value_checked(idx).expect("Invalid argument index");
        value.try_into().expect("Unexpected argument type")
    }

    /// Total number of arguments
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Trait that allows to implement host functions.
///
/// # Examples
///
/// ```rust
/// use wasmi::{
///     Externals, RuntimeValue, RuntimeArgs, Error, ModuleImportResolver,
///     FuncRef, ValueType, Signature, FuncInstance,
/// };
///
/// struct HostExternals {
///     counter: usize,
/// }
///
/// const ADD_FUNC_INDEX: usize = 0;
///
/// impl Externals for HostExternals {
///     type Error = wasmi::Error;
///     fn invoke_index(
///         &mut self,
///         index: usize,
///         args: RuntimeArgs,
///     ) -> Result<Option<RuntimeValue>, Self::Error> {
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
    /// Type of error the host can yield while executing a host function.
    type Error: CanResume + From<Error>;
    /// Perform invoke of a host function by specified `index`.
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Self::Error>;
}

/// Implementation of [`Externals`] that just traps on [`invoke_index`].
///
/// [`Externals`]: trait.Externals.html
/// [`invoke_index`]: trait.Externals.html#tymethod.invoke_index
pub struct NopExternals<E>(pub PhantomData<E>);

impl<E> Externals for NopExternals<E>
where
    E: CanResume + From<Error>,
{
    type Error = E;
    fn invoke_index(
        &mut self,
        _index: usize,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Self::Error> {
        Err(Error::Trap(TrapCode::Unreachable).into())
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeArgs;
    use crate::RuntimeValue;

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
}
