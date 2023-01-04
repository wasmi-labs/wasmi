use super::Func;
use crate::{engine::Stack, AsContextMut, Engine, Error};
use core::mem::replace;
use wasmi_core::{Trap, Value};

/// Returned by calling a function in a resumable way.
#[derive(Debug)]
pub enum ResumableCall<T> {
    /// The resumable call has finished properly and returned a result.
    Finished(T),
    /// The resumable call encountered a host error and can be resumed.
    Resumable(ResumableInvocation),
}

/// State required to resume a function invocation.
#[derive(Debug)]
pub struct ResumableInvocation {
    /// The engine in use for the function invokation.
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
    host_error: Trap,
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

impl ResumableInvocation {
    /// Creates a new [`ResumableInvocation`].
    pub(super) fn new(
        engine: Engine,
        func: Func,
        host_func: Func,
        host_error: Trap,
        stack: Stack,
    ) -> Self {
        Self {
            engine,
            func,
            host_func,
            host_error,
            stack,
        }
    }

    /// Replaces the internal stack with an empty one that has no heap allocations.
    pub(super) fn take_stack(&mut self) -> Stack {
        replace(&mut self.stack, Stack::empty())
    }

    /// Updates the [`ResumableInvocation`] with the new `host_func` and a `host_error`.
    pub(super) fn update(&mut self, host_func: Func, host_error: Trap) {
        self.host_func = host_func;
        self.host_error = host_error;
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
    pub fn host_error(&self) -> &Trap {
        &self.host_error
    }

    /// Resumes the call to the Wasm or host function with the given inputs.
    ///
    /// The result is written back into the `outputs` buffer.
    ///
    /// Returns a resumable handle to the function invocation upon
    /// enountering host errors with which it is possible to handle
    /// the error and continue the execution as if no error occured.
    ///
    /// # Errors
    ///
    /// - If the function resumption returned a Wasm [`Trap`].
    /// - If the types or the number of values in `inputs` does not match
    ///   the types and number of result values of the errorneous host function.
    /// - If the number of output values does not match the expected number of
    ///   outputs required by the called function.
    pub fn resume<T>(
        self,
        mut ctx: impl AsContextMut<UserState = T>,
        inputs: &[Value],
        outputs: &mut [Value],
    ) -> Result<ResumableCall<()>, Error> {
        self.engine
            .resolve_func_type(self.host_func().signature(ctx.as_context()), |func_type| {
                func_type.match_results(inputs, true)
            })?;
        self.engine
            .resolve_func_type(self.func.signature(ctx.as_context()), |func_type| {
                func_type.match_results(outputs, false)?;
                func_type.prepare_outputs(outputs);
                <Result<(), Error>>::Ok(()) // TODO: why do we need types here?
            })?;
        self.engine
            .clone()
            .resume_func(ctx.as_context_mut(), self, inputs, outputs)
            .map_err(Into::into)
    }
}
