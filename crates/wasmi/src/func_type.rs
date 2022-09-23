use crate::core::ValueType;
use alloc::{sync::Arc, vec::Vec};
use core::fmt::{self, Display};

/// A function type representing a function's parameter and result types.
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
        if let Some((first, rest)) = params.split_first() {
            write!(f, "{}", first)?;
            for param in rest {
                write!(f, ", {}", param)?;
            }
        }
        write!(f, ")")?;
        if let Some((first, rest)) = results.split_first() {
            write!(f, " -> ")?;
            if !rest.is_empty() {
                write!(f, "(")?;
            }
            write!(f, "{}", first)?;
            for result in rest {
                write!(f, ", {}", result)?;
            }
            if !rest.is_empty() {
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

impl FuncType {
    /// Creates a new function signature.
    pub fn new<I, O>(inputs: I, outputs: O) -> Self
    where
        I: IntoIterator<Item = ValueType>,
        O: IntoIterator<Item = ValueType>,
    {
        let mut inputs_outputs = inputs.into_iter().collect::<Vec<_>>();
        let len_inputs = inputs_outputs.len();
        inputs_outputs.extend(outputs);
        Self {
            params_results: inputs_outputs.into(),
            len_params: len_inputs,
        }
    }

    /// Returns the parameter types of the function type.
    pub fn params(&self) -> &[ValueType] {
        &self.params_results[..self.len_params]
    }

    // pub fn into_params(self) -> impl ExactSizeIterator<Item = ValueType> + 'static {
    //     self.params_results[..self.len_params].iter().copied()
    // }

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

    #[test]
    fn display_0in_0out() {
        let func_type = FuncType::new([], []);
        assert_eq!(format!("{}", func_type), String::from("fn()"),);
    }

    #[test]
    fn display_1in_1out() {
        let func_type = FuncType::new([ValueType::I32], [ValueType::I32]);
        assert_eq!(format!("{}", func_type), String::from("fn(i32) -> i32"),);
    }

    #[test]
    fn display_4in_0out() {
        let func_type = FuncType::new(
            [
                ValueType::I32,
                ValueType::I64,
                ValueType::F32,
                ValueType::F64,
            ],
            [],
        );
        assert_eq!(
            format!("{}", func_type),
            String::from("fn(i32, i64, f32, f64)"),
        );
    }

    #[test]
    fn display_0in_4out() {
        let func_type = FuncType::new(
            [],
            [
                ValueType::I32,
                ValueType::I64,
                ValueType::F32,
                ValueType::F64,
            ],
        );
        assert_eq!(
            format!("{}", func_type),
            String::from("fn() -> (i32, i64, f32, f64)"),
        );
    }

    #[test]
    fn display_4in_4out() {
        let func_type = FuncType::new(
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
        );
        assert_eq!(
            format!("{}", func_type),
            String::from("fn(i32, i64, f32, f64) -> (i32, i64, f32, f64)"),
        );
    }
}
