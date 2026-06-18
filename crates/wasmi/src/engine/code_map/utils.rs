use alloc::boxed::Box;
use core::{ops, slice};

/// A boxed byte slice that can store some bytes inline.
#[derive(Debug)]
pub enum SmallByteSlice {
    /// The byte slice fits in the inline buffer.
    Small {
        /// The length of the byte slice.
        len: u8,
        /// The bytes stored inline.
        ///
        /// Bytes beyond `len` are zeroed.
        bytes: [u8; Self::MAX_INLINE_SIZE],
    },
    /// The byte slice is too big and allocated on the heap.
    Big(Box<[u8]>),
}

impl Default for SmallByteSlice {
    fn default() -> Self {
        Self::Small {
            len: 0,
            bytes: [0x00; Self::MAX_INLINE_SIZE],
        }
    }
}

impl SmallByteSlice {
    /// The maximum amount of bytes that can be stored inline.
    ///
    /// This value was chosen because it allows for the maximum
    /// amount of bytes to be stored inline with minimal `size_of`.
    const MAX_INLINE_SIZE: usize = 22;

    /// Returns the underlying slice of bytes.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            SmallByteSlice::Small { len, bytes } => &bytes[..usize::from(*len)],
            SmallByteSlice::Big(bytes) => &bytes[..],
        }
    }
}

impl<I> ops::Index<I> for SmallByteSlice
where
    I: slice::SliceIndex<[u8]>,
{
    type Output = <I as slice::SliceIndex<[u8]>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<'a> From<&'a [u8]> for SmallByteSlice {
    fn from(bytes: &'a [u8]) -> Self {
        if bytes.len() <= Self::MAX_INLINE_SIZE {
            let len = bytes.len() as u8;
            let mut buffer = [0x00_u8; Self::MAX_INLINE_SIZE];
            buffer[..usize::from(len)].copy_from_slice(bytes);
            return Self::Small { len, bytes: buffer };
        }
        Self::Big(bytes.into())
    }
}
