use super::{bytecode::RegisterSpan, Func};
use crate::{engine::Stack, func::CallResultsTuple, AsContextMut, Engine, Error, Val, WasmResults};
use core::{fmt, marker::PhantomData, mem::replace, ops::Deref};

/// Returned by [`Engine`] methods for calling a function in a resumable way.
///
/// # Note
///
/// This is the base type for resumable call results and can be converted into
/// either the dynamically typed [`ResumableCall`] or the statically typed
/// [`TypedResumableCall`] that act as user facing API. Therefore this type
/// must provide all the information necessary to be properly converted into
/// either user facing types.
#[derive(Debug)]
pub(crate) enum ResumableCallBase<T> {
    /// The resumable call has finished properly and returned a result.
    Finished(T),
    /// The resumable call encountered a host error and can be resumed.
    Resumable(ResumableInvocation),
}

/// Returned by calling a [`Func`] in a resumable way.
#[derive(Debug)]
pub enum ResumableCall {
    /// The resumable call has finished properly and returned a result.
    Finished,
    /// The resumable call encountered a host error and can be resumed.
    Resumable(ResumableInvocation),
}

impl ResumableCall {
    /// Creates a [`ResumableCall`] from the [`Engine`]'s base [`ResumableCallBase`].
    pub(crate) fn new(call: ResumableCallBase<()>) -> Self {
        match call {
            ResumableCallBase::Finished(()) => Self::Finished,
            ResumableCallBase::Resumable(invocation) => Self::Resumable(invocation),
        }
    }
}

/// State required to resume a [`Func`] invocation.
#[derive(Debug)]
pub struct ResumableInvocation {
    /// The engine in use for the function invocation.
    ///
    /// # Note
    ///
    /// - This handle is required to resolve function types
    ///   of both `func` and `host_func` fields as well as in
    ///   the `Drop` impl to recycle the stack.
    engine: Engine,
    /// The underlying root function to be executed.
    ///
    /// # Note
    ///
    /// The results of this function must always match with the
    /// results given when resuming the call.
    func: Func,
    /// The host function that returned a host error.
    ///
    /// # Note
    ///
    /// - This is required to receive its result values that are
    ///   needed to be fed back in manually by the user. This way we
    ///   avoid heap memory allocations.
    /// - The results of this function must always match with the
    ///   arguments given when resuming the call.
    host_func: Func,
    /// The host error that was returned by the `host_func` which
    /// caused the resumable function invocation to break.
    ///
    /// # Note
    ///
    /// This might be useful to users of this API to inspect the
    /// actual host error. This is therefore guaranteed to never
    /// be a Wasm trap.
    host_error: Error,
    /// The registers where to put provided host function results upon resumption.
    ///
    /// # Note
    ///
    /// This is only needed for the register-machine Wasmi engine backend.
    caller_results: RegisterSpan,
    /// The value and call stack in use by the [`ResumableInvocation`].
    ///
    /// # Note
    ///
    /// - We need to keep the stack around since the user might want to
    ///   resume the execution.
    /// - This stack is borrowed from the engine and needs to be given
    ///   back to the engine when the [`ResumableInvocation`] goes out
    ///   of scope.
    pub(super) stack: Stack,
}

// # Safety
//
// `ResumableInvocation` is not `Sync` because of the following sequence of fields:
//
// - `ResumableInvocation`'s `Stack` is not `Sync`
// - `Stack`'s `CallStack` is not `Sync`
// - `CallStack`'s `CallFrame` sequence is not `Sync`
// - `CallFrame`'s `InstructionPtr` is not `Sync` because it is a raw pointer to an `Instruction` buffer owned by the [`Engine`].
//
// Since `Engine` is owned by `ResumableInvocation` it cannot be outlived.
// Also the `Instruction` buffers that are pointed to by the `InstructionPtr` are immutable.
//
// Therefore `ResumableInvocation` can safely be assumed to be `Sync`.
unsafe impl Sync for ResumableInvocation {}

impl ResumableInvocation {
    /// Creates a new [`ResumableInvocation`].
    pub(super) fn new(
        engine: Engine,
        func: Func,
        host_func: Func,
        host_error: Error,
        caller_results: RegisterSpan,
        stack: Stack,
    ) -> Self {
        Self {
            engine,
            func,
            host_func,
            host_error,
            caller_results,
            stack,
        }
    }

    /// Replaces the internal stack with an empty one that has no heap allocations.
    pub(super) fn take_stack(&mut self) -> Stack {
        replace(&mut self.stack, Stack::empty())
    }

    /// Updates the [`ResumableInvocation`] with the new `host_func`, `host_error` and `caller_results`.
    ///
    /// # Note
    ///
    /// This should only be called from the register-machine Wasmi engine backend.
    pub(super) fn update(
        &mut self,
        host_func: Func,
        host_error: Error,
        caller_results: RegisterSpan,
    ) {
        self.host_func = host_func;
        self.host_error = host_error;
        self.caller_results = caller_results;
    }
}

impl Drop for ResumableInvocation {
    fn drop(&mut self) {
        let stack = self.take_stack();
        self.engine.recycle_stack(stack);
    }
}

