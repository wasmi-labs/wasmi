#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

mod r#enum;
mod error;
mod for_each_op;
mod immeditate;
pub mod index;
mod primitive;
mod sequence;
mod span;
mod visit_regs;

#[cfg(test)]
mod tests;

use wasmi_core as core;

#[doc(inline)]
pub use self::{
    error::Error,
    immeditate::{AnyConst16, AnyConst32, Const16, Const32},
    index::Instr,
    index::Reg,
    primitive::{
        BlockFuel,
        BranchOffset,
        BranchOffset16,
        Comparator,
        ComparatorAndOffset,
        IntoShiftAmount,
        ShiftAmount,
        Sign,
    },
    r#enum::Instruction,
    sequence::{InstrIter, InstrIterMut, InstrSequence},
    span::{BoundedRegSpan, FixedRegSpan, RegSpan, RegSpanIter},
    visit_regs::VisitRegs,
};
