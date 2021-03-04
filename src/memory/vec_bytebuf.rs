//! An implementation of `ByteBuf` based on a plain `Vec`.

use alloc::{string::String, vec::Vec};

pub struct ByteBuf {
    buf: Vec<u8>,
}

impl ByteBuf {
    pub fn new(len: usize) -> Result<Self, String> {
        let buf = vec![0; len];
        Ok(Self { buf })
    }

    pub fn realloc(&mut self, new_len: usize) -> Result<(), String> {
        self.buf.resize(new_len, 0u8);
        Ok(())
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

    pub fn erase(&mut self) -> Result<(), String> {
        for v in &mut self.buf {
            *v = 0;
        }
        Ok(())
    }
}
