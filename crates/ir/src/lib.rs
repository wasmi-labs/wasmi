#![allow(clippy::len_without_is_empty)]

mod primitive;

#[cfg(test)]
mod tests;

pub use self::{
    decode::{CheckedOpDecoder, DecodeError, UncheckedOpDecoder},
    dispatch::{OpVariant, UncheckedOpVariantDecoder},
    encode::{OpEncoder, OpIter, OpPos, PatchError},
    primitive::*,
    r#enum::*,
    slice::*,
    visit::Visitor,
};
use wasmi_core as core;

mod decode;
mod dispatch;
mod encode;
mod r#enum;
mod for_each_op;
mod slice;
mod visit;
