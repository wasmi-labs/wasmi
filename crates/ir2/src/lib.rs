#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod decode;
mod encode;
mod error;
pub mod index;
mod op;
mod primitive;
mod span;

use wasmi_core as core;

#[doc(inline)]
pub use self::{
    decode::{Decode, Decoder},
    encode::{Encode, Encoder},
    error::Error,
    index::Slot,
    op::{Op, OpCode},
    primitive::{Address, BlockFuel, BranchOffset, Offset16, Sign},
    span::{BoundedSlotSpan, FixedSlotSpan, SlotSpan, SlotSpanIter},
};
