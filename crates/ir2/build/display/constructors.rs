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
        TernaryOp,
        UnaryOp,
        V128ExtractLaneOp,
        V128LoadLaneOp,
        V128ReplaceLaneOp,
    },
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

macro_rules! impl_display_constructor {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Display for DisplayConstructor<&'_ $ty> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let fields = self.value.fields().map(Option::from);
                    self.display_constructor(f, &fields)
                }
            }
        )*
    };
}
impl_display_constructor! {
    UnaryOp,
    BinaryOp,
    TernaryOp,
    CmpBranchOp,
    CmpSelectOp,
    LoadOp,
    StoreOp,
    TableGetOp,
    TableSetOp,
    V128ReplaceLaneOp,
    V128ExtractLaneOp,
    V128LoadLaneOp,
}

impl<const N: usize> Display for DisplayConstructor<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields.map(Option::from);
        self.display_constructor(f, &fields)
    }
}
