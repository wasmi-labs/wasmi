use crate::build::{
    Isa,
    display::{
        Indent,
        Suffix,
        ident::DisplayIdent,
        utils::{DisplayConcat, DisplayMaybe, DisplaySequence, IntoDisplayMaybe as _},
    },
    ident::{CamelCase, Ident, SnakeCase},
    op::{
        BinaryOp,
        CmpBranchOp,
        GenericOp,
        LaneWidth,
        LoadKind,
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
        V128ReplaceLaneOp,
    },
    ty::FieldTy,
};
use core::fmt::{self, Display};

pub struct DisplayDecode<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayDecode<T> {
    pub fn new(value: T, indent: Indent) -> Self {
        Self { value, indent }
    }

    pub fn map<V>(&self, value: V) -> DisplayDecode<V> {
        DisplayDecode {
            value,
            indent: self.indent,
        }
    }
}

impl Display for DisplayDecode<&'_ Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let impls = DisplaySequence::new(
            "",
            self.value
                .ops
                .iter()
                .map(|op| DisplayDecode::new(op, indent)),
        );
        impls.fmt(f)
    }
}

impl Display for DisplayDecode<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let camel_ident = DisplayIdent::camel(self.value);
        let value_ty = self.value.value_field().ty;
        writeln!(f, "pub type {camel_ident} = UnaryOp<{value_ty}>;")
    }
}

impl Display for DisplayDecode<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let lhs = op.lhs_field().ty;
        let rhs = op.rhs_field().ty;
        writeln!(f, "pub type {camel_ident} = BinaryOp<{lhs}, {rhs}>;")
    }
}

impl Display for DisplayDecode<&'_ TernaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let a = op.a_field().ty;
        let b = op.b_field().ty;
        let c = op.c_field().ty;
        writeln!(f, "pub type {camel_ident} = TernaryOp<{a}, {b}, {c}>;")
    }
}

impl Display for DisplayDecode<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let lhs = op.lhs_field().ty;
        let rhs = op.rhs_field().ty;
        writeln!(f, "pub type {camel_ident} = CmpBranchOp<{lhs}, {rhs}>;")
    }
}

impl Display for DisplayDecode<&'_ SelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let lhs = op.true_val_field().ty;
        let rhs = op.false_val_field().ty;
        writeln!(f, "pub type {camel_ident} = SelectOp<{lhs}, {rhs}>;")
    }
}

impl Display for DisplayDecode<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let mem0_suffix = match op.mem {
            MemoryOperand::Immediate => "",
            MemoryOperand::Mem0 => "Mem0",
        };
        let offset16_suffix = match op.offset {
            OffsetOperand::Offset => "",
            OffsetOperand::Offset16 => "Offset16",
        };
        let (lane_suffix, lane_param) = match op.kind {
            LoadKind::Lane { width, .. } => {
                let lane_param = DisplayConcat(('<', FieldTy::from(width), '>'));
                (
                    Some(CamelCase(Ident::Lane)),
                    Some(lane_param).display_maybe(),
                )
            }
            _ => (None, DisplayMaybe::None),
        };
        let lane_suffix = lane_suffix.display_maybe();
        let result_suffix = CamelCase(Suffix(OperandKind::Slot));
        let ptr_suffix = SnakeCase(Suffix(op.ptr));
        writeln!(
            f,
            "pub type {camel_ident} = Load{lane_suffix}Op{mem0_suffix}{offset16_suffix}_{result_suffix}{ptr_suffix}{lane_param};"
        )
    }
}

impl Display for DisplayDecode<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let lane_ident = op
            .laneidx_field()
            .map(|_| CamelCase(Ident::Lane))
            .display_maybe();
        let mem0_suffix = match op.mem {
            MemoryOperand::Immediate => "",
            MemoryOperand::Mem0 => "Mem0",
        };
        let offset16_suffix = match op.offset {
            OffsetOperand::Offset => "",
            OffsetOperand::Offset16 => "Offset16",
        };
        let ptr_suffix = CamelCase(Suffix(op.ptr));
        let value_ty = op.value_field().ty;
        let laneidx_ty = op
            .laneidx_field()
            .map(|field| (", ", field.ty))
            .map(DisplayConcat)
            .display_maybe();
        writeln!(
            f,
            "pub type {camel_ident} = Store{lane_ident}Op{mem0_suffix}{offset16_suffix}_{ptr_suffix}<{value_ty}{laneidx_ty}>;"
        )
    }
}

impl Display for DisplayDecode<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let index_ty = op.index_field().ty;
        writeln!(f, "pub type {camel_ident} = TableGet<{index_ty}>;")
    }
}

impl Display for DisplayDecode<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let index_ty = op.index_field().ty;
        let value_ty = op.value_field().ty;
        writeln!(
            f,
            "pub type {camel_ident} = TableSet<{index_ty}, {value_ty}>;"
        )
    }
}

impl<const N: usize> Display for DisplayDecode<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let op = self.value;
        if op.fields.is_empty() {
            // No need to decode type with no operands (a.k.a. fields).
            return Ok(());
        }
        let camel_ident = DisplayIdent::camel(self.value);
        let fields = DisplaySequence::new(
            ",\n",
            op.fields
                .iter()
                .map(|field| (indent.inc(), "pub ", field))
                .map(DisplayConcat),
        );
        let constructors = DisplaySequence::new(
            ",\n",
            op.fields
                .iter()
                .map(|field| field.ident)
                .map(SnakeCase)
                .map(|ident| (indent.inc_by(3), ident, ": Decode::decode(decoder)?"))
                .map(DisplayConcat),
        );
        write!(
            f,
            "\
            {indent}pub struct {camel_ident} {{\n\
                        {fields}\n\
            {indent}}}\n\
            {indent}impl Decode for {camel_ident} {{\n\
            {indent}    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {{\n\
            {indent}        Ok(Self {{\n\
                                {constructors}\n\
            {indent}        }})\n\
            {indent}    }}\n\
            {indent}}}\n\
            "
        )
    }
}

impl Display for DisplayDecode<&'_ V128ReplaceLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let value_ty = op.value_field().ty;
        let len_lanes = op.width.len_lanes();
        writeln!(
            f,
            "pub type {camel_ident} = V128ReplaceLaneOp<{value_ty}, {len_lanes}>;"
        )
    }
}

impl Display for DisplayDecode<&'_ V128ExtractLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let len_lanes = LaneWidth::from(op.ty).len_lanes();
        writeln!(
            f,
            "pub type {camel_ident} = V128ExtractLaneOp<{len_lanes}>;"
        )
    }
}
