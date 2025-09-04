#![allow(non_camel_case_types)]

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
    FixedSlotSpan,
    Offset16,
    OpCode,
    Sign,
    Slot,
    SlotSpan,
};
use core::{mem, num::NonZero};

/// Types that can be used to decode types implementing [`Decode`].
pub trait Decoder {
    /// Reads enough bytes from `self` to populate `buffer`.
    fn read_bytes(&mut self, buffer: &mut [u8]);
}

/// Types that can be decoded using a type that implements [`Decoder`].
pub trait Decode {
    /// Decodes `Self` via `decoder`.
    ///
    /// # Safety
    ///
    /// It is the callers responsibility to ensure that the decoder
    /// decodes items in the order they have been encoded and on valid
    /// positions within the decode stream.
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self;
}

impl Decode for BoundedSlotSpan {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        let span = SlotSpan::decode(decoder);
        let len = u16::decode(decoder);
        Self::new(span, len)
    }
}

impl<const N: u16> Decode for FixedSlotSpan<N> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self::new_unchecked(SlotSpan::decode(decoder))
    }
}

macro_rules! impl_decode_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Decode for $ty {
                unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
                    let mut bytes = [0_u8; mem::size_of::<$ty>()];
                    decoder.read_bytes(&mut bytes);
                    Self::from_ne_bytes(bytes)
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
                unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
                    $e(<$as as Decode>::decode(decoder))
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
    Address as u64 = |address| unsafe { Address::try_from(address).unwrap_unchecked() },
    Sign<f32> as bool = Sign::new,
    Sign<f64> as bool = Sign::new,

    Slot as u16 = Into::into,
    Func as u32 = Into::into,
    FuncType as u32 = Into::into,
    InternalFunc as u32 = Into::into,
    Global as u32 = Into::into,
    Memory as u16 = Into::into,
    Table as u32 = Into::into,
    Data as u32 = Into::into,
    Elem as u32 = Into::into,

    SlotSpan as Slot = SlotSpan::new,

    NonZero<u32> as u32 = |value| unsafe { NonZero::new_unchecked(value) },
    NonZero<u64> as u64 = |value| unsafe { NonZero::new_unchecked(value) },

    TrapCode as u8 = |code: u8| -> TrapCode {
        TrapCode::try_from(code).unwrap_unchecked()
    },
    OpCode as u16 = |code: u16| -> OpCode {
        OpCode::try_from(code).unwrap_unchecked()
    }
}

impl<const N: usize, T: Decode> Decode for [T; N] {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        core::array::from_fn(|_| <T as Decode>::decode(decoder))
    }
}

#[cfg(feature = "simd")]
impl<const N: u8> Decode for ImmLaneIdx<N> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        ImmLaneIdx::try_from(u8::decode(decoder)).unwrap_unchecked()
    }
}

include!(concat!(env!("OUT_DIR"), "/decode.rs"));
