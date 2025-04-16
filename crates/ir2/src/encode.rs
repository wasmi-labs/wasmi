use crate::{Address, BranchOffset, Offset, RefAccess, Reg, Stack};
use alloc::vec::Vec;
use core::{
    fmt,
    fmt::{Debug, Display},
    mem,
    mem::ManuallyDrop,
    slice,
};

/// Trait implemented by types that can be encoded to an [`Encoder`].
pub trait Encode<E: Encoder>: Copy {
    /// Encodes `self` to `encoder`.
    ///
    /// # Errors
    ///
    /// If the `encoder` failed to emit `self`.
    fn encode(&self, encoder: &mut E) -> Result<(), E::Error>;
}

impl<E: Encoder> Encode<E> for Reg {
    fn encode(&self, _encoder: &mut E) -> Result<(), <E>::Error> {
        Ok(())
    }
}

macro_rules! impl_encode_for_primitives {
    ( $( $ty:ty ),* $(,)? ) => {
        $(
            impl<E: Encoder> Encode<E> for $ty {
                fn encode(&self, encoder: &mut E) -> Result<(), <E>::Error> {
                    encoder.encode_bytes(&self.to_ne_bytes())
                }
            }
        )*
    };
}
impl_encode_for_primitives! {
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
}

impl<E: Encoder> Encode<E> for bool {
    fn encode(&self, encoder: &mut E) -> Result<(), <E>::Error> {
        u8::from(*self).encode(encoder)
    }
}

macro_rules! impl_encode_for_newtypes {
    ( $( $ty:ty ),* $(,)? ) => {
        $(
            impl<E: Encoder> Encode<E> for $ty {
                fn encode(&self, encoder: &mut E) -> Result<(), <E>::Error> {
                    self.0.encode(encoder)
                }
            }
        )*
    };
}
impl_encode_for_newtypes! {
    Stack,
    Address,
    Offset,
    BranchOffset,
}

/// Implemented by types that can encode types that implement [`Encode`].
pub trait Encoder {
    /// Errors returned by the encoder.
    type Error: Debug + Display + core::error::Error;

    /// Encodes an array of `bytes` to `self`.
    ///
    /// # Errors
    ///
    /// If `self` failed to encode `bytes`.`
    fn encode_bytes<const N: usize>(&mut self, bytes: &[u8; N]) -> Result<(), Self::Error>;
}

/// A safe encoder that performs bounds checks upon encoding.
#[derive(Debug)]
pub struct CheckedEncoder {
    /// The underlying bytes for encoding.
    bytes: Vec<u8>,
}

impl CheckedEncoder {
    /// Ensures there is enough system memory to encode another `additional` bytes.
    ///
    /// # Errors
    ///
    /// If there is not enough system memory left.
    fn ensure_enough_memory(&mut self, additional: usize) -> Result<(), EncoderError> {
        self.bytes
            .try_reserve(additional)
            .map_err(|_| EncoderError::OutOfMemory)
    }
}

impl Encoder for CheckedEncoder {
    type Error = EncoderError;

    fn encode_bytes<const N: usize>(&mut self, bytes: &[u8; N]) -> Result<(), Self::Error> {
        self.ensure_enough_memory(N)?;
        self.bytes.extend(bytes);
        Ok(())
    }
}

/// Errors returned by [`CheckedEncoder`].
#[derive(Debug)]
pub enum EncoderError {
    /// Encountered when the system has not enough memory for an encoding.
    OutOfMemory,
}

impl core::error::Error for EncoderError {}
impl Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            EncoderError::OutOfMemory => "out of system memory",
        };
        write!(f, "{message}")
    }
}

pub struct CopyEncoder {
    start: *mut u8,
    top: *mut u8,
    end: *mut u8,
}

unsafe impl Send for CopyEncoder {}
unsafe impl Sync for CopyEncoder {}

impl Clone for CopyEncoder {
    fn clone(&self) -> Self {
        let vec = self.restore_vec();
        let cloned_vec = vec.clone();
        let (start, top, end) = Self::extract_vec(cloned_vec);
        Self { start, top, end }
    }
}

impl PartialEq for CopyEncoder {
    fn eq(&self, other: &Self) -> bool {
        let vec_self = self.restore_vec();
        let vec_other = other.restore_vec();
        vec_self == vec_other
    }
}

impl CopyEncoder {
    pub fn new() -> Self {
        let vec = ManuallyDrop::new(<Vec<u8>>::new());
        let (start, top, end) = Self::extract_vec(vec);
        Self { start, top, end }
    }

