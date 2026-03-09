use crate::build::{
    display::utils::{DisplayConcat, IntoDisplayMaybe as _},
    ident::{CamelCase, Case, Ident, Sep, SnakeCase},
    op::{
        BinaryOp,
        CmpBranchOp,
        GenericOp,
        LoadOp,
        MemoryOperand,
        OffsetOperand,
        OperandKind,
        SelectOp,
        StoreOp,
        TableGetOp,
        TableSetOp,
        TernaryOp,
        UnaryOp,
        V128ExtractLaneOp,
        V128LoadLaneOp,
        V128ReplaceLaneOp,
    },
    ty::Ty,
};
use core::fmt::{self, Display};

pub struct DisplayIdent<T> {
    pub value: T,
    pub case: Case,
}

impl<T> DisplayIdent<T> {
    pub fn camel(value: T) -> Self {
        Self {
            value,
            case: Case::Camel,
        }
    }

    pub fn snake(value: T) -> Self {
        Self {
            value,
            case: Case::Snake,
        }
    }

    pub fn map<V>(&self, value: V) -> DisplayIdent<V> {
        DisplayIdent {
            value,
            case: self.case,
        }
    }
}

/// [`Display`] wrapper for types that can act as operator identifier prefices.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct IdentPrefix<T>(pub T);

impl Display for CamelCase<Ty> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0 {
            Ty::Bits8 => "8",
            Ty::Bits16 => "16",
            Ty::Bits32 => "32",
            Ty::Bits64 => "64",
            Ty::I32 => "I32",
            Ty::I64 => "I64",
            Ty::U8 => "U8",
            Ty::U32 => "U32",
            Ty::U64 => "U64",
            Ty::NonZeroI32 => "I32",
            Ty::NonZeroI64 => "I64",
            Ty::NonZeroU32 => "U32",
            Ty::NonZeroU64 => "U64",
            Ty::F32 => "F32",
            Ty::F64 => "F64",
            Ty::SignF32 => "F32",
            Ty::SignF64 => "F64",
            Ty::V128 => "V128",
            Ty::I8x16 => "I8x16",
            Ty::I16x8 => "I16x8",
            Ty::I32x4 => "I32x4",
            Ty::I64x2 => "I64x2",
            Ty::U8x16 => "U8x16",
            Ty::U16x8 => "U16x8",
            Ty::U32x4 => "U32x4",
            Ty::U64x2 => "U64x2",
            Ty::F32x4 => "F32x4",
            Ty::F64x2 => "F64x2",
        };
        f.write_str(s)
    }
}

impl Display for SnakeCase<Ty> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0 {
            Ty::Bits8 => "8",
            Ty::Bits16 => "16",
            Ty::Bits32 => "32",
            Ty::Bits64 => "64",
            Ty::I32 => "i32",
            Ty::I64 => "i64",
            Ty::U8 => "u8",
            Ty::U32 => "u32",
            Ty::U64 => "u64",
            Ty::NonZeroI32 => "i32",
            Ty::NonZeroI64 => "i64",
            Ty::NonZeroU32 => "u32",
            Ty::NonZeroU64 => "u64",
            Ty::F32 => "f32",
            Ty::F64 => "f64",
            Ty::SignF32 => "f32",
            Ty::SignF64 => "f64",
            Ty::V128 => "v128",
            Ty::I8x16 => "i8x16",
            Ty::I16x8 => "i16x8",
            Ty::I32x4 => "i32x4",
            Ty::I64x2 => "i64x2",
            Ty::U8x16 => "u8x16",
            Ty::U16x8 => "u16x8",
            Ty::U32x4 => "u32x4",
            Ty::U64x2 => "u64x2",
            Ty::F32x4 => "f32x4",
            Ty::F64x2 => "f64x2",
        };
        f.write_str(s)
    }
}

impl Display for CamelCase<IdentPrefix<Ty>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        CamelCase(self.0.0).fmt(f)
    }
}

impl Display for SnakeCase<IdentPrefix<Ty>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        SnakeCase(self.0.0).fmt(f)?;
        SnakeCase(Sep).fmt(f)?;
        Ok(())
    }
}

/// [`Display`] wrapper for types that can act as operator identifier suffices.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct IdentSuffix<T>(pub T);

impl Display for CamelCase<IdentSuffix<Ty>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        CamelCase(self.0.0).fmt(f)
    }
}

