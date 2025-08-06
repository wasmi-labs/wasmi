use super::InstantiationError;
use crate::{module::FuncIdx, AsContextMut, Error, Instance, InstanceEntityBuilder};

/// A partially instantiated [`Instance`] where the `start` function has not yet been executed.
///
/// # Note
///
/// Some users require Wasm modules to not have a `start` function that is required for
/// conformant module instantiation. This API provides control over the precise instantiation
/// process with regard to this need.
#[derive(Debug)]
#[deprecated(
    since = "0.49.0",
    note = "enable fuel-metering and set fuel to zero if you want to prevent `start` function execution."
)]
pub struct InstancePre {
    handle: Instance,
    builder: InstanceEntityBuilder,
}

#[expect(deprecated)]
impl InstancePre {
    /// Creates a new [`InstancePre`].
    pub(super) fn new(handle: Instance, builder: InstanceEntityBuilder) -> Self {
        Self { handle, builder }
    }

    /// Returns the index of the `start` function if any.
    ///
    /// Returns `None` if the Wasm module does not have a `start` function.
    fn start_fn(&self) -> Option<u32> {
        self.builder.get_start().map(FuncIdx::into_u32)
    }

    /// Runs the `start` function of the [`Instance`] and returns its handle.
    ///
    /// # Note
    ///
    /// This finishes the instantiation procedure.
    ///
    /// # Errors
    ///
    /// If executing the `start` function traps.
    ///
    /// # Panics
    ///
    /// If the `start` function is invalid albeit successful validation.
    pub fn start(self, mut context: impl AsContextMut) -> Result<Instance, Error> {
        let opt_start_index = self.start_fn();
        context
            .as_context_mut()
            .store
            .inner
            .initialize_instance(self.handle, self.builder.finish());
        if let Some(start_index) = opt_start_index {
            let start_func = self
                .handle
                .get_func_by_index(&mut context, start_index)
                .unwrap_or_else(|| {
                    panic!("encountered invalid start function after validation: {start_index}")
                });
            start_func.call(context.as_context_mut(), &[], &mut [])?
        }
        Ok(self.handle)
    }

    /// Finishes instantiation ensuring that no `start` function exists.
    ///
    /// # Errors
    ///
    /// If a `start` function exists that needs to be called for conformant module instantiation.
    pub fn ensure_no_start(
        self,
        mut context: impl AsContextMut,
    ) -> Result<Instance, InstantiationError> {
        if let Some(index) = self.start_fn() {
            return Err(InstantiationError::UnexpectedStartFn { index });
        }
        context
            .as_context_mut()
            .store
            .inner
            .initialize_instance(self.handle, self.builder.finish());
        Ok(self.handle)
    }
}
