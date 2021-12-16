mod caller;
mod into_func;
mod locals;

pub use self::caller::Caller;
pub use self::into_func::IntoFunc;
use self::locals::{Local, Locals, LocalsBuilder};
use super::{AsContext, AsContextMut};
use super::Index;
use super::Signature;
use super::Stored;
use crate::RuntimeValue;
use crate::ValueType;
use crate::{isa, Trap};
use core::fmt;
use core::fmt::Debug;

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

/// A Wasm function instance.
#[derive(Debug)]
struct WasmFuncEntity {
    signature: Signature,
    body: FuncBody,
}

impl WasmFuncEntity {
    /// Returns the signature of the Wasm function.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

/// The function body of a Wasm function.
#[derive(Debug)]
struct FuncBody {
    locals: Locals,
    code: isa::Instructions,
}

/// A host function instance.
struct HostFuncEntity<T> {
    signature: Signature,
    trampoline: HostFuncTrampoline<T>,
}

pub struct HostFuncTrampoline<T> {
    closure: Box<
        dyn Fn(Caller<T>, &[RuntimeValue], &mut [RuntimeValue]) -> Result<(), Trap>
            + Send
            + Sync
            + 'static,
    >,
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
}
