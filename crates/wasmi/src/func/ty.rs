use crate::{Val, ValType, core::CoreFuncType, errors::FuncError};

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

    /// Returns `true` when the [`FuncType`] is a subtype of the `other` [`FuncType`].
    ///
    /// # Note
    ///
    /// Currently, `.matches` and `==` give the same result because [`ValType`] has no subtyping yet.
    /// That may change with `function-references`.
    ///
    /// The parameter types are checked in the opposite direction
    /// (contravariantly), while the result types are checked in the same
    /// direction (covariantly), following the
    /// WebAssembly [subtyping rules](https://webassembly.github.io/spec/core/valid/types.html#import-subtyping).
    pub fn matches(&self, other: &Self) -> bool {
        self.core.matches(&other.core)
    }

    /// Returns the number of parameter types of the function type.
    pub(crate) fn len_params(&self) -> u16 {
        self.core.len_params()
    }

    /// Returns the number of result types of the function type.
    pub(crate) fn len_results(&self) -> u16 {
        self.core.len_results()
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
            *output = Val::default_for_ty(*result_ty);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_works() {
        // Two void FuncType instances should match
        let self_ft = FuncType::new([], []);
        let other_ft = FuncType::new([], []);
        assert!(self_ft.matches(&other_ft));
        assert!(other_ft.matches(&self_ft));

        // Two different FuncType instances should not match
        let self_ft = FuncType::new([ValType::I32, ValType::F32], []);
        let other_ft = FuncType::new([ValType::F32], [ValType::F32]);
        assert!(!self_ft.matches(&other_ft));
        assert!(!other_ft.matches(&self_ft));
    }
}