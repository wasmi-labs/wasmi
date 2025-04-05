use crate::{
    core::{DynamicallyTyped, FuncType as CoreFuncType, ValType},
    func::FuncError,
    Val,
};

impl DynamicallyTyped for Val {
    fn ty(&self) -> ValType {
        self.ty()
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
