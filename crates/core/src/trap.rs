use crate::HostError;
use alloc::{boxed::Box, string::String};
use core::fmt::{self, Display};

#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Error type which can be returned by Wasm code or by the host environment.
///
/// Under some conditions, Wasm execution may produce a [`Trap`],
/// which immediately aborts execution.
/// Traps cannot be handled by WebAssembly code, but are reported to the
/// host embedder.
#[derive(Debug)]
pub struct Trap {
    /// The cloneable reason of a [`Trap`].
    reason: Box<TrapReason>,
}

#[test]
fn trap_size() {
    assert_eq!(
        core::mem::size_of::<Trap>(),
        core::mem::size_of::<*const ()>()
    );
}

/// The reason of a [`Trap`].
#[derive(Debug)]
enum TrapReason {
    /// Traps during Wasm execution.
    InstructionTrap(TrapCode),
    /// An `i32` exit status code.
    ///
    /// # Note
    ///
    /// This is useful for some WASI functions.
    I32Exit(i32),
    /// An error described by a display message.
    Message(Box<str>),
    /// Traps and errors during host execution.
    Host(Box<dyn HostError>),
}

impl TrapReason {
    /// Returns the classic `i32` exit program code of a `Trap` if any.
    ///
    /// Otherwise returns `None`.
    pub fn i32_exit_status(&self) -> Option<i32> {
        if let Self::I32Exit(status) = self {
            return Some(*status);
        }
        None
    }

    /// Returns a shared reference to the [`HostError`] if any.
    #[inline]
    pub fn as_host(&self) -> Option<&dyn HostError> {
        if let Self::Host(host_error) = self {
            return Some(&**host_error);
        }
        None
    }

    /// Returns an exclusive reference to the [`HostError`] if any.
    #[inline]
    pub fn as_host_mut(&mut self) -> Option<&mut dyn HostError> {
        if let Self::Host(host_error) = self {
            return Some(&mut **host_error);
        }
        None
    }

    /// Consumes `self` to return the [`HostError`] if any.
    #[inline]
    pub fn into_host(self) -> Option<Box<dyn HostError>> {
        if let Self::Host(host_error) = self {
            return Some(host_error);
        }
        None
    }

    /// Returns the [`TrapCode`] traps originating from Wasm execution.
    #[inline]
    pub fn trap_code(&self) -> Option<TrapCode> {
        if let Self::InstructionTrap(trap_code) = self {
            return Some(*trap_code);
        }
        None
    }
}

impl Trap {
    /// Create a new [`Trap`] from the [`TrapReason`].
    fn with_reason(reason: TrapReason) -> Self {
        Self {
            reason: Box::new(reason),
        }
    }

    /// Creates a new [`Trap`] described by a `message`.
    #[cold] // traps are exceptional, this helps move handling off the main path
    pub fn new<T>(message: T) -> Self
    where
        T: Into<String>,
    {
        Self::with_reason(TrapReason::Message(message.into().into_boxed_str()))
    }

    /// Downcasts the [`Trap`] into the `T: HostError` if possible.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: HostError,
    {
        self.reason
            .as_host()
            .and_then(<dyn HostError + 'static>::downcast_ref)
    }

    /// Downcasts the [`Trap`] into the `T: HostError` if possible.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: HostError,
    {
        self.reason
            .as_host_mut()
            .and_then(<dyn HostError + 'static>::downcast_mut)
    }

    /// Consumes `self` to downcast the [`Trap`] into the `T: HostError` if possible.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn downcast<T>(self) -> Option<T>
    where
        T: HostError,
    {
        self.reason
            .into_host()
            .and_then(|error| error.downcast().ok())
            .map(|boxed| *boxed)
    }

    /// Creates a new `Trap` representing an explicit program exit with a classic `i32`
    /// exit status value.
    #[cold] // see Trap::new
    pub fn i32_exit(status: i32) -> Self {
        Self::with_reason(TrapReason::I32Exit(status))
    }

    /// Returns the classic `i32` exit program code of a `Trap` if any.
    ///
    /// Otherwise returns `None`.
    #[inline]
    pub fn i32_exit_status(&self) -> Option<i32> {
        self.reason.i32_exit_status()
    }

    /// Returns the [`TrapCode`] traps originating from Wasm execution.
    #[inline]
    pub fn trap_code(&self) -> Option<TrapCode> {
        self.reason.trap_code()
    }
}

impl From<TrapCode> for Trap {
    #[cold] // see Trap::new
    fn from(error: TrapCode) -> Self {
        Self::with_reason(TrapReason::InstructionTrap(error))
    }
}

impl<E> From<E> for Trap
where
    E: HostError,
{
    #[inline]
    #[cold] // see Trap::new
    fn from(host_error: E) -> Self {
        Self::with_reason(TrapReason::Host(Box::new(host_error)))
    }
}

impl Display for TrapReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InstructionTrap(trap_code) => Display::fmt(trap_code, f),
            Self::I32Exit(status) => write!(f, "Exited with i32 exit status {status}"),
            Self::Message(message) => write!(f, "{message}"),
            Self::Host(host_error) => Display::fmt(host_error, f),
        }
    }
}

impl Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <TrapReason as Display>::fmt(&self.reason, f)
    }
}

#[cfg(feature = "std")]
impl StdError for Trap {
    fn description(&self) -> &str {
        self.trap_code().map_or("", |code| code.trap_message())
    }
}

/// An invalid [`TrapCode`] integer value.
#[derive(Debug, Copy, Clone)]
pub struct InvalidTrapCode;

