#![cfg_attr(not(feature = "std"), no_std)]

mod r#enum;
mod error;
mod for_each_op;
mod immeditate;
pub mod index;
mod primitive;
mod relink_result;
mod sequence;
mod visit_input_regs;

use wasmi_core as core;

#[doc(inline)]
pub use self::{
    error::Error,
    immeditate::{AnyConst32, Const16, Const32},
    index::Reg,
    primitive::{
        BlockFuel,
        BranchOffset,
        BranchOffset16,
        Comparator,
        ComparatorAndOffset,
        Instr,
        RegSpan,
        RegSpanIter,
        Sign,
    },
    r#enum::Instruction,
    sequence::{InstrIter, InstrIterMut, InstrSequence},
};
