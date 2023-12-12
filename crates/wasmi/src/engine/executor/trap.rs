use crate::{
    core::{Trap, TrapCode},
    engine::bytecode::RegisterSpan,
    Error,
    Func,
};

/// Either a Wasm trap or a host trap with its originating host [`Func`].
#[derive(Debug)]
pub enum TaggedTrap {
    /// The trap is originating from Wasm.
    Wasm(Error),
    /// The trap is originating from a host function.
    Host {
        host_error: Error,
        host_func: Func,
        caller_results: RegisterSpan,
    },
}

impl TaggedTrap {
    /// Creates a [`TaggedTrap`] from a host error.
    pub fn host(host_func: Func, host_error: Error, caller_results: RegisterSpan) -> Self {
        Self::Host {
            host_func,
            host_error,
            caller_results,
        }
    }

    /// Returns the [`Error`] of the [`TaggedTrap`].
    pub fn into_error(self) -> Error {
        match self {
            TaggedTrap::Wasm(error) => error,
            TaggedTrap::Host { host_error, .. } => host_error,
        }
    }
}

impl From<Trap> for TaggedTrap {
    fn from(trap: Trap) -> Self {
        Self::Wasm(trap.into())
    }
}

impl From<Error> for TaggedTrap {
    fn from(error: Error) -> Self {
        Self::Wasm(error)
    }
}

impl From<TrapCode> for TaggedTrap {
    fn from(trap_code: TrapCode) -> Self {
        Self::Wasm(Trap::from(trap_code).into())
    }
}