macro_rules! generate_trap_code {
    (
        $( $(#[$attr:meta])* $ident:ident = $discr:literal ),* $(,)?
    ) => {
        /// Error type which can be thrown by wasm code or by host environment.
        ///
        /// See [`Trap`] for details.
        ///
        /// [`Trap`]: struct.Trap.html
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr(u8)]
        pub enum TrapCode {
            $(
                $( #[$attr] )*
                $ident = $discr
            ),*
        }

        impl From<TrapCode> for u8 {
            fn from(trap_code: TrapCode) -> Self {
                trap_code as _
            }
        }

        impl TryFrom<u8> for TrapCode {
            type Error = InvalidTrapCode;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $( $discr => Ok(TrapCode::$ident), )*
                    _ => Err(InvalidTrapCode),
                }
            }
        }

        #[test]
        fn trap_code_conversion() {
            $(
                assert_eq!(
                    TrapCode::try_from(TrapCode::$ident as u8).unwrap(),
                    TrapCode::$ident,
                );
            )*
            assert!(TrapCode::try_from(u8::MAX).is_err());
        }
    };
}
generate_trap_code! {
    // Note: we deliberately do _not_ assign the zero value so that it can be used as success state
    //       in `Result<(), TrapCode>` or `Control<(), TrapCode>` as the compiler can then generate
    //       slightly better code.

    /// Wasm code executed `unreachable` opcode.
    ///
    /// This indicates that unreachable Wasm code was actually reached.
    /// This opcode have a similar purpose as `ud2` in x86.
    UnreachableCodeReached = 1,

    /// Attempt to load or store at the address which
    /// lies outside of bounds of the memory.
    ///
    /// Since addresses are interpreted as unsigned integers, out of bounds access
    /// can't happen with negative addresses (i.e. they will always wrap).
    MemoryOutOfBounds = 2,

    /// Attempt to access table element at index which
    /// lies outside of bounds.
    ///
    /// This typically can happen when `call_indirect` is executed
    /// with index that lies out of bounds.
    ///
    /// Since indexes are interpreted as unsigned integers, out of bounds access
    /// can't happen with negative indexes (i.e. they will always wrap).
    TableOutOfBounds = 3,

    /// Indicates that a `call_indirect` instruction called a function at
    /// an uninitialized (i.e. `null`) table index.
    IndirectCallToNull = 4,

    /// Attempt to divide by zero.
    ///
    /// This trap typically can happen if `div` or `rem` is executed with
    /// zero as divider.
    IntegerDivisionByZero = 5,

    /// An integer arithmetic operation caused an overflow.
    ///
    /// This can happen when trying to do signed division (or get the remainder)
    /// -2<sup>N-1</sup> over -1. This is because the result +2<sup>N-1</sup>
    /// isn't representable as a N-bit signed integer.
    IntegerOverflow = 6,

    /// Attempted to make an invalid conversion to an integer type.
    ///
    /// This can for example happen when trying to truncate NaNs,
    /// infinity, or value for which the result is out of range into an integer.
    BadConversionToInteger = 7,

    /// Stack overflow.
    ///
    /// This is likely caused by some infinite or very deep recursion.
    /// Extensive inlining might also be the cause of stack overflow.
    StackOverflow = 8,

    /// Attempt to invoke a function with mismatching signature.
    ///
    /// This can happen with indirect calls as they always
    /// specify the expected signature of function. If an indirect call is executed
    /// with an index that points to a function with signature different of what is
    /// expected by this indirect call, this trap is raised.
    BadSignature = 9,

    /// This trap is raised when a WebAssembly execution ran out of fuel.
    ///
    /// The Wasmi execution engine can be configured to instrument its
    /// internal bytecode so that fuel is consumed for each executed instruction.
    /// This is useful to deterministically halt or yield a WebAssembly execution.
    OutOfFuel = 10,

    /// This trap is raised when a growth operation was attempted and an
    /// installed `wasmi::ResourceLimiter` returned `Err(...)` from the
    /// associated `table_growing` or `memory_growing` method, indicating a
    /// desire on the part of the embedder to trap the interpreter rather than
    /// merely fail the growth operation.
    GrowthOperationLimited = 11,

    /// This trap is raised when a WebAssembly operation demanded a memory
    /// allocation and the host system could not supply the requested amount.
    OutOfSystemMemory = 12,
}

impl TrapCode {
    /// Returns the trap message as specified by the WebAssembly specification.
    ///
    /// # Note
    ///
    /// This API is primarily useful for the Wasm spec testsuite but might have
    /// other uses since it avoid heap memory allocation in certain cases.
    pub fn trap_message(&self) -> &'static str {
        match self {
            Self::UnreachableCodeReached => "wasm `unreachable` instruction executed",
            Self::MemoryOutOfBounds => "out of bounds memory access",
            Self::TableOutOfBounds => "undefined element: out of bounds table access",
            Self::IndirectCallToNull => "uninitialized element 2", // TODO: fixme, remove the trailing " 2" again
            Self::IntegerDivisionByZero => "integer divide by zero",
            Self::IntegerOverflow => "integer overflow",
            Self::BadConversionToInteger => "invalid conversion to integer",
            Self::StackOverflow => "call stack exhausted",
            Self::BadSignature => "indirect call type mismatch",
            Self::OutOfFuel => "all fuel consumed by WebAssembly",
            Self::GrowthOperationLimited => "growth operation limited",
            Self::OutOfSystemMemory => "out of system memory",
        }
    }
}

impl Display for TrapCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.trap_message())
    }
}
