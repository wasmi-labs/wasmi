use crate::build::{
    display::{
        ident::DisplayIdent,
        utils::{DisplayConcat, DisplaySequence},
        Indent,
    },
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
    },
};
use core::fmt::{self, Display};

pub struct DisplayOp<T> {
    pub val: T,
    pub indent: Indent,
}

impl<T> DisplayOp<T> {
    pub fn new(val: T, indent: Indent) -> Self {
        Self { val, indent }
    }

    pub fn map<V>(&self, val: V) -> DisplayOp<V> {
        DisplayOp {
            val,
            indent: self.indent,
        }
    }
}

impl<'a, T> DisplayOp<&'a T>
where
    DisplayIdent<&'a T>: Display,
{
    fn display_variant(&self, f: &mut fmt::Formatter<'_>, fields: &[Option<Field>]) -> fmt::Result {
        let indent = self.indent;
        let ident = DisplayIdent::camel(self.val);
        let fields = DisplaySequence::new(
            "",
            fields
                .iter()
                .filter_map(Option::as_ref)
                .map(|field| (indent.inc(), field, ",\n"))
                .map(DisplayConcat),
        );
        write!(
            f,
            "\
            {indent}{ident} {{\n\
                {fields}\
            {indent}}},\n\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let variants = DisplaySequence::new(
            "",
            self.val
                .ops
                .iter()
                .map(|op| DisplayOp::new(op, indent.inc())),
        );
        write!(
            f,
            "\
            {indent}#[allow(non_camel_case_types)]
            {indent}pub enum Op {{\n\
                        {variants}\
            {indent}}}\n\
        "
        )
    }
}

impl Display for DisplayOp<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.val {
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

impl Display for DisplayOp<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields();
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl<const N: usize> Display for DisplayOp<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields.map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.val.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}