impl Display for SnakeCase<IdentSuffix<Ty>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.0 {
            Ty::Bits8 | Ty::Bits16 | Ty::Bits32 | Ty::Bits64 => {}
            _ => SnakeCase(Sep).fmt(f)?,
        }
        SnakeCase(self.0.0).fmt(f)
    }
}

/// [`Display`] wrapper for types that can act as suffices for operator identifiers.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Suffix<T>(pub T);

impl Display for CamelCase<Suffix<OperandKind>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0.0 {
            OperandKind::Slot => "S",
            OperandKind::Immediate => "I",
        };
        f.write_str(s)
    }
}

impl Display for SnakeCase<Suffix<OperandKind>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.0.0 {
            OperandKind::Slot => "s",
            OperandKind::Immediate => "i",
        };
        f.write_str(s)
    }
}

impl Display for DisplayIdent<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let ident = case.wrap(op.ident);
        let ident_prefix = case.wrap(IdentPrefix(op.result_ty));
        let ident_suffix = match op.value_ty != op.result_ty {
            true => Some(IdentSuffix(op.value_ty)),
            false => None,
        };
        let ident_suffix = ident_suffix.map(|i| case.wrap(i)).display_maybe();
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let value_suffix = SnakeCase(Suffix(op.value));
        write!(
            f,
            "{ident_prefix}{ident}{ident_suffix}_{result_suffix}{value_suffix}"
        )
    }
}

impl Display for DisplayIdent<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let ident = case.wrap(op.ident);
        let is_cmp = op.caps.is_cmp();
        let ident_prefix = match is_cmp {
            true => case.wrap(IdentPrefix(op.lhs_ty)),
            false => case.wrap(IdentPrefix(op.result_ty)),
        };
        let ident_suffix = match op.result_ty != op.lhs_ty {
            true => Some(IdentSuffix(op.rhs_ty)),
            false => None,
        };
        let ident_suffix = ident_suffix
            .filter(|_| !is_cmp)
            .map(|i| case.wrap(i))
            .display_maybe();
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let lhs_suffix = SnakeCase(Suffix(op.lhs));
        let rhs_suffix = SnakeCase(Suffix(op.rhs));
        write!(
            f,
            "{ident_prefix}{ident}{ident_suffix}_{result_suffix}{lhs_suffix}{rhs_suffix}"
        )
    }
}

impl Display for DisplayIdent<&'_ TernaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let kind = op.kind;
        let ident = case.wrap(kind.ident());
        let ident_prefix = case.wrap(IdentPrefix(kind.ident_prefix()));
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let a_suffix = SnakeCase(Suffix(OperandKind::Slot));
        let b_suffix = SnakeCase(Suffix(OperandKind::Slot));
        let c_suffix = SnakeCase(Suffix(OperandKind::Slot));
        write!(
            f,
            "{ident_prefix}{ident}_{result_suffix}{a_suffix}{b_suffix}{c_suffix}"
        )
    }
}

impl Display for DisplayIdent<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let sep = case.wrap(Sep);
        let op = self.value;
        let branch = case.wrap(Ident::Branch);
        let ident = case.wrap(op.ident);
        let input_ident = case.wrap(IdentPrefix(op.input_ty));
        let lhs_suffix = case.wrap(Suffix(self.value.lhs));
        let rhs_suffix = SnakeCase(Suffix(self.value.rhs));
        write!(
            f,
            "{branch}{sep}{input_ident}{ident}_{lhs_suffix}{rhs_suffix}"
        )
    }
}

impl Display for DisplayIdent<&'_ SelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let select = case.wrap(Ident::Select);
        let width = self.value.width;
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let condition_suffix = SnakeCase(Suffix(OperandKind::Slot));
        let tval_suffix = SnakeCase(Suffix(self.value.true_val));
        let fval_suffix = SnakeCase(Suffix(self.value.false_val));
        write!(
            f,
            "{select}{width}_{result_suffix}{condition_suffix}{tval_suffix}{fval_suffix}"
        )
    }
}

impl Display for DisplayIdent<MemoryOperand> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let MemoryOperand::Mem0 = self.value {
            let case = self.case;
            case.wrap(Sep).fmt(f)?;
            case.wrap(Ident::Mem0).fmt(f)?;
        }
        Ok(())
    }
}

impl Display for DisplayIdent<OffsetOperand> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let OffsetOperand::Offset16 = self.value {
            let case = self.case;
            case.wrap(Sep).fmt(f)?;
            case.wrap(Ident::Offset16).fmt(f)?;
        }
        Ok(())
    }
}

