use crate::build::{
    display::{Indent, ident::DisplayIdent, utils::DisplaySequence},
    ident::{Ident, SnakeCase},
    isa::Isa,
    op::{
        BinaryOp,
        BranchTableOp,
        CallIndirectOp,
        CmpBranchOp,
        Field,
        GenericOp,
        GlobalGetOp,
        GlobalSetOp,
        LoadOp,
        ReplaceLaneOp,
        ReturnOp,
        SelectOp,
        StoreOp,
        TableGetOp,
        TableSetOp,
        TernaryOp,
        UnaryOp,
        V128ExtractLaneOp,
    },
    ty::FieldTy,
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
        fn is_default_init(ty: FieldTy) -> bool {
            ty.is_reg() || ty.is_zero()
        }

        let indent = self.indent;
        let snake_ident = DisplayIdent::snake(self.value);
        let camel_ident = DisplayIdent::camel(self.value);
        let fn_params = DisplaySequence::new(
            ", ",
            fields
                .iter()
                .filter_map(Option::as_ref)
                .filter(|field| !is_default_init(field.ty)),
        );
        let struct_params = DisplaySequence::new(
            ", ",
            fields
                .iter()
                .filter_map(Option::as_ref)
                .map(|param| match param.ty {
                    ty if is_default_init(ty) => DisplayConstructorInit::Default(*param),
                    _ => DisplayConstructorInit::Param(param.ident),
                }),
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

enum DisplayConstructorInit {
    Param(Ident),
    Default(Field),
}

impl Display for DisplayConstructorInit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Param(ident) => SnakeCase(*ident).fmt(f),
            Self::Default(field) => {
                SnakeCase(field.ident).fmt(f)?;
                f.write_str(": <")?;
                field.ty.fmt(f)?;
                f.write_str(">::default()")?;
                Ok(())
            }
        }
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
    ReturnOp,
    UnaryOp,
    BinaryOp,
    TernaryOp,
    CmpBranchOp,
    BranchTableOp,
    SelectOp,
    LoadOp,
    StoreOp,
    GlobalGetOp,
    GlobalSetOp,
    TableGetOp,
    TableSetOp,
    CallIndirectOp,
    ReplaceLaneOp,
    V128ExtractLaneOp,
}

impl<const N: usize> Display for DisplayConstructor<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields.map(Option::from);
        self.display_constructor(f, &fields)
    }
}
