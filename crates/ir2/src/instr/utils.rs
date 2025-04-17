use crate::{CopyEncoder, OpCode, EncoderError};

/// Trait to query the [`OpCode`] of operators.
///
/// Implemented by [`Op`] and all operators found in the [`crate::Op`] module.
pub trait OperatorCode {
    fn op_code(&self) -> crate::OpCode;
}
