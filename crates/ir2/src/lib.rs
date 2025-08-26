#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod encode;
mod error;
pub mod index;
mod op;
mod primitive;
mod span;

use wasmi_core as core;

pub use self::{
    encode::{Encode, Encoder},
    error::Error,
    index::Stack,
    op::{Op, OpCode},
    primitive::{Address, BlockFuel, BranchOffset, Offset16, Sign},
    span::{BoundedStackSpan, FixedStackSpan, StackSpan, StackSpanIter},
};
