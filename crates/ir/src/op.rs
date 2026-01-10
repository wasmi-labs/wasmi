#[cfg(feature = "simd")]
use crate::core::simd::ImmLaneIdx;
use crate::{
    Address,
    BlockFuel,
    BoundedSlotSpan,
    BranchOffset,
    FixedSlotSpan,
    Offset16,
    Sign,
    Slot,
    SlotSpan,
    core::TrapCode,
    index::{Data, Elem, Func, FuncType, Global, InternalFunc, Memory, Table},
};
use core::num::NonZero;

include!(concat!(env!("OUT_DIR"), "/op.rs"));

impl Copy for Op {}
impl Clone for Op {
    fn clone(&self) -> Self {
        *self
    }
}

#[test]
fn op_size_of_and_alignment() {
    assert_eq!(core::mem::size_of::<Op>(), 24);
    assert_eq!(core::mem::align_of::<Op>(), 8);
}
