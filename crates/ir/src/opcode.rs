use crate::Op;
use core::mem;

include!(concat!(env!("OUT_DIR"), "/op_code.rs"));

impl Copy for OpCode {}
impl Clone for OpCode {
    fn clone(&self) -> Self {
        *self
    }
}
impl From<OpCode> for u16 {
    #[inline]
    fn from(code: OpCode) -> Self {
        code as u16
    }
}

impl OpCode {
    /// Creates a new [`OpCode`] from `code` if `code` is within bounds.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn new(code: u16) -> Option<Self> {
        if usize::from(code) >= crate::LEN_OPS {
            return None;
        }
        // SAFETY: `OpCode` forms a contiguous set of indices up to `LEN_OPS`.
        //         Since `code` has been asserted to be `< LEN_OPS` this transmute is safe.
        Some(unsafe { mem::transmute::<u16, Self>(code) })
    }
}
