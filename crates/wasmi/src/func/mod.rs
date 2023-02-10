mod caller;
mod error;
mod func_type;
mod funcref;
mod into_func;
mod typed_func;

pub(crate) use self::typed_func::CallResultsTuple;
pub use self::{
    caller::Caller,
    error::FuncError,
    func_type::FuncType,
    funcref::FuncRef,
    into_func::{IntoFunc, WasmRet, WasmType, WasmTypeList},
    typed_func::{TypedFunc, WasmParams, WasmResults},
};
use super::{
    engine::{DedupFuncType, FuncBody, FuncParams, FuncResults},
    AsContext,
    AsContextMut,
    Instance,
    StoreContext,
    Stored,
};
use crate::{core::Trap, engine::ResumableCall, Error, Value};
use alloc::sync::Arc;
use core::{fmt, fmt::Debug, num::NonZeroU32};
use wasmi_arena::ArenaIndex;

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

/// A function instance.
#[derive(Debug)]
pub struct FuncEntity<T> {
    /// We wrap this enum in a struct so that we can make its
    /// variants private. This is advantageous since they are
    /// implementation details and not important to the user.
    internal: FuncEntityInternal<T>,
}

impl<T> Clone for FuncEntity<T> {
    fn clone(&self) -> Self {
        Self {
            internal: self.internal.clone(),
        }
    }
}

impl<T> FuncEntity<T> {
    /// Creates a new Wasm function from the given raw parts.
    pub fn new_wasm(signature: DedupFuncType, body: FuncBody, instance: Instance) -> Self {
        Self {
            internal: FuncEntityInternal::Wasm(WasmFuncEntity::new(signature, body, instance)),
        }
    }

    /// Creates a new host function from the given dynamically typed closure.
    pub fn new(
        ctx: impl AsContextMut<UserState = T>,
        ty: FuncType,
        func: impl Fn(Caller<'_, T>, &[Value], &mut [Value]) -> Result<(), Trap> + Send + Sync + 'static,
    ) -> Self {
        Self {
            internal: FuncEntityInternal::Host(HostFuncEntity::new(ctx, ty, func)),
        }
    }

    /// Creates a new host function from the given dynamically typed closure.
    pub fn wrap<Params, Results>(
        ctx: impl AsContextMut<UserState = T>,
        func: impl IntoFunc<T, Params, Results>,
    ) -> Self {
        Self {
            internal: FuncEntityInternal::Host(HostFuncEntity::wrap(ctx, func)),
        }
    }

    /// Returns the internal function entity.
    ///
    /// # Note
    ///
    /// This can be used to efficiently match against host or Wasm
    /// function entities and efficiently extract their properties.
    pub(crate) fn as_internal(&self) -> &FuncEntityInternal<T> {
        &self.internal
    }

    /// Returns the signature of the Wasm function.
    pub fn ty_dedup(&self) -> &DedupFuncType {
        match self.as_internal() {
            FuncEntityInternal::Wasm(func) => func.ty_dedup(),
            FuncEntityInternal::Host(func) => func.ty_dedup(),
        }
    }
}

/// The internal representation of a function instance.
///
/// This can either be a host function or a Wasm function.
#[derive(Debug)]
pub(crate) enum FuncEntityInternal<T> {
    /// A Wasm function instance.
    Wasm(WasmFuncEntity),
    /// A host function instance.
    Host(HostFuncEntity<T>),
}

impl<T> Clone for FuncEntityInternal<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Wasm(func) => Self::Wasm(func.clone()),
            Self::Host(func) => Self::Host(func.clone()),
        }
    }
}

/// A Wasm function instance.
#[derive(Debug, Clone)]
pub(crate) struct WasmFuncEntity {
    signature: DedupFuncType,
    body: FuncBody,
    instance: Instance,
}

impl WasmFuncEntity {
    /// Creates a new Wasm function from the given raw parts.
    pub fn new(signature: DedupFuncType, body: FuncBody, instance: Instance) -> Self {
        Self {
            signature,
            body,
            instance,
        }
    }

    /// Returns the signature of the Wasm function.
    pub fn ty_dedup(&self) -> &DedupFuncType {
        &self.signature
    }

    /// Returns the instance where the [`Func`] belong to.
    pub fn instance(&self) -> Instance {
        self.instance
    }

    /// Returns the Wasm function body of the [`Func`].
    pub fn func_body(&self) -> FuncBody {
        self.body
    }
}

