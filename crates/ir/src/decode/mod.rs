#![allow(non_camel_case_types)]

//! Definitions and utilities to decode encoded byte streams.

mod op;

use self::op::{
    BinaryOp,
    CmpBranchOp,
    CmpSelectOp,
    LoadOpMem0Offset16_Ss,
    LoadOp_Si,
    LoadOp_Ss,
    StoreOpMem0Offset16_S,
    StoreOp_I,
    StoreOp_S,
    TableGet,
    TableSet,
    UnaryOp,
};
#[cfg(feature = "simd")]
use self::op::{
    StoreLaneOpMem0Offset16_S,
    StoreLaneOp_S,
    TernaryOp,
    V128ExtractLaneOp,
    V128LoadLaneOpMem0Offset16_Ss,
    V128LoadLaneOp_Ss,
    V128ReplaceLaneOp,
};
#[cfg(feature = "simd")]
use crate::core::simd::ImmLaneIdx;
use crate::{
    core::TrapCode,
    index::{Data, Elem, Func, FuncType, Global, InternalFunc, Memory, Table},
    Address,
    BlockFuel,
    BoundedSlotSpan,
    BranchOffset,
    BranchTableTarget,
    FixedSlotSpan,
    Offset16,
    OpCode,
    Sign,
    Slot,
    SlotSpan,
};
use core::{
    error::Error as CoreError,
    fmt,
    mem::{self, MaybeUninit},
    num::NonZero,
};

/// Types that can be used to decode types implementing [`Decode`].
pub trait Decoder {
    /// Reads enough bytes from `self` to populate `buffer`.
    fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), DecodeError>;
}

/// An error that may be returned when decoding items.
#[derive(Debug, Copy, Clone)]
pub enum DecodeError {
    /// The decoder ran out of bytes.
    OutOfBytes,
    /// The decoder found an invalid bit pattern.
    InvalidBitPattern,
}

impl CoreError for DecodeError {}
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DecodeError::OutOfBytes => "ran out of bytes to decode",
            DecodeError::InvalidBitPattern => "encountered invalid bit pattern",
        };
        f.write_str(s)
    }
}

impl Decoder for &'_ [u8] {
    fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), DecodeError> {
        let Some((bytes, rest)) = self.split_at_checked(buffer.len()) else {
            return Err(DecodeError::OutOfBytes);
        };
        buffer.copy_from_slice(bytes);
        *self = rest;
        Ok(())
    }
}

/// Types that can be decoded using a type that implements [`Decoder`].
pub trait Decode: Sized {
    /// Decodes `Self` via `decoder`.
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError>;
}

impl Decode for BoundedSlotSpan {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let span = SlotSpan::decode(decoder)?;
        let len = u16::decode(decoder)?;
        Ok(Self::new(span, len))
    }
}

impl<const N: u16> Decode for FixedSlotSpan<N> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let span = SlotSpan::decode(decoder)?;
        Self::new(span).map_err(|_| DecodeError::InvalidBitPattern)
    }
}

impl Decode for BranchTableTarget {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let results = SlotSpan::decode(decoder)?;
        let offset = BranchOffset::decode(decoder)?;
        Ok(Self::new(results, offset))
    }
}

macro_rules! impl_decode_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Decode for $ty {
                fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
                    let mut bytes = [0_u8; mem::size_of::<$ty>()];
                    decoder.read_bytes(&mut bytes)?;
                    Ok(Self::from_ne_bytes(bytes))
                }
            }
        )*
    };
}
impl_decode_for_primitive!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

macro_rules! impl_decode_using {
    ( $($ty:ty as $as:ty = $e:expr),* $(,)? ) => {
        $(
            impl Decode for $ty {
                fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
                    Ok($e(<$as as Decode>::decode(decoder)?))
                }
            }
        )*
    };
}
impl_decode_using! {
    bool as u8 = |value| value != 0,
    Offset16 as u16 = Into::into,
    BranchOffset as i32 = Into::into,
    BlockFuel as u64 = Into::into,
    Slot as u16 = Into::into,
    Func as u32 = Into::into,
    FuncType as u32 = Into::into,
    InternalFunc as u32 = Into::into,
    Global as u32 = Into::into,
    Memory as u16 = Into::into,
    Table as u32 = Into::into,
    Data as u32 = Into::into,
    Elem as u32 = Into::into,

    Sign<f32> as bool = Sign::new,
    Sign<f64> as bool = Sign::new,
    SlotSpan as Slot = SlotSpan::new,
}

macro_rules! impl_decode_fallible_using {
    ( $($ty:ty as $as:ty = $e:expr),* $(,)? ) => {
        $(
            impl Decode for $ty {
                #[inline(always)]
                fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
                    $e(<$as as Decode>::decode(decoder)?)
                }
            }
        )*
    };
}
impl_decode_fallible_using! {
    Address as u64 = |address| {
        Address::try_from(address).map_err(|_| DecodeError::InvalidBitPattern)
    },
    NonZero<i32> as i32 = |value| {
        NonZero::new(value).ok_or(DecodeError::InvalidBitPattern)
    },
    NonZero<i64> as i64 = |value| {
        NonZero::new(value).ok_or(DecodeError::InvalidBitPattern)
    },
    NonZero<u32> as u32 = |value| {
        NonZero::new(value).ok_or(DecodeError::InvalidBitPattern)
    },
    NonZero<u64> as u64 = |value| {
        NonZero::new(value).ok_or(DecodeError::InvalidBitPattern)
    },
    TrapCode as u8 = |code: u8| {
        TrapCode::try_from(code).map_err(|_| DecodeError::InvalidBitPattern)
    },
    OpCode as u16 = |code: u16| {
        OpCode::try_from(code).map_err(|_| DecodeError::InvalidBitPattern)
    }
}

impl<const N: usize, T: Decode> Decode for [T; N] {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let mut array = <MaybeUninit<[T; N]>>::uninit();
        // Safety: we are going to decode and initialize all array items and won't read any.
        let items = unsafe { &mut *array.as_mut_ptr() };
        for item in items {
            *item = <T as Decode>::decode(decoder)?;
        }
        // Safety: we have decoded and thus initialized all array items.
        let array = unsafe { array.assume_init() };
        Ok(array)
    }
}

#[cfg(feature = "simd")]
impl<const N: u8> Decode for ImmLaneIdx<N> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let byte = u8::decode(decoder)?;
        let lane = ImmLaneIdx::try_from(byte).map_err(|_| DecodeError::InvalidBitPattern)?;
        Ok(lane)
    }
}

include!(concat!(env!("OUT_DIR"), "/decode.rs"));
