#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod decode;
mod encode;
mod error;
pub mod index;
mod op;
mod opcode;
mod primitive;
mod span;

use wasmi_core as core;

#[doc(inline)]
pub use self::{
    decode::{Decode, DecodeError, Decoder},
    encode::{Encode, Encoder},
    error::Error,
    index::Slot,
    op::Op,
    opcode::{InvalidOpCode, OpCode},
    primitive::{
        Address,
        BlockFuel,
        BranchOffset,
        BranchTableTarget,
        Freg32,
        Freg64,
        Ireg,
        Offset16,
        Reg,
        Sign,
    },
    span::{BoundedSlotSpan, FixedSlotSpan, SlotSpan, SlotSpanIter},
};
