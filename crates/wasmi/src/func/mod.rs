mod caller;
mod error;
mod func_inout;
mod into_func;
mod ty;
mod typed_func;

use self::func_inout::FuncFinished;
pub(crate) use self::typed_func::CallResultsTuple;
pub use self::{
    caller::Caller,
    error::FuncError,
    func_inout::FuncInOut,
    into_func::{IntoFunc, WasmRet, WasmTy, WasmTyList},
    ty::FuncType,
    typed_func::{TypedFunc, WasmParams, WasmResults},
};
use super::{
    engine::{DedupFuncType, EngineFunc},
    AsContext,
    AsContextMut,
    Instance,
    StoreContext,
    Stored,
};
use crate::{collections::arena::ArenaIndex, engine::ResumableCall, Engine, Error, Val};
use alloc::{boxed::Box, sync::Arc};
use core::{fmt, fmt::Debug, num::NonZeroU32};

/// A raw index to a function entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FuncIdx(NonZeroU32);

impl ArenaIndex for FuncIdx {
    fn into_usize(self) -> usize {
        self.0.get().wrapping_sub(1) as usize
    }

    fn from_usize(index: usize) -> Self {
        index
            .try_into()
            .ok()
            .map(|index: u32| index.wrapping_add(1))
            .and_then(NonZeroU32::new)
            .map(Self)
            .unwrap_or_else(|| panic!("out of bounds func index {index}"))
    }
}

/// A raw index to a host function trampoline entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrampolineIdx(usize);

impl ArenaIndex for TrampolineIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(index: usize) -> Self {
        Self(index)
    }
}

/// A host function reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Trampoline(Stored<TrampolineIdx>);

impl Trampoline {
    /// Creates a new host function reference.
    pub(super) fn from_inner(stored: Stored<TrampolineIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn as_inner(&self) -> &Stored<TrampolineIdx> {
        &self.0
    }
}

/// A Wasm or host function instance.
#[derive(Debug)]
pub enum FuncEntity {
    /// A Wasm function.
    Wasm(WasmFuncEntity),
    /// A host function.
    Host(HostFuncEntity),
}

impl From<WasmFuncEntity> for FuncEntity {
    fn from(func: WasmFuncEntity) -> Self {
        Self::Wasm(func)
    }
}

impl From<HostFuncEntity> for FuncEntity {
    fn from(func: HostFuncEntity) -> Self {
        Self::Host(func)
    }
}

/// A host function reference and its function type.
#[derive(Debug, Copy, Clone)]
pub struct HostFuncEntity {
    /// The number of parameters of the [`HostFuncEntity`].
    len_params: u16,
    /// The number of results of the [`HostFuncEntity`].
    len_results: u16,
    /// The function type of the host function.
    ty: DedupFuncType,
    /// A reference to the trampoline of the host function.
    func: Trampoline,
}

impl HostFuncEntity {
    /// Creates a new [`HostFuncEntity`].
    pub fn new(engine: &Engine, ty: &FuncType, func: Trampoline) -> Self {
        let len_params = ty.len_params();
        let len_results = ty.len_results();
        let ty = engine.alloc_func_type(ty.clone());
        Self {
            len_params,
            len_results,
            ty,
            func,
        }
    }

    /// Returns the number of parameters of the [`HostFuncEntity`].
    pub fn len_params(&self) -> u16 {
        self.len_params
    }

    /// Returns the number of results of the [`HostFuncEntity`].
    pub fn len_results(&self) -> u16 {
        self.len_results
    }

    /// Returns the signature of the host function.
    pub fn ty_dedup(&self) -> &DedupFuncType {
        &self.ty
    }

