use super::{into_func::WasmTypeList, Func, FuncError};
use crate::{
    core::Value,
    engine::{CallParams, CallResults},
    AsContext,
    AsContextMut,
    Error,
};
use core::{fmt, fmt::Debug, marker::PhantomData};
use wasmi_core::Trap;

/// A typed [`Func`] instance.
///
/// # Note
///
/// This allows a more efficient execution by avoiding type checks
/// upon function call since those type checks are performed upon [`TypedFunc`]
/// construction and enforced by the Rust type system.
///
/// Use [`TypedFunc`] instead of [`Func`] if possible.
#[repr(transparent)]
pub struct TypedFunc<Params, Results> {
    /// The parameter and result typed encoded in Rust type system.
    _signature: PhantomData<fn(Params) -> Results>,
    /// The underlying [`Func`] instance.
    func: Func,
}

impl<Params, Results> Debug for TypedFunc<Params, Results> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedFunc")
            .field("_signature", &self._signature)
            .field("func", &self.func)
            .finish()
    }
}

impl<Params, Results> Copy for TypedFunc<Params, Results> {}

impl<Params, Results> Clone for TypedFunc<Params, Results> {
    fn clone(&self) -> TypedFunc<Params, Results> {
        *self
    }
}

impl<Params, Results> TypedFunc<Params, Results> {
    /// Returns the underlying [`Func`].
    ///
    /// # Note
    ///
    /// This loses the static type information in the process.
    pub fn func(&self) -> &Func {
        &self.func
    }
}

impl<Params, Results> TypedFunc<Params, Results>
where
    Params: WasmParams,
    Results: WasmResults,
{
    /// Creates a new [`TypedFunc`] for the given [`Func`] using the static typing.
    ///
    /// # Errors
    ///
    /// If the provided static types `Params` and `Results` for the parameters
    /// and result types of `func` mismatch the signature of `func`.
    pub(crate) fn new(ctx: impl AsContext, func: Func) -> Result<Self, Error> {
        let func_type = func.func_type(&ctx);
        let (expected_params, expected_results) = func_type.params_results();
        let (actual_params, actual_results) = (
            <Params as WasmTypeList>::value_types(),
            <Results as WasmTypeList>::value_types(),
        );
        if actual_params.as_ref() != expected_params {
            return Err(Error::Func(FuncError::MismatchingParameters { func }));
        }
        if actual_results.as_ref() != expected_results {
            return Err(Error::Func(FuncError::MismatchingResults { func }));
        }
        Ok(Self {
            _signature: PhantomData,
            func,
        })
    }

    /// Invokes this Wasm or host function with the specified parameters.
    ///
    /// Returns either the results of the call, or a [`Trap`] if one happened.
    ///
    /// For more information, see the [`Func::typed`] and [`Func::call`]
    /// documentation.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`TypedFunc`].
    pub fn call(&self, mut ctx: impl AsContextMut, params: Params) -> Result<Results, Trap> {
        // Note: Cloning an [`Engine`] is intentionally a cheap operation.
        ctx.as_context().store.engine().clone().execute_func(
            ctx.as_context_mut(),
            self.func,
            params,
            <CallResultsTuple<Results>>::default(),
        )
    }
}

impl<Params> CallParams for Params
where
    Params: WasmParams,
{
    type Params = <Params as WasmTypeList>::ValuesIter;

    fn len_params(&self) -> usize {
        <Params as WasmTypeList>::LEN
    }

    fn feed_params(self) -> Self::Params {
        <Params as WasmTypeList>::values(self).into_iter()
    }
}

/// Wrapper around the result tuple types of a [`TypedFunc`].
///
/// # Note
///
/// This type is a utility in order to provide an efficient implementation
/// of the [`CallResults`] trait required for executing the [`TypedFunc`]
/// via the [`Engine`].
///
/// [`Engine`]: [`crate::Engine`].
pub struct CallResultsTuple<Results> {
    _marker: PhantomData<fn() -> Results>,
}

impl<Results> Default for CallResultsTuple<Results> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
impl<Results> Copy for CallResultsTuple<Results> {}
impl<Results> Clone for CallResultsTuple<Results> {
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Results> CallResults for CallResultsTuple<Results>
where
    Results: WasmResults,
{
    type Results = Results;

    fn len_results(&self) -> usize {
        <Results as WasmTypeList>::LEN
    }

    fn feed_results<T>(self, results: T) -> Self::Results
    where
        T: IntoIterator<Item = Value>,
        T::IntoIter: ExactSizeIterator,
    {
        let results = results.into_iter();
        assert_eq!(self.len_results(), results.len());
        <Results as WasmTypeList>::from_values(results)
            .expect("unable to construct typed results from value iterator")
    }
}

/// The typed parameters of a [`TypedFunc`].
pub trait WasmParams: WasmTypeList {}
impl<T> WasmParams for T where T: WasmTypeList {}

/// The typed results of a [`TypedFunc`].
pub trait WasmResults: WasmTypeList {}
impl<T> WasmResults for T where T: WasmTypeList {}
