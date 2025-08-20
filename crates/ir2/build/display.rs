use crate::build::{
    isa::Isa,
    op::{BinaryOp, BinaryOpKind, FieldTy, Input, Op, Ty, UnaryOp, CmpOpKind},
    token::{CamelCase, Ident, SnakeCase},
};
use core::fmt::{self, Display};

#[derive(Copy, Clone, Default)]
pub struct Indent(usize);

impl Indent {
    pub fn inc(self) -> Self {
        Self(self.0 + 1)
    }
}

impl Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, "    ")?;
        }
        Ok(())
    }
}

pub struct DisplayEnum<T> {
    pub val: T,
    pub indent: Indent,
}

impl<T> DisplayEnum<T> {
    pub fn new(val: T, indent: Indent) -> Self {
        Self { val, indent }
    }

    pub fn scoped<V>(&self, val: V) -> DisplayEnum<V> {
        DisplayEnum {
            val,
            indent: self.indent.inc(),
        }
    }

    pub fn map<V>(&self, val: V) -> DisplayEnum<V> {
        DisplayEnum {
            val,
            indent: self.indent,
        }
    }
}

impl Display for DisplayEnum<Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        write!(
            f,
            "\
            {indent}pub enum Op {{\n\
        "
        )?;
        for op in &self.val.ops {
            write!(f, "{}", self.scoped(op))?;
        }
        write!(
            f,
            "\
            {indent}}}\n\
        "
        )?;
        Ok(())
    }
}

impl Display for DisplayEnum<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.val {
            Op::Unary(op) => self.map(op).fmt(f),
            Op::Binary(op) => self.map(op).fmt(f),
            Op::CmpBranch(_op) => Ok(()),
            Op::CmpSelect(_op) => Ok(()),
            Op::Load(_op) => Ok(()),
            Op::Store(_op) => Ok(()),
            Op::Generic0(_op) => Ok(()),
            Op::Generic1(_op) => Ok(()),
            Op::Generic2(_op) => Ok(()),
            Op::Generic3(_op) => Ok(()),
            Op::Generic4(_op) => Ok(()),
        }
    }
}

impl Display for DisplayEnum<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.val.kind.is_conversion() {
            self.display_conversion(f)
        } else {
            self.display_unary(f)
        }
    }
}

impl DisplayEnum<&'_ UnaryOp> {
    fn display_unary(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.val.kind;
        let ident = CamelCase(kind.ident());
        let result_ident = CamelCase(Ident::from(kind.result_ty()));
        let result_field = FieldTy::Stack;
        let value_field = FieldTy::Stack;
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        write!(
            f,
            "\
            {indent0}{result_ident}{ident}_Ss {{\n\
            {indent1}result: {result_field},\n\
            {indent1}value: {value_field},\n\
            {indent0}}},\n\
            ",
        )
    }

    fn display_conversion(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.val.kind;
        let ident = CamelCase(kind.ident());
        let result_ident = CamelCase(Ident::from(kind.result_ty()));
        let input_ident = CamelCase(Ident::from(kind.input_ty()));
        let result_id = CamelCase(Input::Stack);
        let value_id = CamelCase(Input::Stack);
        let result_field = FieldTy::Stack;
        let value_field = FieldTy::Stack;
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        write!(
            f,
            "\
            {indent0}{result_ident}{ident}{input_ident}_Ss {{\n\
            {indent1}result: {result_field},\n\
            {indent1}value: {value_field},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayEnum<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let kind = self.val.kind;
        let ident = CamelCase(kind.ident());
        let result_ident = CamelCase(Ident::from(kind.result_ty()));
        let result_ty = FieldTy::Stack;
        let lhs_ty = kind.lhs_field(self.val.lhs);
        let rhs_ty = kind.rhs_field(self.val.rhs);
        let result_suffix = CamelCase(Input::Stack);
        let lhs_suffix = SnakeCase(self.val.lhs);
        let rhs_suffix = SnakeCase(self.val.rhs);
        write!(
            f,
            "\
            {indent0}{result_ident}{ident}_S{lhs_suffix}{rhs_suffix} {{\n\
            {indent1}result: {result_ty},\n\
            {indent1}lhs: {lhs_ty},\n\
            {indent1}rhs: {rhs_ty},\n\
            {indent0}}},\n\
            ",
        )
    }
}
