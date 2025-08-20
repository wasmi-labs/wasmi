#![allow(warnings)]
#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod error;
mod index;
mod ops;
mod primitive;
mod span;

use wasmi_core as core;

pub use self::{
    error::Error,
    index::Stack,
    ops::Instruction,
    primitive::{Address, BlockFuel, BranchOffset, Offset16, Sign},
    span::{BoundedStackSpan, FixedStackSpan, StackSpan},
};
