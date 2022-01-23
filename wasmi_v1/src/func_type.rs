use super::{Index, Store, StoreContext, Stored};
use crate::ValueType;
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

    /// Returns the result types of the function type.
    pub fn results(&self) -> &[ValueType] {
        &self.params_results[self.len_params..]
    }

    /// Returns the pair of parameter and result types of the function type.
    pub(crate) fn params_results(&self) -> (&[ValueType], &[ValueType]) {
        self.params_results.split_at(self.len_params)
    }
}

/// A raw index to a function signature entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignatureIdx(usize);

impl Index for SignatureIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

/// A Wasm function signature reference.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Signature(Stored<SignatureIdx>);

impl Signature {
    /// Creates a new function signature reference.
    pub(super) fn from_inner(stored: Stored<SignatureIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn into_inner(self) -> Stored<SignatureIdx> {
        self.0
    }

    /// Creates a new function signature to the store.
    pub fn new<T, I, O>(ctx: &mut Store<T>, inputs: I, outputs: O) -> Self
    where
        I: IntoIterator<Item = ValueType>,
        O: IntoIterator<Item = ValueType>,
    {
        ctx.alloc_func_type(FuncType::new(inputs, outputs))
    }

    /// Returns the parameters of the function type.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Signature`].
    pub fn params<'a, T: 'a>(&self, ctx: impl Into<StoreContext<'a, T>>) -> &'a [ValueType] {
        ctx.into().store.resolve_func_type(*self).params()
    }

    /// Returns the results of the function signature.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Signature`].
    pub fn results<'a, T: 'a>(&self, ctx: impl Into<StoreContext<'a, T>>) -> &'a [ValueType] {
        ctx.into().store.resolve_func_type(*self).results()
    }

    /// Returns the parameter and result types of the function type.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`Signature`].
    pub fn params_results<'a, T: 'a>(
        &self,
        ctx: impl Into<StoreContext<'a, T>>,
    ) -> (&'a [ValueType], &'a [ValueType]) {
        ctx.into().store.resolve_func_type(*self).params_results()
    }
}