impl Display for DisplayIdent<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let kind = self.value.kind;
        let ident = case.wrap(kind.ident());
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let ptr_suffix = SnakeCase(Suffix(self.value.ptr));
        let ident_prefix = self
            .value
            .kind
            .ident_prefix()
            .map(IdentPrefix)
            .map(|v| case.wrap(v))
            .display_maybe();
        let mem_suffix = self.map(self.value.mem);
        let offset_suffix = self.map(self.value.offset);
        write!(
            f,
            "{ident_prefix}{ident}{mem_suffix}{offset_suffix}_{result_suffix}{ptr_suffix}",
        )
    }
}

impl Display for DisplayIdent<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let kind = self.value.kind;
        let ident = case.wrap(kind.ident());
        let ptr_suffix = case.wrap(Suffix(self.value.ptr));
        let value_suffix = SnakeCase(Suffix(self.value.value));
        let sep = case.wrap(Sep);
        let ident_prefix = self
            .value
            .kind
            .ident_prefix()
            .map(|v| (case.wrap(v), sep))
            .map(DisplayConcat)
            .display_maybe();
        let mem0_ident = self
            .value
            .mem0
            .then_some(Ident::Mem0)
            .map(|v| (sep, case.wrap(v)))
            .map(DisplayConcat)
            .display_maybe();
        let offset16_ident = self
            .value
            .offset16
            .then_some(Ident::Offset16)
            .map(|v| (sep, case.wrap(v)))
            .map(DisplayConcat)
            .display_maybe();
        write!(
            f,
            "{ident_prefix}{ident}{mem0_ident}{offset16_ident}_{ptr_suffix}{value_suffix}",
        )
    }
}

impl<const N: usize> Display for DisplayIdent<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.case.wrap(self.value.ident).fmt(f)
    }
}

impl Display for DisplayIdent<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let ident = case.wrap(Ident::TableGet);
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let index_suffix = SnakeCase(Suffix(self.value.index));
        write!(f, "{ident}_{result_suffix}{index_suffix}")
    }
}

impl Display for DisplayIdent<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let ident = case.wrap(Ident::TableSet);
        let index_suffix = case.wrap(Suffix(self.value.index));
        let value_suffix = SnakeCase(Suffix(self.value.value));
        write!(f, "{ident}_{index_suffix}{value_suffix}")
    }
}

impl Display for DisplayIdent<&'_ V128ExtractLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let ident = case.wrap(Ident::ExtractLane);
        let lane_ty = case.wrap(IdentPrefix(Ty::from(op.ty)));
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let v128_suffix = SnakeCase(Suffix(OperandKind::Slot));
        write!(f, "{lane_ty}{ident}_{result_suffix}{v128_suffix}")
    }
}

impl Display for DisplayIdent<&'_ V128ReplaceLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let sep = case.wrap(Sep);
        let v128 = case.wrap(Ident::V128);
        let ident = case.wrap(Ident::ReplaceLane);
        let width = op.width;
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let v128_suffix = SnakeCase(Suffix(OperandKind::Slot));
        let value_suffix = SnakeCase(Suffix(op.value));
        write!(
            f,
            "{v128}{sep}{ident}{width}_{result_suffix}{v128_suffix}{value_suffix}"
        )
    }
}

impl Display for DisplayIdent<&'_ V128LoadLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = self.case;
        let op = self.value;
        let sep = case.wrap(Sep);
        let v128 = case.wrap(Ident::V128);
        let ident = case.wrap(Ident::LoadLane);
        let width = u8::from(op.width);
        let result_suffix = case.wrap(Suffix(OperandKind::Slot));
        let ptr_suffix = SnakeCase(Suffix(op.ptr));
        let v128_suffix = SnakeCase(Suffix(OperandKind::Slot));
        let mem0_ident = self
            .value
            .mem0
            .then_some(Ident::Mem0)
            .map(|v| (sep, case.wrap(v)))
            .map(DisplayConcat)
            .display_maybe();
        let offset16_ident = self
            .value
            .offset16
            .then_some(Ident::Offset16)
            .map(|v| (sep, case.wrap(v)))
            .map(DisplayConcat)
            .display_maybe();
        write!(
            f,
            "{v128}{sep}{ident}{width}{mem0_ident}{offset16_ident}_{result_suffix}{ptr_suffix}{v128_suffix}"
        )
    }
}
