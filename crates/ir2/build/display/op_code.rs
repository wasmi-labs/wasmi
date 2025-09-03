use crate::build::{
    display::{
        ident::DisplayIdent,
        utils::{DisplayConcat, DisplaySequence},
        Indent,
    },
    isa::Isa,
    op::Op,
};
use core::fmt::{self, Display};

pub struct DisplayOpCode<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayOpCode<T> {
    pub fn new(value: T, indent: Indent) -> Self {
        Self { value, indent }
    }
}

impl Display for DisplayOpCode<&'_ Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let variants = DisplaySequence::new(
            ",\n",
            self.value
                .ops
                .iter()
                .map(|op| (indent.inc(), DisplayIdent::camel(op)))
                .map(DisplayConcat),
        );
        let match_arms_code = DisplaySequence::new(
            ",\n",
            self.value
                .ops
                .iter()
                .map(|op| DisplayOpCode::new(op, indent.inc_by(3))),
        );
        let match_arms_tryfrom = DisplaySequence::new(
            ",\n",
            self.value
                .ops
                .iter()
                .map(DisplayTryFromU16)
                .map(|op| DisplayOpCode::new(op, indent.inc_by(3))),
        );
        write!(
            f,
            "\
            {indent}#[allow(non_camel_case_types)]\n\
            {indent}#[repr(u16)]\n\
            {indent}pub enum OpCode {{\n\
                        {variants}\n\
            {indent}}}\n\
            \n\
            {indent}impl Op {{\n\
            {indent}    pub fn code(&self) -> OpCode {{\n\
            {indent}        match self {{\n\
                                {match_arms_code}\n\
            {indent}        }}\n\
            {indent}    }}\n\
            {indent}}}\n\
            \n\
            {indent}impl TryFrom<u16> for OpCode {{\n\
            {indent}    type Error = InvalidOpCode;\n\
            {indent}    fn try_from(value: u16) -> Result<Self, Self::Error> {{\n\
            {indent}        let op_code = match value {{\n\
                                {match_arms_tryfrom},\n\
            {indent}            _ => return Err(InvalidOpCode),\n\
            {indent}        }};\n\
            {indent}        Ok(op_code)\n\
            {indent}    }}\n\
            {indent}}}\n\
        "
        )
    }
}

impl Display for DisplayOpCode<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let ident = DisplayIdent::camel(self.value);
        write!(f, "{indent}Self::{ident} {{ .. }} => OpCode::{ident}")
    }
}

pub struct DisplayTryFromU16<T>(T);
impl Display for DisplayOpCode<DisplayTryFromU16<&'_ Op>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let ident = DisplayIdent::camel(self.value.0);
        write!(f, "{indent}x if x == Self::{ident} as _ => Self::{ident}")
    }
}
