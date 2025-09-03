use crate::build::{CamelCase, Ident, SnakeCase};
use core::fmt::{self, Display};

macro_rules! apply_macro_for_ops {
    ($mac:ident $(, $param:ident)* $(,)?) => {
        $mac! {
            $($param,)*
            Unary(UnaryOp),
            Binary(BinaryOp),
            CmpBranch(CmpBranchOp),
            CmpSelect(CmpSelectOp),
            Load(LoadOp),
            Store(StoreOp),
            TableGet(TableGetOp),
            TableSet(TableSetOp),
            Generic0(GenericOp<0>),
            Generic1(GenericOp<1>),
            Generic2(GenericOp<2>),
            Generic3(GenericOp<3>),
            Generic4(GenericOp<4>),
            Generic5(GenericOp<5>),
            V128ReplaceLane(V128ReplaceLaneOp),
            V128LoadLane(V128LoadLaneOp),
        }
    };
}

macro_rules! impl_from_for_op {
    (
        $($variant:ident($op_ty:ty)),* $(,)?
    ) => {
        #[derive(Copy, Clone)]
        pub enum Op {
            $(
                $variant($op_ty),
            )*
        }

        $(
            impl From<$op_ty> for Op {
                fn from(op: $op_ty) -> Self {
                    Op::$variant(op)
                }
            }
        )*
    };
}
apply_macro_for_ops!(impl_from_for_op);

#[derive(Copy, Clone)]
pub struct Field {
    pub ident: Ident,
    pub ty: FieldTy,
}

impl Field {
    pub fn new(ident: Ident, ty: FieldTy) -> Self {
        Self { ident, ty }
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ident = SnakeCase(self.ident);
        let ty = self.ty;
        write!(f, "{ident}: {ty}")
    }
}

/// The kind of an operand of an [`Op`].
#[derive(Copy, Clone)]
pub enum OperandKind {
    /// The operand is a [`Slot`] index.
    Slot,
    /// The operand is an immediate value.
    Immediate,
}

impl Display for CamelCase<OperandKind> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0 {
            OperandKind::Slot => "S",
            OperandKind::Immediate => "I",
        };
        write!(f, "{s}")
    }
}

impl Display for SnakeCase<OperandKind> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0 {
            OperandKind::Slot => "s",
            OperandKind::Immediate => "i",
        };
        write!(f, "{s}")
    }
}

#[derive(Copy, Clone)]
pub struct GenericOp<const N: usize> {
    pub ident: Ident,
    pub fields: [Field; N],
}

impl<const N: usize> GenericOp<N> {
    pub fn new(ident: Ident, fields: [Field; N]) -> Self {
        Self { ident, fields }
    }

    pub fn has_result(&self) -> bool {
        self.fields
            .iter()
            .any(|field| matches!(field.ident, Ident::Result))
    }
}

#[derive(Copy, Clone)]
pub struct UnaryOp {
    pub kind: UnaryOpKind,
    pub value: OperandKind,
}

impl UnaryOp {
    pub fn new(kind: UnaryOpKind, value: OperandKind) -> Self {
        Self { kind, value }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn value_field(&self) -> Field {
        let ty = match self.value {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => {
                let value_ty = self.kind.value_ty();
                match value_ty.to_field_ty() {
                    Some(ty) => ty,
                    None => panic!("no `FieldTy` for `Ty`: {value_ty}"),
                }
            }
        };
        Field::new(Ident::Value, ty)
    }

    pub fn fields(&self) -> [Field; 2] {
        [self.result_field(), self.value_field()]
    }
}

#[derive(Copy, Clone)]
pub enum UnaryOpKind {
    I32Clz,
    I32Ctz,
    I32Popcnt,

    I64Clz,
    I64Ctz,
    I64Popcnt,

    I32WrapI64,

    I32Sext8,
    I32Sext16,
    I64Sext8,
    I64Sext16,
    I64Sext32,

    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,

    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,

    S32TruncF32,
    U32TruncF32,
    S32TruncF64,
    U32TruncF64,
    S64TruncF32,
    U64TruncF32,
    S64TruncF64,
    U64TruncF64,

    S32TruncSatF32,
    U32TruncSatF32,
    S32TruncSatF64,
    U32TruncSatF64,
    S64TruncSatF32,
    U64TruncSatF32,
    S64TruncSatF64,
    U64TruncSatF64,

    F32DemoteF64,
    F64PromoteF32,

    F32ConvertS32,
    F32ConvertU32,
    F32ConvertS64,
    F32ConvertU64,

    F64ConvertS32,
    F64ConvertU32,
    F64ConvertS64,
    F64ConvertU64,

    // SIMD: Generic Unary Ops
    V128Splat32,
    V128Splat64,
    V128Not,
    V128AnyTrue,
    // SIMD: `i8x16` Unary Ops
    I8x16Abs,
    I8x16Neg,
    I8x16Popcnt,
    I8x16AllTrue,
    I8x16Bitmask,
    // SIMD: `i16x8` Unary Ops
    I16x8Abs,
    I16x8Neg,
    I16x8AllTrue,
    I16x8Bitmask,
    S16x8ExtaddPairwiseI8x16,
    U16x8ExtaddPairwiseI8x16,
    S16x8ExtendLowI8x16,
    U16x8ExtendLowI8x16,
    S16x8ExtendHighI8x16,
    U16x8ExtendHighI8x16,
    // SIMD: `i32x4` Unary Ops
    I32x4Abs,
    I32x4Neg,
    I32x4AllTrue,
    I32x4Bitmask,
    S32x4ExtaddPairwiseI16x8,
    U32x4ExtaddPairwiseI16x8,
    S32x4ExtendLowI16x8,
    U32x4ExtendLowI16x8,
    S32x4ExtendHighI16x8,
    U32x4ExtendHighI16x8,
    // SIMD: `i64x2` Unary Ops
    I64x2Abs,
    I64x2Neg,
    I64x2AllTrue,
    I64x2Bitmask,
    S64x2ExtendLowI32x4,
    U64x2ExtendLowI32x4,
    S64x2ExtendHighI32x4,
    U64x2ExtendHighI32x4,
    // SIMD: `f32x4` Unary Ops
    F32x4DemoteZeroF64x2,
    F32x4Ceil,
    F32x4Floor,
    F32x4Trunc,
    F32x4Nearest,
    F32x4Abs,
    F32x4Neg,
    F32x4Sqrt,
    // SIMD: `f64x2` Unary Ops
    F64x2PromoteLowF32x4,
    F64x2Ceil,
    F64x2Floor,
    F64x2Trunc,
    F64x2Nearest,
    F64x2Abs,
    F64x2Neg,
    F64x2Sqrt,
    // SIMD: Conversions
    S32x4TruncSatF32x4,
    U32x4TruncSatF32x4,
    S32x4TruncSatZeroF64x2,
    U32x4TruncSatZeroF64x2,
    F32x4ConvertS32x4,
    F32x4ConvertU32x4,
    F64x2ConvertLowS32x4,
    F64x2ConvertLowU32x4,
}

impl UnaryOpKind {
    pub fn is_conversion(&self) -> bool {
        self.value_ty() != self.result_ty()
    }

    pub fn value_ty(&self) -> Ty {
        match self {
            | Self::I32Clz | Self::I32Ctz | Self::I32Popcnt => Ty::I32,
            | Self::I64Clz | Self::I64Ctz | Self::I64Popcnt | Self::I32WrapI64 => Ty::I64,
            | Self::I32Sext8 | Self::I32Sext16 => Ty::I32,
            | Self::I64Sext8 | Self::I64Sext16 | Self::I64Sext32 => Ty::I64,
            | Self::F32Abs
            | Self::F32Neg
            | Self::F32Ceil
            | Self::F32Floor
            | Self::F32Trunc
            | Self::F32Nearest
            | Self::F32Sqrt => Ty::F32,
            | Self::F64Abs
            | Self::F64Neg
            | Self::F64Ceil
            | Self::F64Floor
            | Self::F64Trunc
            | Self::F64Nearest
            | Self::F64Sqrt => Ty::F64,
            | Self::S32TruncF32 | Self::U32TruncF32 => Ty::F32,
            | Self::S32TruncF64 | Self::U32TruncF64 => Ty::F64,
            | Self::S64TruncF32 | Self::U64TruncF32 => Ty::F32,
            | Self::S64TruncF64 | Self::U64TruncF64 => Ty::F64,
            | Self::S32TruncSatF32 | Self::U32TruncSatF32 => Ty::F32,
            | Self::S32TruncSatF64 | Self::U32TruncSatF64 => Ty::F64,
            | Self::S64TruncSatF32 | Self::U64TruncSatF32 => Ty::F32,
            | Self::S64TruncSatF64 | Self::U64TruncSatF64 | Self::F32DemoteF64 => Ty::F64,
            | Self::F64PromoteF32 => Ty::F32,
            | Self::F32ConvertS32 => Ty::I32,
            | Self::F32ConvertU32 => Ty::U32,
            | Self::F32ConvertS64 => Ty::I64,
            | Self::F32ConvertU64 => Ty::U64,
            | Self::F64ConvertS32 => Ty::I32,
            | Self::F64ConvertU32 => Ty::U32,
            | Self::F64ConvertS64 => Ty::I64,
            | Self::F64ConvertU64 => Ty::U64,

            // SIMD: Generic Unary Ops
            | Self::V128Splat32 => Ty::B32,
            | Self::V128Splat64 => Ty::B64,
            | Self::V128Not | Self::V128AnyTrue => Ty::V128,
            // SIMD: `i8x16` Unary Ops
            | Self::I8x16Abs
            | Self::I8x16Neg
            | Self::I8x16Popcnt
            | Self::I8x16AllTrue
            | Self::I8x16Bitmask => Ty::I8x16,
            // SIMD: `i16x8` Unary Ops
            | Self::I16x8Abs | Self::I16x8Neg | Self::I16x8AllTrue | Self::I16x8Bitmask => {
                Ty::I16x8
            }
            | Self::S16x8ExtaddPairwiseI8x16
            | Self::S16x8ExtendLowI8x16
            | Self::S16x8ExtendHighI8x16
            | Self::U16x8ExtaddPairwiseI8x16
            | Self::U16x8ExtendLowI8x16
            | Self::U16x8ExtendHighI8x16 => Ty::I8x16,
            // SIMD: `i32x4` Unary Ops
            | Self::I32x4Abs | Self::I32x4Neg | Self::I32x4AllTrue | Self::I32x4Bitmask => {
                Ty::I32x4
            }
            | Self::S32x4ExtaddPairwiseI16x8
            | Self::S32x4ExtendLowI16x8
            | Self::S32x4ExtendHighI16x8
            | Self::U32x4ExtaddPairwiseI16x8
            | Self::U32x4ExtendLowI16x8
            | Self::U32x4ExtendHighI16x8 => Ty::I16x8,
            // SIMD: `i64x2` Unary Ops
            | Self::I64x2Abs | Self::I64x2Neg | Self::I64x2AllTrue | Self::I64x2Bitmask => {
                Ty::I64x2
            }
            | Self::S64x2ExtendLowI32x4
            | Self::S64x2ExtendHighI32x4
            | Self::U64x2ExtendLowI32x4
            | Self::U64x2ExtendHighI32x4 => Ty::I32x4,
            // SIMD: `f32x4` Unary Ops
            | Self::F32x4DemoteZeroF64x2 => Ty::F64x2,
            | Self::F32x4Ceil
            | Self::F32x4Floor
            | Self::F32x4Trunc
            | Self::F32x4Nearest
            | Self::F32x4Abs
            | Self::F32x4Neg
            | Self::F32x4Sqrt => Ty::F32x4,
            // SIMD: `f64x2` Unary Ops
            | Self::F64x2PromoteLowF32x4 => Ty::F32x4,
            | Self::F64x2Ceil
            | Self::F64x2Floor
            | Self::F64x2Trunc
            | Self::F64x2Nearest
            | Self::F64x2Abs
            | Self::F64x2Neg
            | Self::F64x2Sqrt => Ty::F64x2,
            // SIMD: Conversions
            | Self::S32x4TruncSatF32x4 => Ty::F32x4,
            | Self::S32x4TruncSatZeroF64x2 => Ty::F64x2,
            | Self::U32x4TruncSatF32x4 => Ty::F32x4,
            | Self::U32x4TruncSatZeroF64x2 => Ty::F64x2,
            | Self::F32x4ConvertS32x4 => Ty::S32x4,
            | Self::F32x4ConvertU32x4 => Ty::U32x4,
            | Self::F64x2ConvertLowS32x4 => Ty::S32x4,
            | Self::F64x2ConvertLowU32x4 => Ty::U32x4,
        }
    }

