use crate::{CopyEncoder, EncoderError, OpCode};
use core::ops::Deref;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RefAccess<T>(T);

impl<T> RefAccess<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub unsafe fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for RefAccess<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Trait to query the [`OpCode`] of operators.
///
/// Implemented by [`Op`] and all operators found in the [`crate::Op`] module.
pub trait OperatorCode {
    /// Returns the [`OpCode`] associated to `self`.
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
        f: impl Fn(OpCode) -> T,
    ) -> Result<(), EncoderError>;
}
