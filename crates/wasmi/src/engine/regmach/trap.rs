use super::bytecode::RegisterSpan;
use crate::{
    core::{Trap, TrapCode},
    Func,
};

/// Either a Wasm trap or a host trap with its originating host [`Func`].
#[derive(Debug)]
pub enum TaggedTrap {
    /// The trap is originating from Wasm.
    Wasm(Trap),
    /// The trap is originating from a host function.
    Host {
        host_func: Func,
        host_trap: Trap,
        caller_results: RegisterSpan,
    },
}

impl TaggedTrap {
    /// Creates a [`TaggedTrap`] from a host error.
    pub fn host(host_func: Func, host_trap: Trap, caller_results: RegisterSpan) -> Self {
        Self::Host {
            host_func,
            host_trap,
            caller_results,
        }
    }

    /// Returns the [`Trap`] of the [`TaggedTrap`].
    pub fn into_trap(self) -> Trap {
        match self {
            TaggedTrap::Wasm(trap) => trap,
            TaggedTrap::Host { host_trap, .. } => host_trap,
        }
    }
}

impl From<Trap> for TaggedTrap {
    fn from(trap: Trap) -> Self {
        Self::Wasm(trap)
    }
}

impl From<TrapCode> for TaggedTrap {
    fn from(trap_code: TrapCode) -> Self {
        Self::Wasm(trap_code.into())
    }
}
