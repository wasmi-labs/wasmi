mod caller;
mod into_func;
mod locals;

pub use self::caller::Caller;
pub use self::into_func::IntoFunc;
use self::locals::{Local, Locals, LocalsBuilder};
use super::Index;
use super::Signature;
use super::Stored;
use super::{AsContextMut, StoreContextMut};
use crate::RuntimeValue;
use crate::TrapKind;
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
    /// Create a new host function with the given signature.
    ///
    /// # Note
    ///
    /// Dynamically checks upon calling the host function if the given inputs
    /// and the resulting outputs match the host function signature.
    /// This dynamic checking is kind of costly compared to a statically typed
    /// version of the host function.
    pub fn new_host<'a, F>(signature: Signature, func: F) -> Self
    where
        T: 'a,
        F: Fn(Caller<'_, T>, &[RuntimeValue], &mut [RuntimeValue]) -> Result<(), Trap>
            + Send
            + Sync
            + 'static,
    {
        #[rustfmt::skip]
        let trampoline = move |
            mut caller: Caller<T>,
            inputs: &[RuntimeValue],
            outputs: &mut [RuntimeValue],
        | -> Result<(), Trap> {
            let ctx = caller.as_context_mut();
            let expected_inputs = signature.inputs(&ctx);
            // Need to put expected outputs into `Vec` to avoid lifetime issues.
            let expected_outputs = signature.outputs(&ctx).to_vec();
            let inputs_match = inputs
                .iter()
                .map(RuntimeValue::value_type)
                .ne(expected_inputs.iter().copied());
            let outputs_match = outputs.len() == expected_outputs.len();
            if !(inputs_match && outputs_match) {
                // Bail out due to one of the following cases:
                //
                // - The length of the given inputs do not match the signature.
                // - The length of the given outputs do not match the signature.
                // - The type of the inputs do not match the signature.
                return Err(Trap::new(TrapKind::UnexpectedSignature))
            }
            func(caller, inputs, outputs)?;
            let outputs_match = outputs
                .iter()
                .map(RuntimeValue::value_type)
                .ne(expected_outputs);
            if !outputs_match {
                // The returned output types do not match the function signature.
                // This is likely a bug in the provided host function closure.
                return Err(Trap::new(TrapKind::UnexpectedSignature))
            }
            Ok(())
        };
        Self {
            internal: FuncEntityInternal::Host(HostFuncEntity {
                signature,
                trampoline: HostFuncTrampoline {
                    closure: Box::new(trampoline),
                },
            }),
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

    /// Create a new host function with the given signature.
    ///
    /// # Note
    ///
    /// Dynamically checks upon calling the host function if the given inputs
    /// and the resulting outputs match the host function signature.
    /// This dynamic checking is kind of costly compared to a statically typed
    /// version of the host function.
    pub fn new_host<'a, F, T>(
        mut ctx: impl AsContextMut<UserState = T>,
        signature: Signature,
        func: F,
    ) -> Self
    where
        T: 'a,
        F: Fn(Caller<'_, T>, &[RuntimeValue], &mut [RuntimeValue]) -> Result<(), Trap>
            + Send
            + Sync
            + 'static,
    {
        ctx.as_context_mut()
            .store
            .alloc_func(FuncEntity::new_host(signature, func))
    }

    /// Creates a new host function from the given closure.
    pub fn wrap<T, Params, Results>(
        mut ctx: impl AsContextMut<UserState = T>,
        func: impl IntoFunc<T, Params, Results>,
    ) -> Self {
        let func = FuncEntity::wrap(ctx.as_context_mut(), func);
        ctx.as_context_mut().store.alloc_func(func)
    }
}
