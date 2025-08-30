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
            V128Splat(V128SplatOp),
            V128ReplaceLane(V128ReplaceLaneOp),
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
    /// The operand is a [`Stack`] index.
    Stack,
    /// The operand is an immediate value.
    Immediate,
}

impl Display for CamelCase<OperandKind> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0 {
            OperandKind::Stack => "S",
            OperandKind::Immediate => "I",
        };
        write!(f, "{s}")
    }
}

impl Display for SnakeCase<OperandKind> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0 {
            OperandKind::Stack => "s",
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
}

impl UnaryOp {
    pub fn new(kind: UnaryOpKind) -> Self {
        Self { kind }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Stack)
    }

    pub fn value_field(&self) -> Field {
        Field::new(Ident::Value, FieldTy::Stack)
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
}

impl UnaryOpKind {
    pub fn is_conversion(&self) -> bool {
        self.input_ty() != self.result_ty()
    }

    pub fn input_ty(&self) -> Ty {
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
        Field::new(Ident::Result, FieldTy::Stack)
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
        }
    }

    pub fn ident_prefix(&self) -> Ident {
        let ty = match self {
            BinaryOpKind::Cmp(op) => op.input_ty(),
            _ => self.result_ty(),
        };
        Ident::from(ty)
    }

    fn lhs_field(&self, input: OperandKind) -> FieldTy {
        match input {
            OperandKind::Stack => FieldTy::Stack,
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
            OperandKind::Stack => FieldTy::Stack,
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
                _ => panic!("operator cannot have an immediate `rhs` field"),
            },
        }
    }

    pub fn result_ty(&self) -> Ty {
        match self {
            | Self::Cmp(_) => Ty::I32,
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
        Field::new(Ident::Result, FieldTy::Stack)
    }

    pub fn lhs_field(&self) -> Field {
        Field::new(Ident::Lhs, self.cmp.input_field(self.lhs))
    }

    pub fn rhs_field(&self) -> Field {
        Field::new(Ident::Rhs, self.cmp.input_field(self.rhs))
    }

    pub fn val_true_field(&self) -> Field {
        Field::new(Ident::ValTrue, FieldTy::Stack)
    }

    pub fn val_false_field(&self) -> Field {
        Field::new(Ident::ValFalse, FieldTy::Stack)
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

impl Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Ty::I32 => "i32",
            Ty::I64 => "i64",
            Ty::S32 => "i32",
            Ty::S64 => "i64",
            Ty::U32 => "u32",
            Ty::U64 => "u64",
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

impl From<Ty> for Ident {
    fn from(ty: Ty) -> Self {
        match ty {
            Ty::I32 => Self::I32,
            Ty::I64 => Self::I64,
            Ty::S32 => Self::S32,
            Ty::S64 => Self::S64,
            Ty::U32 => Self::U32,
            Ty::U64 => Self::U64,
            Ty::F32 => Self::F32,
            Ty::F64 => Self::F64,
            Ty::V128 => Self::V128,
            Ty::I8x16 => Self::I8x16,
            Ty::I16x8 => Self::I16x8,
            Ty::I32x4 => Self::I32x4,
            Ty::I64x2 => Self::I64x2,
            Ty::U8x16 => Self::U8x16,
            Ty::U16x8 => Self::U16x8,
            Ty::U32x4 => Self::U32x4,
            Ty::U64x2 => Self::U64x2,
            Ty::S8x16 => Self::S8x16,
            Ty::S16x8 => Self::S16x8,
            Ty::S32x4 => Self::S32x4,
            Ty::S64x2 => Self::S64x2,
            Ty::F32x4 => Self::F32x4,
            Ty::F64x2 => Self::F64x2,
        }
    }
}

#[derive(Copy, Clone)]
pub enum FieldTy {
    Stack,
    StackSpan,
    FixedStackSpan2,
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
}

impl Display for FieldTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Stack => "Stack",
            Self::StackSpan => "StackSpan",
            Self::FixedStackSpan2 => "FixedStackSpan<2>",
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
            OperandKind::Stack => FieldTy::Stack,
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

    pub fn input_ty(&self) -> Ty {
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
        Field::new(Ident::Result, FieldTy::Stack)
    }

    pub fn ptr_field(&self) -> Field {
        let ptr_ty = match self.ptr {
            OperandKind::Stack => FieldTy::Stack,
            OperandKind::Immediate => FieldTy::Address,
        };
        Field::new(Ident::Ptr, ptr_ty)
    }

    pub fn offset_field(&self) -> Option<Field> {
        let offset_ty = match self.ptr {
            OperandKind::Stack => match self.offset16 {
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
        }
    }

    pub fn ident_prefix(&self) -> Option<Ident> {
        match self {
            Self::Load32 => None,
            Self::Load64 => None,
            Self::S32Load8 => Some(Ident::S32),
            Self::U32Load8 => Some(Ident::U32),
            Self::S32Load16 => Some(Ident::S32),
            Self::U32Load16 => Some(Ident::U32),
            Self::S64Load8 => Some(Ident::S64),
            Self::U64Load8 => Some(Ident::U64),
            Self::S64Load16 => Some(Ident::S64),
            Self::U64Load16 => Some(Ident::U64),
            Self::S64Load32 => Some(Ident::S64),
            Self::U64Load32 => Some(Ident::U64),
        }
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
            OperandKind::Stack => FieldTy::Stack,
            OperandKind::Immediate => FieldTy::Address,
        };
        Field::new(Ident::Ptr, ptr_ty)
    }

    pub fn offset_field(&self) -> Option<Field> {
        let offset_ty = match self.ptr {
            OperandKind::Stack => match self.offset16 {
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

    pub fn fields(&self) -> [Option<Field>; 4] {
        [
            Some(self.ptr_field()),
            self.offset_field(),
            Some(self.value_field()),
            self.memory_field(),
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
        }
    }

    fn value_ty(&self, input: OperandKind) -> FieldTy {
        match input {
            OperandKind::Stack => FieldTy::Stack,
            OperandKind::Immediate => match self {
                Self::Store32 => FieldTy::U32,
                Self::Store64 => FieldTy::U64,
                Self::I32Store8 => FieldTy::I8,
                Self::I32Store16 => FieldTy::I16,
                Self::I64Store8 => FieldTy::I8,
                Self::I64Store16 => FieldTy::I16,
                Self::I64Store32 => FieldTy::I32,
            },
        }
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
        Field::new(Ident::Result, FieldTy::Stack)
    }

    pub fn index_field(&self) -> Field {
        let index_ty = match self.index {
            OperandKind::Stack => FieldTy::Stack,
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
            OperandKind::Stack => FieldTy::Stack,
            OperandKind::Immediate => FieldTy::U32,
        };
        Field::new(Ident::Index, index_ty)
    }

    pub fn value_field(&self) -> Field {
        let value_ty = match self.value {
            OperandKind::Stack => FieldTy::Stack,
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
pub struct V128SplatOp {
    /// The type of the value to be splatted.
    pub ty: SplatType,
    /// The `value` to be splatted.
    pub value: OperandKind,
}

#[derive(Copy, Clone)]
pub enum SplatType {
    U32,
    U64,
}

impl V128SplatOp {
    pub fn new(ty: SplatType, value: OperandKind) -> Self {
        Self { ty, value }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Stack)
    }

    pub fn value_field(&self) -> Field {
        let value_ty = match self.value {
            OperandKind::Stack => FieldTy::Stack,
            OperandKind::Immediate => match self.ty {
                SplatType::U32 => FieldTy::U32,
                SplatType::U64 => FieldTy::U64,
            },
        };
        Field::new(Ident::Value, value_ty)
    }

    pub fn fields(&self) -> [Field; 2] {
        [self.result_field(), self.value_field()]
    }
}

#[derive(Copy, Clone)]
pub struct V128ReplaceLaneOp {
    /// The type of the value to be splatted.
    pub width: ReplaceLaneWidth,
    /// The `value` used for replacing.
    pub value: OperandKind,
}

#[derive(Copy, Clone)]
pub enum ReplaceLaneWidth {
    W8,
    W16,
    W32,
    W64,
}

impl V128ReplaceLaneOp {
    pub fn new(width: ReplaceLaneWidth, value: OperandKind) -> Self {
        Self { width, value }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Stack)
    }

    pub fn v128_field(&self) -> Field {
        Field::new(Ident::V128, FieldTy::Stack)
    }

    pub fn value_field(&self) -> Field {
        let value_ty = match self.value {
            OperandKind::Stack => FieldTy::Stack,
            OperandKind::Immediate => match self.width {
                ReplaceLaneWidth::W8 => FieldTy::U8,
                ReplaceLaneWidth::W16 => FieldTy::U16,
                ReplaceLaneWidth::W32 => FieldTy::U32,
                ReplaceLaneWidth::W64 => FieldTy::U64,
            },
        };
        Field::new(Ident::Value, value_ty)
    }

    pub fn lane_field(&self) -> Field {
        let lane_ty = match self.width {
            ReplaceLaneWidth::W8 => FieldTy::ImmLaneIdx16,
            ReplaceLaneWidth::W16 => FieldTy::ImmLaneIdx8,
            ReplaceLaneWidth::W32 => FieldTy::ImmLaneIdx4,
            ReplaceLaneWidth::W64 => FieldTy::ImmLaneIdx2,
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
