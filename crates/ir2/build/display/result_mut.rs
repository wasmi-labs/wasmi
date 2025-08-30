use crate::build::{
    display::{ident::DisplayIdent, utils::DisplaySequence, Indent},
    isa::Isa,
    op::{
        BinaryOp,
        CmpBranchOp,
        CmpSelectOp,
        GenericOp,
        LoadOp,
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
        V128ReplaceLaneOp,
    },
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
        writeln!(f, "{indent}Self::{ident} {{ result, .. }} => result,")
    }
}

impl Display for DisplayResultMut<&'_ Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let variants = DisplaySequence::new(
            "",
            self.value
                .ops
                .iter()
                .map(|op| DisplayResultMut::new(op, indent.inc_by(3))),
        );
        write!(
            f,
            "\
            {indent}impl Op {{\n\
            {indent}    pub fn result_mut(&mut self) -> Option<&mut Stack> {{\n\
            {indent}        let res = match self {{\n\
                                {variants}\
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
