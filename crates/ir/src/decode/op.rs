#[cfg(feature = "simd")]
use crate::core::simd::ImmLaneIdx;
use crate::{
    Address,
    BoundedSlotSpan,
    BranchOffset,
    Decode,
    Decoder,
    Offset16,
    Slot,
    decode::DecodeError,
    index::{FuncType, Global, Memory, Table},
};

#[derive(Copy, Clone)]
pub struct ReturnOp<Value> {
    pub value: Value,
}

impl<V: Decode> Decode for ReturnOp<V> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            value: Decode::decode(decoder)?,
        })
    }
}

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
pub struct SelectOp<Res, Cond, Tval, Fval> {
    pub result: Res,
    pub condition: Cond,
    pub true_val: Tval,
    pub false_val: Fval,
}

impl<Res, Cond, Tval, Fval> Decode for SelectOp<Res, Cond, Tval, Fval>
where
    Res: Decode,
    Cond: Decode,
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
pub struct LoadOp<Res, Ptr> {
    pub result: Res,
    pub ptr: Ptr,
    pub offset: u64,
    pub memory: Memory,
}

impl<Res, Ptr> Decode for LoadOp<Res, Ptr>
where
    Res: Decode,
    Ptr: Decode,
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
pub struct LoadAtOp<Res> {
    pub result: Res,
    pub address: Address,
    pub memory: Memory,
}

impl<Res> Decode for LoadAtOp<Res>
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
pub struct LoadOpMem0Offset16<Res, Ptr> {
    pub result: Res,
    pub ptr: Ptr,
    pub offset: Offset16,
}

impl<Res, Ptr> Decode for LoadOpMem0Offset16<Res, Ptr>
where
    Res: Decode,
    Ptr: Decode,
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
pub struct StoreOp<Ptr, Val> {
    pub ptr: Ptr,
    pub offset: u64,
    pub value: Val,
    pub memory: Memory,
}

impl<Ptr: Decode, Val: Decode> Decode for StoreOp<Ptr, Val> {
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
pub struct StoreAtOp<T> {
    pub address: Address,
    pub value: T,
    pub memory: Memory,
}

impl<T: Decode> Decode for StoreAtOp<T> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            address: Decode::decode(decoder)?,
            value: Decode::decode(decoder)?,
            memory: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct StoreOpMem0Offset16<Ptr, Val> {
    pub ptr: Ptr,
    pub offset: Offset16,
    pub value: Val,
}

impl<Ptr: Decode, Val: Decode> Decode for StoreOpMem0Offset16<Ptr, Val> {
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
pub struct StoreLaneOp<Ptr, Val, LaneIdx> {
    pub ptr: Ptr,
    pub offset: u64,
    pub value: Val,
    pub memory: Memory,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<Ptr: Decode, Val: Decode, LaneIdx: Decode> Decode for StoreLaneOp<Ptr, Val, LaneIdx> {
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
pub struct StoreLaneOpMem0Offset16<Ptr, Val, LaneIdx> {
    pub ptr: Ptr,
    pub offset: Offset16,
    pub value: Val,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<Ptr: Decode, Val: Decode, LaneIdx: Decode> Decode
    for StoreLaneOpMem0Offset16<Ptr, Val, LaneIdx>
{
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
pub struct GlobalGet<T> {
    pub result: T,
    pub global: Global,
}

impl<T: Decode> Decode for GlobalGet<T> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            result: Decode::decode(decoder)?,
            global: Decode::decode(decoder)?,
        })
    }
}

#[derive(Copy, Clone)]
pub struct GlobalSet<T> {
    pub value: T,
    pub global: Global,
}

impl<T: Decode> Decode for GlobalSet<T> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            value: Decode::decode(decoder)?,
            global: Decode::decode(decoder)?,
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
pub struct CallIndirect<I> {
    pub table: Table,
    pub func_type: FuncType,
    pub params: BoundedSlotSpan,
    pub index: I,
}

impl<I: Decode> Decode for CallIndirect<I> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            table: Decode::decode(decoder)?,
            func_type: Decode::decode(decoder)?,
            params: Decode::decode(decoder)?,
            index: Decode::decode(decoder)?,
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
pub struct LoadLaneOp<Res, Ptr, LaneIdx> {
    pub result: Res,
    pub ptr: Ptr,
    pub offset: u64,
    pub memory: Memory,
    pub v128: Slot,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<Res: Decode, Ptr: Decode, LaneIdx: Decode> Decode for LoadLaneOp<Res, Ptr, LaneIdx> {
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
pub struct LoadLaneOpMem0Offset16<Res, Ptr, LaneIdx> {
    pub result: Res,
    pub ptr: Ptr,
    pub offset: Offset16,
    pub v128: Slot,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<Res: Decode, Ptr: Decode, LaneIdx: Decode> Decode
    for LoadLaneOpMem0Offset16<Res, Ptr, LaneIdx>
{
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
