#[cfg(feature = "simd")]
use crate::core::simd::ImmLaneIdx;
use crate::{
    Address,
    BranchOffset,
    Decode,
    Decoder,
    Offset16,
    Slot,
    decode::DecodeError,
    index::{Memory, Table},
};

#[derive(Copy, Clone)]
pub struct UnaryOp<Result, Value> {
    pub result: Result,
    pub value: Value,
}

impl<R: Decode, V: Decode> Decode for UnaryOp<R, V> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct BinaryOp<Res, Lhs, Rhs> {
    pub result: Res,
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Res, Lhs, Rhs> Decode for BinaryOp<Res, Lhs, Rhs>
where
    Res: Decode,
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
            offset: Decode::decode(decoder)?,
            lhs: Decode::decode(decoder)?,
            rhs: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct SelectOp<Tval, Fval> {
    pub result: Slot,
    pub condition: Slot,
    pub true_val: Tval,
    pub false_val: Fval,
}

impl<Tval, Fval> Decode for SelectOp<Tval, Fval>
where
    Tval: Decode,
    Fval: Decode,
{
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            condition: Decode::decode(decoder)?,
            true_val: Decode::decode(decoder)?,
            false_val: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct LoadOp_Ss<Res> {
    pub result: Res,
    pub ptr: Slot,
    pub offset: u64,
    pub memory: Memory,
}

impl<Res> Decode for LoadOp_Ss<Res>
where
    Res: Decode,
{
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
pub struct LoadOp_Si<Res> {
    pub result: Res,
    pub address: Address,
    pub memory: Memory,
}

impl<Res> Decode for LoadOp_Si<Res>
where
    Res: Decode,
{
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            address: Decode::decode(decoder)?,
            memory: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct LoadOpMem0Offset16_Ss<Res> {
    pub result: Res,
    pub ptr: Slot,
    pub offset: Offset16,
}

impl<Res> Decode for LoadOpMem0Offset16_Ss<Res>
where
    Res: Decode,
{
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
pub struct V128ExtractLaneOp<const N: u8> {
    pub result: Slot,
    pub value: Slot,
    pub lane: ImmLaneIdx<N>,
}

#[cfg(feature = "simd")]
impl<const N: u8> Decode for V128ExtractLaneOp<N> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
            lane: Decode::decode(decoder)?,
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
pub struct LoadLaneOp_Ss<Res, LaneIdx> {
    pub result: Res,
    pub ptr: Slot,
    pub offset: u64,
    pub memory: Memory,
    pub v128: Slot,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<Res: Decode, LaneIdx: Decode> Decode for LoadLaneOp_Ss<Res, LaneIdx> {
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
pub struct LoadLaneOpMem0Offset16_Ss<Res, LaneIdx> {
    pub result: Res,
    pub ptr: Slot,
    pub offset: Offset16,
    pub v128: Slot,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<Res: Decode, LaneIdx: Decode> Decode for LoadLaneOpMem0Offset16_Ss<Res, LaneIdx> {
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