    /// Returns the [`Trampoline`] of the host function.
    pub fn trampoline(&self) -> &Trampoline {
        &self.func
    }
}

impl FuncEntity {
    /// Returns the signature of the Wasm function.
    pub fn ty_dedup(&self) -> &DedupFuncType {
        match self {
            Self::Wasm(func) => func.ty_dedup(),
            Self::Host(func) => func.ty_dedup(),
        }
    }
}

/// A Wasm function instance.
#[derive(Debug, Clone)]
pub struct WasmFuncEntity {
    /// The function type of the Wasm function.
    ty: DedupFuncType,
    /// The compiled function body of the Wasm function.
    body: EngineFunc,
    /// The instance associated to the Wasm function.
    instance: Instance,
}

impl WasmFuncEntity {
    /// Creates a new Wasm function from the given raw parts.
    pub fn new(signature: DedupFuncType, body: EngineFunc, instance: Instance) -> Self {
        Self {
            ty: signature,
            body,
            instance,
        }
    }

    /// Returns the signature of the Wasm function.
    pub fn ty_dedup(&self) -> &DedupFuncType {
        &self.ty
    }

    /// Returns the instance where the [`Func`] belong to.
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Returns the Wasm function body of the [`Func`].
    pub fn func_body(&self) -> EngineFunc {
        self.body
    }
}

/// A host function instance.
pub struct HostFuncTrampolineEntity<T> {
    /// The type of the associated host function.
    ty: FuncType,
    /// The trampoline of the associated host function.
    trampoline: TrampolineEntity<T>,
}

impl<T> Clone for HostFuncTrampolineEntity<T> {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            trampoline: self.trampoline.clone(),
        }
    }
}

impl<T> Debug for HostFuncTrampolineEntity<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.ty, f)
    }
}

impl<T> HostFuncTrampolineEntity<T> {
    /// Creates a new host function trampoline from the given dynamically typed closure.
    pub fn new(
        // engine: &Engine,
        ty: FuncType,
        func: impl Fn(Caller<'_, T>, &[Val], &mut [Val]) -> Result<(), Error> + Send + Sync + 'static,
    ) -> Self {
        // Preprocess parameters and results buffers so that we can reuse those
        // computations within the closure implementation. We put both parameters
        // and results into a single buffer which we can split to minimize the
        // amount of allocations per trampoline invocation.
        let params_iter = ty.params().iter().copied().map(Val::default);
        let results_iter = ty.results().iter().copied().map(Val::default);
        let len_params = ty.params().len();
        let params_results: Box<[Val]> = params_iter.chain(results_iter).collect();
        let trampoline = <TrampolineEntity<T>>::new(move |caller, args| {
            // We are required to clone the buffer because we are operating within a `Fn`.
            // This way the trampoline closure only has to own a single slice buffer.
            // Note: An alternative solution is to use interior mutability but that solution
            //       comes with its own downsides.
            let mut params_results = params_results.clone();
            let (params, results) = params_results.split_at_mut(len_params);
            let func_results = args.decode_params_into_slice(params).unwrap();
            func(caller, params, results)?;
            Ok(func_results.encode_results_from_slice(results).unwrap())
        });
        Self { ty, trampoline }
    }

    /// Creates a new host function trampoline from the given statically typed closure.
    pub fn wrap<Params, Results>(func: impl IntoFunc<T, Params, Results>) -> Self {
        let (ty, trampoline) = func.into_func();
        // let ty = engine.alloc_func_type(signature);
        Self { ty, trampoline }
    }

    /// Returns the [`FuncType`] of the host function.
    pub fn func_type(&self) -> &FuncType {
        &self.ty
    }

    /// Returns the trampoline of the host function.
    pub fn trampoline(&self) -> &TrampolineEntity<T> {
        &self.trampoline
    }
}

type TrampolineFn<T> =
    dyn Fn(Caller<T>, FuncInOut) -> Result<FuncFinished, Error> + Send + Sync + 'static;

pub struct TrampolineEntity<T> {
    closure: Arc<TrampolineFn<T>>,
}

impl<T> Debug for TrampolineEntity<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrampolineEntity").finish()
    }
}

