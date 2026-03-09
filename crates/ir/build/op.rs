use crate::build::{FieldTy, Ident, SnakeCase, Ty};
use core::{
    fmt::{self, Display},
    ops::{BitAnd, BitOr},
};

macro_rules! apply_macro_for_ops {
    ($mac:ident $(, $param:ident)* $(,)?) => {
        $mac! {
            $($param,)*
            Unary(UnaryOp),
            Binary(BinaryOp),
            Ternary(TernaryOp),
            CmpBranch(CmpBranchOp),
            Select(SelectOp),
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
            V128ExtractLane(V128ExtractLaneOp),
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

impl OperandKind {
    pub fn field_ty(self, hint: Ty) -> FieldTy {
        match self {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => FieldTy::from(hint),
        }
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
    pub ident: Ident,
    pub result_ty: Ty,
    pub value_ty: Ty,
    pub value: OperandKind,
}

impl UnaryOp {
    pub fn new(ident: Ident, result_ty: Ty, value_ty: Ty, value: OperandKind) -> Self {
        Self {
            ident,
            result_ty,
            value_ty,
            value,
        }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn value_field(&self) -> Field {
        Field::new(Ident::Value, self.value.field_ty(self.value_ty))
    }

    pub fn fields(&self) -> [Field; 2] {
        [self.result_field(), self.value_field()]
    }
}

#[derive(Copy, Clone)]
pub struct BinaryOp {
    pub ident: Ident,
    pub result_ty: Ty,
    pub lhs_ty: Ty,
    pub rhs_ty: Ty,
    pub lhs: OperandKind,
    pub rhs: OperandKind,
    pub caps: BinaryOpCaps,
}

impl BinaryOp {
    pub fn new(
        ident: Ident,
        result_ty: Ty,
        lhs_ty: Ty,
        rhs_ty: Ty,
        lhs: OperandKind,
        rhs: OperandKind,
        caps: BinaryOpCaps,
    ) -> Self {
        Self {
            ident,
            result_ty,
            lhs_ty,
            rhs_ty,
            lhs,
            rhs,
            caps,
        }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn lhs_field(&self) -> Field {
        Field::new(Ident::Lhs, self.lhs.field_ty(self.lhs_ty))
    }

    pub fn rhs_field(&self) -> Field {
        Field::new(Ident::Rhs, self.rhs.field_ty(self.rhs_ty))
    }

    pub fn fields(&self) -> [Field; 3] {
        [self.result_field(), self.lhs_field(), self.rhs_field()]
    }
}

#[derive(Copy, Clone)]
pub struct BinaryOpCaps(u8);

impl BinaryOpCaps {
    pub const NONE: Self = Self(0b0000);
    pub const COMMUTATIVE: Self = Self(0b0001);
    pub const CMP: Self = Self(0b0010);

    pub fn is_cmp(self) -> bool {
        (self & Self::CMP).0 != 0
    }

    pub fn is_commutative(self) -> bool {
        (self & Self::COMMUTATIVE).0 != 0
    }
}

impl BitAnd for BinaryOpCaps {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for BinaryOpCaps {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Copy, Clone)]
pub struct TernaryOp {
    pub kind: TernaryOpKind,
}

impl TernaryOp {
    pub fn new(kind: TernaryOpKind) -> Self {
        Self { kind }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn a_field(&self) -> Field {
        Field::new(Ident::A, FieldTy::Slot)
    }

    pub fn b_field(&self) -> Field {
        Field::new(Ident::B, FieldTy::Slot)
    }

    pub fn c_field(&self) -> Field {
        Field::new(Ident::C, FieldTy::Slot)
    }

    pub fn fields(&self) -> [Field; 4] {
        [
            self.result_field(),
            self.a_field(),
            self.b_field(),
            self.c_field(),
        ]
    }
}

#[derive(Copy, Clone)]
pub enum TernaryOpKind {
    V128Bitselect,
    F32x4RelaxedMadd,
    F32x4RelaxedNmadd,
    F64x2RelaxedMadd,
    F64x2RelaxedNmadd,
    I32x4RelaxedDotI8x16I7x16Add,
}

impl TernaryOpKind {
    pub fn ident(&self) -> Ident {
        match self {
            TernaryOpKind::V128Bitselect => Ident::Bitselect,
            TernaryOpKind::F32x4RelaxedMadd => Ident::RelaxedMadd,
            TernaryOpKind::F32x4RelaxedNmadd => Ident::RelaxedNmadd,
            TernaryOpKind::F64x2RelaxedMadd => Ident::RelaxedMadd,
            TernaryOpKind::F64x2RelaxedNmadd => Ident::RelaxedNmadd,
            TernaryOpKind::I32x4RelaxedDotI8x16I7x16Add => Ident::RelaxedDotI8x16I7x16Add,
        }
    }

    pub fn ident_prefix(&self) -> Ty {
        match self {
            TernaryOpKind::V128Bitselect => Ty::V128,
            TernaryOpKind::F32x4RelaxedMadd => Ty::F32x4,
            TernaryOpKind::F32x4RelaxedNmadd => Ty::F32x4,
            TernaryOpKind::F64x2RelaxedMadd => Ty::F64x2,
            TernaryOpKind::F64x2RelaxedNmadd => Ty::F64x2,
            TernaryOpKind::I32x4RelaxedDotI8x16I7x16Add => Ty::I32x4,
        }
    }
}

#[derive(Copy, Clone)]
pub struct CmpBranchOp {
    pub ident: Ident,
    pub input_ty: Ty,
    pub lhs: OperandKind,
    pub rhs: OperandKind,
}

impl CmpBranchOp {
    pub fn new(ident: Ident, input_ty: Ty, lhs: OperandKind, rhs: OperandKind) -> Self {
        Self {
            ident,
            input_ty,
            lhs,
            rhs,
        }
    }

    pub fn lhs_field(&self) -> Field {
        Field::new(Ident::Lhs, self.lhs.field_ty(self.input_ty))
    }

    pub fn rhs_field(&self) -> Field {
        Field::new(Ident::Rhs, self.rhs.field_ty(self.input_ty))
    }

    pub fn offset_field(&self) -> Field {
        Field::new(Ident::Offset, FieldTy::BranchOffset)
    }

    pub fn fields(&self) -> [Field; 3] {
        [self.offset_field(), self.lhs_field(), self.rhs_field()]
    }
}

#[derive(Copy, Clone)]
pub enum SelectWidth {
    None,
    Bits32,
    Bits64,
}

impl SelectWidth {
    fn field_ty(&self, kind: OperandKind) -> FieldTy {
        match kind {
            OperandKind::Slot => FieldTy::Slot,
            OperandKind::Immediate => match self {
                Self::Bits32 => FieldTy::U32,
                Self::Bits64 => FieldTy::U64,
                Self::None => panic!("must not have immediate operands"),
            },
        }
    }
}

impl Display for SelectWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SelectWidth::None => "",
            SelectWidth::Bits32 => "32",
            SelectWidth::Bits64 => "64",
        };
        f.write_str(s)
    }
}

#[derive(Copy, Clone)]
pub struct SelectOp {
    pub width: SelectWidth,
    pub true_val: OperandKind,
    pub false_val: OperandKind,
}

impl SelectOp {
    pub fn new(width: SelectWidth, true_val: OperandKind, false_val: OperandKind) -> Self {
        Self {
            width,
            true_val,
            false_val,
        }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn condition_field(&self) -> Field {
        Field::new(Ident::Condition, FieldTy::Slot)
    }

    pub fn true_val_field(&self) -> Field {
        Field::new(Ident::TrueVal, self.width.field_ty(self.true_val))
    }

    pub fn false_val_field(&self) -> Field {
        Field::new(Ident::FalseVal, self.width.field_ty(self.false_val))
    }

    pub fn fields(&self) -> [Field; 4] {
        [
            self.result_field(),
            self.condition_field(),
            self.true_val_field(),
            self.false_val_field(),
        ]
    }
}

#[derive(Copy, Clone)]
pub struct LoadOp {
    /// The kind of the load operator.
    pub kind: LoadOpKind,
    /// The `ptr` field type.
    pub ptr: OperandKind,
    /// True, if the operator is always operating on (`memory 0`).
    pub mem: MemoryOperand,
    /// True, if the operator uses a 16-bit offset field.
    pub offset: OffsetOperand,
}

/// Describes the memory operand for `load` and `store` operators.
#[derive(Copy, Clone)]
pub enum MemoryOperand {
    /// The `(memory $m)` is provided via immediate operand.
    Immediate,
    /// The operator only operates on `(memory 0)`.
    Mem0,
}

/// Describes the offset operand for `load` and `store` operators.
#[derive(Copy, Clone)]
pub enum OffsetOperand {
    /// A full 64-bit encoded immediate `offset` operand.
    Offset,
    /// An optimized 16-bit encoded immediate `offset`operand.
    Offset16,
}

impl LoadOp {
    pub fn new(
        kind: LoadOpKind,
        ptr: OperandKind,
        mem: MemoryOperand,
        offset: OffsetOperand,
    ) -> Self {
        Self {
            kind,
            ptr,
            mem,
            offset,
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
            OperandKind::Slot => match self.offset {
                OffsetOperand::Offset => FieldTy::U64,
                OffsetOperand::Offset16 => FieldTy::Offset16,
            },
            OperandKind::Immediate => return None,
        };
        Some(Field::new(Ident::Offset, offset_ty))
    }

    pub fn memory_field(&self) -> Option<Field> {
        match self.mem {
            MemoryOperand::Immediate => Some(Field::new(Ident::Memory, FieldTy::Memory)),
            MemoryOperand::Mem0 => None,
        }
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
    I16x8Load8x8,
    U16x8Load8x8,
    I32x4Load16x4,
    U32x4Load16x4,
    I64x2Load32x2,
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
            Self::I16x8Load8x8 => Ident::Load8x8,
            Self::U16x8Load8x8 => Ident::Load8x8,
            Self::I32x4Load16x4 => Ident::Load16x4,
            Self::U32x4Load16x4 => Ident::Load16x4,
            Self::I64x2Load32x2 => Ident::Load32x2,
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
            | Self::S32Load8 => Ty::I32,
            | Self::U32Load8 => Ty::U32,
            | Self::S32Load16 => Ty::I32,
            | Self::U32Load16 => Ty::U32,
            | Self::S64Load8 => Ty::I64,
            | Self::U64Load8 => Ty::U64,
            | Self::S64Load16 => Ty::I64,
            | Self::U64Load16 => Ty::U64,
            | Self::S64Load32 => Ty::I64,
            | Self::U64Load32 => Ty::U64,
            | Self::V128Load => Ty::V128,
            | Self::I16x8Load8x8 => Ty::I16x8,
            | Self::U16x8Load8x8 => Ty::U16x8,
            | Self::I32x4Load16x4 => Ty::I32x4,
            | Self::U32x4Load16x4 => Ty::U32x4,
            | Self::I64x2Load32x2 => Ty::I64x2,
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
            OperandKind::Immediate => FieldTy::U32,
        };
        Field::new(Ident::Value, value_ty)
    }

    pub fn table_field(&self) -> Field {
        Field::new(Ident::Table, FieldTy::Table)
    }

    pub fn fields(&self) -> [Field; 3] {
        [self.table_field(), self.index_field(), self.value_field()]
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
pub enum SimdTy {
    I8x16,
    U8x16,
    I16x8,
    U16x8,
    U32x4,
    U64x2,
}

impl From<SimdTy> for Ty {
    fn from(value: SimdTy) -> Self {
        match value {
            SimdTy::I8x16 => Self::I8x16,
            SimdTy::U8x16 => Self::U8x16,
            SimdTy::I16x8 => Self::I16x8,
            SimdTy::U16x8 => Self::U16x8,
            SimdTy::U32x4 => Self::U32x4,
            SimdTy::U64x2 => Self::U64x2,
        }
    }
}

impl From<SimdTy> for LaneWidth {
    fn from(value: SimdTy) -> Self {
        match value {
            SimdTy::I8x16 => Self::W8,
            SimdTy::U8x16 => Self::W8,
            SimdTy::I16x8 => Self::W16,
            SimdTy::U16x8 => Self::W16,
            SimdTy::U32x4 => Self::W32,
            SimdTy::U64x2 => Self::W64,
        }
    }
}

impl SimdTy {
    pub fn lane_ty(self) -> FieldTy {
        match self {
            SimdTy::I8x16 => FieldTy::ImmLaneIdx16,
            SimdTy::U8x16 => FieldTy::ImmLaneIdx16,
            SimdTy::I16x8 => FieldTy::ImmLaneIdx8,
            SimdTy::U16x8 => FieldTy::ImmLaneIdx8,
            SimdTy::U32x4 => FieldTy::ImmLaneIdx4,
            SimdTy::U64x2 => FieldTy::ImmLaneIdx2,
        }
    }
}

#[derive(Copy, Clone)]
pub struct V128ExtractLaneOp {
    pub ty: SimdTy,
}

impl V128ExtractLaneOp {
    pub fn new(ty: SimdTy) -> Self {
        Self { ty }
    }

    pub fn result_field(&self) -> Field {
        Field::new(Ident::Result, FieldTy::Slot)
    }

    pub fn value_field(&self) -> Field {
        Field::new(Ident::Value, FieldTy::Slot)
    }

    pub fn lane_field(&self) -> Field {
        Field::new(Ident::Lane, self.ty.lane_ty())
    }

    pub fn fields(&self) -> [Field; 3] {
        [self.result_field(), self.value_field(), self.lane_field()]
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
