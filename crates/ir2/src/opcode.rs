use crate::Op;

include!(concat!(env!("OUT_DIR"), "/op_code.rs"));

impl Copy for OpCode {}
impl Clone for OpCode {
    fn clone(&self) -> Self {
        *self
    }
}
impl From<OpCode> for u16 {
    fn from(code: OpCode) -> Self {
        code as u16
    }
}

/// Indicated an invalid `u16` value for an [`OpCode`].
#[derive(Debug, Copy, Clone)]
pub struct InvalidOpCode;