    pub fn result_ty(&self) -> Ty {
        match self {
            | Self::I32Clz | Self::I32Ctz | Self::I32Popcnt => Ty::I32,
            | Self::I64Clz | Self::I64Ctz | Self::I64Popcnt => Ty::I64,
            | Self::I32WrapI64 | Self::I32Sext8 | Self::I32Sext16 => Ty::I32,
            | Self::I64Sext8 | Self::I64Sext16 | Self::I64Sext32 => Ty::I64,
            | Self::F32Abs
            | Self::F32Neg
            | Self::F32Ceil
            | Self::F32Floor
            | Self::F32Trunc
            | Self::F32Nearest
            | Self::F32Sqrt => Ty::F32,
            | Self::F64Abs
            | Self::F64Neg
            | Self::F64Ceil
            | Self::F64Floor
            | Self::F64Trunc
            | Self::F64Nearest
            | Self::F64Sqrt => Ty::F64,
            | Self::S32TruncF32 | Self::S32TruncF64 => Ty::I32,
            | Self::U32TruncF32 | Self::U32TruncF64 => Ty::U32,
            | Self::S64TruncF32 | Self::S64TruncF64 => Ty::I64,
            | Self::U64TruncF32 | Self::U64TruncF64 => Ty::U64,
            | Self::S32TruncSatF32 | Self::S32TruncSatF64 => Ty::I32,
            | Self::U32TruncSatF32 | Self::U32TruncSatF64 => Ty::U32,
            | Self::S64TruncSatF32 | Self::S64TruncSatF64 => Ty::I64,
            | Self::U64TruncSatF32 | Self::U64TruncSatF64 => Ty::U64,
            | Self::F32DemoteF64 => Ty::F32,
            | Self::F64PromoteF32 => Ty::F64,
            | Self::F32ConvertS32
            | Self::F32ConvertU32
            | Self::F32ConvertS64
            | Self::F32ConvertU64 => Ty::F32,
            | Self::F64ConvertS32
            | Self::F64ConvertU32
            | Self::F64ConvertS64
            | Self::F64ConvertU64 => Ty::F64,

            // SIMD: Generic Unary Ops
            | Self::V128Splat32 | Self::V128Splat64 | Self::V128Not | Self::V128AnyTrue => Ty::V128,
            // SIMD: `i8x16` Unary Ops
            | Self::I8x16Abs
            | Self::I8x16Neg
            | Self::I8x16Popcnt
            | Self::I8x16AllTrue
            | Self::I8x16Bitmask => Ty::I8x16,
            // SIMD: `i16x8` Unary Ops
            | Self::I16x8Abs => Ty::I16x8,
            | Self::I16x8Neg => Ty::I16x8,
            | Self::I16x8AllTrue => Ty::I16x8,
            | Self::I16x8Bitmask => Ty::I16x8,
            | Self::S16x8ExtaddPairwiseI8x16
            | Self::S16x8ExtendLowI8x16
            | Self::S16x8ExtendHighI8x16 => Ty::S16x8,
            | Self::U16x8ExtaddPairwiseI8x16
            | Self::U16x8ExtendLowI8x16
            | Self::U16x8ExtendHighI8x16 => Ty::U16x8,
            // SIMD: `i32x4` Unary Ops
            | Self::I32x4Abs | Self::I32x4Neg | Self::I32x4AllTrue | Self::I32x4Bitmask => {
                Ty::I32x4
            }
            | Self::S32x4ExtaddPairwiseI16x8
            | Self::S32x4ExtendLowI16x8
            | Self::S32x4ExtendHighI16x8 => Ty::S32x4,
            | Self::U32x4ExtaddPairwiseI16x8
            | Self::U32x4ExtendLowI16x8
            | Self::U32x4ExtendHighI16x8 => Ty::U32x4,
            // SIMD: `i64x2` Unary Ops
            | Self::I64x2Abs | Self::I64x2Neg | Self::I64x2AllTrue | Self::I64x2Bitmask => {
                Ty::I64x2
            }
            | Self::S64x2ExtendLowI32x4 | Self::S64x2ExtendHighI32x4 => Ty::S64x2,
            | Self::U64x2ExtendLowI32x4 | Self::U64x2ExtendHighI32x4 => Ty::U64x2,
            // SIMD: `f32x4` Unary Ops
            | Self::F32x4DemoteZeroF64x2
            | Self::F32x4Ceil
            | Self::F32x4Floor
            | Self::F32x4Trunc
            | Self::F32x4Nearest
            | Self::F32x4Abs
            | Self::F32x4Neg
            | Self::F32x4Sqrt => Ty::F32x4,
            // SIMD: `f64x2` Unary Ops
            | Self::F64x2PromoteLowF32x4
            | Self::F64x2Ceil
            | Self::F64x2Floor
            | Self::F64x2Trunc
            | Self::F64x2Nearest
            | Self::F64x2Abs
            | Self::F64x2Neg
            | Self::F64x2Sqrt => Ty::F64x2,
            // SIMD: Conversions
            | Self::S32x4TruncSatF32x4 | Self::S32x4TruncSatZeroF64x2 => Ty::S32x4,
            | Self::U32x4TruncSatF32x4 | Self::U32x4TruncSatZeroF64x2 => Ty::U32x4,
            | Self::F32x4ConvertS32x4 | Self::F32x4ConvertU32x4 => Ty::F32x4,
            | Self::F64x2ConvertLowS32x4 | Self::F64x2ConvertLowU32x4 => Ty::F64x2,
        }
    }

