use crate::build::{
    display::{Indent, ident::DisplayIdent, utils::DisplaySequence},
    ident::Ident,
    isa::Isa,
    op::{GenericOp, Op, OperandKind},
    ty::FieldTy,
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

pub struct DisplayResultLoc<T> {
    pub value: T,
    pub indent: Indent,
}

impl<T> DisplayResultLoc<T> {
    pub fn new(value: T, indent: Indent) -> Self {
        Self { value, indent }
    }
}

impl Display for DisplayResultLoc<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let ident = DisplayIdent::camel(self.value);
        write!(
            f,
            "{indent}| Self::{ident} {{ result, .. }} => result.location(),"
        )
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
                .filter(|op| op.has_result_slot())
                .map(|op| DisplayResultMut::new(op, indent.inc_by(3))),
        );
        let variants_loc = DisplaySequence::new(
            "\n",
            self.value
                .ops
                .iter()
                .filter(|op| op.result_loc().is_some())
                .map(|loc| DisplayResultLoc::new(loc, indent.inc_by(3))),
        );
        write!(
            f,
            "\
            {indent}impl Op {{\n\
            {indent}    /// Returns the [`Location`] of the result of `self` if any.\n\
            {indent}    pub fn result_loc(&self) -> Option<Location> {{\n\
            {indent}        let loc = match self {{\n\
                                {variants_loc}\n\
            {indent}            _ => return None,\n\
            {indent}        }};\n\
            {indent}        Some(loc)\n\
            {indent}    }}\n\
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

/// The location of an operand.
#[derive(Copy, Clone)]
pub enum Location {
    /// The operand resides in a register of a certain type.
    Reg,
    /// The operand resides in a stack slot.
    Slot,
}

impl<const N: usize> GenericOp<N> {
    /// Returns `true` if `self` has a `Slot` result field.
    pub fn result_loc(&self) -> Option<Location> {
        let field = self
            .fields
            .iter()
            .find(|field| matches!(field.ident, Ident::Result))?;
        let loc = match field.ty {
            FieldTy::Slot => Location::Slot,
            FieldTy::RegInt | FieldTy::RegF32 | FieldTy::RegF64 => Location::Reg,
            _ => return None,
        };
        Some(loc)
    }
}

impl OperandKind {
    pub fn result_loc(&self) -> Option<Location> {
        match self {
            OperandKind::Reg => Some(Location::Reg),
            OperandKind::Slot => Some(Location::Slot),
            OperandKind::Immediate => unreachable!(),
        }
    }
}

impl Op {
    /// Returns `true` if `self` has a `Slot` result field.
    pub fn result_loc(&self) -> Option<Location> {
        match self {
            Op::Return(_) => None,
            Op::Unary(op) => op.result.result_loc(),
            Op::Binary(op) => op.result.result_loc(),
            Op::Ternary(_) => Some(Location::Slot),
            Op::CmpBranch(_) => None,
            Op::BranchTable(_) => None,
            Op::Select(_) => Some(Location::Reg),
            Op::Load(op) => op.result.result_loc(),
            Op::Store(_) => None,
            Op::GlobalGet(op) => op.result.result_loc(),
            Op::GlobalSet(_) => None,
            Op::TableGet(op) => op.result.result_loc(),
            Op::TableSet(_) => None,
            Op::CallIndirect(_) => None,
            Op::Generic0(op) => op.result_loc(),
            Op::Generic1(op) => op.result_loc(),
            Op::Generic2(op) => op.result_loc(),
            Op::Generic3(op) => op.result_loc(),
            Op::Generic4(op) => op.result_loc(),
            Op::Generic5(op) => op.result_loc(),
            Op::V128ReplaceLane(_) => Some(Location::Slot),
            Op::V128ExtractLane(_) => Some(Location::Slot),
        }
    }

    /// Returns `true` if `self` has a `Slot` result field.
    pub fn has_result_slot(&self) -> bool {
        matches!(self.result_loc(), Some(Location::Slot))
    }
}
