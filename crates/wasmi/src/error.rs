use super::errors::{
    EnforcedLimitsError,
    FuncError,
    GlobalError,
    InstantiationError,
    IrError,
    LinkerError,
};
use crate::{
    core::{FuelError, HostError, MemoryError, TableError, TrapCode},
    engine::{ResumableHostTrapError, TranslationError},
    module::ReadError,
};
use alloc::{boxed::Box, string::String};
use core::{fmt, fmt::Display};
use wasmparser::BinaryReaderError as WasmError;

#[cfg(feature = "wat")]
use wat::Error as WatError;

/// The generic Wasmi root error type.
#[derive(Debug)]
pub struct Error {
    /// The underlying kind of the error and its specific information.
    kind: Box<ErrorKind>,
}

#[test]
fn error_size() {
    use core::mem;
    assert_eq!(mem::size_of::<Error>(), 8);
}

impl Error {
    /// Creates a new [`Error`] from the [`ErrorKind`].
    fn from_kind(kind: ErrorKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }

    /// Creates a new [`Error`] described by a `message`.
    #[inline]
    #[cold]
    pub fn new<T>(message: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_kind(ErrorKind::Message(message.into().into_boxed_str()))
    }

    /// Creates a custom [`HostError`].
    #[inline]
    #[cold]
    pub fn host<E>(host_error: E) -> Self
    where
        E: HostError,
    {
        Self::from_kind(ErrorKind::Host(Box::new(host_error)))
    }

    /// Creates a new `Error` representing an explicit program exit with a classic `i32` exit status value.
    ///
    /// # Note
    ///
    /// This is usually used as return code by WASI applications.
    #[inline]
    #[cold]
    pub fn i32_exit(status: i32) -> Self {
        Self::from_kind(ErrorKind::I32ExitStatus(status))
    }

    /// Returns the [`ErrorKind`] of the [`Error`].
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Returns a reference to [`TrapCode`] if [`Error`] is a [`TrapCode`].
    pub fn as_trap_code(&self) -> Option<TrapCode> {
        self.kind().as_trap_code()
    }

    /// Returns the classic `i32` exit program code of a `Trap` if any.
    ///
    /// Otherwise returns `None`.
    pub fn i32_exit_status(&self) -> Option<i32> {
        self.kind().as_i32_exit_status()
    }

