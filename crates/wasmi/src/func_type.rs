use crate::{core::ValueType, func::FuncError};
use alloc::{sync::Arc, vec::Vec};
use core::fmt::{self, Display};
use wasmi_core::Value;

/// A function type representing a function's parameter and result types.
///
/// # Note
///
/// Can be cloned cheaply.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct FuncType {
    /// The number of function parameters.
    len_params: usize,
    /// The ordered and merged parameter and result types of the function type.
    ///
    /// # Note
    ///
    /// The parameters and results are ordered and merged in a single
    /// vector starting with parameters in their order and followed
    /// by results in their order.
    /// The `len_params` field denotes how many parameters there are in
    /// the head of the vector before the results.
    params_results: Arc<[ValueType]>,
}

impl Display for FuncType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn(")?;
        let (params, results) = self.params_results();
        write_slice(f, params, ",")?;
        write!(f, ")")?;
        if let Some((first, rest)) = results.split_first() {
            write!(f, " -> ")?;
            if !rest.is_empty() {
                write!(f, "(")?;
            }
            write!(f, "{first}")?;
            for result in rest {
                write!(f, ", {result}")?;
            }
            if !rest.is_empty() {
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

/// Writes the elements of a `slice` separated by the `separator`.
fn write_slice<T>(f: &mut fmt::Formatter, slice: &[T], separator: &str) -> fmt::Result
where
    T: Display,
{
    if let Some((first, rest)) = slice.split_first() {
        write!(f, "{first}")?;
        for param in rest {
            write!(f, "{separator} {param}")?;
        }
    }
    Ok(())
}

impl FuncType {
    /// Creates a new [`FuncType`].
    pub fn new<P, R>(params: P, results: R) -> Self
    where
        P: IntoIterator<Item = ValueType>,
        R: IntoIterator<Item = ValueType>,
    {
        let mut params_results = params.into_iter().collect::<Vec<_>>();
        let len_params = params_results.len();
        params_results.extend(results);
        Self {
            params_results: params_results.into(),
            len_params,
        }
    }

    /// Returns the parameter types of the function type.
    pub fn params(&self) -> &[ValueType] {
        &self.params_results[..self.len_params]
    }

    /// Returns the result types of the function type.
    pub fn results(&self) -> &[ValueType] {
        &self.params_results[self.len_params..]
    }

    /// Returns the pair of parameter and result types of the function type.
    pub(crate) fn params_results(&self) -> (&[ValueType], &[ValueType]) {
        self.params_results.split_at(self.len_params)
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
            return Err(FuncError::MismatchingParameterLen).map_err(Into::into);
        }
        if self
            .params()
            .iter()
            .copied()
            .ne(params.iter().map(<T as Ty>::ty))
        {
            return Err(FuncError::MismatchingParameterType).map_err(Into::into);
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
            return Err(FuncError::MismatchingResultLen).map_err(Into::into);
        }
        if check_type
            && self
                .results()
                .iter()
                .copied()
                .ne(results.iter().map(<T as Ty>::ty))
        {
            return Err(FuncError::MismatchingResultType).map_err(Into::into);
        }
        Ok(())
    }

    /// Initializes the values in `outputs` to match the types expected by the [`FuncType`].
    ///
    /// # Note
    ///
    /// This is required by an implementation detail of how function result passing is current
    /// implemented in the `wasmi` execution engine and might change in the future.
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
        self.value_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::borrow::Borrow;

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

    fn assert_display(func_type: impl Borrow<FuncType>, expected: &str) {
        assert_eq!(format!("{}", func_type.borrow()), String::from(expected),);
    }

    #[test]
    fn display_0in_0out() {
        assert_display(FuncType::new([], []), "fn()");
    }

    #[test]
    fn display_1in_0out() {
        assert_display(FuncType::new([ValueType::I32], []), "fn(i32)");
    }

    #[test]
    fn display_0in_1out() {
        assert_display(FuncType::new([], [ValueType::I32]), "fn() -> i32");
    }

    #[test]
    fn display_1in_1out() {
        assert_display(
            FuncType::new([ValueType::I32], [ValueType::I32]),
            "fn(i32) -> i32",
        );
    }

    #[test]
    fn display_4in_0out() {
        assert_display(
            FuncType::new(
                [
                    ValueType::I32,
                    ValueType::I64,
                    ValueType::F32,
                    ValueType::F64,
                ],
                [],
            ),
            "fn(i32, i64, f32, f64)",
        );
    }

    #[test]
    fn display_0in_4out() {
        assert_display(
            FuncType::new(
                [],
                [
                    ValueType::I32,
                    ValueType::I64,
                    ValueType::F32,
                    ValueType::F64,
                ],
            ),
            "fn() -> (i32, i64, f32, f64)",
        );
    }

    #[test]
    fn display_4in_4out() {
        assert_display(
            FuncType::new(
                [
                    ValueType::I32,
                    ValueType::I64,
                    ValueType::F32,
                    ValueType::F64,
                ],
                [
                    ValueType::I32,
                    ValueType::I64,
                    ValueType::F32,
                    ValueType::F64,
                ],
            ),
            "fn(i32, i64, f32, f64) -> (i32, i64, f32, f64)",
        );
    }
}
