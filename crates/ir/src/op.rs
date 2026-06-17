#[cfg(feature = "simd")]
use crate::core::V128;
#[cfg(feature = "simd")]
use crate::core::simd::ImmLaneIdx;
use crate::{
    Address,
    BlockFuel,
    BoundedSlotSpan,
    BranchOffset,
    FixedSlotSpan,
    Local,
    Offset16,
    Reg,
    Slot,
    SlotSpan,
    core::{ShiftAmount, Sign, TrapCode, ValType},
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

/// The location of an operand.
#[derive(Debug, Copy, Clone)]
pub enum Location {
    /// The operand resides in a register of a certain type.
    Reg(ValType),
    /// The operand resides in a stack slot.
    Slot(Slot),
}

impl Location {
    /// Returns `true` if `self` is a [`Location::Reg`].
    pub fn is_reg(&self) -> bool {
        matches!(self, Self::Reg(_))
    }
}

impl Slot {
    #[inline]
    pub fn location(&self) -> Location {
        Location::Slot(*self)
    }
}

impl Reg<i64> {
    #[inline]
    pub fn location(&self) -> Location {
        Location::Reg(ValType::I64)
    }
}

impl Reg<f32> {
    #[inline]
    pub fn location(&self) -> Location {
        Location::Reg(ValType::F32)
    }
}

impl Reg<f64> {
    #[inline]
    pub fn location(&self) -> Location {
        Location::Reg(ValType::F64)
    }
}

#[test]
fn op_size_of_and_alignment() {
    assert_eq!(
        core::mem::size_of::<Op>(),
        match cfg!(feature = "slot16") {
            true => 24,
            false => 32,
        }
    );
    assert_eq!(core::mem::align_of::<Op>(), 8);
}
