use super::{Func, into_func::WasmTyList};
use crate::{
    core::UntypedVal,
    engine::{CallParams, CallResults, LoadFromCells, StoreToCells},
    AsContext,
    AsContextMut,
    Error,
    TypedResumableCall,
    core::UntypedVal,
    engine::{CallParams, CallResults},
};
use core::{fmt, fmt::Debug, marker::PhantomData};

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
    signature: PhantomData<fn(Params) -> Results>,
    /// The underlying [`Func`] instance.
    func: Func,
}

impl<Params, Results> Debug for TypedFunc<Params, Results> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedFunc")
            .field("signature", &self.signature)
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
        let func_type = func.ty(&ctx);
        let (actual_params, actual_results) = (
            <Params as WasmTyList>::types(),
            <Results as WasmTyList>::types(),
        );
        func_type.match_params(actual_params.as_ref())?;
        func_type.match_results(actual_results.as_ref())?;
        Ok(Self {
            signature: PhantomData,
            func,
        })
    }

    /// Calls this Wasm or host function with the specified parameters.
    ///
    /// Returns either the results of the call, or a [`Error`] if one happened.
    ///
    /// For more information, see the [`Func::typed`] and [`Func::call`]
    /// documentation.
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`TypedFunc`].
    ///
    /// # Errors
    ///
    /// If the execution of the called Wasm function traps.
    pub fn call(&self, mut ctx: impl AsContextMut, params: Params) -> Result<Results, Error> {
        // Note: Cloning an [`Engine`] is intentionally a cheap operation.
        ctx.as_context().store.engine().clone().execute_func(
            ctx.as_context_mut(),
            &self.func,
            params,
            <CallResultsTuple<Results>>::default(),
        )
    }

    /// Calls this Wasm or host function with the specified parameters.
    ///
    /// Returns a resumable handle to the function invocation upon
    /// encountering host errors with which it is possible to handle
    /// the error and continue the execution as if no error occurred.
    ///
    /// # Note
    ///
    /// This is a non-standard WebAssembly API and might not be available
    /// at other WebAssembly engines. Please be aware that depending on this
    /// feature might mean a lock-in to Wasmi for users.
    ///
    /// # Errors
    ///
    /// If the function returned a [`Error`] originating from WebAssembly.
    pub fn call_resumable(
        &self,
        mut ctx: impl AsContextMut,
        params: Params,
    ) -> Result<TypedResumableCall<Results>, Error> {
        // Note: Cloning an [`Engine`] is intentionally a cheap operation.
        ctx.as_context()
            .store
            .engine()
            .clone()
            .execute_func_resumable(
                ctx.as_context_mut(),
                &self.func,
                params,
                <CallResultsTuple<Results>>::default(),
            )
            .map(TypedResumableCall::new)
    }
}

impl<Params> CallParams for Params
where
    Params: WasmParams,
{
    type Params = <Params as WasmTyList>::ValuesIter;

    #[inline]
    fn call_params(self) -> Self::Params {
        <Params as WasmTyList>::values(self).into_iter()
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
        *self
    }
}

impl<Results> CallResults for CallResultsTuple<Results>
where
    Results: WasmResults,
{
    type Results = Results;

    fn len_results(&self) -> usize {
        <Results as WasmTyList>::LEN
    }

    fn call_results(self, results: &[UntypedVal]) -> Self::Results {
        <Results as WasmTyList>::from_values(results)
            .expect("unable to construct typed results from call results")
    }
}

/// The typed parameters of a [`TypedFunc`].
pub trait WasmParams: WasmTyList + StoreToCells {}
impl<T> WasmParams for T where T: WasmTyList + StoreToCells {}

/// The typed results of a [`TypedFunc`].
pub trait WasmResults: WasmTyList + LoadFromCells {}
impl<T> WasmResults for T where T: WasmTyList + LoadFromCells {}