/// A host function instance.
pub(crate) struct HostFuncEntity<T> {
    signature: DedupFuncType,
    trampoline: HostFuncTrampoline<T>,
}

impl<T> Clone for HostFuncEntity<T> {
    fn clone(&self) -> Self {
        Self {
            signature: self.signature,
            trampoline: self.trampoline.clone(),
        }
    }
}

type HostFuncTrampolineFn<T> =
    dyn Fn(Caller<T>, FuncParams) -> Result<FuncResults, Trap> + Send + Sync + 'static;

pub struct HostFuncTrampoline<T> {
    closure: Arc<HostFuncTrampolineFn<T>>,
}

impl<T> HostFuncTrampoline<T> {
    /// Creates a new [`HostFuncTrampoline`] from the given trampoline function.
    pub fn new<F>(trampoline: F) -> Self
    where
        F: Fn(Caller<T>, FuncParams) -> Result<FuncResults, Trap> + Send + Sync + 'static,
    {
        Self {
            closure: Arc::new(trampoline),
        }
    }
}

impl<T> Clone for HostFuncTrampoline<T> {
    fn clone(&self) -> Self {
        Self {
            closure: self.closure.clone(),
        }
    }
}

impl<T> Debug for HostFuncEntity<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.signature, f)
    }
}

