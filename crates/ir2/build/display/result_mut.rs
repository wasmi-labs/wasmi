use crate::build::{
    display::{ident::DisplayIdent, utils::DisplaySequence, Indent},
    isa::Isa,
    op::{
        Op,
        BinaryOp,
        CmpBranchOp,
        CmpSelectOp,
        GenericOp,
        LoadOp,
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

impl Op {
    /// Returns `true` if `self` has a result field.
    fn has_result(&self) -> bool {
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
            Op::V128LoadLane(_) => true,
        }
    }
}

pub struct DisplayResultMut<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayResultMut<T> {
    pub fn new(value: T, indent: Indent) -> Self {
        Self { value, indent }
    }

    pub fn map<V>(&self, value: V) -> DisplayResultMut<V> {
        DisplayResultMut {
            value,
            indent: self.indent,
        }
    }
}

impl<'a, T> DisplayResultMut<&'a T> {
    fn display_match_arm(&self, f: &mut fmt::Formatter) -> fmt::Result
    where
        DisplayIdent<&'a T>: Display,
    {
        let indent = self.indent;
        let ident = DisplayIdent::camel(self.value);
        write!(f, "{indent}| Self::{ident} {{ result, .. }}")
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

impl Display for DisplayResultMut<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}

impl Display for DisplayResultMut<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}

impl Display for DisplayResultMut<&'_ TernaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}

impl Display for DisplayResultMut<&'_ CmpBranchOp> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for DisplayResultMut<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}

impl Display for DisplayResultMut<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}

impl Display for DisplayResultMut<&'_ StoreOp> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for DisplayResultMut<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}

impl Display for DisplayResultMut<&'_ TableSetOp> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl<const N: usize> Display for DisplayResultMut<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.value.has_result() {
            return Ok(());
        }
        self.display_match_arm(f)
    }
}

impl Display for DisplayResultMut<&'_ V128ReplaceLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}

impl Display for DisplayResultMut<&'_ V128LoadLaneOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}
