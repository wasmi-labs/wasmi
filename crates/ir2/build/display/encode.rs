use crate::build::{
    display::{ident::DisplayIdent, utils::DisplaySequence, Indent},
    ident::SnakeCase,
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

macro_rules! impl_display_encode {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Display for DisplayEncode<&'_ $ty> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let fields = self.value.fields().map(Option::from);
                    self.display_encode(f, &fields)
                }
            }
        )*
    };
}
impl_display_encode! {
    UnaryOp,
    BinaryOp,
    CmpBranchOp,
    CmpSelectOp,
    LoadOp,
    StoreOp,
    TableGetOp,
    TableSetOp,
    V128ReplaceLaneOp,
    V128LoadLaneOp,
}

impl<const N: usize> Display for DisplayEncode<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields.map(Option::from);
        self.display_encode(f, &fields)
    }
}
