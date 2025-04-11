use crate::ValType;
use alloc::{sync::Arc, vec::Vec};
use core::{fmt, fmt::Display};

/// Errors that can occur upon type checking function signatures.
#[derive(Debug, Copy, Clone)]
pub enum FuncTypeError {
    /// Too many function parameters.
    TooManyFunctionParams,
    /// Too many function results.
    TooManyFunctionResults,
}

impl core::error::Error for FuncTypeError {}

impl Display for FuncTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncTypeError::TooManyFunctionParams => {
                write!(f, "encountered a function with too many parameters")
            }
            FuncTypeError::TooManyFunctionResults => {
                write!(f, "encountered a function with too many results")
            }
        }
    }
}

/// A function type representing a function's parameter and result types.
///
/// # Note
///
/// Can be cloned cheaply.
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct FuncType {
    /// The inner function type internals.
    inner: FuncTypeInner,
}

/// Internal details of [`FuncType`].
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum FuncTypeInner {
    /// Stores the value types of the parameters and results inline.
    Inline {
        /// The number of parameters.
        len_params: u8,
        /// The number of results.
        len_results: u8,
        /// The parameter types, followed by the result types, followed by unspecified elements.
        params_results: [ValType; Self::INLINE_SIZE],
    },
    /// Stores the value types of the parameters and results on the heap.
    Big {
        /// The number of parameters.
        len_params: u16,
        /// Combined parameter and result types allocated on the heap.
        params_results: Arc<[ValType]>,
    },
}

impl FuncTypeInner {
    /// The inline buffer size on 32-bit platforms.
    ///
    /// # Note
    ///
    /// On 32-bit platforms we target a `size_of<FuncTypeInner>()` of 16 bytes.
    #[cfg(target_pointer_width = "32")]
    const INLINE_SIZE: usize = 14;

    /// The inline buffer size on 64-bit platforms.
    ///
    /// # Note
    ///
    /// On 64-bit platforms we target a `size_of<FuncTypeInner>()` of 24 bytes.
    #[cfg(target_pointer_width = "64")]
    const INLINE_SIZE: usize = 21;

    /// The maximum number of parameter types allowed of a [`FuncType`].
    const MAX_LEN_PARAMS: usize = 1_000;

    /// The maximum number of result types allowed of a [`FuncType`].
    const MAX_LEN_RESULTS: usize = 1_000;

    /// Creates a new [`FuncTypeInner`].
    ///
    /// # Errors
    ///
    /// If an out of bounds number of parameters or results are given.
    pub fn new<P, R>(params: P, results: R) -> Result<Self, FuncTypeError>
    where
        P: IntoIterator,
        R: IntoIterator,
        <P as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
        <R as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
    {
        let mut params = params.into_iter();
        let mut results = results.into_iter();
        let len_params = params.len();
        let len_results = results.len();
        if len_params > Self::MAX_LEN_PARAMS {
            return Err(FuncTypeError::TooManyFunctionParams);
        }
        if len_results > Self::MAX_LEN_RESULTS {
            return Err(FuncTypeError::TooManyFunctionResults);
        }
        if let Some(small) = Self::try_new_small(&mut params, &mut results) {
            return Ok(small);
        }
        let Ok(len_params) = u16::try_from(len_params) else {
            unreachable!("already ensured that `len_params` is well within bounds of `u16::MAX`")
        };
        let mut params_results = params.collect::<Vec<_>>();
        params_results.extend(results);
        Ok(Self::Big {
            params_results: params_results.into(),
            len_params,
        })
    }

    /// Tries to create a [`FuncTypeInner::Inline`] variant from the given inputs.
    ///
    /// # Note
    ///
    /// - Returns `None` if creation was not possible.
    /// - Does not mutate `params` or `results` if this method returns `None`.
    pub fn try_new_small<P, R>(params: &mut P, results: &mut R) -> Option<Self>
    where
        P: Iterator<Item = ValType> + ExactSizeIterator,
        R: Iterator<Item = ValType> + ExactSizeIterator,
    {
        let params = params.into_iter();
        let results = results.into_iter();
        let len_params = u8::try_from(params.len()).ok()?;
        let len_results = u8::try_from(results.len()).ok()?;
        let len_inout = len_params.checked_add(len_results)?;
        if usize::from(len_inout) > Self::INLINE_SIZE {
            return None;
        }
        let mut params_results = [ValType::I32; Self::INLINE_SIZE];
        let cells = &mut params_results[..usize::from(len_inout)];
        let (cells_params, cells_results) = cells.split_at_mut(usize::from(len_params));
        for (cell, param) in cells_params.iter_mut().zip(params) {
            *cell = param;
        }
        for (cell, result) in cells_results.iter_mut().zip(results) {
            *cell = result;
        }
        Some(Self::Inline {
            len_params,
            len_results,
            params_results,
        })
    }

