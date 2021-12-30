#![allow(dead_code)] // TODO: remove

mod caller;
mod into_func;

pub use self::{caller::Caller, into_func::IntoFunc};
use super::{
    engine::FuncBody,
    AsContext,
    AsContextMut,
    Index,
    Instance,
    Signature,
    StoreContext,
    Stored,
};
use crate::{RuntimeValue, Trap, ValueType};
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
    pub(crate) fn new_wasm(signature: Signature, body: FuncBody, instance: Instance) -> Self {
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
    pub fn signature(&self) -> Signature {
        match self.as_internal() {
            FuncEntityInternal::Wasm(func) => func.signature(),
            FuncEntityInternal::Host(func) => func.signature(),
        }
    }

    /// Returns the associated [`Instance`] of the [`Func`] if any.
    ///
    /// # Note
    ///
    /// All Wasm functions have an associated [`Instance`].
    pub(crate) fn instance(&self) -> Option<Instance> {
        match self.as_internal() {
            FuncEntityInternal::Wasm(func) => func.instance().into(),
            FuncEntityInternal::Host(_) => None,
        }
    }

    /// Returns the associated Wasm function body of the [`Func`] if any.
    ///
    /// # Note
    ///
    /// All Wasm functions have an associated Wasm function body.
    pub(crate) fn func_body(&self) -> Option<FuncBody> {
        match self.as_internal() {
            FuncEntityInternal::Wasm(func) => func.func_body().into(),
            FuncEntityInternal::Host(_) => None,
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
    signature: Signature,
    body: FuncBody,
    instance: Instance,
}

impl WasmFuncEntity {
    /// Creates a new Wasm function from the given raw parts.
    pub fn new(signature: Signature, body: FuncBody, instance: Instance) -> Self {
        Self {
            signature,
            body,
            instance,
        }
    }

    /// Returns the signature of the Wasm function.
    pub fn signature(&self) -> Signature {
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
    signature: Signature,
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

type HostFuncTrampolineFn<T> = dyn Fn(Caller<T>, &[RuntimeValue], &mut [RuntimeValue]) -> Result<(), Trap>
    + Send
    + Sync
    + 'static;

pub struct HostFuncTrampoline<T> {
    closure: Arc<HostFuncTrampolineFn<T>>,
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
        let signature = ctx.as_context_mut().store.alloc_signature(signature);
        Self {
            signature,
            trampoline,
        }
    }

    /// Returns the signature of the host function.
    pub fn signature(&self) -> Signature {
        self.signature
    }

    /// Calls the host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    pub fn call(
        &self,
        mut ctx: impl AsContextMut<UserState = T>,
        inputs: &[RuntimeValue],
        outputs: &mut [RuntimeValue],
    ) -> Result<(), Trap> {
        let caller = <Caller<T>>::from(&mut ctx);
        (self.trampoline.closure)(caller, inputs, outputs)
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
    pub fn signature(&self, ctx: impl AsContext) -> Signature {
        ctx.as_context().store.resolve_func(*self).signature()
    }

    /// Calls the Wasm or host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    pub fn call<T>(
        &self,
        mut ctx: impl AsContextMut<UserState = T>,
        inputs: &[RuntimeValue],
        outputs: &mut [RuntimeValue],
    ) -> Result<(), Trap> {
        // Cloning an engine is a cheap operation.
        ctx.as_context().store.engine().clone().execute_func(
            ctx.as_context_mut(),
            *self,
            inputs,
            outputs,
        )
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
