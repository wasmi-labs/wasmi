use crate::{
    core::TrapCode,
    index::{Data, Elem, Memory, Table},
    Address,
    BlockFuel,
    BranchOffset,
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
