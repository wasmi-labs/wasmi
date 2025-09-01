#[cfg(feature = "simd")]
use crate::core::simd::ImmLaneIdx;
use crate::{
    index::{Memory, Table},
    Address,
    BranchOffset,
    Decode,
    Decoder,
    Offset16,
    Stack,
};

#[derive(Copy, Clone)]
pub struct UnaryOp<V> {
    pub result: Stack,
    pub value: V,
}

impl<V: Decode> Decode for UnaryOp<V> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            value: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct BinaryOp<Lhs, Rhs> {
    pub result: Stack,
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Decode for BinaryOp<Lhs, Rhs>
where
    Lhs: Decode,
    Rhs: Decode,
{
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            lhs: Decode::decode(decoder),
            rhs: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct CmpBranchOp<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
    pub offset: BranchOffset,
}

impl<Lhs, Rhs> Decode for CmpBranchOp<Lhs, Rhs>
where
    Lhs: Decode,
    Rhs: Decode,
{
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            lhs: Decode::decode(decoder),
            rhs: Decode::decode(decoder),
            offset: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct CmpSelectOp<Lhs, Rhs> {
    pub result: Stack,
    pub lhs: Lhs,
    pub rhs: Rhs,
    pub val_true: Stack,
    pub val_false: Stack,
}

impl<Lhs, Rhs> Decode for CmpSelectOp<Lhs, Rhs>
where
    Lhs: Decode,
    Rhs: Decode,
{
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            lhs: Decode::decode(decoder),
            rhs: Decode::decode(decoder),
            val_true: Decode::decode(decoder),
            val_false: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct LoadOp_Ss {
    pub result: Stack,
    pub ptr: Stack,
    pub offset: u64,
    pub memory: Memory,
}

impl Decode for LoadOp_Ss {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            ptr: Decode::decode(decoder),
            offset: Decode::decode(decoder),
            memory: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct LoadOp_Si {
    pub result: Stack,
    pub address: Address,
    pub memory: Memory,
}

impl Decode for LoadOp_Si {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            address: Decode::decode(decoder),
            memory: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct LoadOpMem0Offset16_Ss {
    pub result: Stack,
    pub ptr: Stack,
    pub offset: Offset16,
}

impl Decode for LoadOpMem0Offset16_Ss {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            ptr: Decode::decode(decoder),
            offset: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct StoreOp_S<T> {
    pub ptr: Stack,
    pub offset: u64,
    pub value: T,
    pub memory: Memory,
}

impl<T: Decode> Decode for StoreOp_S<T> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            ptr: Decode::decode(decoder),
            offset: Decode::decode(decoder),
            value: Decode::decode(decoder),
            memory: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct StoreOp_I<T> {
    pub address: Address,
    pub value: T,
    pub memory: Memory,
}

impl<T: Decode> Decode for StoreOp_I<T> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            address: Decode::decode(decoder),
            value: Decode::decode(decoder),
            memory: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct StoreOpMem0Offset16_S<T> {
    pub ptr: Stack,
    pub offset: Offset16,
    pub value: T,
}

impl<T: Decode> Decode for StoreOpMem0Offset16_S<T> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            ptr: Decode::decode(decoder),
            offset: Decode::decode(decoder),
            value: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct TableGet<T> {
    pub result: Stack,
    pub index: T,
    pub table: Table,
}

impl<T: Decode> Decode for TableGet<T> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            index: Decode::decode(decoder),
            table: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
pub struct TableSet<I, V> {
    pub table: Table,
    pub index: I,
    pub value: V,
}

impl<I: Decode, V: Decode> Decode for TableSet<I, V> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            table: Decode::decode(decoder),
            index: Decode::decode(decoder),
            value: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct V128ReplaceLaneOp<V, const N: u8> {
    pub result: Stack,
    pub v128: Stack,
    pub value: V,
    pub lane: ImmLaneIdx<N>,
}

#[cfg(feature = "simd")]
impl<V: Decode, const N: u8> Decode for V128ReplaceLaneOp<V, N> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            v128: Decode::decode(decoder),
            value: Decode::decode(decoder),
            lane: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct V128LoadLaneOp_Ss<LaneIdx> {
    pub result: Stack,
    pub ptr: Stack,
    pub offset: u64,
    pub memory: Memory,
    pub v128: Stack,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<LaneIdx: Decode> Decode for V128LoadLaneOp_Ss<LaneIdx> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            ptr: Decode::decode(decoder),
            offset: Decode::decode(decoder),
            memory: Decode::decode(decoder),
            v128: Decode::decode(decoder),
            lane: Decode::decode(decoder),
        }
    }
}

#[derive(Copy, Clone)]
#[cfg(feature = "simd")]
pub struct V128LoadLaneOpMem0Offset16_Ss<LaneIdx> {
    pub result: Stack,
    pub ptr: Stack,
    pub offset: Offset16,
    pub v128: Stack,
    pub lane: LaneIdx,
}

#[cfg(feature = "simd")]
impl<LaneIdx: Decode> Decode for V128LoadLaneOpMem0Offset16_Ss<LaneIdx> {
    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {
        Self {
            result: Decode::decode(decoder),
            ptr: Decode::decode(decoder),
            offset: Decode::decode(decoder),
            v128: Decode::decode(decoder),
            lane: Decode::decode(decoder),
        }
    }
}
