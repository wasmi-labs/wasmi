use crate::{core::ValueType, func::FuncError, Value};
use core::fmt;
use std::{sync::Arc, vec::Vec};

/// A function type representing a function's parameter and result types.
///
/// # Note
///
/// Can be cloned cheaply.
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct FuncType {
    /// The inner function type internals.
    inner: FuncTypeInner,
}

/// Internal details of [`FuncType`].
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum FuncTypeInner {
    /// Stores the value types of the parameters and results inline.
    Inline {
        /// The number of parameters.
        len_params: u8,
        /// The number of results.
        len_results: u8,
        /// The parameter types, followed by the result types, followed by unspecified elements.
        params_results: [ValueType; Self::INLINE_SIZE],
    },
    /// Stores the value types of the parameters and results on the heap.
    Big {
        len_params: u32,
        params_results: Arc<[ValueType]>,
    },
}

impl FuncTypeInner {
    /// The inline buffer size on 32-bit platforms.
    ///
    /// # Note
    ///
    /// On 64-bit platforms we target a `size_of<FuncTypeInner>()` of 16 bytes.
    #[cfg(target_pointer_width = "32")]
    const INLINE_SIZE: usize = 14;

    /// The inline buffer size on 64-bit platforms.
    ///
    /// # Note
    ///
    /// On 64-bit platforms we target a `size_of<FuncTypeInner>()` of 16 bytes.
    /// On 64-bit platforms we target a `size_of<FuncTypeInner>()` of 24 bytes.
    #[cfg(target_pointer_width = "64")]
    const INLINE_SIZE: usize = 21;

    /// Creates a new [`FuncTypeInner`].
    ///
    /// # Panics
    ///
    /// If an out of bounds number of parameters or results are given.
    pub fn new<P, R>(params: P, results: R) -> Self
    where
        P: IntoIterator,
        R: IntoIterator,
        <P as IntoIterator>::IntoIter: Iterator<Item = ValueType> + ExactSizeIterator,
        <R as IntoIterator>::IntoIter: Iterator<Item = ValueType> + ExactSizeIterator,
    {
        let mut params = params.into_iter();
        let mut results = results.into_iter();
        let len_params = params.len();
        let len_results = results.len();
        if let Some(small) = Self::try_new_small(&mut params, &mut results) {
            return small;
        }
        let mut params_results = params.collect::<Vec<_>>();
        let len_params = u32::try_from(params_results.len()).unwrap_or_else(|_| {
            panic!("out of bounds parameters (={len_params}) and results (={len_results}) for FuncType")
        });
        params_results.extend(results);
        Self::Big {
            params_results: params_results.into(),
            len_params,
        }
    }

    /// Tries to create a [`FuncTypeInner::Inline`] variant from the given inputs.
    ///
    /// # Note
    ///
    /// - Returns `None` if creation was not possible.
    /// - Does not mutate `params` or `results` if this method returns `None`.
    pub fn try_new_small<P, R>(params: &mut P, results: &mut R) -> Option<Self>
    where
        P: Iterator<Item = ValueType> + ExactSizeIterator,
        R: Iterator<Item = ValueType> + ExactSizeIterator,
    {
        let params = params.into_iter();
        let results = results.into_iter();
        let len_params = u8::try_from(params.len()).ok()?;
        let len_results = u8::try_from(results.len()).ok()?;
        let len = len_params.checked_add(len_results)?;
        if usize::from(len) > Self::INLINE_SIZE {
            return None;
        }
        let mut params_results = [ValueType::I32; Self::INLINE_SIZE];
        params_results
            .iter_mut()
            .zip(params.chain(results))
            .for_each(|(cell, param_or_result)| {
                *cell = param_or_result;
            });
        Some(Self::Inline {
            len_params,
            len_results,
            params_results,
        })
    }

    /// Returns the parameter types of the function type.
    pub fn params(&self) -> &[ValueType] {
        match self {
            FuncTypeInner::Inline {
                len_params,
                params_results,
                ..
            } => &params_results[..usize::from(*len_params)],
            FuncTypeInner::Big {
                len_params,
                params_results,
            } => &params_results[..(*len_params as usize)],
        }
    }

    /// Returns the result types of the function type.
    pub fn results(&self) -> &[ValueType] {
        match self {
            FuncTypeInner::Inline {
                len_params,
                len_results,
                params_results,
                ..
            } => {
                let start_results = usize::from(*len_params);
                let end_results = start_results + usize::from(*len_results);
                &params_results[start_results..end_results]
            }
            FuncTypeInner::Big {
                len_params,
                params_results,
            } => &params_results[(*len_params as usize)..],
        }
    }

    /// Returns the number of result types of the function type.
    pub fn len_results(&self) -> usize {
        match self {
            FuncTypeInner::Inline { len_results, .. } => usize::from(*len_results),
            FuncTypeInner::Big {
                len_params,
                params_results,
            } => {
                let len_buffer = params_results.len();
                let len_params = *len_params as usize;
                len_buffer - len_params
            }
        }
    }

