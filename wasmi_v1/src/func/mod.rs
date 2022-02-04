mod caller;
mod error;
mod into_func;
mod typed_func;

pub use self::{
    caller::Caller,
    error::FuncError,
    into_func::IntoFunc,
    typed_func::{TypedFunc, WasmParams, WasmResults},
};
use super::{
    engine::{DedupFuncType, FuncBody, FuncParams, FuncResults},
    AsContext,
    AsContextMut,
    Index,
    Instance,
    StoreContext,
    Stored,
};
use crate::{
    core::{Trap, Value},
    Error,
    FuncType,
};
use alloc::sync::Arc;
use core::{fmt, fmt::Debug};

/// A raw index to a function entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FuncIdx(usize);

impl Index for FuncIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
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
    pub(crate) fn new_wasm(signature: DedupFuncType, body: FuncBody, instance: Instance) -> Self {
        Self {
            internal: FuncEntityInternal::Wasm(WasmFuncEntity::new(signature, body, instance)),
        }
    }

    /// Creates a new host function from the given closure.
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
    pub fn signature(&self) -> DedupFuncType {
        match self.as_internal() {
            FuncEntityInternal::Wasm(func) => func.signature(),
            FuncEntityInternal::Host(func) => func.signature(),
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
    pub fn signature(&self) -> DedupFuncType {
        self.signature
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
        F: Fn(Caller<T>, FuncParams) -> Result<FuncResults, Trap>,
        F: Send + Sync + 'static,
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
    /// Creates a new host function from the given closure.
    pub fn wrap<Params, Results>(
        mut ctx: impl AsContextMut,
        func: impl IntoFunc<T, Params, Results>,
    ) -> Self {
        let (signature, trampoline) = func.into_func();
        let signature = ctx.as_context_mut().store.alloc_func_type(signature);
        Self {
            signature,
            trampoline,
        }
    }

    /// Returns the signature of the host function.
    pub fn signature(&self) -> DedupFuncType {
        self.signature
    }

    /// Calls the host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    pub fn call(
        &self,
        mut ctx: impl AsContextMut<UserState = T>,
        instance: Option<Instance>,
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
    pub(super) fn into_inner(self) -> Stored<FuncIdx> {
        self.0
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
    pub(crate) fn signature(&self, ctx: impl AsContext) -> DedupFuncType {
        ctx.as_context().store.resolve_func(*self).signature()
    }

    /// Returns the function type of the [`Func`].
    pub fn func_type(&self, ctx: impl AsContext) -> FuncType {
        ctx.as_context()
            .store
            .resolve_func_type(self.signature(&ctx))
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
        // Since [`Func`] is a dynamically typed function instance there is
        // a need to verify that the given input parameters match the required
        // types and that the given output slice matches the expected length.
        //
        // These checks can be avoided using the [`TypedFunc`] API.
        let func_type = self.func_type(&ctx);
        let (expected_inputs, expected_outputs) = func_type.params_results();
        let actual_inputs = inputs.iter().map(|value| value.value_type());
        if expected_inputs.iter().copied().ne(actual_inputs) {
            return Err(FuncError::MismatchingParameters { func: *self }).map_err(Into::into);
        }
        if expected_outputs.len() != outputs.len() {
            return Err(FuncError::MismatchingResults { func: *self }).map_err(Into::into);
        }
        // Note: Cloning an [`Engine`] is intentionally a cheap operation.
        ctx.as_context().store.engine().clone().execute_func(
            ctx.as_context_mut(),
            *self,
            inputs,
            outputs,
        )?;
        Ok(())
    }

    /// Creates a new [`TypedFunc`] from this [`Func`].
    ///
    /// # Note
    ///
    /// This performs static type checks given `Params` as parameter types
    /// to [`Func`] and `Results` as result types of [`Func`] so that those
    /// type checks can be avoided when calling the created [`TypedFunc`].
    pub fn typed<Params, Results, S>(&self, ctx: S) -> Result<TypedFunc<Params, Results>, Error>
    where
        Params: WasmParams,
        Results: WasmResults,
        S: AsContext,
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
        ctx.into().store.resolve_func(*self).as_internal()
    }
}