impl<T> HostFuncEntity<T> {
    /// Creates a new host function from the given dynamically typed closure.
    pub fn new(
        mut ctx: impl AsContextMut<UserState = T>,
        ty: FuncType,
        func: impl Fn(Caller<'_, T>, &[Value], &mut [Value]) -> Result<(), Trap> + Send + Sync + 'static,
    ) -> Self {
        // Preprocess parameters and results buffers so that we can reuse those
        // computations within the closure implementation. We put both parameters
        // and results into a single buffer which we can split to minimize the
        // amount of allocations per trampoline invokation.
        let params_iter = ty.params().iter().copied().map(Value::default);
        let results_iter = ty.results().iter().copied().map(Value::default);
        let len_params = ty.params().len();
        let params_results: Box<[Value]> = params_iter.chain(results_iter).collect();
        let trampoline = <HostFuncTrampoline<T>>::new(move |caller, args| {
            // We are required to clone the buffer because we are operating within a `Fn`.
            // This way the trampoline closure only has to own a single slice buffer.
            // Note: An alternative solution is to use interior mutability but that solution
            //       comes with its own downsides.
            let mut params_results = params_results.clone();
            let (params, results) = params_results.split_at_mut(len_params);
            args.decode_params_into_slice(params).unwrap();
            func(caller, params, results)?;
            Ok(args.encode_results_from_slice(results).unwrap())
        });
        let signature = ctx.as_context_mut().store.inner.alloc_func_type(ty.clone());
        Self {
            signature,
            trampoline,
        }
    }

    /// Creates a new host function from the given statically typed closure.
    pub fn wrap<Params, Results>(
        mut ctx: impl AsContextMut,
        func: impl IntoFunc<T, Params, Results>,
    ) -> Self {
        let (signature, trampoline) = func.into_func();
        let signature = ctx.as_context_mut().store.inner.alloc_func_type(signature);
        Self {
            signature,
            trampoline,
        }
    }

    /// Returns the signature of the host function.
    pub fn ty_dedup(&self) -> &DedupFuncType {
        &self.signature
    }

    /// Calls the host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    pub fn call(
        &self,
        mut ctx: impl AsContextMut<UserState = T>,
        instance: Option<&Instance>,
        params: FuncParams,
    ) -> Result<FuncResults, Trap> {
        let caller = <Caller<T>>::new(&mut ctx, instance);
        (self.trampoline.closure)(caller, params)
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
    /// - `ty`: the signature that the given closure adheres to,
    ///         used to indicate what the inputs and outputs are.
    /// - `func`: the native code invoked whenever this Func will be called.
    ///           The closure is provided a [`Caller`] as its first argument
    ///           which allows it to query information about the [`Instance`]
    ///           that is assocaited to the call.
    ///
    /// # Note
    ///
    /// - It is the responsibility of the caller of [`Func::new`] to guarantee that
    ///   the correct amount and types of results are written into the results buffer
    ///   from the `func` closure. If an incorrect amount of results or types of results
    ///   is written into the buffer then the remaining computation may fail in unexpected
    ///   ways. This footgun can be avoided by using the typed [`Func::wrap`] method instead.
    /// - Prefer using [`Func::wrap`] over this method if possible since [`Func`] instances
    ///   created using this constructor have runtime overhead for every invokation that
    ///   can be avoided by using [`Func::wrap`].
    pub fn new<T>(
        mut ctx: impl AsContextMut<UserState = T>,
        ty: FuncType,
        func: impl Fn(Caller<'_, T>, &[Value], &mut [Value]) -> Result<(), Trap> + Send + Sync + 'static,
    ) -> Self {
        let func = FuncEntity::new(ctx.as_context_mut(), ty, func);
        ctx.as_context_mut().store.alloc_func(func)
    }

    /// Creates a new host function from the given closure.
    pub fn wrap<T, Params, Results>(
        mut ctx: impl AsContextMut<UserState = T>,
        func: impl IntoFunc<T, Params, Results>,
    ) -> Self {
        let func = FuncEntity::wrap(ctx.as_context_mut(), func);
        ctx.as_context_mut().store.alloc_func(func)
    }

    /// Returns the signature of the function.
    pub(crate) fn ty_dedup<'a, T: 'a>(
        &self,
        ctx: impl Into<StoreContext<'a, T>>,
    ) -> &'a DedupFuncType {
        ctx.into().store.resolve_func(self).ty_dedup()
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
    /// - If the function returned a [`Trap`].
    /// - If the types of the `inputs` do not match the expected types for the
    ///   function signature of `self`.
    /// - If the number of input values does not match the expected number of
    ///   inputs required by the function signature of `self`.
    /// - If the number of output values does not match the expected number of
    ///   outputs required by the function signature of `self`.
    pub fn call<T>(
        &self,
        mut ctx: impl AsContextMut<UserState = T>,
        inputs: &[Value],
        outputs: &mut [Value],
    ) -> Result<(), Error> {
        self.verify_and_prepare_inputs_outputs(ctx.as_context(), inputs, outputs)?;
        // Note: Cloning an [`Engine`] is intentionally a cheap operation.
        ctx.as_context().store.engine().clone().execute_func(
            ctx.as_context_mut(),
            *self,
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
    /// enountering host errors with which it is possible to handle
    /// the error and continue the execution as if no error occured.
    ///
    /// # Note
    ///
    /// This is a non-standard WebAssembly API and might not be available
    /// at other WebAssembly engines. Please be aware that depending on this
    /// feature might mean a lock-in to `wasmi` for users.
    ///
    /// # Errors
    ///
    /// - If the function returned a Wasm [`Trap`].
    /// - If the types of the `inputs` do not match the expected types for the
    ///   function signature of `self`.
    /// - If the number of input values does not match the expected number of
    ///   inputs required by the function signature of `self`.
    /// - If the number of output values does not match the expected number of
    ///   outputs required by the function signature of `self`.
    pub fn call_resumable<T>(
        &self,
        mut ctx: impl AsContextMut<UserState = T>,
        inputs: &[Value],
        outputs: &mut [Value],
    ) -> Result<ResumableCall, Error> {
        self.verify_and_prepare_inputs_outputs(ctx.as_context(), inputs, outputs)?;
        // Note: Cloning an [`Engine`] is intentionally a cheap operation.
        ctx.as_context()
            .store
            .engine()
            .clone()
            .execute_func_resumable(ctx.as_context_mut(), *self, inputs, outputs)
            .map_err(Into::into)
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
        inputs: &[Value],
        outputs: &mut [Value],
    ) -> Result<(), FuncError> {
        let fn_type = self.ty_dedup(ctx.as_context());
        ctx.as_context()
            .store
            .inner
            .resolve_func_type_with(fn_type, |func_type| {
                func_type.match_params(inputs)?;
                func_type.match_results(outputs, false)?;
                func_type.prepare_outputs(outputs);
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

    /// Returns the internal representation of the [`Func`] instance.
    ///
    /// # Note
    ///
    /// This is intentionally a private API and mainly provided for efficient
    /// execution of the `wasmi` interpreter upon function dispatch.
    pub(crate) fn as_internal<'a, T: 'a>(
        &self,
        ctx: impl Into<StoreContext<'a, T>>,
    ) -> &'a FuncEntityInternal<T> {
        ctx.into().store.resolve_func(self).as_internal()
    }
}
