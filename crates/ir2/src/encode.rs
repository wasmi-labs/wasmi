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
    Op,
    OpCode,
    Sign,
    Slot,
    SlotSpan,
};
use core::num::NonZero;

/// Types that can encode types that implement [`Encode`].
pub trait Encoder {
    /// Position of encoded items.
    type Pos: Copy;
    /// Errors that may be returned during encoding.
    type Error;

    /// Writes `bytes` to the encoder.
    ///
    /// # Errors
    ///
    /// If the encoder cannot encode more `bytes`.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<Self::Pos, Self::Error>;

    /// Encodes the [`OpCode`] to `self`.
    ///
    /// # Note
    /// This API allows the encoder to customize encoding of [`OpCode`], e.g. to
    /// allow for direct or indirect threading encodings where the [`OpCode`] is
    /// either encoded as function pointer or as `u16` value respectively.
    fn encode_op_code(&mut self, code: OpCode) -> Result<Self::Pos, Self::Error>;

    /// Registers an encoded [`BranchOffset`] to the encoder.
    ///
    /// # Errors
    ///
    /// If the encoder cannot register the `branch_offset`.
    fn branch_offset(
        &mut self,
        pos: Self::Pos,
        branch_offset: BranchOffset,
    ) -> Result<(), Self::Error>;
}

/// Types that can be encoded by types that implement [`Encoder`].
pub trait Encode {
    /// Encode `self` to `encoder` and return its position within the `encoder`.
    fn encode<E>(&self, encoder: &mut E) -> Result<E::Pos, E::Error>
    where
        E: Encoder;
}

impl Encode for OpCode {
    fn encode<E>(&self, encoder: &mut E) -> Result<E::Pos, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_op_code(*self)
    }
}

impl Encode for BranchOffset {
    fn encode<E>(&self, encoder: &mut E) -> Result<E::Pos, E::Error>
    where
        E: Encoder,
    {
        let pos = self.to_i32().encode(encoder)?;
        encoder.branch_offset(pos, *self)?;
        Ok(pos)
    }
}

impl Encode for BoundedSlotSpan {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<E::Pos, E::Error> {
        (self.span(), self.len()).encode(encoder)
    }
}

impl<const N: u16> Encode for FixedSlotSpan<N> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<E::Pos, E::Error> {
        self.span().encode(encoder)
    }
}

impl Encode for BranchTableTarget {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<E::Pos, E::Error> {
        (self.results, self.offset).encode(encoder)
    }
}

macro_rules! impl_encode_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Encode for $ty {
                fn encode<E>(&self, encoder: &mut E) -> Result<E::Pos, E::Error>
                where
                    E: Encoder,
                {
                    encoder.write_bytes(&self.to_ne_bytes())
                }
            }
        )*
    };
}
impl_encode_for_primitive!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

macro_rules! impl_encode_using {
    ( $($ty:ty as $prim:ty = $e:expr),* $(,)? ) => {
        $(
            impl Encode for $ty {
                fn encode<E>(&self, encoder: &mut E) -> Result<E::Pos, E::Error>
                where
                    E: Encoder,
                {
                    let conv = |value: &Self| -> $prim { $e(*value) };
                    conv(self).encode(encoder)
                }
            }
        )*
    };
}
impl_encode_using! {
    bool as u8 = Into::into,
    Offset16 as u16 = Into::into,
    BlockFuel as u64 = Into::into,
    Address as u64 = Into::into,
    Slot as u16 = Into::into,
    Func as u32 = Into::into,
    FuncType as u32 = Into::into,
    InternalFunc as u32 = Into::into,
    Global as u32 = Into::into,
    Memory as u16 = Into::into,
    Table as u32 = Into::into,
    Data as u32 = Into::into,
    Elem as u32 = Into::into,

    Sign<f32> as bool = Sign::is_positive,
    Sign<f64> as bool = Sign::is_positive,
    SlotSpan as Slot = SlotSpan::head,
    NonZero<u32> as u32 = NonZero::get,
    NonZero<u64> as u64 = NonZero::get,
    TrapCode as u8 = |code: TrapCode| -> u8 { code as _ },
}

#[cfg(feature = "simd")]
impl<const N: u8> Encode for ImmLaneIdx<N> {
    fn encode<E>(&self, encoder: &mut E) -> Result<E::Pos, E::Error>
    where
        E: Encoder,
    {
        u8::from(*self).encode(encoder)
    }
}

macro_rules! for_tuple {
    ( $mac:ident ) => {
        $mac! { T0 }
        $mac! { T0, T1 }
        $mac! { T0, T1, T2 }
        $mac! { T0, T1, T2, T3 }
        $mac! { T0, T1, T2, T3, T4 }
        $mac! { T0, T1, T2, T3, T4, T5 }
        $mac! { T0, T1, T2, T3, T4, T5, T6 }
    };
}
macro_rules! impl_encode_for_tuple {
    ( $t0:ident $(, $t:ident)* $(,)? ) => {
        impl<$t0: Encode $(, $t: Encode)*> Encode for ($t0, $($t,)*) {
            fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<E::Pos, E::Error> {
                #[allow(non_snake_case)]
                let ($t0, $($t,)*) = self;
                let pos = $t0.encode(encoder)?;
                $( $t.encode(encoder)?; )*
                Ok(pos)
            }
        }
    };
}
for_tuple!(impl_encode_for_tuple);

impl<T: Encode> Encode for &'_ T {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<E::Pos, E::Error> {
        <T as Encode>::encode(*self, encoder)
    }
}

impl<const N: usize, T: Encode> Encode for [T; N] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<E::Pos, E::Error> {
        let Some((first, rest)) = self.split_first() else {
            panic!("cannot encode zero-sized arrays")
        };
        let pos = first.encode(encoder)?;
        for item in rest {
            item.encode(encoder)?;
        }
        Ok(pos)
    }
}

include!(concat!(env!("OUT_DIR"), "/encode.rs"));