    /// Returns the pair of parameter and result types of the function type.
    pub(crate) fn params_results(&self) -> (&[ValueType], &[ValueType]) {
        match self {
            FuncTypeInner::Inline {
                len_params,
                len_results,
                params_results,
            } => {
                let len_params = usize::from(*len_params);
                let len_results = usize::from(*len_results);
                params_results[..len_params + len_results].split_at(len_params)
            }
            FuncTypeInner::Big {
                len_params,
                params_results,
            } => params_results.split_at(*len_params as usize),
        }
    }
}

#[test]
fn size_of_func_type() {
    #[cfg(target_pointer_width = "32")]
    assert!(core::mem::size_of::<FuncTypeInner>() <= 16);
    #[cfg(target_pointer_width = "64")]
    assert!(core::mem::size_of::<FuncTypeInner>() <= 24);
}

impl fmt::Debug for FuncType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FuncType")
            .field("params", &self.params())
            .field("results", &self.results())
            .finish()
    }
}

impl FuncType {
    /// Creates a new [`FuncType`].
    pub fn new<P, R>(params: P, results: R) -> Self
    where
        P: IntoIterator,
        R: IntoIterator,
        <P as IntoIterator>::IntoIter: Iterator<Item = ValueType> + ExactSizeIterator,
        <R as IntoIterator>::IntoIter: Iterator<Item = ValueType> + ExactSizeIterator,
    {
        Self {
            inner: FuncTypeInner::new(params, results),
        }
    }

    /// Returns the parameter types of the function type.
    pub fn params(&self) -> &[ValueType] {
        self.inner.params()
    }

    /// Returns the result types of the function type.
    pub fn results(&self) -> &[ValueType] {
        self.inner.results()
    }

    /// Returns the number of result types of the function type.
    pub fn len_results(&self) -> usize {
        self.inner.len_results()
    }

    /// Returns the pair of parameter and result types of the function type.
    pub(crate) fn params_results(&self) -> (&[ValueType], &[ValueType]) {
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
        T: Ty,
    {
        if self.params().len() != params.len() {
            return Err(FuncError::MismatchingParameterLen);
        }
        if self
            .params()
            .iter()
            .copied()
            .ne(params.iter().map(<T as Ty>::ty))
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
    pub(crate) fn match_results<T>(&self, results: &[T], check_type: bool) -> Result<(), FuncError>
    where
        T: Ty,
    {
        if self.results().len() != results.len() {
            return Err(FuncError::MismatchingResultLen);
        }
        if check_type
            && self
                .results()
                .iter()
                .copied()
                .ne(results.iter().map(<T as Ty>::ty))
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
    /// # Panics
    ///
    /// If the number of items in `outputs` does not match the number of results of the [`FuncType`].
    pub(crate) fn prepare_outputs(&self, outputs: &mut [Value]) {
        assert_eq!(
            self.results().len(),
            outputs.len(),
            "must have the same number of items in outputs as results of the function type"
        );
        let init_values = self.results().iter().copied().map(Value::default);
        outputs
            .iter_mut()
            .zip(init_values)
            .for_each(|(output, init)| *output = init);
    }
}

/// Types that have a [`ValueType`].
///
/// # Note
///
/// Primarily used to allow `match_params` and `match_results`
/// to be called with both [`Value`] and [`ValueType`] parameters.
pub(crate) trait Ty {
    fn ty(&self) -> ValueType;
}

impl Ty for ValueType {
    fn ty(&self) -> ValueType {
        *self
    }
}

impl Ty for Value {
    fn ty(&self) -> ValueType {
        self.ty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty_works() {
        let ft = FuncType::new([], []);
        assert!(ft.params().is_empty());
        assert!(ft.results().is_empty());
        assert_eq!(ft.params(), ft.params_results().0);
        assert_eq!(ft.results(), ft.params_results().1);
    }

    #[test]
    fn new_works() {
        let types = [
            &[ValueType::I32][..],
            &[ValueType::I64][..],
            &[ValueType::F32][..],
            &[ValueType::F64][..],
            &[ValueType::I32, ValueType::I32][..],
            &[ValueType::I32, ValueType::I32, ValueType::I32][..],
            &[
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
            ][..],
            &[
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
                ValueType::I32,
            ][..],
            &[
                ValueType::I32,
                ValueType::I64,
                ValueType::F32,
                ValueType::F64,
            ][..],
        ];
        for params in types {
            for results in types {
                let ft = FuncType::new(params.iter().copied(), results.iter().copied());
                assert_eq!(ft.params(), params);
                assert_eq!(ft.results(), results);
                assert_eq!(ft.params(), ft.params_results().0);
                assert_eq!(ft.results(), ft.params_results().1);
            }
        }
    }
}
