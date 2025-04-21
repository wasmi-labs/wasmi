use crate::{Op, Reg, Stack};
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

/// Trait to query the [`OpCode`][`crate::OpCode`] of operators.
///
/// Implemented by [`Op`][crate::Op] and all operators found in the [`crate::Op`] module.
pub trait OperatorCode {
    /// Returns the [`OpCode`][crate::OpCode] associated to `self`.
    fn op_code(&self) -> crate::OpCode;
}

/// Trait implemented by all operator types.
pub trait Operator: Copy + OperatorCode + Into<Op> {}
impl<T> Operator for T where T: Copy + OperatorCode + Into<Op> {}

/// Class of unary operators.
pub trait UnaryOperator {
    /// The name of the operator class.
    const NAME: &'static str;

    /// The operator variant that takes a [`Reg`] and returns a [`Reg`].
    type OpRr: Operator;

    /// The operator variant that takes a [`Stack`] and returns a [`Reg`].
    type OpRs: Operator;

    /// The operator variant that takes a [`Reg`] and returns a [`Stack`].
    type OpSr: Operator;

    /// The operator variant that takes a [`Stack`] and returns a [`Stack`].
    type OpSs: Operator;

    /// Creates the operator variant that takes a [`Reg`] and returns a [`Reg`].
    fn make_rr(result: Reg, input: Reg) -> Self::OpRr;
    /// Creates the operator variant that takes a [`Stack`] and returns a [`Reg`].
    fn make_rs(result: Reg, input: Stack) -> Self::OpRs;
    /// Creates the operator variant that takes a [`Reg`] and returns a [`Stack`].
    fn make_sr(result: Stack, input: Reg) -> Self::OpSr;
    /// Creates the operator variant that takes a [`Stack`] and returns a [`Stack`].
    fn make_ss(result: Stack, input: Stack) -> Self::OpSs;
}