    pub fn ident(&self) -> Ident {
        match self {
            Self::I32Clz => Ident::Clz,
            Self::I32Ctz => Ident::Ctz,
            Self::I32Popcnt => Ident::Popcnt,
            Self::I64Clz => Ident::Clz,
            Self::I64Ctz => Ident::Ctz,
            Self::I64Popcnt => Ident::Popcnt,
            Self::I32WrapI64 => Ident::Wrap,
            Self::I32Sext8 => Ident::Sext8,
            Self::I32Sext16 => Ident::Sext16,
            Self::I64Sext8 => Ident::Sext8,
            Self::I64Sext16 => Ident::Sext16,
            Self::I64Sext32 => Ident::Sext32,
            Self::F32Abs => Ident::Abs,
            Self::F32Neg => Ident::Neg,
            Self::F32Ceil => Ident::Ceil,
            Self::F32Floor => Ident::Floor,
            Self::F32Trunc => Ident::Trunc,
            Self::F32Nearest => Ident::Nearest,
            Self::F32Sqrt => Ident::Sqrt,
            Self::F64Abs => Ident::Abs,
            Self::F64Neg => Ident::Neg,
            Self::F64Ceil => Ident::Ceil,
            Self::F64Floor => Ident::Floor,
            Self::F64Trunc => Ident::Trunc,
            Self::F64Nearest => Ident::Nearest,
            Self::F64Sqrt => Ident::Sqrt,
            Self::S32TruncF32 => Ident::Trunc,
            Self::U32TruncF32 => Ident::Trunc,
            Self::S32TruncF64 => Ident::Trunc,
            Self::U32TruncF64 => Ident::Trunc,
            Self::S64TruncF32 => Ident::Trunc,
            Self::U64TruncF32 => Ident::Trunc,
            Self::S64TruncF64 => Ident::Trunc,
            Self::U64TruncF64 => Ident::Trunc,
            Self::S32TruncSatF32 => Ident::TruncSat,
            Self::U32TruncSatF32 => Ident::TruncSat,
            Self::S32TruncSatF64 => Ident::TruncSat,
            Self::U32TruncSatF64 => Ident::TruncSat,
            Self::S64TruncSatF32 => Ident::TruncSat,
            Self::U64TruncSatF32 => Ident::TruncSat,
            Self::S64TruncSatF64 => Ident::TruncSat,
            Self::U64TruncSatF64 => Ident::TruncSat,
            Self::F32DemoteF64 => Ident::Demote,
            Self::F64PromoteF32 => Ident::Promote,
            Self::F32ConvertS32 => Ident::Convert,
            Self::F32ConvertU32 => Ident::Convert,
            Self::F32ConvertS64 => Ident::Convert,
            Self::F32ConvertU64 => Ident::Convert,
            Self::F64ConvertS32 => Ident::Convert,
            Self::F64ConvertU32 => Ident::Convert,
            Self::F64ConvertS64 => Ident::Convert,
            Self::F64ConvertU64 => Ident::Convert,

            // SIMD: Generic Unary Ops
            Self::V128Splat32 => Ident::Splat,
            Self::V128Splat64 => Ident::Splat,
            Self::V128Not => Ident::Not,
            Self::V128AnyTrue => Ident::AnyTrue,
            // SIMD: `i8x16` Unary Ops
            Self::I8x16Abs => Ident::Abs,
            Self::I8x16Neg => Ident::Neg,
            Self::I8x16Popcnt => Ident::Popcnt,
            Self::I8x16AllTrue => Ident::AllTrue,
            Self::I8x16Bitmask => Ident::Bitmask,
            // SIMD: `i16x8` Unary Ops
            Self::I16x8Abs => Ident::Abs,
            Self::I16x8Neg => Ident::Neg,
            Self::I16x8AllTrue => Ident::AllTrue,
            Self::I16x8Bitmask => Ident::Bitmask,
            Self::S16x8ExtaddPairwiseI8x16 => Ident::ExtaddPairwise,
            Self::U16x8ExtaddPairwiseI8x16 => Ident::ExtaddPairwise,
            Self::S16x8ExtendLowI8x16 => Ident::ExtendLow,
            Self::U16x8ExtendLowI8x16 => Ident::ExtendLow,
            Self::S16x8ExtendHighI8x16 => Ident::ExtendHigh,
            Self::U16x8ExtendHighI8x16 => Ident::ExtendHigh,
            // SIMD: `i32x4` Unary Ops
            Self::I32x4Abs => Ident::Abs,
            Self::I32x4Neg => Ident::Neg,
            Self::I32x4AllTrue => Ident::AllTrue,
            Self::I32x4Bitmask => Ident::Bitmask,
            Self::S32x4ExtaddPairwiseI16x8 => Ident::ExtaddPairwise,
            Self::U32x4ExtaddPairwiseI16x8 => Ident::ExtaddPairwise,
            Self::S32x4ExtendLowI16x8 => Ident::ExtendLow,
            Self::U32x4ExtendLowI16x8 => Ident::ExtendLow,
            Self::S32x4ExtendHighI16x8 => Ident::ExtendHigh,
            Self::U32x4ExtendHighI16x8 => Ident::ExtendHigh,
            // SIMD: `i64x2` Unary Ops
            Self::I64x2Abs => Ident::Abs,
            Self::I64x2Neg => Ident::Neg,
            Self::I64x2AllTrue => Ident::AllTrue,
            Self::I64x2Bitmask => Ident::Bitmask,
            Self::S64x2ExtendLowI32x4 => Ident::ExtendLow,
            Self::U64x2ExtendLowI32x4 => Ident::ExtendLow,
            Self::S64x2ExtendHighI32x4 => Ident::ExtendHigh,
            Self::U64x2ExtendHighI32x4 => Ident::ExtendHigh,
            // SIMD: `f32x4` Unary Ops
            Self::F32x4DemoteZeroF64x2 => Ident::DemoteZero,
            Self::F32x4Ceil => Ident::Ceil,
            Self::F32x4Floor => Ident::Floor,
            Self::F32x4Trunc => Ident::Trunc,
            Self::F32x4Nearest => Ident::Nearest,
            Self::F32x4Abs => Ident::Abs,
            Self::F32x4Neg => Ident::Neg,
            Self::F32x4Sqrt => Ident::Sqrt,
            // SIMD: `f64x2` Unary Ops
            Self::F64x2PromoteLowF32x4 => Ident::PromoteLow,
            Self::F64x2Ceil => Ident::Ceil,
            Self::F64x2Floor => Ident::Floor,
            Self::F64x2Trunc => Ident::Trunc,
            Self::F64x2Nearest => Ident::Nearest,
            Self::F64x2Abs => Ident::Abs,
            Self::F64x2Neg => Ident::Neg,
            Self::F64x2Sqrt => Ident::Sqrt,
            // SIMD: Conversions
            Self::S32x4TruncSatF32x4 => Ident::TruncSat,
            Self::U32x4TruncSatF32x4 => Ident::TruncSat,
            Self::S32x4TruncSatZeroF64x2 => Ident::TruncSatZero,
            Self::U32x4TruncSatZeroF64x2 => Ident::TruncSatZero,
            Self::F32x4ConvertS32x4 => Ident::Convert,
            Self::F32x4ConvertU32x4 => Ident::Convert,
            Self::F64x2ConvertLowS32x4 => Ident::ConvertLow,
            Self::F64x2ConvertLowU32x4 => Ident::ConvertLow,
        }
    }
}

#[derive(Copy, Clone)]
pub struct BinaryOp {
    pub kind: BinaryOpKind,
    pub lhs: OperandKind,
    pub rhs: OperandKind,
}

impl BinaryOp {
    pub fn new(kind: BinaryOpKind, lhs: OperandKind, rhs: OperandKind) -> Self {
        Self { kind, lhs, rhs }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn lhs_field(&self) -> Field {
        Field::new(Ident::Lhs, self.kind.lhs_field(self.lhs))
    }

    pub fn rhs_field(&self) -> Field {
        Field::new(Ident::Rhs, self.kind.rhs_field(self.rhs))
    }

    pub fn fields(&self) -> [Field; 3] {
        [self.result_field(), self.lhs_field(), self.rhs_field()]
    }
}

#[derive(Copy, Clone)]
pub enum BinaryOpKind {
    // Compare operators.
    Cmp(CmpOpKind),
    // Binary operators: i32
    I32Add,
    I32Sub,
    I32Mul,
    S32Div,
    U32Div,
    S32Rem,
    U32Rem,
    I32BitAnd,
    I32BitOr,
    I32BitXor,
    I32Shl,
    S32Shr,
    U32Shr,
    I32Rotl,
    I32Rotr,
    // Binary operators: i64
    I64Add,
    I64Sub,
    I64Mul,
    S64Div,
    U64Div,
    S64Rem,
    U64Rem,
    I64BitAnd,
    I64BitOr,
    I64BitXor,
    I64Shl,
    S64Shr,
    U64Shr,
    I64Rotl,
    I64Rotr,
    // Binary operators: f32
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    // Binary operators: f64
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    // Simd Operators
    I8x16Swizzle,
    I8x16Eq,
    I8x16NotEq,
    I16x8Eq,
    I16x8NotEq,
    I32x4Eq,
    I32x4NotEq,
    I64x2Eq,
    I64x2NotEq,
    S8x16Lt,
    S8x16Le,
    S16x8Lt,
    S16x8Le,
    S32x4Lt,
    S32x4Le,
    S64x2Lt,
    S64x2Le,
    U8x16Lt,
    U8x16Le,
    U16x8Lt,
    U16x8Le,
    U32x4Lt,
    U32x4Le,
    U64x2Lt,
    U64x2Le,
    F32x4Eq,
    F32x4NotEq,
    F32x4Lt,
    F32x4Le,
    F64x2Eq,
    F64x2NotEq,
    F64x2Lt,
    F64x2Le,
    V128And,
    V128AndNot,
    V128Or,
    V128Xor,
    // i8x16 Ops
    S8x16NarrowI16x8,
    U8x16NarrowI16x8,
    I8x16Add,
    S8x16AddSat,
    U8x16AddSat,
    I8x16Sub,
    S8x16SubSat,
    U8x16SubSat,
    S8x16Min,
    U8x16Min,
    S8x16Max,
    U8x16Max,
    U8x16Avgr,
    // i16x8 Ops
    S16x8Q15MulrSat,
    S16x8NarrowI32x4,
    U16x8NarrowI32x4,
    S16x8ExtmulLowI8x16,
    U16x8ExtmulLowI8x16,
    S16x8ExtmulHighI8x16,
    U16x8ExtmulHighI8x16,
    I16x8Add,
    S16x8AddSat,
    U16x8AddSat,
    I16x8Sub,
    S16x8SubSat,
    U16x8SubSat,
    I16x8Mul,
    S16x8Min,
    U16x8Min,
    S16x8Max,
    U16x8Max,
    U16x8Avgr,
    // i32x4 Ops
    I32x4Add,
    I32x4Sub,
    I32x4Mul,
    S32x4Min,
    U32x4Min,
    S32x4Max,
    U32x4Max,
    S32x4DotI16x8,
    S32x4ExtmulLowI16x8,
    U32x4ExtmulLowI16x8,
    S32x4ExtmulHighI16x8,
    U32x4ExtmulHighI16x8,
    // i64x2 Ops
    I64x2Add,
    I64x2Sub,
    I64x2Mul,
    S64x2ExtmulLowI32x4,
    U64x2ExtmulLowI32x4,
    S64x2ExtmulHighI32x4,
    U64x2ExtmulHighI32x4,
    // f32x4 Ops
    F32x4Add,
    F32x4Sub,
    F32x4Mul,
    F32x4Div,
    F32x4Min,
    F32x4Max,
    F32x4Pmin,
    F32x4Pmax,
    // f64x2 Ops
    F64x2Add,
    F64x2Sub,
    F64x2Mul,
    F64x2Div,
    F64x2Min,
    F64x2Max,
    F64x2Pmin,
    F64x2Pmax,
    // Simd Shift Ops
    I8x16Shl,
    S8x16Shr,
    U8x16Shr,
    I16x8Shl,
    S16x8Shr,
    U16x8Shr,
    I32x4Shl,
    S32x4Shr,
    U32x4Shr,
    I64x2Shl,
    S64x2Shr,
    U64x2Shr,
    // Relaxed SIMD
    S16x8RelaxedDotI8x16I7x16,
    S32x4RelaxedDotI8x16I7x16Add,
    F32x4RelaxedMadd,
    F32x4RelaxedNmadd,
    F64x2RelaxedMadd,
    F64x2RelaxedNmadd,
}

impl BinaryOpKind {
    pub fn ident(&self) -> Ident {
        match self {
            Self::Cmp(cmp) => cmp.ident(),
            Self::I32Add => Ident::Add,
            Self::I32Sub => Ident::Sub,
            Self::I32Mul => Ident::Mul,
            Self::S32Div => Ident::Div,
            Self::U32Div => Ident::Div,
            Self::S32Rem => Ident::Rem,
            Self::U32Rem => Ident::Rem,
            Self::I32BitAnd => Ident::BitAnd,
            Self::I32BitOr => Ident::BitOr,
            Self::I32BitXor => Ident::BitXor,
            Self::I32Shl => Ident::Shl,
            Self::S32Shr => Ident::Shr,
            Self::U32Shr => Ident::Shr,
            Self::I32Rotl => Ident::Rotl,
            Self::I32Rotr => Ident::Rotr,
            Self::I64Add => Ident::Add,
            Self::I64Sub => Ident::Sub,
            Self::I64Mul => Ident::Mul,
            Self::S64Div => Ident::Div,
            Self::U64Div => Ident::Div,
            Self::S64Rem => Ident::Rem,
            Self::U64Rem => Ident::Rem,
            Self::I64BitAnd => Ident::BitAnd,
            Self::I64BitOr => Ident::BitOr,
            Self::I64BitXor => Ident::BitXor,
            Self::I64Shl => Ident::Shl,
            Self::S64Shr => Ident::Shr,
            Self::U64Shr => Ident::Shr,
            Self::I64Rotl => Ident::Rotl,
            Self::I64Rotr => Ident::Rotr,
            Self::F32Add => Ident::Add,
            Self::F32Sub => Ident::Sub,
            Self::F32Mul => Ident::Mul,
            Self::F32Div => Ident::Div,
            Self::F32Min => Ident::Min,
            Self::F32Max => Ident::Max,
            Self::F32Copysign => Ident::Copysign,
            Self::F64Add => Ident::Add,
            Self::F64Sub => Ident::Sub,
            Self::F64Mul => Ident::Mul,
            Self::F64Div => Ident::Div,
            Self::F64Min => Ident::Min,
            Self::F64Max => Ident::Max,
            Self::F64Copysign => Ident::Copysign,
            // Simd Ops
            Self::I8x16Swizzle => Ident::Swizzle,
            Self::I8x16Eq => Ident::Eq,
            Self::I8x16NotEq => Ident::NotEq,
            Self::I16x8Eq => Ident::Eq,
            Self::I16x8NotEq => Ident::NotEq,
            Self::I32x4Eq => Ident::Eq,
            Self::I32x4NotEq => Ident::NotEq,
            Self::I64x2Eq => Ident::Eq,
            Self::I64x2NotEq => Ident::NotEq,
            Self::S8x16Lt => Ident::Lt,
            Self::S8x16Le => Ident::Le,
            Self::S16x8Lt => Ident::Lt,
            Self::S16x8Le => Ident::Le,
            Self::S32x4Lt => Ident::Lt,
            Self::S32x4Le => Ident::Le,
            Self::S64x2Lt => Ident::Lt,
            Self::S64x2Le => Ident::Le,
            Self::U8x16Lt => Ident::Lt,
            Self::U8x16Le => Ident::Le,
            Self::U16x8Lt => Ident::Lt,
            Self::U16x8Le => Ident::Le,
            Self::U32x4Lt => Ident::Lt,
            Self::U32x4Le => Ident::Le,
            Self::U64x2Lt => Ident::Lt,
            Self::U64x2Le => Ident::Le,
            Self::F32x4Eq => Ident::Eq,
            Self::F32x4NotEq => Ident::NotEq,
            Self::F32x4Lt => Ident::Lt,
            Self::F32x4Le => Ident::Le,
            Self::F64x2Eq => Ident::Eq,
            Self::F64x2NotEq => Ident::NotEq,
            Self::F64x2Lt => Ident::Lt,
            Self::F64x2Le => Ident::Le,
            Self::V128And => Ident::And,
            Self::V128AndNot => Ident::AndNot,
            Self::V128Or => Ident::Or,
            Self::V128Xor => Ident::Xor,
            // i8x16 Ops
            Self::S8x16NarrowI16x8 => Ident::NarrowI16x8,
            Self::U8x16NarrowI16x8 => Ident::NarrowI16x8,
            Self::I8x16Add => Ident::Add,
            Self::S8x16AddSat => Ident::AddSat,
            Self::U8x16AddSat => Ident::AddSat,
            Self::I8x16Sub => Ident::Sub,
            Self::S8x16SubSat => Ident::SubSat,
            Self::U8x16SubSat => Ident::SubSat,
            Self::S8x16Min => Ident::Min,
            Self::U8x16Min => Ident::Min,
            Self::S8x16Max => Ident::Max,
            Self::U8x16Max => Ident::Max,
            Self::U8x16Avgr => Ident::Avgr,
            // i16x8 Ops
            Self::S16x8Q15MulrSat => Ident::Q15MulrSat,
            Self::S16x8NarrowI32x4 => Ident::NarrowI32x4,
            Self::U16x8NarrowI32x4 => Ident::NarrowI32x4,
            Self::S16x8ExtmulLowI8x16 => Ident::ExtmulLowI8x16,
            Self::U16x8ExtmulLowI8x16 => Ident::ExtmulLowI8x16,
            Self::S16x8ExtmulHighI8x16 => Ident::ExtmulHighI8x16,
            Self::U16x8ExtmulHighI8x16 => Ident::ExtmulHighI8x16,
            Self::I16x8Add => Ident::Add,
            Self::S16x8AddSat => Ident::AddSat,
            Self::U16x8AddSat => Ident::AddSat,
            Self::I16x8Sub => Ident::Sub,
            Self::S16x8SubSat => Ident::SubSat,
            Self::U16x8SubSat => Ident::SubSat,
            Self::I16x8Mul => Ident::Mul,
            Self::S16x8Min => Ident::Min,
            Self::U16x8Min => Ident::Min,
            Self::S16x8Max => Ident::Max,
            Self::U16x8Max => Ident::Max,
            Self::U16x8Avgr => Ident::Avgr,
            // i32x4 Ops
            Self::I32x4Add => Ident::Add,
            Self::I32x4Sub => Ident::Sub,
            Self::I32x4Mul => Ident::Mul,
            Self::S32x4Min => Ident::Min,
            Self::U32x4Min => Ident::Min,
            Self::S32x4Max => Ident::Max,
            Self::U32x4Max => Ident::Max,
            Self::S32x4DotI16x8 => Ident::DotI16x8,
            Self::S32x4ExtmulLowI16x8 => Ident::ExtmulLowI16x8,
            Self::U32x4ExtmulLowI16x8 => Ident::ExtmulLowI16x8,
            Self::S32x4ExtmulHighI16x8 => Ident::ExtmulHighI16x8,
            Self::U32x4ExtmulHighI16x8 => Ident::ExtmulHighI16x8,
            // i64x2 Ops
            Self::I64x2Add => Ident::Add,
            Self::I64x2Sub => Ident::Sub,
            Self::I64x2Mul => Ident::Mul,
            Self::S64x2ExtmulLowI32x4 => Ident::ExtmulLowI32x4,
            Self::U64x2ExtmulLowI32x4 => Ident::ExtmulLowI32x4,
            Self::S64x2ExtmulHighI32x4 => Ident::ExtmulHighI32x4,
            Self::U64x2ExtmulHighI32x4 => Ident::ExtmulHighI32x4,
            // f32x4 Ops
            Self::F32x4Add => Ident::Add,
            Self::F32x4Sub => Ident::Sub,
            Self::F32x4Mul => Ident::Mul,
            Self::F32x4Div => Ident::Div,
            Self::F32x4Min => Ident::Min,
            Self::F32x4Max => Ident::Max,
            Self::F32x4Pmin => Ident::Pmin,
            Self::F32x4Pmax => Ident::Pmax,
            // f64x2 Ops
            Self::F64x2Add => Ident::Add,
            Self::F64x2Sub => Ident::Sub,
            Self::F64x2Mul => Ident::Mul,
            Self::F64x2Div => Ident::Div,
            Self::F64x2Min => Ident::Min,
            Self::F64x2Max => Ident::Max,
            Self::F64x2Pmin => Ident::Pmin,
            Self::F64x2Pmax => Ident::Pmax,
            // Simd Shift Ops
            Self::I8x16Shl => Ident::Shl,
            Self::S8x16Shr => Ident::Shr,
            Self::U8x16Shr => Ident::Shr,
            Self::I16x8Shl => Ident::Shl,
            Self::S16x8Shr => Ident::Shr,
            Self::U16x8Shr => Ident::Shr,
            Self::I32x4Shl => Ident::Shl,
            Self::S32x4Shr => Ident::Shr,
            Self::U32x4Shr => Ident::Shr,
            Self::I64x2Shl => Ident::Shl,
            Self::S64x2Shr => Ident::Shr,
            Self::U64x2Shr => Ident::Shr,
            // Relaxed SIMD
            Self::S16x8RelaxedDotI8x16I7x16 => Ident::RelaxedDotI8x16I7x16,
            Self::S32x4RelaxedDotI8x16I7x16Add => Ident::RelaxedDotI8x16I7x16Add,
            Self::F32x4RelaxedMadd => Ident::RelaxedMadd,
            Self::F32x4RelaxedNmadd => Ident::RelaxedNmadd,
            Self::F64x2RelaxedMadd => Ident::RelaxedMadd,
            Self::F64x2RelaxedNmadd => Ident::RelaxedNmadd,
        }
    }