impl ResumableInvocation {
    /// Returns the host [`Func`] that returned the host error.
    ///
    /// # Note
    ///
    /// When using [`ResumableInvocation::resume`] the `inputs`
    /// need to match the results of this host function so that
    /// the function invocation can properly resume. For that
    /// number and types of the values provided must match.
    pub fn host_func(&self) -> Func {
        self.host_func
    }

    /// Returns a shared reference to the encountered host error.
    ///
    /// # Note
    ///
    /// This is guaranteed to never be a Wasm trap.
    pub fn host_error(&self) -> &Error {
        &self.host_error
    }

    /// Returns the caller results [`RegisterSpan`].
    ///
    /// # Note
    ///
    /// This is `Some` only for [`ResumableInvocation`] originating from the register-machine Wasmi engine.
    pub(crate) fn caller_results(&self) -> RegisterSpan {
        self.caller_results
    }

    /// Resumes the call to the [`Func`] with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer upon success.
    ///
    /// Returns a resumable handle to the function invocation upon
    /// encountering host errors with which it is possible to handle
    /// the error and continue the execution as if no error occurred.
    ///
    /// # Errors
    ///
    /// - If the function resumption returned a Wasm [`Error`].
    /// - If the types or the number of values in `inputs` does not match
    ///   the types and number of result values of the erroneous host function.
    /// - If the number of output values does not match the expected number of
    ///   outputs required by the called function.
    pub fn resume<T>(
        self,
        mut ctx: impl AsContextMut<Data = T>,
        inputs: &[Val],
        outputs: &mut [Val],
    ) -> Result<ResumableCall, Error> {
        self.engine
            .resolve_func_type(self.host_func().ty_dedup(ctx.as_context()), |func_type| {
                func_type.match_results(inputs, true)
            })?;
        self.engine
            .resolve_func_type(self.func.ty_dedup(ctx.as_context()), |func_type| {
                func_type.match_results(outputs, false)?;
                func_type.prepare_outputs(outputs);
                <Result<(), Error>>::Ok(()) // TODO: why do we need types here?
            })?;
        self.engine
            .clone()
            .resume_func(ctx.as_context_mut(), self, inputs, outputs)
            .map_err(Into::into)
            .map(ResumableCall::new)
    }
}

/// Returned by calling a [`TypedFunc`] in a resumable way.
///
/// [`TypedFunc`]: [`crate::TypedFunc`]
#[derive(Debug)]
pub enum TypedResumableCall<T> {
    /// The resumable call has finished properly and returned a result.
    Finished(T),
    /// The resumable call encountered a host error and can be resumed.
    Resumable(TypedResumableInvocation<T>),
}

impl<Results> TypedResumableCall<Results> {
    /// Creates a [`TypedResumableCall`] from the [`Engine`]'s base [`ResumableCallBase`].
    pub(crate) fn new(call: ResumableCallBase<Results>) -> Self {
        match call {
            ResumableCallBase::Finished(results) => Self::Finished(results),
            ResumableCallBase::Resumable(invocation) => {
                Self::Resumable(TypedResumableInvocation::new(invocation))
            }
        }
    }
}

/// State required to resume a [`TypedFunc`] invocation.
///
/// [`TypedFunc`]: [`crate::TypedFunc`]
pub struct TypedResumableInvocation<Results> {
    invocation: ResumableInvocation,
    /// The parameter and result typed encoded in Rust type system.
    results: PhantomData<fn() -> Results>,
}

impl<Results> TypedResumableInvocation<Results> {
    /// Creates a [`TypedResumableInvocation`] wrapper for the given [`ResumableInvocation`].
    pub(crate) fn new(invocation: ResumableInvocation) -> Self {
        Self {
            invocation,
            results: PhantomData,
        }
    }

    /// Resumes the call to the [`TypedFunc`] with the given inputs.
    ///
    /// Returns a resumable handle to the function invocation upon
    /// encountering host errors with which it is possible to handle
    /// the error and continue the execution as if no error occurred.
    ///
    /// # Errors
    ///
    /// - If the function resumption returned a Wasm [`Error`].
    /// - If the types or the number of values in `inputs` does not match
    ///   the types and number of result values of the erroneous host function.
    ///
    /// [`TypedFunc`]: [`crate::TypedFunc`]
    pub fn resume<T>(
        self,
        mut ctx: impl AsContextMut<Data = T>,
        inputs: &[Val],
    ) -> Result<TypedResumableCall<Results>, Error>
    where
        Results: WasmResults,
    {
        self.engine
            .resolve_func_type(self.host_func().ty_dedup(ctx.as_context()), |func_type| {
                func_type.match_results(inputs, true)
            })?;
        self.engine
            .clone()
            .resume_func(
                ctx.as_context_mut(),
                self.invocation,
                inputs,
                <CallResultsTuple<Results>>::default(),
            )
            .map_err(Into::into)
            .map(TypedResumableCall::new)
    }
}

impl<Results> Deref for TypedResumableInvocation<Results> {
    type Target = ResumableInvocation;

    fn deref(&self) -> &Self::Target {
        &self.invocation
    }
}

impl<Results> fmt::Debug for TypedResumableInvocation<Results> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedResumableInvocation")
            .field("invocation", &self.invocation)
            .field("results", &self.results)
            .finish()
    }
}
