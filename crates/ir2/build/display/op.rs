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
        TernaryOp,
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

pub struct DisplayForEachOpBody<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayForEachOpBody<T> {
    pub fn new(value: T, indent: Indent) -> Self {
        Self { value, indent }
    }
}

impl Display for DisplayForEachOpBody<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent.inc();
        let camel_ident = DisplayIdent::camel(self.value);
        let snake_ident = DisplayIdent::snake(self.value);
        write!(f, "{indent}{snake_ident} => {camel_ident}")
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
        let for_each_op_body = DisplaySequence::new(
            ",\n",
            self.value
                .ops
                .iter()
                .map(|op| DisplayForEachOpBody::new(op, indent.inc())),
        );
        write!(
            f,
            "\
            {indent}/// A Wasmi bytecode operator or instruction.\n\
            {indent}///\n\
            {indent}/// The [`Op`] type features a small utility API:\n\
            {indent}///\n\
            {indent}/// - [`Op::result_ref`]\n\
            {indent}/// - [`Op::result_mut`]\n\
            {indent}/// - [`Op::code`]\n\
            {indent}#[allow(non_camel_case_types)]\n\
            {indent}#[derive(Debug)]\n\
            {indent}pub enum Op {{\n\
                        {variants}\n\
            {indent}}}\n\
            \n\
            {indent}/// Expands `mac` using the snake-case and camel-case identifiers of all operators.
            {indent}/// \n\
            {indent}/// # Note\n\
            {indent}/// \n\
            {indent}/// Simd related operators are only included if the `simd` crate feature is enabled.\n\
            {indent}/// \n\
            {indent}/// # Example\n\
            {indent}/// \n\
            {indent}/// The expanded code format fed to the `mac` macro is as follows:\n\
            {indent}/// \n\
            {indent}/// ```no-compile\n\
            {indent}/// i32_add_sss => I32Add_Sss,\n\
            {indent}/// i32_add_ssi => I32Add_Ssi,\n\
            {indent}/// i32_sub_sss => I32Sub_Sss,\n\
            {indent}/// i32_sub_ssi => I32Sub_Ssi,\n\
            {indent}/// i32_sub_sis => I32Sub_Sis,\n\
            {indent}/// i32_mul_sss => I32Mul_Sss,\n\
            {indent}/// i32_mul_ssi => I32Mul_Ssi,\n\
            {indent}/// etc ..\n\
            {indent}/// ```\n\
            {indent}#[macro_export]\n\
            {indent}macro_rules! for_each_op {{\n\
            {indent}    ($mac:ident) => {{\n\
                            {for_each_op_body},\n\
            {indent}    }};\n\
            {indent}}}\n\
        "
        )
    }
}

macro_rules! impl_display_variant {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Display for DisplayOp<&'_ $ty> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let fields = self.value.fields().map(Option::from);
                    self.display_variant(f, &fields)
                }
            }
        )*
    };
}
impl_display_variant! {
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
    V128LoadLaneOp,
}

impl<const N: usize> Display for DisplayOp<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.value.fields.map(Option::from);
        self.display_variant(f, &fields)
    }
}
