use crate::HostError;
use alloc::boxed::Box;
use core::fmt::{self, Display};

#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Error type which can be thrown by wasm code or by host environment.
///
/// Under some conditions, wasm execution may produce a `Trap`, which immediately aborts execution.
/// Traps can't be handled by WebAssembly code, but are reported to the embedder.
#[derive(Debug)]
pub enum Trap {
    /// Traps during Wasm execution.
    Code(TrapCode),
    /// Traps and errors during host execution.
    Host(Box<dyn HostError>),
}

impl Trap {
    /// Wraps the host error in a [`Trap`].
    #[inline]
    pub fn host<U>(host_error: U) -> Self
    where
        U: HostError + Sized,
    {
        Self::Host(Box::new(host_error))
    }

    /// Returns `true` if `self` trap originating from host code.
    #[inline]
    pub fn is_host(&self) -> bool {
        matches!(self, Self::Host(_))
    }

    /// Returns the [`TrapCode`] traps originating from Wasm execution.
    #[inline]
    pub fn code(&self) -> Option<TrapCode> {
        if let Self::Code(trap_code) = self {
            return Some(*trap_code);
        }
        None
    }
}

impl From<TrapCode> for Trap {
    #[inline]
    fn from(error: TrapCode) -> Self {
        Self::Code(error)
    }
}

impl<U> From<U> for Trap
where
    U: HostError + Sized,
{
    #[inline]
    fn from(e: U) -> Self {
        Trap::host(e)
    }
}

impl Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Trap::Code(trap_code) => Display::fmt(trap_code, f),
            Trap::Host(host_error) => Display::fmt(host_error, f),
        }
    }
}

#[cfg(feature = "std")]
impl StdError for Trap {
    fn description(&self) -> &str {
        self.code().map(|code| code.trap_message()).unwrap_or("")
    }
}

/// Error type which can be thrown by wasm code or by host environment.
///
/// See [`Trap`] for details.
///
/// [`Trap`]: struct.Trap.html
#[derive(Debug, Copy, Clone)]
pub enum TrapCode {
    /// Wasm code executed `unreachable` opcode.
    ///
    /// `unreachable` is a special opcode which always traps upon execution.
    /// This opcode have a similar purpose as `ud2` in x86.
    Unreachable,

    /// Attempt to load or store at the address which
    /// lies outside of bounds of the memory.
    ///
    /// Since addresses are interpreted as unsigned integers, out of bounds access
    /// can't happen with negative addresses (i.e. they will always wrap).
    MemoryAccessOutOfBounds,

    /// Attempt to access table element at index which
    /// lies outside of bounds.
    ///
    /// This typically can happen when `call_indirect` is executed
    /// with index that lies out of bounds.
    ///
    /// Since indexes are interpreted as unsinged integers, out of bounds access
    /// can't happen with negative indexes (i.e. they will always wrap).
    TableAccessOutOfBounds,

    /// Attempt to access table element which is uninitialized (i.e. `None`).
    ///
    /// This typically can happen when `call_indirect` is executed.
    ElemUninitialized,

    /// Attempt to divide by zero.
    ///
    /// This trap typically can happen if `div` or `rem` is executed with
    /// zero as divider.
    DivisionByZero,

    /// An integer arithmetic operation caused an overflow.
    ///
    /// This can happen when:
    ///
    /// - Trying to do signed division (or get the remainder) -2<sup>N-1</sup> over -1. This is
    ///   because the result +2<sup>N-1</sup> isn't representable as a N-bit signed integer.
    IntegerOverflow,

    /// Attempt to make a conversion to an int failed.
    ///
    /// This can happen when:
    ///
    /// - Trying to truncate NaNs, infinity, or value for which the result is out of range into an integer.
    InvalidConversionToInt,

    /// Stack overflow.
    ///
    /// This is likely caused by some infinite or very deep recursion.
    /// Extensive inlining might also be the cause of stack overflow.
    StackOverflow,

    /// Attempt to invoke a function with mismatching signature.
    ///
    /// This can happen if a Wasm or host function was invoked
    /// with mismatching parameters or result values.
    ///
    /// This can always happen with indirect calls as they always
    /// specify the expected signature of function. If an indirect call is executed
    /// with an index that points to a function with signature different of what is
    /// expected by this indirect call, this trap is raised.
    UnexpectedSignature,
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
            TrapCode::Unreachable => "unreachable",
            TrapCode::MemoryAccessOutOfBounds => "out of bounds memory access",
            TrapCode::TableAccessOutOfBounds => "undefined element",
            TrapCode::ElemUninitialized => "uninitialized element",
            TrapCode::DivisionByZero => "integer divide by zero",
            TrapCode::IntegerOverflow => "integer overflow",
            TrapCode::InvalidConversionToInt => "invalid conversion to integer",
            TrapCode::StackOverflow => "call stack exhausted",
            TrapCode::UnexpectedSignature => "indirect call type mismatch",
        }
    }
}

impl Display for TrapCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.trap_message())
    }
}
