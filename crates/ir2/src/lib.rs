#![allow(warnings)]
#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod error;
mod index;
mod op;
mod primitive;
mod span;

use wasmi_core as core;

pub use self::{
    error::Error,
    index::Stack,
    op::Op,
    primitive::{Address, BlockFuel, BranchOffset, Offset16, Sign},
    span::{BoundedStackSpan, FixedStackSpan, StackSpan},
};