    pub fn ident_prefix(&self) -> Ty {
        match self {
            | BinaryOpKind::Cmp(op) => op.ident_prefix(),
            | Self::I32Add
            | Self::I32Sub
            | Self::I32Mul
            | Self::I32BitAnd
            | Self::I32BitOr
            | Self::I32BitXor
            | Self::I32Shl
            | Self::I32Rotl
            | Self::I32Rotr => Ty::I32,
            | Self::S32Div | Self::S32Rem | Self::S32Shr => Ty::S32,
            | Self::U32Div | Self::U32Rem | Self::U32Shr => Ty::U32,
            | Self::I64Add
            | Self::I64Sub
            | Self::I64Mul
            | Self::I64BitAnd
            | Self::I64BitOr
            | Self::I64BitXor
            | Self::I64Shl
            | Self::I64Rotl
            | Self::I64Rotr => Ty::I64,
            | Self::S64Div | Self::S64Rem | Self::S64Shr => Ty::S64,
            | Self::U64Div | Self::U64Rem | Self::U64Shr => Ty::U64,
            | Self::F32Add
            | Self::F32Sub
            | Self::F32Mul
            | Self::F32Div
            | Self::F32Min
            | Self::F32Max
            | Self::F32Copysign => Ty::F32,
            | Self::F64Add
            | Self::F64Sub
            | Self::F64Mul
            | Self::F64Div
            | Self::F64Min
            | Self::F64Max
            | Self::F64Copysign => Ty::F64,
            | Self::I8x16Swizzle => Ty::I8x16,
            | Self::I8x16Eq | Self::I8x16NotEq => Ty::I8x16,
            | Self::I16x8Eq | Self::I16x8NotEq => Ty::I16x8,
            | Self::I32x4Eq | Self::I32x4NotEq => Ty::I32x4,
            | Self::I64x2Eq | Self::I64x2NotEq => Ty::I64x2,
            | Self::S8x16Lt | Self::S8x16Le => Ty::S8x16,
            | Self::S16x8Lt | Self::S16x8Le => Ty::S16x8,
            | Self::S32x4Lt | Self::S32x4Le => Ty::S32x4,
            | Self::S64x2Lt | Self::S64x2Le => Ty::S64x2,
            | Self::U8x16Lt | Self::U8x16Le => Ty::U8x16,
            | Self::U16x8Lt | Self::U16x8Le => Ty::U16x8,
            | Self::U32x4Lt | Self::U32x4Le => Ty::U32x4,
            | Self::U64x2Lt | Self::U64x2Le => Ty::U64x2,
            | Self::F32x4Eq | Self::F32x4NotEq | Self::F32x4Lt | Self::F32x4Le => Ty::F32x4,
            | Self::F64x2Eq | Self::F64x2NotEq | Self::F64x2Lt | Self::F64x2Le => Ty::F64x2,
            | Self::V128And | Self::V128AndNot | Self::V128Or | Self::V128Xor => Ty::V128,
            // i8x16 Ops
            | Self::S8x16NarrowI16x8 => Ty::S8x16,
            | Self::U8x16NarrowI16x8 => Ty::U8x16,
            | Self::I8x16Add => Ty::I8x16,
            | Self::S8x16AddSat => Ty::S8x16,
            | Self::U8x16AddSat => Ty::U8x16,
            | Self::I8x16Sub => Ty::I8x16,
            | Self::S8x16SubSat => Ty::S8x16,
            | Self::U8x16SubSat => Ty::U8x16,
            | Self::S8x16Min => Ty::S8x16,
            | Self::U8x16Min => Ty::U8x16,
            | Self::S8x16Max => Ty::S8x16,
            | Self::U8x16Max => Ty::U8x16,
            | Self::U8x16Avgr => Ty::U8x16,
            // i16x8 Ops
            | Self::S16x8Q15MulrSat => Ty::S16x8,
            | Self::S16x8NarrowI32x4 => Ty::S16x8,
            | Self::U16x8NarrowI32x4 => Ty::U16x8,
            | Self::S16x8ExtmulLowI8x16 => Ty::S16x8,
            | Self::U16x8ExtmulLowI8x16 => Ty::U16x8,
            | Self::S16x8ExtmulHighI8x16 => Ty::S16x8,
            | Self::U16x8ExtmulHighI8x16 => Ty::U16x8,
            | Self::I16x8Add => Ty::I16x8,
            | Self::S16x8AddSat => Ty::S16x8,
            | Self::U16x8AddSat => Ty::U16x8,
            | Self::I16x8Sub => Ty::I16x8,
            | Self::S16x8SubSat => Ty::S16x8,
            | Self::U16x8SubSat => Ty::U16x8,
            | Self::I16x8Mul => Ty::I16x8,
            | Self::S16x8Min => Ty::S16x8,
            | Self::U16x8Min => Ty::U16x8,
            | Self::S16x8Max => Ty::S16x8,
            | Self::U16x8Max => Ty::U16x8,
            | Self::U16x8Avgr => Ty::U16x8,
            // i32x4 Ops
            | Self::I32x4Add | Self::I32x4Sub | Self::I32x4Mul => Ty::I32x4,
            | Self::S32x4Min => Ty::S32x4,
            | Self::U32x4Min => Ty::U32x4,
            | Self::S32x4Max => Ty::S32x4,
            | Self::U32x4Max => Ty::U32x4,
            | Self::S32x4DotI16x8 => Ty::S32x4,
            | Self::S32x4ExtmulLowI16x8 => Ty::S32x4,
            | Self::U32x4ExtmulLowI16x8 => Ty::U32x4,
            | Self::S32x4ExtmulHighI16x8 => Ty::S32x4,
            | Self::U32x4ExtmulHighI16x8 => Ty::U32x4,
            // i64x2 Ops
            | Self::I64x2Add | Self::I64x2Sub | Self::I64x2Mul => Ty::I64x2,
            | Self::S64x2ExtmulLowI32x4 => Ty::S64x2,
            | Self::U64x2ExtmulLowI32x4 => Ty::U64x2,
            | Self::S64x2ExtmulHighI32x4 => Ty::S64x2,
            | Self::U64x2ExtmulHighI32x4 => Ty::U64x2,
            // f32x4 Ops
            | Self::F32x4Add
            | Self::F32x4Sub
            | Self::F32x4Mul
            | Self::F32x4Div
            | Self::F32x4Min
            | Self::F32x4Max
            | Self::F32x4Pmin
            | Self::F32x4Pmax => Ty::F32x4,
            // f64x2 Ops
            | Self::F64x2Add
            | Self::F64x2Sub
            | Self::F64x2Mul
            | Self::F64x2Div
            | Self::F64x2Min
            | Self::F64x2Max
            | Self::F64x2Pmin
            | Self::F64x2Pmax => Ty::F64x2,
            // Simd Shift Ops
            | Self::I8x16Shl => Ty::I8x16,
            | Self::S8x16Shr => Ty::S8x16,
            | Self::U8x16Shr => Ty::U8x16,
            | Self::I16x8Shl => Ty::I16x8,
            | Self::S16x8Shr => Ty::S16x8,
            | Self::U16x8Shr => Ty::U16x8,
            | Self::I32x4Shl => Ty::I32x4,
            | Self::S32x4Shr => Ty::S32x4,
            | Self::U32x4Shr => Ty::U32x4,
            | Self::I64x2Shl => Ty::I64x2,
            | Self::S64x2Shr => Ty::S64x2,
            | Self::U64x2Shr => Ty::U64x2,
            // Relaxed SIMD
            | Self::S16x8RelaxedDotI8x16I7x16 => Ty::S16x8,
            | Self::S32x4RelaxedDotI8x16I7x16Add => Ty::S32x4,
            | Self::F32x4RelaxedMadd | Self::F32x4RelaxedNmadd => Ty::F32x4,
            | Self::F64x2RelaxedMadd | Self::F64x2RelaxedNmadd => Ty::F64x2,
        }
    }

