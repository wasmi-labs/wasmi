use crate::build::{
    display::{ident::DisplayIdent, utils::DisplaySequence, Indent},
    isa::Isa,
    op::Op,
};
use core::fmt::{self, Display};

pub struct DisplayResultMut<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayResultMut<T> {
    pub fn new(value: T, indent: Indent) -> Self {
        Self { value, indent }
    }
}

impl Display for DisplayResultMut<&'_ Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let variants = DisplaySequence::new(
            "\n",
            self.value
                .ops
                .iter()
                .filter(|op| op.has_result())
                .map(|op| DisplayResultMut::new(op, indent.inc_by(3))),
        );
        write!(
            f,
            "\
            {indent}impl Op {{\n\
            {indent}    /// Returns a shared reference to the result [`Slot`] of `self` if any.\n\
            {indent}    pub fn result_ref(&self) -> Option<&Slot> {{\n\
            {indent}        let res = match self {{\n\
                                {variants} => result,\n\
            {indent}            _ => return None,\n\
            {indent}        }};\n\
            {indent}        Some(res)\n\
            {indent}    }}\n\
            \n\
            {indent}    /// Returns an exclusive reference to the result [`Slot`] of `self` if any.\n\
            {indent}    pub fn result_mut(&mut self) -> Option<&mut Slot> {{\n\
            {indent}        let res = match self {{\n\
                                {variants} => result,\n\
            {indent}            _ => return None,\n\
            {indent}        }};\n\
            {indent}        Some(res)\n\
            {indent}    }}\n\
            {indent}}}\n\
        "
        )
    }
}

impl Display for DisplayResultMut<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let ident = DisplayIdent::camel(self.value);
        write!(f, "{indent}| Self::{ident} {{ result, .. }}")
    }
}

impl Op {
    /// Returns `true` if `self` has a result field.
    pub fn has_result(&self) -> bool {
        match self {
            Op::Unary(_) => true,
            Op::Binary(_) => true,
            Op::Ternary(_) => true,
            Op::CmpBranch(_) => false,
            Op::CmpSelect(_) => true,
            Op::Load(_) => true,
            Op::Store(_) => false,
            Op::TableGet(_) => true,
            Op::TableSet(_) => false,
            Op::Generic0(op) => op.has_result(),
            Op::Generic1(op) => op.has_result(),
            Op::Generic2(op) => op.has_result(),
            Op::Generic3(op) => op.has_result(),
            Op::Generic4(op) => op.has_result(),
            Op::Generic5(op) => op.has_result(),
            Op::V128ReplaceLane(_) => true,
            Op::V128ExtractLane(_) => true,
            Op::V128LoadLane(_) => true,
        }
    }
}
