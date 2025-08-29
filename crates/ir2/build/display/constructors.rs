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
        V128Splat,
    },
    token::SnakeCase,
};
use core::fmt::{self, Display};

pub struct DisplayConstructor<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayConstructor<T> {
    pub fn new(value: T, indent: Indent) -> Self {
        Self { value, indent }
    }

    pub fn map<V>(&self, value: V) -> DisplayConstructor<V> {
        DisplayConstructor {
            value,
            indent: self.indent,
        }
    }
}

impl<'a, T> DisplayConstructor<&'a T> {
    fn display_constructor(&self, f: &mut fmt::Formatter, fields: &[Option<Field>]) -> fmt::Result
    where
        DisplayIdent<&'a T>: Display,
    {
        let indent = self.indent;
        let snake_ident = DisplayIdent::snake(self.value);
        let camel_ident = DisplayIdent::camel(self.value);
        let fn_params = DisplaySequence::new(", ", fields.iter().filter_map(Option::as_ref));
        let struct_params = DisplaySequence::new(
            ", ",
            fields
                .iter()
                .filter_map(Option::as_ref)
                .map(|param| param.ident)
                .map(SnakeCase),
        );
        write!(
            f,
            "\
            {indent}pub fn {snake_ident}({fn_params}) -> Self {{\n\
            {indent}    Self::{camel_ident} {{ {struct_params} }}\n\
            {indent}}}\n\
            "
        )
    }
}

impl Display for DisplayConstructor<&'_ Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let variants = DisplaySequence::new(
            "",
            self.value
                .ops
                .iter()
                .map(|op| DisplayConstructor::new(op, indent.inc_by(1))),
        );
        write!(
            f,
            "\
            {indent}impl Op {{\n\
                        {variants}\
            {indent}}}\n\
        "
        )
    }
}

impl Display for DisplayConstructor<&'_ Op> {
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

impl Display for DisplayConstructor<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_constructor(f, &fields)
    }
}

impl Display for DisplayConstructor<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_constructor(f, &fields)
    }
}

impl Display for DisplayConstructor<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_constructor(f, &fields)
    }
}

impl Display for DisplayConstructor<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_constructor(f, &fields)
    }
}

impl Display for DisplayConstructor<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields();
        self.display_constructor(f, &fields)
    }
}

impl Display for DisplayConstructor<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields();
        self.display_constructor(f, &fields)
    }
}

impl Display for DisplayConstructor<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_constructor(f, &fields)
    }
}

impl Display for DisplayConstructor<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_constructor(f, &fields)
    }
}

impl<const N: usize> Display for DisplayConstructor<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields.map(Option::from);
        self.display_constructor(f, &fields)
    }
}

impl Display for DisplayConstructor<&'_ V128Splat> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_constructor(f, &fields)
    }
}
