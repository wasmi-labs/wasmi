use core::{fmt, fmt::Display};

#[cfg(feature = "std")]
use std::io;

/// Errors returned by [`Read::read`].
#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
    /// The source has reached the end of the stream.
    EndOfStream,
    /// An unknown error occurred.
    UnknownError,
}

impl Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::EndOfStream => write!(f, "encountered unexpected end of stream"),
            ReadError::UnknownError => write!(f, "encountered unknown error"),
        }
    }
}

/// Types implementing this trait act as byte streams.
///
/// # Note
///
/// Provides a subset of the interface provided by Rust's [`std::io::Read`][std_io_read] trait.
///
/// [`Module::new`]: [`crate::Module::new`]
/// [std_io_read]: https://doc.rust-lang.org/std/io/trait.Read.html
pub trait Read {
    /// Pull some bytes from this source into the specified buffer, returning how many bytes were read.
    ///
    /// # Note
    ///
    /// Provides the same guarantees to the caller as [`std::io::Read::read`][io_read_read].
    ///
    /// [io_read_read]: https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read
    ///
    /// # Errors
    ///
    /// - If `self` stream is already at its end.
    /// - For any unknown error returned by the generic [`Read`] implementer.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ReadError>;
}

#[cfg(feature = "std")]
impl<T> Read for T
where
    T: io::Read,
{
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, ReadError> {
        <T as io::Read>::read(self, buffer).map_err(|error| match error.kind() {
            io::ErrorKind::UnexpectedEof => ReadError::EndOfStream,
            _ => ReadError::UnknownError,
        })
    }
}

#[cfg(not(feature = "std"))]
impl<'a> Read for &'a [u8] {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, ReadError> {
        let len_copy = self.len().min(buffer.len());
        buffer[..len_copy].copy_from_slice(&self[..len_copy]);
        *self = &self[len_copy..];
        Ok(len_copy)
    }
}
