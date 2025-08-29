use crate::build::{
    display::{ident::DisplayIdent, utils::DisplaySequence, Indent},
    isa::Isa,
    op::{
        BinaryOp,
        CmpBranchOp,
        CmpSelectOp,
        Field,
        GenericOp,
        LoadOp,
        Op,
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
        V128SplatOp,
    },
    token::SnakeCase,
};
use core::fmt::{self, Display};

pub struct DisplayEncode<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayEncode<T> {
    pub fn new(value: T, indent: Indent) -> Self {
        Self { value, indent }
    }

    pub fn map<V>(&self, value: V) -> DisplayEncode<V> {
        DisplayEncode {
            value,
            indent: self.indent,
        }
    }
}

impl<'a, T> DisplayEncode<&'a T> {
    fn display_encode(&self, f: &mut fmt::Formatter, fields: &[Option<Field>]) -> fmt::Result
    where
        DisplayIdent<&'a T>: Display,
    {
        let indent = self.indent;
        let camel_ident = DisplayIdent::camel(self.value);
        let match_params = DisplaySequence::new(
            ", ",
            fields
                .iter()
                .filter_map(Option::as_ref)
                .map(|field| field.ident)
                .map(SnakeCase),
        );
        write!(
            f,
            "\
            {indent}Self::{camel_ident} {{ {match_params} }} => {{\n\
            {indent}    (OpCode::{camel_ident}, {match_params}).encode(encoder)\n\
            {indent}}}\n\
            "
        )
    }
}

impl Display for DisplayEncode<&'_ Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let impls = DisplaySequence::new(
            "",
            self.value
                .ops
                .iter()
                .map(|op| DisplayEncode::new(op, indent.inc_by(3))),
        );
        write!(
            f,
            "\
            {indent}impl Encode for Op {{\n\
            {indent}    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<E::Pos, E::Error> {{\n\
            {indent}        match self {{\n\
                                {impls}\n\
            {indent}        }}\n\
            {indent}    }}\n\
            {indent}}}\n\
        "
        )
    }
}

impl Display for DisplayEncode<&'_ Op> {
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
            Op::V128Splat(op) => self.map(op).fmt(f),
        }
    }
}

impl Display for DisplayEncode<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_encode(f, &fields)
    }
}

impl Display for DisplayEncode<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_encode(f, &fields)
    }
}

impl Display for DisplayEncode<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_encode(f, &fields)
    }
}

impl Display for DisplayEncode<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_encode(f, &fields)
    }
}

impl Display for DisplayEncode<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields();
        self.display_encode(f, &fields)
    }
}

impl Display for DisplayEncode<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields();
        self.display_encode(f, &fields)
    }
}

impl Display for DisplayEncode<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_encode(f, &fields)
    }
}

impl Display for DisplayEncode<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_encode(f, &fields)
    }
}

impl<const N: usize> Display for DisplayEncode<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields.map(Option::from);
        self.display_encode(f, &fields)
    }
}

impl Display for DisplayEncode<&'_ V128SplatOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_encode(f, &fields)
    }
}
