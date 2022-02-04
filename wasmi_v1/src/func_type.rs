use crate::core::ValueType;
use alloc::{sync::Arc, vec::Vec};

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