impl<T> TrampolineEntity<T> {
    /// Creates a new [`TrampolineEntity`] from the given host function.
    pub fn new<F>(trampoline: F) -> Self
    where
        F: Fn(Caller<T>, FuncInOut) -> Result<FuncFinished, Error> + Send + Sync + 'static,
    {
        Self {
            closure: Arc::new(trampoline),
        }
    }

    /// Calls the host function trampoline with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    pub fn call(
        &self,
        mut ctx: impl AsContextMut<Data = T>,
        instance: Option<&Instance>,
        params: FuncInOut,
    ) -> Result<FuncFinished, Error> {
        let caller = <Caller<T>>::new(&mut ctx, instance);
        (self.closure)(caller, params)
    }
}

impl<T> Clone for TrampolineEntity<T> {
    fn clone(&self) -> Self {
        Self {
            closure: self.closure.clone(),
        }
    }
}

/// A Wasm or host function reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Func(Stored<FuncIdx>);

impl Func {
    /// Creates a new Wasm or host function reference.
    pub(super) fn from_inner(stored: Stored<FuncIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn as_inner(&self) -> &Stored<FuncIdx> {
        &self.0
    }

    /// Creates a new [`Func`] with the given arguments.
    ///
    /// This is typically used to create a host-defined function to pass as an import to a Wasm module.
    ///
    /// - `ty`:
    ///   The signature that the given closure adheres to,
    ///   used to indicate what the inputs and outputs are.
    /// - `func`:
    ///   The native code invoked whenever this Func will be called.
    ///   The closure is provided a [`Caller`] as its first argument
    ///   which allows it to query information about the [`Instance`]
    ///   that is associated to the call.
    ///
    /// # Note
    ///
    /// - The given [`FuncType`] `ty` must match the parameters and results otherwise
    ///   the resulting host [`Func`] might trap during execution.
    /// - It is the responsibility of the caller of [`Func::new`] to guarantee that
    ///   the correct amount and types of results are written into the results buffer
    ///   from the `func` closure. If an incorrect amount of results or types of results
    ///   is written into the buffer then the remaining computation may fail in unexpected
    ///   ways. This footgun can be avoided by using the typed [`Func::wrap`] method instead.
    /// - Prefer using [`Func::wrap`] over this method if possible since [`Func`] instances
    ///   created using this constructor have runtime overhead for every invocation that
    ///   can be avoided by using [`Func::wrap`].
    pub fn new<T>(
        mut ctx: impl AsContextMut<Data = T>,
        ty: FuncType,
        func: impl Fn(Caller<'_, T>, &[Val], &mut [Val]) -> Result<(), Error> + Send + Sync + 'static,
    ) -> Self {
        let host_func = HostFuncTrampolineEntity::new(ty.clone(), func);
        let trampoline = host_func.trampoline().clone();
        let func = ctx.as_context_mut().store.alloc_trampoline(trampoline);
        let host_func = HostFuncEntity::new(ctx.as_context().engine(), &ty, func);
        ctx.as_context_mut()
            .store
            .inner
            .alloc_func(host_func.into())
    }

    /// Creates a new host function from the given closure.
    pub fn wrap<T, Params, Results>(
        mut ctx: impl AsContextMut<Data = T>,
        func: impl IntoFunc<T, Params, Results>,
    ) -> Self {
        let host_func = HostFuncTrampolineEntity::wrap(func);
        let ty = host_func.func_type();
        let trampoline = host_func.trampoline().clone();
        let func = ctx.as_context_mut().store.alloc_trampoline(trampoline);
        let host_func = HostFuncEntity::new(ctx.as_context().engine(), ty, func);
        ctx.as_context_mut()
            .store
            .inner
            .alloc_func(host_func.into())
    }

    /// Returns the signature of the function.
    pub(crate) fn ty_dedup<'a, T: 'a>(
        &self,
        ctx: impl Into<StoreContext<'a, T>>,
    ) -> &'a DedupFuncType {
        ctx.into().store.inner.resolve_func(self).ty_dedup()
    }

    /// Returns the function type of the [`Func`].
    pub fn ty(&self, ctx: impl AsContext) -> FuncType {
        ctx.as_context()
            .store
            .inner
            .resolve_func_type(self.ty_dedup(&ctx))
    }

    /// Calls the Wasm or host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    ///
    /// # Errors
    ///
    /// - If the function returned a [`Error`].
    /// - If the types of the `inputs` do not match the expected types for the
    ///   function signature of `self`.
    /// - If the number of input values does not match the expected number of
    ///   inputs required by the function signature of `self`.
    /// - If the number of output values does not match the expected number of
    ///   outputs required by the function signature of `self`.
    pub fn call<T>(
        &self,
        mut ctx: impl AsContextMut<Data = T>,
        inputs: &[Val],
        outputs: &mut [Val],
    ) -> Result<(), Error> {
        self.verify_and_prepare_inputs_outputs(ctx.as_context(), inputs, outputs)?;
        // Note: Cloning an [`Engine`] is intentionally a cheap operation.
        ctx.as_context().store.engine().clone().execute_func(
            ctx.as_context_mut(),
            self,
            inputs,
            outputs,
        )?;
        Ok(())
    }

    /// Calls the Wasm or host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
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
    /// - If the function returned a Wasm [`Error`].
    /// - If the types of the `inputs` do not match the expected types for the
    ///   function signature of `self`.
    /// - If the number of input values does not match the expected number of
    ///   inputs required by the function signature of `self`.
    /// - If the number of output values does not match the expected number of
    ///   outputs required by the function signature of `self`.
    pub fn call_resumable<T>(
        &self,
        mut ctx: impl AsContextMut<Data = T>,
        inputs: &[Val],
        outputs: &mut [Val],
    ) -> Result<ResumableCall, Error> {
        self.verify_and_prepare_inputs_outputs(ctx.as_context(), inputs, outputs)?;
        // Note: Cloning an [`Engine`] is intentionally a cheap operation.
        ctx.as_context()
            .store
            .engine()
            .clone()
            .execute_func_resumable(ctx.as_context_mut(), self, inputs, outputs)
            .map(ResumableCall::new)
    }

    /// Verify that the `inputs` and `outputs` value types match the function signature.
    ///
    /// Since [`Func`] is a dynamically typed function instance there is
    /// a need to verify that the given input parameters match the required
    /// types and that the given output slice matches the expected length.
    ///
    /// These checks can be avoided using the [`TypedFunc`] API.
    ///
    /// # Errors
    ///
    /// - If the `inputs` value types do not match the function input types.
    /// - If the number of `inputs` do not match the function input types.
    /// - If the number of `outputs` do not match the function output types.
    fn verify_and_prepare_inputs_outputs(
        &self,
        ctx: impl AsContext,
        inputs: &[Val],
        outputs: &mut [Val],
    ) -> Result<(), FuncError> {
        let fn_type = self.ty_dedup(ctx.as_context());
        ctx.as_context()
            .store
            .inner
            .resolve_func_type_with(fn_type, |func_type| {
                func_type.match_params(inputs)?;
                func_type.prepare_outputs(outputs)?;
                Ok(())
            })
    }

    /// Creates a new [`TypedFunc`] from this [`Func`].
    ///
    /// # Note
    ///
    /// This performs static type checks given `Params` as parameter types
    /// to [`Func`] and `Results` as result types of [`Func`] so that those
    /// type checks can be avoided when calling the created [`TypedFunc`].
    ///
    /// # Errors
    ///
    /// If the function signature of `self` does not match `Params` and `Results`
    /// as parameter types and result types respectively.
    pub fn typed<Params, Results>(
        &self,
        ctx: impl AsContext,
    ) -> Result<TypedFunc<Params, Results>, Error>
    where
        Params: WasmParams,
        Results: WasmResults,
    {
        TypedFunc::new(ctx, *self)
    }
}
