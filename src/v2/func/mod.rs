#![allow(dead_code)] // TODO: remove

mod caller;
mod into_func;
mod locals;

use self::locals::Locals;
pub use self::{caller::Caller, into_func::IntoFunc};
use super::{engine::FuncBody, AsContext, AsContextMut, Index, Signature, Stored};
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
    /// Creates a new host function from the given closure.
    pub fn wrap<Params, Results>(
        ctx: impl AsContextMut<UserState = T>,
        func: impl IntoFunc<T, Params, Results>,
    ) -> Self {
        Self {
            internal: FuncEntityInternal::Host(HostFuncEntity::wrap(ctx, func)),
        }
    }

    /// Returns the signature of the Wasm function.
    pub fn signature(&self) -> &Signature {
        match &self.internal {
            FuncEntityInternal::Wasm(func) => func.signature(),
            FuncEntityInternal::Host(func) => func.signature(),
        }
    }

    /// Calls the Wasm or host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    pub fn call(
        &self,
        ctx: impl AsContextMut<UserState = T>,
        inputs: &[RuntimeValue],
        outputs: &mut [RuntimeValue],
    ) -> Result<(), Trap> {
        match &self.internal {
            FuncEntityInternal::Wasm(_wasm_func) => {
                panic!("calling Wasm function is not yet supported")
            }
            FuncEntityInternal::Host(host_func) => host_func.call(ctx, inputs, outputs),
        }
    }
}

/// The internal representation of a function instance.
///
/// This can either be a host function or a Wasm function.
#[derive(Debug)]
enum FuncEntityInternal<T> {
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
struct WasmFuncEntity {
    signature: Signature,
    locals: Locals,
    body: FuncBody,
}

impl WasmFuncEntity {
    /// Returns the signature of the Wasm function.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

/// A host function instance.
struct HostFuncEntity<T> {
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
    pub fn signature(&self) -> &Signature {
        &self.signature
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
        *ctx.as_context().store.resolve_func(*self).signature()
    }

    /// Calls the Wasm or host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    pub fn call<T>(
        &self,
        ctx: impl AsContextMut<UserState = T>,
        inputs: &[RuntimeValue],
        outputs: &mut [RuntimeValue],
    ) -> Result<(), Trap> {
        ctx.as_context()
            .store
            .resolve_func(*self)
            // TODO: try removing this clone
            //
            // - Note that removing this `clone` will require
            //   unsafe Rust code with the current design.
            // - The invariant without clone is still safe since
            //   we safe guard the access to the underlying
            //   entities of the store in a fashion that cannot
            //   allow shared mutable access.
            .clone()
            .call(ctx, inputs, outputs)
    }
}
