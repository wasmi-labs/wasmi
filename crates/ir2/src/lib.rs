#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

mod decode;
mod encode;
#[rustfmt::skip]
mod instr;
mod utils;

use self::utils::RefAccess;
pub use self::{
    decode::{Decode, Decoder},
    encode::{CheckedEncoder, CopyDecoder, CopyEncoder, Encode, Encoder, EncoderError},
    instr::{class, op, Op, OpCode},
    utils::{
        BinaryCommutativeOperator,
        BinaryOperator,
        LoadOperator,
        Operator,
        OperatorCode,
        UnaryOperator,
    },
};

/// Address to load from or store to memory.
#[derive(Debug, Copy, Clone)]
pub struct Address(pub usize);

/// Address offset for load and store operations.
#[derive(Debug, Copy, Clone)]
pub struct Offset(pub u64);

/// Offset for branch instructions.
#[derive(Debug, Copy, Clone)]
pub struct BranchOffset(pub isize);

/// An instruction register input or output.
#[derive(Debug, Copy, Clone)]
pub struct Reg;

/// An instruction input or output of a location within a function.
#[derive(Debug, Copy, Clone)]
pub struct Stack(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct Global(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct Table(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct Memory(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct Func(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct WasmFunc(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct Data(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct Elem(pub u32);

/// A branch table target.
pub struct BranchTableTarget {
    pub result: Stack,
    pub offset: BranchOffset,
}
