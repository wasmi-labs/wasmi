use crate::{Address, BranchOffset, Offset, Reg, Stack};
use core::fmt::{Debug, Display};

pub trait Decode<D: Decoder>: Sized + Copy {
    fn decode(decoder: &mut D) -> Result<Self, D::Error>;
}

impl<D: Decoder> Decode<D> for Reg {
    fn decode(_decoder: &mut D) -> Result<Self, <D as Decoder>::Error> {
        Ok(Reg)
    }
}

macro_rules! impl_decode_for_primitives {
    ( $( $ty:ty ),* $(,)? ) => {
        $(
            impl<D: Decoder> Decode<D> for $ty {
                fn decode(decoder: &mut D) -> Result<Self, D::Error> {
                    let bytes = decoder.decode_bytes()?;
                    Ok(<$ty>::from_ne_bytes(bytes))
                }
            }
        )*
    };
}
impl_decode_for_primitives! {
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
}

impl<E: Decoder> Decode<E> for bool {
    fn decode(decoder: &mut E) -> Result<Self, <E>::Error> {
        let bytes: [u8; 1] = decoder.decode_bytes()?;
        Ok(bytes[0] != 0)
    }
}

macro_rules! impl_decode_for_newtypes {
    ( $( $ty:ty ),* $(,)? ) => {
        $(
            impl<D: Decoder> Decode<D> for $ty {
                fn decode(decoder: &mut D) -> Result<Self, D::Error> {
                    Ok(Self(<_ as Decode<D>>::decode(decoder)?))
                }
            }
        )*
    };
}
impl_decode_for_newtypes! {
    Address, BranchOffset, Offset, Stack
}

pub trait Decoder {
    type Error: Debug + Display + core::error::Error;

    fn decode_bytes<const N: usize>(&mut self) -> Result<[u8; N], Self::Error>;
}
