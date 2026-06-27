#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
    op::{Location, Op},
    opcode::{InvalidOpCode, OpCode},
    primitive::{
        Address,
        BlockFuel,
        BranchOffset,
        BranchTableTarget,
        Local,
        Offset,
        Offset16,
        Reg,
        SlotAndReg,
    },
    span::{BoundedSlotSpan, FixedSlotSpan, SlotSpan, SlotSpanIter},
};
