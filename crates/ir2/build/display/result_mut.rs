use crate::build::{
    display::{ident::DisplayIdent, utils::DisplaySequence, Indent},
    isa::Isa,
    op::{
        BinaryOp,
        CmpBranchOp,
        CmpSelectOp,
        GenericOp,
        LoadOp,
        Op,
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
        V128Splat,
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

impl Display for DisplayResultMut<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
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
            Op::V128Splat(op) => self.map(op).fmt(f),
        }
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

impl Display for DisplayResultMut<&'_ V128Splat> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_match_arm(f)
    }
}
