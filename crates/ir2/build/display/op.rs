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
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
        V128LoadLaneOp,
        V128ReplaceLaneOp,
    },
};
use core::fmt::{self, Display};

pub struct DisplayOp<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayOp<T> {
    pub fn new(val: T, indent: Indent) -> Self {
        Self { value: val, indent }
    }

    pub fn map<V>(&self, val: V) -> DisplayOp<V> {
        DisplayOp {
            value: val,
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
        let ident = DisplayIdent::camel(self.value);
        let fields = DisplaySequence::new(
            ",\n",
            fields
                .iter()
                .filter_map(Option::as_ref)
                .map(|field| (indent.inc(), field))
                .map(DisplayConcat),
        );
        write!(
            f,
            "\
            {indent}{ident} {{\n\
                {fields}\n\
            {indent}}}\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let variants = DisplaySequence::new(
            ",\n",
            self.value
                .ops
                .iter()
                .map(|op| DisplayOp::new(op, indent.inc())),
        );
        write!(
            f,
            "\
            {indent}#[allow(non_camel_case_types)]\n\
            {indent}pub enum Op {{\n\
                        {variants}\n\
            {indent}}}\n\
        "
        )
    }
}

impl Display for DisplayOp<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields();
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl<const N: usize> Display for DisplayOp<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields.map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ V128ReplaceLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}

impl Display for DisplayOp<&'_ V128LoadLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields().map(Option::from);
        self.display_variant(f, &fields)
    }
}
