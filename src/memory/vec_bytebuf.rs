//! An implementation of `ByteBuf` based on a plain `Vec`.

use alloc::prelude::v1::*;

pub struct ByteBuf {
    buf: Vec<u8>,
}

impl ByteBuf {
    pub fn new(len: usize) -> Self {
        let mut buf = Vec::new();
        buf.resize(len, 0u8);
        Self {
            buf,
        }
    }

    pub fn realloc(&mut self, new_len: usize) {
        self.buf.resize(new_len, 0u8);
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_ref()
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.buf.as_mut()
    }
}
