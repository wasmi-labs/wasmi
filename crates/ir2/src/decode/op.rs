#[cfg(feature = "simd")]
use crate::core::simd::ImmLaneIdx;
use crate::{
    decode::DecodeError,
    index::{Memory, Table},
    Address,
    BranchOffset,
    Decode,
    Decoder,
    Offset16,
    Slot,
};

#[derive(Copy, Clone)]
pub struct UnaryOp<V> {
    pub result: Slot,
    pub value: V,
}

impl<V: Decode> Decode for UnaryOp<V> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct BinaryOp<Lhs, Rhs> {
    pub result: Slot,
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Decode for BinaryOp<Lhs, Rhs>
where
    Lhs: Decode,
    Rhs: Decode,
{
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            lhs: Decode::decode(decoder)?,
            rhs: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct TernaryOp<A, B, C> {
    pub result: Slot,
    pub a: A,
    pub b: B,
    pub c: C,
}

#[cfg(feature = "simd")]
impl<A, B, C> Decode for TernaryOp<A, B, C>
where
    A: Decode,
    B: Decode,
    C: Decode,
{
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            a: Decode::decode(decoder)?,
            b: Decode::decode(decoder)?,
            c: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct CmpBranchOp<Lhs, Rhs> {
    pub offset: BranchOffset,
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Decode for CmpBranchOp<Lhs, Rhs>
where
    Lhs: Decode,
    Rhs: Decode,
{
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            lhs: Decode::decode(decoder)?,
            rhs: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct CmpSelectOp<Lhs, Rhs> {
    pub result: Slot,
    pub val_true: Slot,
    pub val_false: Slot,
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Decode for CmpSelectOp<Lhs, Rhs>
where
    Lhs: Decode,
    Rhs: Decode,
{
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            val_true: Decode::decode(decoder)?,
            val_false: Decode::decode(decoder)?,
            lhs: Decode::decode(decoder)?,
            rhs: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct LoadOp_Ss {
    pub result: Slot,
    pub ptr: Slot,
    pub offset: u64,
    pub memory: Memory,
}

impl Decode for LoadOp_Ss {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            ptr: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
            memory: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct LoadOp_Si {
    pub result: Slot,
    pub address: Address,
    pub memory: Memory,
}

impl Decode for LoadOp_Si {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            address: Decode::decode(decoder)?,
            memory: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct LoadOpMem0Offset16_Ss {
    pub result: Slot,
    pub ptr: Slot,
    pub offset: Offset16,
}

impl Decode for LoadOpMem0Offset16_Ss {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            ptr: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct StoreOp_S<T> {
    pub ptr: Slot,
    pub offset: u64,
    pub value: T,
    pub memory: Memory,
}

impl<T: Decode> Decode for StoreOp_S<T> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            ptr: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
            memory: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct StoreOp_I<T> {
    pub address: Address,
    pub value: T,
    pub memory: Memory,
}

impl<T: Decode> Decode for StoreOp_I<T> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            address: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
            memory: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct StoreOpMem0Offset16_S<T> {
    pub ptr: Slot,
    pub offset: Offset16,
    pub value: T,
}

impl<T: Decode> Decode for StoreOpMem0Offset16_S<T> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            ptr: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct StoreLaneOp_S<T, LaneIdx> {
    pub ptr: Slot,
    pub offset: u64,
    pub value: T,
    pub memory: Memory,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<T: Decode, LaneIdx: Decode> Decode for StoreLaneOp_S<T, LaneIdx> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            ptr: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
            memory: Decode::decode(decoder)?,
            lane: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct StoreLaneOpMem0Offset16_S<T, LaneIdx> {
    pub ptr: Slot,
    pub offset: Offset16,
    pub value: T,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<T: Decode, LaneIdx: Decode> Decode for StoreLaneOpMem0Offset16_S<T, LaneIdx> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            ptr: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
            lane: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct TableGet<T> {
    pub result: Slot,
    pub index: T,
    pub table: Table,
}

impl<T: Decode> Decode for TableGet<T> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            index: Decode::decode(decoder)?,
            table: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct TableSet<I, V> {
    pub table: Table,
    pub index: I,
    pub value: V,
}

impl<I: Decode, V: Decode> Decode for TableSet<I, V> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            table: Decode::decode(decoder)?,
            index: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct V128ReplaceLaneOp<V, const N: u8> {
    pub result: Slot,
    pub v128: Slot,
    pub value: V,
    pub lane: ImmLaneIdx<N>,
}

#[cfg(feature = "simd")]
impl<V: Decode, const N: u8> Decode for V128ReplaceLaneOp<V, N> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            v128: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
            lane: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct V128LoadLaneOp_Ss<LaneIdx> {
    pub result: Slot,
    pub ptr: Slot,
    pub offset: u64,
    pub memory: Memory,
    pub v128: Slot,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<LaneIdx: Decode> Decode for V128LoadLaneOp_Ss<LaneIdx> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            ptr: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
            memory: Decode::decode(decoder)?,
            v128: Decode::decode(decoder)?,
            lane: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct V128LoadLaneOpMem0Offset16_Ss<LaneIdx> {
    pub result: Slot,
    pub ptr: Slot,
    pub offset: Offset16,
    pub v128: Slot,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<LaneIdx: Decode> Decode for V128LoadLaneOpMem0Offset16_Ss<LaneIdx> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            ptr: Decode::decode(decoder)?,
            offset: Decode::decode(decoder)?,
            v128: Decode::decode(decoder)?,
            lane: Decode::decode(decoder)?,
        })
    }
}
