use crate::core::ValueType;
use alloc::{sync::Arc, vec::Vec};
use core::fmt::{self, Display};

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
