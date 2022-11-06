use crate::HostError;
use alloc::{boxed::Box, sync::Arc};
use core::fmt::{self, Display};

#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Error type which can be thrown by wasm code or by host environment.
///
/// Under some conditions, wasm execution may produce a `Trap`, which immediately aborts execution.
/// Traps can't be handled by WebAssembly code, but are reported to the embedder.
#[derive(Debug, Clone)]
pub struct Trap {
    /// The internal data structure of a [`Trap`].
    inner: Arc<TrapInner>,
}

#[test]
fn trap_size() {
    assert_eq!(
        core::mem::size_of::<Trap>(),
        core::mem::size_of::<*const ()>()
    );
}

/// The internal of a [`Trap`].
#[derive(Debug)]
enum TrapInner {
    /// Traps during Wasm execution.
    Code(TrapCode),
    /// An `i32` exit code.
    ///
    /// # Note
    ///
    /// This is useful for some WASI functions.
    I32Exit(i32),
    /// Traps and errors during host execution.
    Host(Box<dyn HostError>),
}

impl TrapInner {
    /// Returns `true` if `self` trap originating from host code.
    #[inline]
    pub fn is_host(&self) -> bool {
        matches!(self, TrapInner::Host(_))
    }

    /// Returns `true` if `self` trap originating from Wasm code.
    #[inline]
    pub fn is_code(&self) -> bool {
        matches!(self, TrapInner::Code(_))
    }

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

    /// Returns the [`TrapCode`] traps originating from Wasm execution.
    #[inline]
    pub fn trap_code(&self) -> Option<TrapCode> {
        if let Self::Code(trap_code) = self {
            return Some(*trap_code);
        }
        None
    }
}

impl Trap {
    /// Create a new [`Trap`] from the [`TrapInner`].
    fn new(inner: TrapInner) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Wraps the host error in a [`Trap`].
    #[cold] // traps are exceptional, this helps move handling off the main path
    pub fn host<U>(host_error: U) -> Self
    where
        U: HostError + Sized,
    {
        Self::new(TrapInner::Host(Box::new(host_error)))
    }

    /// Creates a new `Trap` representing an explicit program exit with a classic `i32`
    /// exit status value.
    #[cold] // see Trap::host
    pub fn i32_exit(status: i32) -> Self {
        Self::new(TrapInner::I32Exit(status))
    }

    /// Returns `true` if `self` trap originating from host code.
    #[inline]
    pub fn is_host(&self) -> bool {
        self.inner.is_host()
    }

    /// Returns `true` if `self` trap originating from Wasm code.
    #[inline]
    pub fn is_code(&self) -> bool {
        self.inner.is_code()
    }

    /// Returns a shared reference to the [`HostError`] if any.
    ///
    /// Otherwise returns `None`.
    #[inline]
    pub fn as_host(&self) -> Option<&dyn HostError> {
        self.inner.as_host()
    }

    }

    /// Returns the classic `i32` exit program code of a `Trap` if any.
    ///
    /// Otherwise returns `None`.
    #[inline]
    pub fn i32_exit_status(&self) -> Option<i32> {
        self.inner.i32_exit_status()
    }

    /// Returns the [`TrapCode`] traps originating from Wasm execution.
    #[inline]
    pub fn trap_code(&self) -> Option<TrapCode> {
        self.inner.trap_code()
    }
}

impl From<TrapCode> for Trap {
    #[cold]
    fn from(error: TrapCode) -> Self {
        Self::new(TrapInner::Code(error))
    }
}

impl<U> From<U> for Trap
where
    U: HostError + Sized,
{
    #[inline]
    fn from(e: U) -> Self {
        Self::host(e)
    }
}

impl Display for TrapInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Code(trap_code) => Display::fmt(trap_code, f),
            Self::I32Exit(status) => write!(f, "Exited with i32 exit status {}", status),
            Self::Host(host_error) => Display::fmt(host_error, f),
        }
    }
}

impl Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <TrapInner as Display>::fmt(&self.inner, f)
    }
}

#[cfg(feature = "std")]
impl StdError for Trap {
    fn description(&self) -> &str {
        self.trap_code().map_or("", |code| code.trap_message())
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
    /// Since indexes are interpreted as unsigned integers, out of bounds access
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
            Self::Unreachable => "unreachable",
            Self::MemoryAccessOutOfBounds => "out of bounds memory access",
            Self::TableAccessOutOfBounds => "undefined element",
            Self::ElemUninitialized => "uninitialized element",
            Self::DivisionByZero => "integer divide by zero",
            Self::IntegerOverflow => "integer overflow",
            Self::InvalidConversionToInt => "invalid conversion to integer",
            Self::StackOverflow => "call stack exhausted",
            Self::UnexpectedSignature => "indirect call type mismatch",
        }
    }
}

impl Display for TrapCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.trap_message())
    }
}
