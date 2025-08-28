use crate::build::{
    display::{
        ident::DisplayIdent,
        utils::{DisplayConcat, DisplaySequence, IntoDisplayMaybe as _},
        Indent,
    },
    op::{
        BinaryOp,
        CmpBranchOp,
        CmpSelectOp,
        GenericOp,
        Input,
        LoadOp,
        Op,
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
    },
    token::{CamelCase, SnakeCase},
    Isa,
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
        write!(f, "{impls}")
    }
}

impl Display for DisplayDecode<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Op::Unary(op) => self.map(op).fmt(f),
            Op::Binary(op) => self.map(op).fmt(f),
            Op::CmpBranch(op) => self.map(op).fmt(f),
            Op::CmpSelect(op) => self.map(op).fmt(f),
            Op::Load(op) => self.map(op).fmt(f),
            Op::Store(op) => self.map(op).fmt(f),
            Op::TableGet(op) => self.map(op).fmt(f),
            Op::TableSet(op) => self.map(op).fmt(f),
            Op::Generic0(op) => self.map(op).fmt(f),
            Op::Generic1(op) => self.map(op).fmt(f),
            Op::Generic2(op) => self.map(op).fmt(f),
            Op::Generic3(op) => self.map(op).fmt(f),
            Op::Generic4(op) => self.map(op).fmt(f),
            Op::Generic5(op) => self.map(op).fmt(f),
        }
    }
}

impl Display for DisplayDecode<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let camel_ident = DisplayIdent::camel(self.value);
        writeln!(f, "pub type {camel_ident} = UnaryOp;")
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

impl Display for DisplayDecode<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let lhs = op.lhs_field().ty;
        let rhs = op.rhs_field().ty;
        writeln!(f, "pub type {camel_ident} = CmpBranchOp<{lhs}, {rhs}>;")
    }
}

impl Display for DisplayDecode<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let lhs = op.lhs_field().ty;
        let rhs = op.rhs_field().ty;
        writeln!(f, "pub type {camel_ident} = CmpSelectOp<{lhs}, {rhs}>;")
    }
}

impl Display for DisplayDecode<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let mem0_offset16 = (op.mem0 && op.offset16)
            .then_some("Mem0Offset16")
            .display_maybe();
        let result_suffix = CamelCase(Input::Stack);
        let ptr_suffix = SnakeCase(op.ptr);
        writeln!(
            f,
            "pub type {camel_ident} = LoadOp{mem0_offset16}_{result_suffix}{ptr_suffix};"
        )
    }
}

impl Display for DisplayDecode<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.value;
        let camel_ident = DisplayIdent::camel(op);
        let mem0_offset16 = (op.mem0 && op.offset16)
            .then_some("Mem0Offset16")
            .display_maybe();
        let ptr_suffix = CamelCase(op.ptr);
        let value_ty = op.value_field().ty;
        writeln!(
            f,
            "pub type {camel_ident} = StoreOp{mem0_offset16}_{ptr_suffix}<{value_ty}>;"
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
                .map(|ident| (indent.inc_by(3), ident, ": Decode::decode(decoder)"))
                .map(DisplayConcat),
        );
        write!(
            f,
            "\
            {indent}pub struct {camel_ident} {{\n\
                        {fields}\n\
            {indent}}}\n\
            {indent}impl Decode for {camel_ident} {{\n\
            {indent}    unsafe fn decode<D: Decoder>(decoder: &mut D) -> Self {{\n\
            {indent}        Self {{\n\
                                {constructors}\n\
            {indent}        }}\n\
            {indent}    }}\n\
            {indent}}}\n\
            "
        )
    }
}