    fn lhs_field(&self, input: OperandKind) -> FieldTy {
        match input {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => match self {
                | Self::Cmp(cmp) => cmp.input_field(input),
                | Self::I32Add
                | Self::I32Sub
                | Self::I32Mul
                | Self::S32Div
                | Self::U32Div
                | Self::S32Rem
                | Self::U32Rem
                | Self::I32BitAnd
                | Self::I32BitOr
                | Self::I32BitXor
                | Self::I32Shl
                | Self::S32Shr
                | Self::U32Shr
                | Self::I32Rotl
                | Self::I32Rotr => FieldTy::I32,
                | Self::I64Add
                | Self::I64Sub
                | Self::I64Mul
                | Self::S64Div
                | Self::U64Div
                | Self::S64Rem
                | Self::U64Rem
                | Self::I64BitAnd
                | Self::I64BitOr
                | Self::I64BitXor
                | Self::I64Shl
                | Self::S64Shr
                | Self::U64Shr
                | Self::I64Rotl
                | Self::I64Rotr => FieldTy::I64,
                | Self::F32Add
                | Self::F32Sub
                | Self::F32Mul
                | Self::F32Div
                | Self::F32Min
                | Self::F32Max
                | Self::F32Copysign => FieldTy::F32,
                | Self::F64Add
                | Self::F64Sub
                | Self::F64Mul
                | Self::F64Div
                | Self::F64Min
                | Self::F64Max
                | Self::F64Copysign => FieldTy::F64,
                _ => panic!("operator cannot have an immediate `lhs` field"),
            },
        }
    }

