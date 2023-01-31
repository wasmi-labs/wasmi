mod caller;
mod error;
mod into_func;
mod typed_func;

pub(crate) use self::typed_func::CallResultsTuple;
pub use self::{
    caller::Caller,
    error::FuncError,
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
use crate::{core::Trap, engine::ResumableCall, Error, FuncType, Value};
use alloc::sync::Arc;
use core::{fmt, fmt::Debug, num::NonZeroU32};
use wasmi_arena::ArenaIndex;
use wasmi_core::UntypedValue;

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
    /// Creates a new host function from the given closure.
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

/// A nullable [`Func`] reference.
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct FuncRef {
    inner: Option<Func>,
}

impl From<Func> for FuncRef {
    fn from(func: Func) -> Self {
        Self::new(func)
    }
}

/// Type used to convert between [`FuncRef`] and [`UntypedValue`].
union Transposer {
    funcref: FuncRef,
    untyped: UntypedValue,
}

#[test]
fn funcref_sizeof() {
    // These assertions are important in order to convert `FuncRef`
    // from and to 64-bit `UntypedValue` instances.
    //
    // The following equation must be true:
    //     size_of(Func) == size_of(UntypedValue) == size_of(FuncRef)
    use core::mem::size_of;
    assert_eq!(size_of::<Func>(), size_of::<UntypedValue>());
    assert_eq!(size_of::<Func>(), size_of::<FuncRef>());
}

#[test]
fn funcref_null_to_zero() {
    assert_eq!(UntypedValue::from(FuncRef::null()), UntypedValue::from(0));
    assert!(FuncRef::from(UntypedValue::from(0)).is_null());
}

impl From<UntypedValue> for FuncRef {
    fn from(untyped: UntypedValue) -> Self {
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`FuncRef`] instances. Therefore
        //         this operation cannot produce invalid [`FuncRef`]
        //         instances even though the input [`UntypedValue`]
        //         was modified arbitrarily.
        unsafe { Transposer { untyped }.funcref }.canonicalize()
    }
}

impl From<FuncRef> for UntypedValue {
    fn from(funcref: FuncRef) -> Self {
        let funcref = funcref.canonicalize();
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`UntypedValue`] instances. Therefore
        //         this operation cannot produce invalid [`UntypedValue`]
        //         instances even if it was possible to arbitrarily modify
        //         the input [`FuncRef`] instance.
        unsafe { Transposer { funcref }.untyped }
    }
}

impl FuncRef {
    /// Returns `true` if [`FuncRef`] is `null`.
    pub fn is_null(&self) -> bool {
        self.inner.is_none()
    }

    /// Canonicalize `self` so that all `null` values have the same representation.
    ///
    /// # Note
    ///
    /// The underlying issue is that `FuncRef` has many possible values for the
    /// `null` value. However, to simplify operating on encoded `FuncRef` instances
    /// (encoded as `UntypedValue`) we want it to encode to exactly one `null`
    /// value. The most trivial of all possible `null` values is `0_u64`, therefore
    /// we canonicalize all `null` values to be represented by `0_u64`.
    fn canonicalize(self) -> Self {
        if self.is_null() {
            // Safety: This is safe since `0u64` can be bit
            //         interpreted as a valid `FuncRef` value.
            return unsafe {
                Transposer {
                    untyped: UntypedValue::from(0u64),
                }
                .funcref
            };
        }
        self
    }

    /// Creates a new [`FuncRef`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use wasmi::{Func, FuncRef, Store, Engine};
    /// # let engine = Engine::default();
    /// # let mut store = <Store<()>>::new(&engine, ());
    /// assert!(FuncRef::new(None).is_null());
    /// assert!(FuncRef::new(Func::wrap(&mut store, |x: i32| x)).func().is_some());
    /// ```
    pub fn new(nullable_func: impl Into<Option<Func>>) -> Self {
        Self {
            inner: nullable_func.into(),
        }
        .canonicalize()
    }

    /// Returns the inner [`Func`] if [`FuncRef`] is not `null`.
    ///
    /// Otherwise returns `None`.
    pub fn func(&self) -> Option<&Func> {
        self.inner.as_ref()
    }

    /// Creates a `null` [`FuncRef`].
    pub fn null() -> Self {
        Self::new(None).canonicalize()
    }
}
