use crate::{
    core::{FuncType as CoreFuncType, ValType},
    func::FuncError,
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

/// Extension methods for [`FuncType`].
pub trait FuncTypeExt {
    /// Creates a new [`FuncTypeInner`].
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

impl FuncTypeExt for CoreFuncType {
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

/// A function type representing a function's parameter and result types.
///
/// # Note
///
/// Can be cloned cheaply.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct FuncType {
    /// The inner function type internals.
    inner: CoreFuncType,
}

impl FuncType {
    /// Creates a new [`FuncType`].
    pub fn new<P, R>(params: P, results: R) -> Self
    where
        P: IntoIterator,
        R: IntoIterator,
        <P as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
        <R as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
    {
        let inner = match CoreFuncType::new(params, results) {
            Ok(inner) => inner,
            Err(error) => panic!("failed to create `FuncType`: {error}"),
        };
        Self { inner }
    }

    /// Returns the parameter types of the function type.
    pub fn params(&self) -> &[ValType] {
        self.inner.params()
    }

    /// Returns the result types of the function type.
    pub fn results(&self) -> &[ValType] {
        self.inner.results()
    }

    /// Returns the number of parameter types of the function type.
    pub fn len_params(&self) -> u16 {
        self.inner.len_params()
    }

    /// Returns the number of result types of the function type.
    pub fn len_results(&self) -> u16 {
        self.inner.len_results()
    }

    /// Returns the pair of parameter and result types of the function type.
    pub(crate) fn params_results(&self) -> (&[ValType], &[ValType]) {
        self.inner.params_results()
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
        self.inner.match_params::<T>(params).map_err(Into::into)
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
    pub(crate) fn match_results<T>(&self, results: &[T], check_type: bool) -> Result<(), FuncError>
    where
        T: DynamicallyTyped,
    {
        self.inner
            .match_results::<T>(results, check_type)
            .map_err(Into::into)
    }

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
    pub(crate) fn prepare_outputs(&self, outputs: &mut [Val]) {
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
