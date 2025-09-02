#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[macro_use]
mod for_each_op;
mod r#enum;
mod error;
mod immeditate;
pub mod index;
mod primitive;
mod span;
mod visit_results;

#[cfg(test)]
mod tests;

use wasmi_core as core;

#[doc(inline)]
pub use self::{
    error::Error,
    immeditate::{AnyConst16, AnyConst32, Const16, Const32},
    index::Slot,
    primitive::{
        Address,
        Address32,
        BlockFuel,
        BranchOffset,
        BranchOffset16,
        Comparator,
        ComparatorAndOffset,
        IntoShiftAmount,
        Offset16,
        Offset64,
        Offset64Hi,
        Offset64Lo,
        Offset8,
        ShiftAmount,
        Sign,
    },
    r#enum::Op,
    span::{BoundedSlotSpan, FixedSlotSpan, SlotSpan, SlotSpanIter},
    visit_results::VisitResults,
};