    fn restore_vec(&self) -> RefAccess<ManuallyDrop<Vec<u8>>> {
        let vec = unsafe { <Vec<u8>>::from_raw_parts(self.start, self.len(), self.capacity()) };
        RefAccess::new(ManuallyDrop::new(vec))
    }

    fn restore_vec_mut(&mut self) -> ManuallyDrop<Vec<u8>> {
        let vec_ref = self.restore_vec();
        unsafe { vec_ref.into_inner() }
    }

    fn extract_vec(vec: ManuallyDrop<Vec<u8>>) -> (*mut u8, *mut u8, *mut u8) {
        let mut vec = vec;
        let start = vec.as_mut_ptr();
        let top = unsafe { start.add(vec.len()) };
        let end = unsafe { start.add(vec.capacity()) };
        (start, top, end)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.start, self.len()) }
    }

    pub fn capacity(&self) -> usize {
        self.end.addr() - self.start.addr()
    }

    pub fn remaining_capacity(&self) -> usize {
        self.end.addr() - self.top.addr()
    }

    pub fn len(&self) -> usize {
        self.top.addr() - self.start.addr()
    }

    pub fn encode<T: Copy>(&mut self, value: T) -> Result<(), EncoderError> {
        let len_bytes = mem::size_of::<T>();
        if len_bytes > self.remaining_capacity() {
            self.grow(len_bytes)?;
        }
        unsafe { self.top.cast::<T>().write_unaligned(value) };
        self.top = unsafe { self.top.add(len_bytes) };
        Ok(())
    }

    fn grow(&mut self, additional: usize) -> Result<(), EncoderError> {
        let mut vec = self.restore_vec_mut();
        vec.try_reserve(additional)
            .map_err(|_| EncoderError::OutOfMemory)?;
        (self.start, self.top, self.end) = Self::extract_vec(vec);
        Ok(())
    }
}

impl Debug for CopyEncoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CopyEncoder")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("bytes", &self.as_bytes())
            .finish()
    }
}

impl Drop for CopyEncoder {
    fn drop(&mut self) {
        let mut vec = self.restore_vec_mut();
        unsafe { ManuallyDrop::drop(&mut vec) }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CopyDecoder(*const u8);

unsafe impl Send for CopyDecoder {}
unsafe impl Sync for CopyDecoder {}

impl From<&[u8]> for CopyDecoder {
    fn from(slice: &[u8]) -> CopyDecoder {
        Self(slice.as_ptr())
    }
}

impl From<*const u8> for CopyDecoder {
    fn from(ptr: *const u8) -> Self {
        Self(ptr)
    }
}

impl CopyDecoder {
    pub unsafe fn decode<T: Copy>(&mut self) -> T {
        let value = self.0.cast::<T>().read_unaligned();
        self.0 = self.0.add(mem::size_of::<T>());
        value
    }

    pub unsafe fn offset(self, offset: isize) -> Self {
        Self(self.0.offset(offset))
    }

    pub unsafe fn add(self, offset: usize) -> Self {
        Self(self.0.add(offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Copy, Clone)]
    #[repr(C, packed)]
    pub struct TestData {
        result: u16,
        lhs: i32,
        rhs: Reg,
    }

    #[test]
    fn copy_encoder_works() -> Result<(), EncoderError> {
        let mut enc = CopyEncoder::new();
        enc.encode(true)?;
        assert_eq!(enc.as_bytes(), &[0x01]);
        enc.encode(1_u8)?;
        assert_eq!(enc.as_bytes(), &[0x01, 0x01]);
        enc.encode(42_u32)?;
        assert_eq!(enc.as_bytes(), &[0x01, 0x01, 42, 0, 0, 0]);
        enc.encode(TestData {
            result: 3,
            lhs: -1,
            rhs: Reg,
        })?;
        assert_eq!(
            enc.as_bytes(),
            &[0x01, 0x01, 42, 0, 0, 0, 3, 0, 0xFF, 0xFF, 0xFF, 0xFF]
        );
        Ok(())
    }

    #[test]
    fn copy_encoder_clone_works() -> Result<(), EncoderError> {
        let mut enc = CopyEncoder::new();
        enc.encode(TestData {
            result: 42,
            lhs: -1,
            rhs: Reg,
        })?;
        let enc2 = enc.clone();
        assert_eq!(enc, enc2);
        Ok(())
    }
}
