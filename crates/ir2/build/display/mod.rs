mod ident;
mod utils;

pub use self::utils::Indent;
use self::{
    ident::DisplayIdent,
    utils::{DisplayConcat, DisplaySequence, IntoDisplayMaybe as _},
};
use crate::build::{
    isa::Isa,
    op::{
        BinaryOp,
        CmpBranchOp,
        CmpSelectOp,
        FieldTy,
        GenericOp,
        LoadOp,
        Op,
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
    },
};
use core::fmt::{self, Display};

pub struct DisplayOp<T> {
    pub val: T,
    pub indent: Indent,
}

impl<T> DisplayOp<T> {
    pub fn new(val: T, indent: Indent) -> Self {
        Self { val, indent }
    }

    pub fn scoped<V>(&self, val: V) -> DisplayOp<V> {
        DisplayOp {
            val,
            indent: self.indent.inc(),
        }
    }

    pub fn map<V>(&self, val: V) -> DisplayOp<V> {
        DisplayOp {
            val,
            indent: self.indent,
        }
    }
}

impl Display for DisplayOp<Isa> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let variants = DisplaySequence(self.val.ops.iter().map(|op| self.scoped(op)));
        write!(
            f,
            "\
            {indent}#[allow(non_camel_case_types)]
            {indent}pub enum Op {{\n\
            {variants}\
            {indent}}}\n\
        "
        )
    }
}

impl Display for DisplayOp<&'_ Op> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.val {
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
        }
    }
}

impl Display for DisplayOp<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let op = self.val;
        let ident = DisplayIdent::camel(op);
        let result_field = op.result_field();
        let value_field = op.value_field();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}{result_field},\n\
            {indent1}{value_field},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ BinaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let op = self.val;
        let ident = DisplayIdent::camel(op);
        let result_field = op.result_field();
        let lhs_field = op.lhs_field();
        let rhs_field = op.rhs_field();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}{result_field},\n\
            {indent1}{lhs_field},\n\
            {indent1}{rhs_field},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let op = self.val;
        let ident = DisplayIdent::camel(op);
        let lhs_field = op.lhs_field();
        let rhs_field = op.rhs_field();
        let offset_ty = FieldTy::BranchOffset;
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}offset: {offset_ty},\n\
            {indent1}{lhs_field},\n\
            {indent1}{rhs_field},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let op = self.val;
        let ident = DisplayIdent::camel(op);
        let result_field = op.result_field();
        let lhs_field = op.lhs_field();
        let rhs_field = op.rhs_field();
        let val_true_field = op.val_true_field();
        let val_false_field = op.val_false_field();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}{result_field},\n\
            {indent1}{lhs_field},\n\
            {indent1}{rhs_field},\n\
            {indent1}{val_true_field},\n\
            {indent1}{val_false_field},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let op = self.val;
        let ident = DisplayIdent::camel(op);
        let result_field = op.result_field();
        let ptr_field = op.ptr_field();
        let offset_field = op
            .offset_field()
            .map(|field| (indent1, field, ",\n"))
            .map(DisplayConcat)
            .display_maybe();
        let memory_field = op
            .memory_field()
            .map(|field| (indent1, field, ",\n"))
            .map(DisplayConcat)
            .display_maybe();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}{result_field},\n\
            {indent1}{ptr_field},\n\
            {offset_field}\
            {memory_field}\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let op = self.val;
        let ident = DisplayIdent::camel(op);
        let ptr_field = op.ptr_field();
        let offset_field = op
            .offset_field()
            .map(|field| (indent1, field, ",\n"))
            .map(DisplayConcat)
            .display_maybe();
        let value_field = op.value_field();
        let memory_field = op
            .memory_field()
            .map(|field| (indent1, field, ",\n"))
            .map(DisplayConcat)
            .display_maybe();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}{ptr_field},\n\
            {offset_field}\
            {indent1}{value_field},\n\
            {memory_field}\
            {indent0}}},\n\
            ",
        )
    }
}

impl<const N: usize> Display for DisplayOp<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let ident = DisplayIdent::camel(self.val);
        let fields = DisplaySequence(
            self.val
                .fields
                .into_iter()
                .map(move |field| (indent1, field, ",\n"))
                .map(DisplayConcat),
        );
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {fields}\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let op = self.val;
        let ident = DisplayIdent::camel(op);
        let result_field = op.result_field();
        let index_field = op.index_field();
        let table_field = op.table_field();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}{result_field},\n\
            {indent1}{index_field},\n\
            {indent1}{table_field},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayOp<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let op = self.val;
        let ident = DisplayIdent::camel(op);
        let index_field = op.index_field();
        let value_field = op.value_field();
        let table_field = op.table_field();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}{table_field},\n\
            {indent1}{index_field},\n\
            {indent1}{value_field},\n\
            {indent0}}},\n\
            ",
        )
    }
}