    fn rhs_field(&self, input: OperandKind) -> FieldTy {
        match input {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => match self {
                | Self::Cmp(cmp) => cmp.input_field(input),
                | Self::I32Add
                | Self::I32Sub
                | Self::I32Mul
                | Self::I32BitAnd
                | Self::I32BitOr
                | Self::I32BitXor => FieldTy::I32,
                | Self::I32Shl | Self::S32Shr | Self::U32Shr | Self::I32Rotl | Self::I32Rotr => {
                    FieldTy::U8
                }
                | Self::S32Div | Self::U32Div | Self::S32Rem | Self::U32Rem => FieldTy::NonZeroU32,
                | Self::I64Add
                | Self::I64Sub
                | Self::I64Mul
                | Self::I64BitAnd
                | Self::I64BitOr
                | Self::I64BitXor => FieldTy::I64,
                | Self::I64Shl | Self::S64Shr | Self::U64Shr | Self::I64Rotl | Self::I64Rotr => {
                    FieldTy::U8
                }
                | Self::S64Div | Self::U64Div | Self::S64Rem | Self::U64Rem => FieldTy::NonZeroU64,
                | Self::F32Add
                | Self::F32Sub
                | Self::F32Mul
                | Self::F32Div
                | Self::F32Min
                | Self::F32Max => FieldTy::F32,
                | Self::F32Copysign => FieldTy::SignF32,
                | Self::F64Add
                | Self::F64Sub
                | Self::F64Mul
                | Self::F64Div
                | Self::F64Min
                | Self::F64Max => FieldTy::F64,
                | Self::F64Copysign => FieldTy::SignF64,
                | Self::I8x16Shl
                | Self::S8x16Shr
                | Self::U8x16Shr
                | Self::I16x8Shl
                | Self::S16x8Shr
                | Self::U16x8Shr
                | Self::I32x4Shl
                | Self::S32x4Shr
                | Self::U32x4Shr
                | Self::I64x2Shl
                | Self::S64x2Shr
                | Self::U64x2Shr => FieldTy::U8,
                _ => panic!("operator cannot have an immediate `rhs` field"),
            },
        }
    }

