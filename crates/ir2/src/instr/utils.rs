use crate::{CopyEncoder, OpCode, EncoderError};

/// Trait to query the [`OpCode`] of operators.
///
/// Implemented by [`Op`] and all operators found in the [`crate::Op`] module.
pub trait OperatorCode {
    fn op_code(&self) -> crate::OpCode;
}

/// Trait to encode operators with customization of their [`OpCode`] encoding.
///
/// Implemented by [`Op`] and all operators found in the [`crate::op`] module.
pub trait EncodeOpAs {
    /// Encodes the operator allowing encoding customization of its [`OpCode`].
    ///
    /// This is useful to allow both direct and indirect dispatch techniques.
    fn encode_op_as<T: Copy>(
        &self,
        encoder: &mut CopyEncoder,
        f: impl Fn(OpCode) -> T
    ) -> Result<(), EncoderError>;
}