    /// Downcasts the [`Error`] into the `T: HostError` if possible.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: HostError,
    {
        self.kind
            .as_host()
            .and_then(<(dyn HostError + 'static)>::downcast_ref)
    }

    /// Downcasts the [`Error`] into the `T: HostError` if possible.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: HostError,
    {
        self.kind
            .as_host_mut()
            .and_then(<(dyn HostError + 'static)>::downcast_mut)
    }

    /// Consumes `self` to downcast the [`Error`] into the `T: HostError` if possible.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn downcast<T>(self) -> Option<T>
    where
        T: HostError,
    {
        self.kind
            .into_host()
            .and_then(|error| error.downcast().ok())
            .map(|boxed| *boxed)
    }

    pub(crate) fn into_resumable(self) -> Result<ResumableHostTrapError, Error> {
        if matches!(&*self.kind, ErrorKind::ResumableHostTrap(_)) {
            let ErrorKind::ResumableHostTrap(error) = *self.kind else {
                unreachable!("asserted that host error is resumable")
            };
            return Ok(error);
        }
        Err(self)
    }
}

impl core::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

/// An error that may occur upon operating on Wasm modules or module instances.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A trap code as defined by the WebAssembly specification.
    TrapCode(TrapCode),
    /// A message usually provided by Wasmi users of host function calls.
    Message(Box<str>),
    /// An `i32` exit status usually used by WASI applications.
    I32ExitStatus(i32),
    /// A trap as defined by the WebAssembly specification.
    Host(Box<dyn HostError>),
    /// An error stemming from a host function call with resumable state information.
    ///
    /// # Note
    ///
    /// This variant is meant for internal uses only in order to store data necessary
    /// to resume a call after a host function returned an error. This should never
    /// actually reach user code thus we hide its documentation.
    #[doc(hidden)]
    ResumableHostTrap(ResumableHostTrapError),
    /// A global variable error.
    Global(GlobalError),
    /// A linear memory error.
    Memory(MemoryError),
    /// A table error.
    Table(TableError),
    /// A linker error.
    Linker(LinkerError),
    /// A module instantiation error.
    Instantiation(InstantiationError),
    /// A fuel error.
    Fuel(FuelError),
    /// A function error.
    Func(FuncError),
    /// Encountered when there is a problem with the Wasm input stream.
    Read(ReadError),
    /// Encountered when there is a Wasm parsing or validation error.
    Wasm(WasmError),
    /// Encountered when there is a Wasm to Wasmi translation error.
    Translation(TranslationError),
    /// Encountered when an enforced limit is exceeded.
    Limits(EnforcedLimitsError),
    /// Encountered for Wasmi bytecode related errors.
    Ir(IrError),
    /// Encountered an error from the `wat` crate.
    #[cfg(feature = "wat")]
    Wat(WatError),
}

impl ErrorKind {
    /// Returns a reference to [`TrapCode`] if [`ErrorKind`] is a [`TrapCode`].
    pub fn as_trap_code(&self) -> Option<TrapCode> {
        let trap_code = match self {
            | Self::TrapCode(trap_code) => *trap_code,
            | Self::Fuel(FuelError::OutOfFuel)
            | Self::Table(TableError::OutOfFuel)
            | Self::Memory(MemoryError::OutOfFuel) => TrapCode::OutOfFuel,
            | Self::Memory(MemoryError::OutOfBoundsAccess)
            | Self::Memory(MemoryError::OutOfBoundsGrowth) => TrapCode::MemoryOutOfBounds,
            | Self::Table(TableError::ElementTypeMismatch) => TrapCode::BadSignature,
            | Self::Table(TableError::SetOutOfBounds)
            | Self::Table(TableError::FillOutOfBounds)
            | Self::Table(TableError::GrowOutOfBounds)
            | Self::Table(TableError::InitOutOfBounds) => TrapCode::TableOutOfBounds,
            _ => return None,
        };
        Some(trap_code)
    }

    /// Returns a [`i32`] if [`ErrorKind`] is an [`ErrorKind::I32ExitStatus`].
    pub fn as_i32_exit_status(&self) -> Option<i32> {
        match self {
            Self::I32ExitStatus(exit_status) => Some(*exit_status),
            _ => None,
        }
    }

    /// Returns a dynamic reference to [`HostError`] if [`ErrorKind`] is a [`HostError`].
    pub fn as_host(&self) -> Option<&dyn HostError> {
        match self {
            Self::Host(error) => Some(error.as_ref()),
            _ => None,
        }
    }

    /// Returns a dynamic reference to [`HostError`] if [`ErrorKind`] is a [`HostError`].
    pub fn as_host_mut(&mut self) -> Option<&mut dyn HostError> {
        match self {
            Self::Host(error) => Some(error.as_mut()),
            _ => None,
        }
    }

    /// Returns the [`HostError`] if [`ErrorKind`] is a [`HostError`].
    pub fn into_host(self) -> Option<Box<dyn HostError>> {
        match self {
            Self::Host(error) => Some(error),
            _ => None,
        }
    }
}

impl core::error::Error for ErrorKind {}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TrapCode(error) => Display::fmt(error, f),
            Self::I32ExitStatus(status) => writeln!(f, "Exited with i32 exit status {status}"),
            Self::Message(message) => Display::fmt(message, f),
            Self::Host(error) => Display::fmt(error, f),
            Self::Global(error) => Display::fmt(error, f),
            Self::Memory(error) => Display::fmt(error, f),
            Self::Table(error) => Display::fmt(error, f),
            Self::Linker(error) => Display::fmt(error, f),
            Self::Func(error) => Display::fmt(error, f),
            Self::Instantiation(error) => Display::fmt(error, f),
            Self::Fuel(error) => Display::fmt(error, f),
            Self::Read(error) => Display::fmt(error, f),
            Self::Wasm(error) => Display::fmt(error, f),
            Self::Translation(error) => Display::fmt(error, f),
            Self::Limits(error) => Display::fmt(error, f),
            Self::ResumableHostTrap(error) => Display::fmt(error, f),
            Self::Ir(error) => Display::fmt(error, f),
            #[cfg(feature = "wat")]
            Self::Wat(error) => Display::fmt(error, f),
        }
    }
}

macro_rules! impl_from {
    ( $( impl From<$from:ident> for Error::$name:ident );* $(;)? ) => {
        $(
            impl From<$from> for Error {
                #[inline]
                #[cold]
                fn from(error: $from) -> Self {
                    Self::from_kind(ErrorKind::$name(error))
                }
            }
        )*
    }
}
impl_from! {
    impl From<TrapCode> for Error::TrapCode;
    impl From<GlobalError> for Error::Global;
    impl From<MemoryError> for Error::Memory;
    impl From<TableError> for Error::Table;
    impl From<LinkerError> for Error::Linker;
    impl From<InstantiationError> for Error::Instantiation;
    impl From<TranslationError> for Error::Translation;
    impl From<WasmError> for Error::Wasm;
    impl From<ReadError> for Error::Read;
    impl From<FuelError> for Error::Fuel;
    impl From<FuncError> for Error::Func;
    impl From<EnforcedLimitsError> for Error::Limits;
    impl From<ResumableHostTrapError> for Error::ResumableHostTrap;
    impl From<IrError> for Error::Ir;
}
#[cfg(feature = "wat")]
impl_from! {
    impl From<WatError> for Error::Wat;
}
