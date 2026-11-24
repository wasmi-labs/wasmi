#[cfg_attr(
    any(
        feature = "portable-dispatch",
        all(feature = "auto-dispatch", not(wasmi_use_tail_calls))
    ),
    path = "backend/loop.rs"
)]
#[cfg_attr(
    all(
        not(feature = "portable-dispatch"),
        any(not(feature = "auto-dispatch"), wasmi_use_tail_calls)
    ),
    path = "backend/tail.rs"
)]
#[macro_use]
pub mod backend;

pub use self::backend::{Done, Handler, execute_until_done, op_code_to_handler};
use super::state::Ip;
use crate::{
    Error,
    TrapCode,
    engine::{ResumableHostTrapError, ResumableOutOfFuelError},
};
use core::ops::ControlFlow;

#[inline(always)]
pub fn control_break<T>() -> Control<T> {
    Control::Break(Break::WithReason)
}

#[allow(unused)]
#[inline(always)]
fn decode_op_code(ip: Ip) -> crate::ir::OpCode {
    let (_, op_code) = unsafe { ip.decode::<crate::ir::OpCode>() };
    op_code
}

#[allow(unused)]
#[inline(always)]
fn decode_handler(ip: Ip) -> Handler {
    use core::{mem, ptr};
    let (_, addr) = unsafe { ip.decode::<usize>() };
    unsafe { mem::transmute::<*const (), Handler>(ptr::with_exposed_provenance(addr)) }
}

#[derive(Debug)]
pub enum ExecutionOutcome {
    Host(ResumableHostTrapError),
    OutOfFuel(ResumableOutOfFuelError),
    Error(Error),
}

impl From<ExecutionOutcome> for Error {
    fn from(error: ExecutionOutcome) -> Self {
        match error {
            ExecutionOutcome::Host(error) => error.into(),
            ExecutionOutcome::OutOfFuel(error) => error.into(),
            ExecutionOutcome::Error(error) => error,
        }
    }
}

impl From<ResumableHostTrapError> for ExecutionOutcome {
    #[cold]
    #[inline]
    fn from(error: ResumableHostTrapError) -> Self {
        Self::Host(error)
    }
}

impl From<ResumableOutOfFuelError> for ExecutionOutcome {
    #[cold]
    #[inline]
    fn from(error: ResumableOutOfFuelError) -> Self {
        Self::OutOfFuel(error)
    }
}

impl From<TrapCode> for ExecutionOutcome {
    #[cold]
    #[inline]
    fn from(error: TrapCode) -> Self {
        Self::Error(error.into())
    }
}

impl From<Error> for ExecutionOutcome {
    #[cold]
    #[inline]
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

impl ExecutionOutcome {
    /// Converts resumable [`ExecutionOutcome::Host`] and [`ExecutionOutcome::OutOfFuel`] into non-resumable errors.
    #[inline]
    pub fn into_non_resumable(self) -> Error {
        match self {
            Self::Host(error) => error.into_error(),
            Self::OutOfFuel(_error) => Error::from(TrapCode::OutOfFuel),
            Self::Error(error) => error,
        }
    }
}

macro_rules! define_break {
    ( $( $(#[$attr:meta])* $name:ident = $discr:literal ),* $(,)? ) => {
        #[derive(Debug, Copy, Clone)]
        pub enum Break {
            $( $name = TrapCode::$name as _, )*
            /// Signals that there must be a reason stored externally supplying the caller with more information.
            WithReason,
        }

        impl From<TrapCode> for Break {
            #[inline]
            fn from(trap_code: TrapCode) -> Self {
                match trap_code {
                    $( TrapCode::$name => Self::$name, )*
                }
            }
        }

        impl Break {
            #[inline]
            pub fn trap_code(self) -> Option<TrapCode> {
                let trap_code = match self {
                    $( Self::$name => TrapCode::$name, )*
                    Self::WithReason => return None,
                };
                Some(trap_code)
            }
        }
    };
}
wasmi_core::for_each_trap_code!(define_break);

pub type Control<C = (), B = Break> = ControlFlow<B, C>;