    /// Returns the parameter types of the function type.
    pub fn params(&self) -> &[ValType] {
        match self {
            FuncTypeInner::Inline {
                len_params,
                params_results,
                ..
            } => &params_results[..usize::from(*len_params)],
            FuncTypeInner::Big {
                len_params,
                params_results,
            } => &params_results[..usize::from(*len_params)],
        }
    }

    /// Returns the result types of the function type.
    pub fn results(&self) -> &[ValType] {
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
            } => &params_results[usize::from(*len_params)..],
        }
    }

    /// Returns the number of parameter types of the function type.
    pub fn len_params(&self) -> u16 {
        match self {
            FuncTypeInner::Inline { len_params, .. } => u16::from(*len_params),
            FuncTypeInner::Big { len_params, .. } => *len_params,
        }
    }

    /// Returns the number of result types of the function type.
    pub fn len_results(&self) -> u16 {
        match self {
            FuncTypeInner::Inline { len_results, .. } => u16::from(*len_results),
            FuncTypeInner::Big {
                len_params,
                params_results,
            } => {
                // Note: this cast is safe since the number of parameters and results
                //       are both bounded to 1000 maximum and `params_results`
                //       thus contains 2000 items at most.
                let Ok(len_buffer) = u16::try_from(params_results.len()) else {
                    panic!("the size of a `FuncType` buffer must fit into a `u16`")
                };
                let len_params = *len_params;
                len_buffer - len_params
            }
        }
    }

    /// Returns the pair of parameter and result types of the function type.
    pub(crate) fn params_results(&self) -> (&[ValType], &[ValType]) {
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
            } => params_results.split_at(usize::from(*len_params)),
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
    ///
    /// # Errors
    ///
    /// If an out of bounds number of parameters or results are given.
    pub fn new<P, R>(params: P, results: R) -> Result<Self, FuncTypeError>
    where
        P: IntoIterator,
        R: IntoIterator,
        <P as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
        <R as IntoIterator>::IntoIter: Iterator<Item = ValType> + ExactSizeIterator,
    {
        let inner = FuncTypeInner::new(params, results)?;
        Ok(Self { inner })
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
    pub fn params_results(&self) -> (&[ValType], &[ValType]) {
        self.inner.params_results()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty_works() {
        let ft = FuncType::new([], []).unwrap();
        assert!(ft.params().is_empty());
        assert!(ft.results().is_empty());
        assert_eq!(ft.params(), ft.params_results().0);
        assert_eq!(ft.results(), ft.params_results().1);
    }

    #[test]
    fn new_inline_works() {
        let types = [
            &[ValType::I32][..],
            &[ValType::I64][..],
            &[ValType::F32][..],
            &[ValType::F64][..],
            &[ValType::I32, ValType::I32][..],
            &[ValType::I32, ValType::I32, ValType::I32][..],
            &[ValType::I32, ValType::I32, ValType::I32, ValType::I32][..],
            &[
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ][..],
            &[ValType::I32, ValType::I64, ValType::F32, ValType::F64][..],
        ];
        for params in types {
            for results in types {
                let params_iter = params.iter().copied();
                let results_iter = results.iter().copied();
                let ft = FuncType::new(params_iter, results_iter).unwrap();
                assert_eq!(ft.params(), params);
                assert_eq!(ft.results(), results);
                assert_eq!(ft.params(), ft.params_results().0);
                assert_eq!(ft.results(), ft.params_results().1);
            }
        }
    }

    #[test]
    fn new_big_works() {
        let params = [ValType::I32; 100];
        let results = [ValType::I64; 100];
        let params_iter = params.iter().copied();
        let results_iter = results.iter().copied();
        let ft = FuncType::new(params_iter, results_iter).unwrap();
        assert_eq!(ft.params(), params);
        assert_eq!(ft.results(), results);
        assert_eq!(ft.params(), ft.params_results().0);
        assert_eq!(ft.results(), ft.params_results().1);
    }
}
