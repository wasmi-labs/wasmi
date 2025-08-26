use crate::{
    core::TrapCode,
    index::{Data, Elem, Func, FuncType, Global, InternalFunc, Memory, Table},
    Address,
    BlockFuel,
    BranchOffset,
    Encode,
    Encoder,
    FixedStackSpan,
    Offset16,
    Sign,
    Stack,
    StackSpan,
};
use core::num::NonZero;

include!(concat!(env!("OUT_DIR"), "/op.rs"));

impl Copy for Op {}
impl Clone for Op {
    fn clone(&self) -> Self {
        *self
    }
}

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

#[test]
fn op_size_of_and_alignment() {
    assert_eq!(core::mem::size_of::<Op>(), 24);
    assert_eq!(core::mem::align_of::<Op>(), 8);
}