    pub fn commutativity(&self) -> Commutativity {
        match self {
            | Self::Cmp(cmp) => cmp.commutativity(),
            | Self::I32Add
            | Self::I32Mul
            | Self::I32BitAnd
            | Self::I32BitOr
            | Self::I32BitXor
            | Self::I64Add
            | Self::I64Mul
            | Self::I64BitAnd
            | Self::I64BitOr
            | Self::I64BitXor => Commutativity::Commutative,
            _ => Commutativity::NonCommutative,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Commutativity {
    Commutative,
    NonCommutative,
}

#[derive(Copy, Clone)]
pub struct CmpBranchOp {
    pub cmp: CmpOpKind,
    pub lhs: OperandKind,
    pub rhs: OperandKind,
}

impl CmpBranchOp {
    pub fn new(cmp: CmpOpKind, lhs: OperandKind, rhs: OperandKind) -> Self {
        Self { cmp, lhs, rhs }
    }

    pub fn lhs_field(&self) -> Field {
        Field::new(Ident::Lhs, self.cmp.input_field(self.lhs))
    }

    pub fn rhs_field(&self) -> Field {
        Field::new(Ident::Rhs, self.cmp.input_field(self.rhs))
    }

    pub fn offset_field(&self) -> Field {
        Field::new(Ident::Offset, FieldTy::BranchOffset)
    }

    pub fn fields(&self) -> [Field; 3] {
        [self.lhs_field(), self.rhs_field(), self.offset_field()]
    }
}

#[derive(Copy, Clone)]
pub struct CmpSelectOp {
    pub cmp: CmpOpKind,
    pub lhs: OperandKind,
    pub rhs: OperandKind,
}

impl CmpSelectOp {
    pub fn new(cmp: CmpOpKind, lhs: OperandKind, rhs: OperandKind) -> Self {
        Self { cmp, lhs, rhs }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn lhs_field(&self) -> Field {
        Field::new(Ident::Lhs, self.cmp.input_field(self.lhs))
    }

    pub fn rhs_field(&self) -> Field {
        Field::new(Ident::Rhs, self.cmp.input_field(self.rhs))
    }

    pub fn val_true_field(&self) -> Field {
        Field::new(Ident::ValTrue, FieldTy::Slot)
    }

    pub fn val_false_field(&self) -> Field {
        Field::new(Ident::ValFalse, FieldTy::Slot)
    }

    pub fn fields(&self) -> [Field; 5] {
        [
            self.result_field(),
            self.lhs_field(),
            self.rhs_field(),
            self.val_true_field(),
            self.val_false_field(),
        ]
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Ty {
    /// A general 32-bit integer type.
    I32,
    /// A general 64-bit integer type.
    I64,
    /// A signed 32-bit integer type.
    S32,
    /// A signed 64-bit integer type.
    S64,
    /// A unsigned 32-bit integer type.
    U32,
    /// A unsigned 64-bit integer type.
    U64,
    /// A generic 32-bits value.
    B32,
    /// A generic 64-bits value.
    B64,
    /// A 32-bit float type.
    F32,
    /// A 64-bit float type.
    F64,
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
    /// A `s8x16` vector type for `simd`.
    S8x16,
    /// A `s16x8` vector type for `simd`.
    S16x8,
    /// A `s32x4` vector type for `simd`.
    S32x4,
    /// A `s64x2` vector type for `simd`.
    S64x2,
    /// A `f32x4` vector type for `simd`.
    F32x4,
    /// A `f64x2` vector type for `simd`.
    F64x2,
}

impl Ty {
    pub fn to_field_ty(self) -> Option<FieldTy> {
        let ty = match self {
            | Ty::S32 | Ty::I32 => FieldTy::I32,
            | Ty::S64 | Ty::I64 => FieldTy::I64,
            | Ty::B32 | Ty::U32 => FieldTy::U32,
            | Ty::B64 | Ty::U64 => FieldTy::U64,
            | Ty::F32 => FieldTy::F32,
            | Ty::F64 => FieldTy::F64,
            _ => return None,
        };
        Some(ty)
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Ty::I32 => "i32",
            Ty::I64 => "i64",
            Ty::S32 => "i32",
            Ty::S64 => "i64",
            Ty::U32 => "u32",
            Ty::U64 => "u64",
            Ty::B32 => "32",
            Ty::B64 => "64",
            Ty::F32 => "f32",
            Ty::F64 => "f64",
            Ty::V128 => "v128",
            Ty::I8x16 => "i8x16",
            Ty::I16x8 => "i16x8",
            Ty::I32x4 => "i32x4",
            Ty::I64x2 => "i64x2",
            Ty::U8x16 => "u8x16",
            Ty::U16x8 => "u16x8",
            Ty::U32x4 => "u32x4",
            Ty::U64x2 => "u64x2",
            Ty::S8x16 => "s8x16",
            Ty::S16x8 => "s16x8",
            Ty::S32x4 => "s32x4",
            Ty::S64x2 => "s64x2",
            Ty::F32x4 => "f32x4",
            Ty::F64x2 => "f64x2",
        };
        write!(f, "{s}")
    }
}

impl Display for SnakeCase<Ty> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for CamelCase<Ty> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.0 {
            Ty::I32 => "I32",
            Ty::I64 => "I64",
            Ty::S32 => "I32",
            Ty::S64 => "I64",
            Ty::U32 => "U32",
            Ty::U64 => "U64",
            Ty::B32 => "32",
            Ty::B64 => "64",
            Ty::F32 => "F32",
            Ty::F64 => "F64",
            Ty::V128 => "V128",
            Ty::I8x16 => "I8x16",
            Ty::I16x8 => "I16x8",
            Ty::I32x4 => "I32x4",
            Ty::I64x2 => "I64x2",
            Ty::U8x16 => "U8x16",
            Ty::U16x8 => "U16x8",
            Ty::U32x4 => "U32x4",
            Ty::U64x2 => "U64x2",
            Ty::S8x16 => "S8x16",
            Ty::S16x8 => "S16x8",
            Ty::S32x4 => "S32x4",
            Ty::S64x2 => "S64x2",
            Ty::F32x4 => "F32x4",
            Ty::F64x2 => "F64x2",
        };
        write!(f, "{s}")
    }
}

#[derive(Copy, Clone)]
pub enum FieldTy {
    Slot,
    SlotSpan,
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

impl Display for FieldTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Slot => "Slot",
            Self::SlotSpan => "SlotSpan",
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
        write!(f, "{s}")
    }
}

#[derive(Copy, Clone)]
pub enum CmpOpKind {
    I32Eq,
    I32NotEq,
    I32And,
    I32NotAnd,
    I32Or,
    I32NotOr,
    S32Lt,
    U32Lt,
    S32Le,
    U32Le,

    I64Eq,
    I64NotEq,
    I64And,
    I64NotAnd,
    I64Or,
    I64NotOr,
    S64Lt,
    U64Lt,
    S64Le,
    U64Le,

    F32Eq,
    F32NotEq,
    F32Lt,
    F32NotLt,
    F32Le,
    F32NotLe,

    F64Eq,
    F64NotEq,
    F64Lt,
    F64NotLt,
    F64Le,
    F64NotLe,
}

impl CmpOpKind {
    pub fn commutativity(&self) -> Commutativity {
        match self {
            | Self::I32Eq
            | Self::I32NotEq
            | Self::I32And
            | Self::I32NotAnd
            | Self::I32Or
            | Self::I32NotOr
            | Self::I64Eq
            | Self::I64NotEq
            | Self::I64And
            | Self::I64NotAnd
            | Self::I64Or
            | Self::I64NotOr
            | Self::F32Eq
            | Self::F32NotEq
            | Self::F64Eq
            | Self::F64NotEq => Commutativity::Commutative,
            _ => Commutativity::NonCommutative,
        }
    }

    fn input_field(&self, input: OperandKind) -> FieldTy {
        match input {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => match self {
                | Self::I32Eq
                | Self::I32NotEq
                | Self::I32And
                | Self::I32NotAnd
                | Self::I32Or
                | Self::I32NotOr
                | Self::S32Lt
                | Self::S32Le => FieldTy::I32,
                | Self::U32Lt | Self::U32Le => FieldTy::U32,
                | Self::I64Eq
                | Self::I64NotEq
                | Self::I64And
                | Self::I64NotAnd
                | Self::I64Or
                | Self::I64NotOr
                | Self::S64Lt
                | Self::S64Le => FieldTy::I64,
                | Self::U64Lt | Self::U64Le => FieldTy::U64,
                | Self::F32Eq
                | Self::F32NotEq
                | Self::F32Lt
                | Self::F32NotLt
                | Self::F32Le
                | Self::F32NotLe => FieldTy::F32,
                | Self::F64Eq
                | Self::F64NotEq
                | Self::F64Lt
                | Self::F64NotLt
                | Self::F64Le
                | Self::F64NotLe => FieldTy::F64,
            },
        }
    }

    pub fn ident_prefix(&self) -> Ty {
        match self {
            | Self::I32Eq
            | Self::I32NotEq
            | Self::I32And
            | Self::I32NotAnd
            | Self::I32Or
            | Self::I32NotOr => Ty::I32,
            | Self::S32Lt | Self::S32Le => Ty::S32,
            | Self::U32Lt | Self::U32Le => Ty::U32,
            | Self::I64Eq
            | Self::I64NotEq
            | Self::I64And
            | Self::I64NotAnd
            | Self::I64Or
            | Self::I64NotOr => Ty::I64,
            | Self::S64Lt | Self::S64Le => Ty::S64,
            | Self::U64Lt | Self::U64Le => Ty::U64,
            | Self::F32Eq
            | Self::F32NotEq
            | Self::F32Lt
            | Self::F32NotLt
            | Self::F32Le
            | Self::F32NotLe => Ty::F32,
            | Self::F64Eq
            | Self::F64NotEq
            | Self::F64Lt
            | Self::F64NotLt
            | Self::F64Le
            | Self::F64NotLe => Ty::F64,
        }
    }

    pub fn ident(&self) -> Ident {
        match self {
            Self::I32Eq => Ident::Eq,
            Self::I32NotEq => Ident::NotEq,
            Self::I32And => Ident::And,
            Self::I32NotAnd => Ident::NotAnd,
            Self::I32Or => Ident::Or,
            Self::I32NotOr => Ident::NotOr,
            Self::S32Lt => Ident::Lt,
            Self::U32Lt => Ident::Lt,
            Self::S32Le => Ident::Le,
            Self::U32Le => Ident::Le,
            Self::I64Eq => Ident::Eq,
            Self::I64NotEq => Ident::NotEq,
            Self::I64And => Ident::And,
            Self::I64NotAnd => Ident::NotAnd,
            Self::I64Or => Ident::Or,
            Self::I64NotOr => Ident::NotOr,
            Self::S64Lt => Ident::Lt,
            Self::U64Lt => Ident::Lt,
            Self::S64Le => Ident::Le,
            Self::U64Le => Ident::Le,
            Self::F32Eq => Ident::Eq,
            Self::F32NotEq => Ident::NotEq,
            Self::F32Lt => Ident::Lt,
            Self::F32NotLt => Ident::NotLt,
            Self::F32Le => Ident::Le,
            Self::F32NotLe => Ident::NotLe,
            Self::F64Eq => Ident::Eq,
            Self::F64NotEq => Ident::NotEq,
            Self::F64Lt => Ident::Lt,
            Self::F64NotLt => Ident::NotLt,
            Self::F64Le => Ident::Le,
            Self::F64NotLe => Ident::NotLe,
        }
    }
}

#[derive(Copy, Clone)]
pub struct LoadOp {
    /// The kind of the load operator.
    pub kind: LoadOpKind,
    /// The `ptr` field type.
    pub ptr: OperandKind,
    /// True, if the operator is always operating on (`memory 0`).
    pub mem0: bool,
    /// True, if the operator uses a 16-bit offset field.
    pub offset16: bool,
}

impl LoadOp {
    pub fn new(kind: LoadOpKind, ptr: OperandKind, mem0: bool, offset16: bool) -> Self {
        Self {
            kind,
            ptr,
            mem0,
            offset16,
        }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn ptr_field(&self) -> Field {
        let ptr_ty = match self.ptr {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => FieldTy::Address,
        };
        Field::new(Ident::Ptr, ptr_ty)
    }

    pub fn offset_field(&self) -> Option<Field> {
        let offset_ty = match self.ptr {
            OperandKind::Slot => match self.offset16 {
                true => FieldTy::Offset16,
                false => FieldTy::U64,
            },
            OperandKind::Immediate => return None,
        };
        Some(Field::new(Ident::Offset, offset_ty))
    }

    pub fn memory_field(&self) -> Option<Field> {
        if self.mem0 {
            return None;
        }
        Some(Field::new(Ident::Memory, FieldTy::Memory))
    }

    pub fn fields(&self) -> [Option<Field>; 4] {
        [
            Some(self.result_field()),
            Some(self.ptr_field()),
            self.offset_field(),
            self.memory_field(),
        ]
    }
}

#[derive(Copy, Clone)]
pub enum LoadOpKind {
    // Scalar
    Load32,
    Load64,
    S32Load8,
    U32Load8,
    S32Load16,
    U32Load16,
    S64Load8,
    U64Load8,
    S64Load16,
    U64Load16,
    S64Load32,
    U64Load32,
    // Simd
    V128Load,
    S16x8Load8x8,
    U16x8Load8x8,
    S32x4Load16x4,
    U32x4Load16x4,
    S64x2Load32x2,
    U64x2Load32x2,
    V128Load8Splat,
    V128Load16Splat,
    V128Load32Splat,
    V128Load64Splat,
    V128Load32Zero,
    V128Load64Zero,
}

impl LoadOpKind {
    pub fn ident(&self) -> Ident {
        match self {
            Self::Load32 => Ident::Load32,
            Self::Load64 => Ident::Load64,
            Self::S32Load8 => Ident::Load8,
            Self::U32Load8 => Ident::Load8,
            Self::S32Load16 => Ident::Load16,
            Self::U32Load16 => Ident::Load16,
            Self::S64Load8 => Ident::Load8,
            Self::U64Load8 => Ident::Load8,
            Self::S64Load16 => Ident::Load16,
            Self::U64Load16 => Ident::Load16,
            Self::S64Load32 => Ident::Load32,
            Self::U64Load32 => Ident::Load32,
            Self::V128Load => Ident::Load,
            Self::S16x8Load8x8 => Ident::Load8x8,
            Self::U16x8Load8x8 => Ident::Load8x8,
            Self::S32x4Load16x4 => Ident::Load16x4,
            Self::U32x4Load16x4 => Ident::Load16x4,
            Self::S64x2Load32x2 => Ident::Load32x2,
            Self::U64x2Load32x2 => Ident::Load32x2,
            Self::V128Load8Splat => Ident::Load8Splat,
            Self::V128Load16Splat => Ident::Load16Splat,
            Self::V128Load32Splat => Ident::Load32Splat,
            Self::V128Load64Splat => Ident::Load64Splat,
            Self::V128Load32Zero => Ident::Load32Zero,
            Self::V128Load64Zero => Ident::Load64Zero,
        }
    }

    pub fn ident_prefix(&self) -> Option<Ty> {
        let prefix = match self {
            | Self::Load32 | Self::Load64 => return None,
            | Self::S32Load8 => Ty::S32,
            | Self::U32Load8 => Ty::U32,
            | Self::S32Load16 => Ty::S32,
            | Self::U32Load16 => Ty::U32,
            | Self::S64Load8 => Ty::S64,
            | Self::U64Load8 => Ty::U64,
            | Self::S64Load16 => Ty::S64,
            | Self::U64Load16 => Ty::U64,
            | Self::S64Load32 => Ty::S64,
            | Self::U64Load32 => Ty::U64,
            | Self::V128Load => Ty::V128,
            | Self::S16x8Load8x8 => Ty::S16x8,
            | Self::U16x8Load8x8 => Ty::U16x8,
            | Self::S32x4Load16x4 => Ty::S32x4,
            | Self::U32x4Load16x4 => Ty::U32x4,
            | Self::S64x2Load32x2 => Ty::S64x2,
            | Self::U64x2Load32x2 => Ty::U64x2,
            | Self::V128Load8Splat => Ty::V128,
            | Self::V128Load16Splat => Ty::V128,
            | Self::V128Load32Splat => Ty::V128,
            | Self::V128Load64Splat => Ty::V128,
            | Self::V128Load32Zero => Ty::V128,
            | Self::V128Load64Zero => Ty::V128,
        };
        Some(prefix)
    }
}

#[derive(Copy, Clone)]
pub struct StoreOp {
    /// The kind of the load operator.
    pub kind: StoreOpKind,
    /// The `ptr` input type.
    pub ptr: OperandKind,
    /// The `value` input type.
    pub value: OperandKind,
    /// True, if the operator is always operating on (`memory 0`).
    pub mem0: bool,
    /// True, if the operator uses a 16-bit offset field.
    pub offset16: bool,
}

impl StoreOp {
    pub fn new(
        kind: StoreOpKind,
        ptr: OperandKind,
        value: OperandKind,
        mem0: bool,
        offset16: bool,
    ) -> Self {
        Self {
            kind,
            ptr,
            value,
            mem0,
            offset16,
        }
    }

    pub fn ptr_field(&self) -> Field {
        let ptr_ty = match self.ptr {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => FieldTy::Address,
        };
        Field::new(Ident::Ptr, ptr_ty)
    }

    pub fn offset_field(&self) -> Option<Field> {
        let offset_ty = match self.ptr {
            OperandKind::Slot => match self.offset16 {
                true => FieldTy::Offset16,
                false => FieldTy::U64,
            },
            OperandKind::Immediate => return None,
        };
        Some(Field::new(Ident::Offset, offset_ty))
    }

    pub fn value_field(&self) -> Field {
        let value_ty = self.kind.value_ty(self.value);
        Field::new(Ident::Value, value_ty)
    }

    pub fn memory_field(&self) -> Option<Field> {
        if self.mem0 {
            return None;
        }
        Some(Field::new(Ident::Memory, FieldTy::Memory))
    }

    pub fn laneidx_field(&self) -> Option<Field> {
        let ty = self.kind.laneidx_ty()?;
        Some(Field::new(Ident::Lane, ty))
    }

    pub fn fields(&self) -> [Option<Field>; 5] {
        [
            Some(self.ptr_field()),
            self.offset_field(),
            Some(self.value_field()),
            self.memory_field(),
            self.laneidx_field(),
        ]
    }
}

#[derive(Copy, Clone)]
pub enum StoreOpKind {
    // Generic
    Store32,
    Store64,
    // i32
    I32Store8,
    I32Store16,
    // i64
    I64Store8,
    I64Store16,
    I64Store32,
    // v128
    Store128,
    V128Store8Lane,
    V128Store16Lane,
    V128Store32Lane,
    V128Store64Lane,
}

impl StoreOpKind {
    pub fn ident(&self) -> Ident {
        match self {
            Self::Store32 => Ident::Store32,
            Self::Store64 => Ident::Store64,
            Self::I32Store8 => Ident::Store8,
            Self::I32Store16 => Ident::Store16,
            Self::I64Store8 => Ident::Store8,
            Self::I64Store16 => Ident::Store16,
            Self::I64Store32 => Ident::Store32,
            Self::Store128 => Ident::Store128,
            Self::V128Store8Lane => Ident::Store8Lane,
            Self::V128Store16Lane => Ident::Store16Lane,
            Self::V128Store32Lane => Ident::Store32Lane,
            Self::V128Store64Lane => Ident::Store64Lane,
        }
    }

    pub fn ident_prefix(&self) -> Option<Ident> {
        match self {
            Self::Store32 => None,
            Self::Store64 => None,
            Self::I32Store8 => Some(Ident::I32),
            Self::I32Store16 => Some(Ident::I32),
            Self::I64Store8 => Some(Ident::I64),
            Self::I64Store16 => Some(Ident::I64),
            Self::I64Store32 => Some(Ident::I64),
            Self::Store128 => None,
            Self::V128Store8Lane => Some(Ident::V128),
            Self::V128Store16Lane => Some(Ident::V128),
            Self::V128Store32Lane => Some(Ident::V128),
            Self::V128Store64Lane => Some(Ident::V128),
        }
    }

    fn value_ty(&self, input: OperandKind) -> FieldTy {
        match input {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => match self {
                Self::Store32 => FieldTy::U32,
                Self::Store64 => FieldTy::U64,
                Self::I32Store8 => FieldTy::I8,
                Self::I32Store16 => FieldTy::I16,
                Self::I64Store8 => FieldTy::I8,
                Self::I64Store16 => FieldTy::I16,
                Self::I64Store32 => FieldTy::I32,
                Self::Store128 => FieldTy::Bytes16,
                Self::V128Store8Lane => FieldTy::V128,
                Self::V128Store16Lane => FieldTy::V128,
                Self::V128Store32Lane => FieldTy::V128,
                Self::V128Store64Lane => FieldTy::V128,
            },
        }
    }

    fn laneidx_ty(&self) -> Option<FieldTy> {
        let ty = match self {
            Self::V128Store8Lane => FieldTy::ImmLaneIdx16,
            Self::V128Store16Lane => FieldTy::ImmLaneIdx8,
            Self::V128Store32Lane => FieldTy::ImmLaneIdx4,
            Self::V128Store64Lane => FieldTy::ImmLaneIdx2,
            _ => return None,
        };
        Some(ty)
    }
}

#[derive(Copy, Clone)]
pub struct TableGetOp {
    /// The `index` type.
    pub index: OperandKind,
}

impl TableGetOp {
    pub fn new(index: OperandKind) -> Self {
        Self { index }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn index_field(&self) -> Field {
        let index_ty = match self.index {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => FieldTy::U32,
        };
        Field::new(Ident::Index, index_ty)
    }

    pub fn table_field(&self) -> Field {
        Field::new(Ident::Table, FieldTy::Table)
    }

    pub fn fields(&self) -> [Field; 3] {
        [self.result_field(), self.index_field(), self.table_field()]
    }
}

#[derive(Copy, Clone)]
pub struct TableSetOp {
    /// The `index` input.
    pub index: OperandKind,
    /// The `value` input.
    pub value: OperandKind,
}

impl TableSetOp {
    pub fn new(index: OperandKind, value: OperandKind) -> Self {
        Self { index, value }
    }

    pub fn index_field(&self) -> Field {
        let index_ty = match self.index {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => FieldTy::U32,
        };
        Field::new(Ident::Index, index_ty)
    }

    pub fn value_field(&self) -> Field {
        let value_ty = match self.value {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => FieldTy::U64,
        };
        Field::new(Ident::Value, value_ty)
    }

    pub fn table_field(&self) -> Field {
        Field::new(Ident::Table, FieldTy::Table)
    }

    pub fn fields(&self) -> [Field; 3] {
        [self.index_field(), self.value_field(), self.table_field()]
    }
}

#[derive(Copy, Clone)]
pub enum LaneWidth {
    W8,
    W16,
    W32,
    W64,
}

impl Display for LaneWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = u8::from(*self);
        let len_lanes = self.len_lanes();
        write!(f, "{width}x{len_lanes}")
    }
}

impl From<LaneWidth> for u8 {
    fn from(width: LaneWidth) -> Self {
        match width {
            LaneWidth::W8 => 8,
            LaneWidth::W16 => 16,
            LaneWidth::W32 => 32,
            LaneWidth::W64 => 64,
        }
    }
}

impl LaneWidth {
    pub fn len_lanes(self) -> u8 {
        match self {
            Self::W8 => 16,
            Self::W16 => 8,
            Self::W32 => 4,
            Self::W64 => 2,
        }
    }

    pub fn to_laneidx(self) -> FieldTy {
        match self {
            Self::W8 => FieldTy::ImmLaneIdx16,
            Self::W16 => FieldTy::ImmLaneIdx8,
            Self::W32 => FieldTy::ImmLaneIdx4,
            Self::W64 => FieldTy::ImmLaneIdx2,
        }
    }
}

#[derive(Copy, Clone)]
pub struct V128ReplaceLaneOp {
    /// The type of the value to be splatted.
    pub width: LaneWidth,
    /// The `value` used for replacing.
    pub value: OperandKind,
}

impl V128ReplaceLaneOp {
    pub fn new(width: LaneWidth, value: OperandKind) -> Self {
        Self { width, value }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn v128_field(&self) -> Field {
        Field::new(Ident::V128, FieldTy::Slot)
    }

    pub fn value_field(&self) -> Field {
        let value_ty = match self.value {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => match self.width {
                LaneWidth::W8 => FieldTy::U8,
                LaneWidth::W16 => FieldTy::U16,
                LaneWidth::W32 => FieldTy::U32,
                LaneWidth::W64 => FieldTy::U64,
            },
        };
        Field::new(Ident::Value, value_ty)
    }

    pub fn lane_field(&self) -> Field {
        let lane_ty = match self.width {
            LaneWidth::W8 => FieldTy::ImmLaneIdx16,
            LaneWidth::W16 => FieldTy::ImmLaneIdx8,
            LaneWidth::W32 => FieldTy::ImmLaneIdx4,
            LaneWidth::W64 => FieldTy::ImmLaneIdx2,
        };
        Field::new(Ident::Lane, lane_ty)
    }

    pub fn fields(&self) -> [Field; 4] {
        [
            self.result_field(),
            self.v128_field(),
            self.value_field(),
            self.lane_field(),
        ]
    }
}

#[derive(Copy, Clone)]
pub struct V128LoadLaneOp {
    /// The type of the value to be splatted.
    pub width: LaneWidth,
    /// The `value` used for replacing.
    pub ptr: OperandKind,
    /// True, if the operator is always operating on (`memory 0`).
    pub mem0: bool,
    /// True, if the operator uses a 16-bit offset field.
    pub offset16: bool,
}

impl V128LoadLaneOp {
    pub fn new(width: LaneWidth, ptr: OperandKind, mem0: bool, offset16: bool) -> Self {
        Self {
            width,
            ptr,
            mem0,
            offset16,
        }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn ptr_field(&self) -> Field {
        let ptr_ty = match self.ptr {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => FieldTy::Address,
        };
        Field::new(Ident::Ptr, ptr_ty)
    }

    pub fn offset_field(&self) -> Option<Field> {
        let offset_ty = match self.ptr {
            OperandKind::Slot => match self.offset16 {
                true => FieldTy::Offset16,
                false => FieldTy::U64,
            },
            OperandKind::Immediate => return None,
        };
        Some(Field::new(Ident::Offset, offset_ty))
    }

    pub fn v128_field(&self) -> Field {
        Field::new(Ident::V128, FieldTy::Slot)
    }

    pub fn memory_field(&self) -> Option<Field> {
        if self.mem0 {
            return None;
        }
        Some(Field::new(Ident::Memory, FieldTy::Memory))
    }

    pub fn laneidx_field(&self) -> Field {
        let ty = match self.width {
            LaneWidth::W8 => FieldTy::ImmLaneIdx16,
            LaneWidth::W16 => FieldTy::ImmLaneIdx8,
            LaneWidth::W32 => FieldTy::ImmLaneIdx4,
            LaneWidth::W64 => FieldTy::ImmLaneIdx2,
        };
        Field::new(Ident::Lane, ty)
    }

    pub fn fields(&self) -> [Option<Field>; 6] {
        [
            Some(self.result_field()),
            Some(self.ptr_field()),
            self.offset_field(),
            self.memory_field(),
            Some(self.v128_field()),
            Some(self.laneidx_field()),
        ]
    }
}
