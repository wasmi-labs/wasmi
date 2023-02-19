pub use super::{
    caller::Caller,
    error::FuncError,
    func_type::FuncType,
    funcref::FuncRef,
    into_func::{IntoFunc, WasmRet, WasmType, WasmTypeList},
    typed_func::{TypedFunc, WasmParams, WasmResults},
};
use crate::{
    core::Trap,
    engine::{DedupFuncType, FuncFinished, FuncParams},
    AsContextMut,
    Engine,
    Instance,
    Stored,
    Value,
};
use alloc::{boxed::Box, sync::Arc};
use core::{fmt, fmt::Debug};

/// An index uniquely identifying a host function.
#[derive(Debug, Copy, Clone)]
pub struct HostFuncIdx(usize);

/// An index uniquely identifying a host function stored in a [`Store`](crate::Store).
#[derive(Debug, Copy, Clone)]
pub struct HostFunc(Stored<HostFuncIdx>);

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
    dyn Fn(Caller<T>, FuncParams) -> Result<FuncFinished, Trap> + Send + Sync + 'static;

pub struct HostFuncTrampoline<T> {
    closure: Arc<HostFuncTrampolineFn<T>>,
}

impl<T> HostFuncTrampoline<T> {
    /// Creates a new [`HostFuncTrampoline`] from the given trampoline function.
    pub fn new<F>(trampoline: F) -> Self
    where
        F: Fn(Caller<T>, FuncParams) -> Result<FuncFinished, Trap> + Send + Sync + 'static,
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
        engine: &Engine,
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
            let func_results = args.decode_params_into_slice(params).unwrap();
            func(caller, params, results)?;
            Ok(func_results.encode_results_from_slice(results).unwrap())
        });
        let signature = engine.alloc_func_type(ty.clone());
        Self {
            signature,
            trampoline,
        }
    }

    /// Creates a new host function from the given statically typed closure.
    pub fn wrap<Params, Results>(engine: &Engine, func: impl IntoFunc<T, Params, Results>) -> Self {
        let (signature, trampoline) = func.into_func();
        let signature = engine.alloc_func_type(signature);
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
    ) -> Result<FuncFinished, Trap> {
        let caller = <Caller<T>>::new(&mut ctx, instance);
        (self.trampoline.closure)(caller, params)
    }
}
