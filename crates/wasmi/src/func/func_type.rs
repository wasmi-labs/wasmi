use crate::{core::ValType, func::FuncError, FuncType, Val};

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

/// Extension methods for [`FuncType`].
pub trait FuncTypeExt {
    /// Creates a new [`FuncType`].
    ///
    /// # Panics
    ///
    /// If an out of bounds number of parameters or results are given.
    fn new_or_panic<P, R>(params: P, results: R) -> Self
    where
        P: IntoIterator,
        R: IntoIterator,
        <P as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
        <R as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator;

    /// Returns `Ok` if the number and types of items in `params` matches as expected by the [`FuncType`].
    ///
    /// # Errors
    ///
    /// - If the number of items in `params` does not match the number of parameters of the function type.
    /// - If any type of an item in `params` does not match the expected type of the function type.
    fn match_params<T>(&self, params: &[T]) -> Result<(), FuncError>
    where
        T: DynamicallyTyped;

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
    fn match_results<T>(&self, results: &[T], check_type: bool) -> Result<(), FuncError>
    where
        T: DynamicallyTyped;

    /// Initializes the values in `outputs` to match the types expected by the [`FuncType`].
    ///
    /// # Note
    ///
    /// This is required by an implementation detail of how function result passing is current
    /// implemented in the Wasmi execution engine and might change in the future.
    ///
    /// # Panics
    ///
    /// If the number of items in `outputs` does not match the number of results of the [`FuncType`].
    fn prepare_outputs(&self, outputs: &mut [Val]);
}

impl FuncTypeExt for FuncType {
    fn new_or_panic<P, R>(params: P, results: R) -> Self
    where
        P: IntoIterator,
        R: IntoIterator,
        <P as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
        <R as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
    {
        match Self::new(params, results) {
            Ok(func_type) => func_type,
            Err(error) => panic!("failed to create function type: {error}"),
        }
    }

    fn match_params<T>(&self, params: &[T]) -> Result<(), FuncError>
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

    fn match_results<T>(&self, results: &[T], check_type: bool) -> Result<(), FuncError>
    where
        T: DynamicallyTyped,
    {
        if self.results().len() != results.len() {
            return Err(FuncError::MismatchingResultLen);
        }
        if check_type
            && self
                .results()
                .iter()
                .copied()
                .ne(results.iter().map(<T as DynamicallyTyped>::ty))
        {
            return Err(FuncError::MismatchingResultType);
        }
        Ok(())
    }

    fn prepare_outputs(&self, outputs: &mut [Val]) {
        assert_eq!(
            self.results().len(),
            outputs.len(),
            "must have the same number of items in outputs as results of the function type"
        );
        let init_values = self.results().iter().copied().map(Val::default);
        outputs
            .iter_mut()
            .zip(init_values)
            .for_each(|(output, init)| *output = init);
    }
}
