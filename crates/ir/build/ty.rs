use core::fmt::{self, Display};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Ty {
    /// A generic signed 8-bit value.
    SignedBits8,
    /// A generic signed 16-bit value.
    SignedBits16,
    /// A generic signed 32-bit value.
    SignedBits32,
    /// A generic signed 64-bit value.
    SignedBits64,
    /// A generic 8-bit value.
    Bits8,
    /// A generic 16-bit value.
    Bits16,
    /// A generic 32-bit value.
    Bits32,
    /// A generic 64-bit value.
    Bits64,
    /// A generic 128-bit value.
    Bits128,
    /// A generic 64-bit (8x8) half-vector value.
    Bits8x8,
    /// A generic 64-bit (16x4) half-vector value.
    Bits16x4,
    /// A generic 64-bit (32x2) half-vector value.
    Bits32x2,
    /// A general 32-bit integer type.
    I32,
    /// A general 64-bit integer type.
    I64,
    /// A unsigned 8-bit integer type.
    U8,
    /// A unsigned 32-bit integer type.
    U32,
    /// A unsigned 64-bit integer type.
    U64,
    /// A non-zero signed 32-bit integer type.
    NonZeroI32,
    /// A non-zero signed 64-bit integer type.
    NonZeroI64,
    /// A non-zero unsigned 32-bit integer type.
    NonZeroU32,
    /// A non-zero unsigned 64-bit integer type.
    NonZeroU64,
    /// A 32-bit float type.
    F32,
    /// A 64-bit float type.
    F64,
    /// A 32-bit float type sign.
    SignF32,
    /// A 64-bit float type sign.
    SignF64,
    /// A generic `simd` vector type.
    V128,
    /// A `i8x16` vector type for `simd`.
    I8x16,
    /// A `i16x8` vector type for `simd`.
    I16x8,
    /// A `i32x4` vector type for `simd`.
    I32x4,
    /// A `i64x2` vector type for `simd`.
    I64x2,
    /// A `u8x16` vector type for `simd`.
    U8x16,
    /// A `u16x8` vector type for `simd`.
    U16x8,
    /// A `u32x4` vector type for `simd`.
    U32x4,
    /// A `u64x2` vector type for `simd`.
    U64x2,
    /// A `f32x4` vector type for `simd`.
    F32x4,
    /// A `f64x2` vector type for `simd`.
    F64x2,
}

#[derive(Copy, Clone)]
pub enum FieldTy {
    Slot,
    SlotSpan,
    BoundedSlotSpan,
    FixedSlotSpan2,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    NonZeroI32,
    NonZeroI64,
    NonZeroU32,
    NonZeroU64,
    SignF32,
    SignF64,
    Address,
    Offset16,
    BranchOffset,
    Memory,
    Table,
    Global,
    Func,
    FuncType,
    InternalFunc,
    Elem,
    Data,
    TrapCode,
    BlockFuel,
    Array16ImmLaneIdx32,
    ImmLaneIdx16,
    ImmLaneIdx8,
    ImmLaneIdx4,
    ImmLaneIdx2,
    Bytes16,
    V128,
}

impl From<Ty> for FieldTy {
    fn from(ty: Ty) -> Self {
        match ty {
            | Ty::SignedBits8 => FieldTy::I8,
            | Ty::SignedBits16 => FieldTy::I16,
            | Ty::SignedBits32 => FieldTy::I32,
            | Ty::SignedBits64 => FieldTy::I64,
            | Ty::Bits8 => FieldTy::U8,
            | Ty::Bits16 => FieldTy::U16,
            | Ty::Bits32 => FieldTy::U32,
            | Ty::Bits64 => FieldTy::U64,
            | Ty::Bits128 => FieldTy::V128,
            | Ty::Bits8x8 => FieldTy::U64,
            | Ty::Bits16x4 => FieldTy::U64,
            | Ty::Bits32x2 => FieldTy::U64,
            | Ty::I32 => FieldTy::I32,
            | Ty::I64 => FieldTy::I64,
            | Ty::U8 => FieldTy::U8,
            | Ty::U32 => FieldTy::U32,
            | Ty::U64 => FieldTy::U64,
            | Ty::NonZeroI32 => FieldTy::NonZeroI32,
            | Ty::NonZeroI64 => FieldTy::NonZeroI64,
            | Ty::NonZeroU32 => FieldTy::NonZeroU32,
            | Ty::NonZeroU64 => FieldTy::NonZeroU64,
            | Ty::F32 => FieldTy::F32,
            | Ty::F64 => FieldTy::F64,
            | Ty::SignF32 => FieldTy::SignF32,
            | Ty::SignF64 => FieldTy::SignF64,
            | Ty::V128
            | Ty::I8x16
            | Ty::I16x8
            | Ty::I32x4
            | Ty::I64x2
            | Ty::U8x16
            | Ty::U16x8
            | Ty::U32x4
            | Ty::U64x2
            | Ty::F32x4
            | Ty::F64x2 => FieldTy::V128,
        }
    }
}

impl Display for FieldTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Slot => "Slot",
            Self::SlotSpan => "SlotSpan",
            Self::BoundedSlotSpan => "BoundedSlotSpan",
            Self::FixedSlotSpan2 => "FixedSlotSpan<2>",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::NonZeroI32 => "NonZero<i32>",
            Self::NonZeroI64 => "NonZero<i64>",
            Self::NonZeroU32 => "NonZero<u32>",
            Self::NonZeroU64 => "NonZero<u64>",
            Self::SignF32 => "Sign<f32>",
            Self::SignF64 => "Sign<f64>",
            Self::Address => "Address",
            Self::Offset16 => "Offset16",
            Self::BranchOffset => "BranchOffset",
            Self::Memory => "Memory",
            Self::Table => "Table",
            Self::Global => "Global",
            Self::Func => "Func",
            Self::FuncType => "FuncType",
            Self::InternalFunc => "InternalFunc",
            Self::Elem => "Elem",
            Self::Data => "Data",
            Self::TrapCode => "TrapCode",
            Self::BlockFuel => "BlockFuel",
            Self::Array16ImmLaneIdx32 => "[ImmLaneIdx<32>; 16]",
            Self::ImmLaneIdx16 => "ImmLaneIdx<16>",
            Self::ImmLaneIdx8 => "ImmLaneIdx<8>",
            Self::ImmLaneIdx4 => "ImmLaneIdx<4>",
            Self::ImmLaneIdx2 => "ImmLaneIdx<2>",
            Self::Bytes16 => "[u8; 16]",
            Self::V128 => "V128",
        };
        f.write_str(s)
    }
}
