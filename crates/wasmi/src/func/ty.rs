use crate::{
    core::{CoreFuncType, ValType},
    errors::FuncError,
    Val,
};

/// Types that are dynamically typed, such as [`ValType`].
pub trait DynamicallyTyped {
    /// Returns the [`ValType`] of `self`.
    fn ty(&self) -> ValType;
}

impl DynamicallyTyped for ValType {
    fn ty(&self) -> ValType {
        *self
    }
}

impl DynamicallyTyped for Val {
    fn ty(&self) -> ValType {
        self.ty()
    }
}

/// A Wasm function descriptor.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncType {
    core: CoreFuncType,
}

impl FuncType {
    /// Creates a new [`FuncType`].
    ///
    /// # Errors
    ///
    /// If an out of bounds number of parameters or results are given.
    pub fn new<P, R>(params: P, results: R) -> Self
    where
        P: IntoIterator,
        R: IntoIterator,
        <P as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
        <R as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
    {
        let core = match CoreFuncType::new(params, results) {
            Ok(func_type) => func_type,
            Err(error) => panic!("failed to create function type: {error}"),
        };
        Self { core }
    }

    /// Returns the parameter types of the function type.
    pub fn params(&self) -> &[ValType] {
        self.core.params()
    }

    /// Returns the result types of the function type.
    pub fn results(&self) -> &[ValType] {
        self.core.results()
    }

    /// Returns the number of parameter types of the function type.
    pub(crate) fn len_params(&self) -> u16 {
        self.core.len_params()
    }

    /// Returns the number of result types of the function type.
    pub(crate) fn len_results(&self) -> u16 {
        self.core.len_results()
    }

    /// Returns the pair of parameter and result types of the function type.
    pub(crate) fn params_results(&self) -> (&[ValType], &[ValType]) {
        self.core.params_results()
    }

    /// Returns `Ok` if the number and types of items in `params` matches as expected by the [`FuncType`].
    ///
    /// # Errors
    ///
    /// - If the number of items in `params` does not match the number of parameters of the function type.
    /// - If any type of an item in `params` does not match the expected type of the function type.
    pub(crate) fn match_params<T>(&self, params: &[T]) -> Result<(), FuncError>
    where
        T: DynamicallyTyped,
    {
        if self.params().len() != params.len() {
            return Err(FuncError::MismatchingParameterLen);
        }
        if self
            .params()
            .iter()
            .copied()
            .ne(params.iter().map(<T as DynamicallyTyped>::ty))
        {
            return Err(FuncError::MismatchingParameterType);
        }
        Ok(())
    }

    /// Returns `Ok` if the number and types of items in `results` matches as expected by the [`FuncType`].
    ///
    /// # Note
    ///
    /// Only checks types if `check_type` is set to `true`.
    ///
    /// # Errors
    ///
    /// - If the number of items in `results` does not match the number of results of the function type.
    /// - If any type of an item in `results` does not match the expected type of the function type.
    pub(crate) fn match_results<T>(&self, results: &[T]) -> Result<(), FuncError>
    where
        T: DynamicallyTyped,
    {
        if self.results().len() != results.len() {
            return Err(FuncError::MismatchingResultLen);
        }
        if self
            .results()
            .iter()
            .copied()
            .ne(results.iter().map(<T as DynamicallyTyped>::ty))
        {
            return Err(FuncError::MismatchingResultType);
        }
        Ok(())
    }

    /// Initializes the values in `outputs` to match the types expected by the [`FuncType`].
    ///
    /// # Note
    ///
    /// This is required by an implementation detail of how function result passing is current
    /// implemented in the Wasmi execution engine and might change in the future.
    ///
    /// # Errors
    ///
    /// If the number of items in `outputs` does not match the number of results of the [`FuncType`].
    pub(crate) fn prepare_outputs(&self, outputs: &mut [Val]) -> Result<(), FuncError> {
        if self.results().len() != outputs.len() {
            return Err(FuncError::MismatchingResultLen);
        }
        for (output, result_ty) in outputs.iter_mut().zip(self.results()) {
            *output = Val::default(*result_ty);
        }
        Ok(())
    }
}
