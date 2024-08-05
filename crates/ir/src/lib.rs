#![allow(clippy::len_without_is_empty)]

mod primitive;

pub use self::{decode::*, encode::*, primitive::*, r#enum::*, slice::*, visit::*};
use wasmi_core as core;

mod decode;
mod encode;
mod r#enum;
mod for_each_op;
mod slice;
mod visit;
