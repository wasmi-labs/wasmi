mod ident;
mod utils;

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
        Field,
        FieldTy,
        GenericOp,
        Input,
        LoadOp,
        Op,
        StoreOp,
        TableGetOp,
        TableSetOp,
        UnaryOp,
    },
    token::Ident,
};
use core::{
    fmt::{self, Display},
    ops::Not,
};

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

impl Display for DisplayEnum<&'_ Op> {
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

impl Display for DisplayEnum<&'_ UnaryOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ident = DisplayIdent::camel(self.val);
        let result_field = FieldTy::Stack;
        let value_field = FieldTy::Stack;
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
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
        let ident = DisplayIdent::camel(self.val);
        let kind = self.val.kind;
        let result_ty = FieldTy::Stack;
        let lhs_ty = kind.lhs_field(self.val.lhs);
        let rhs_ty = kind.rhs_field(self.val.rhs);
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}result: {result_ty},\n\
            {indent1}lhs: {lhs_ty},\n\
            {indent1}rhs: {rhs_ty},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayEnum<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let cmp = self.val.cmp;
        let ident = DisplayIdent::camel(self.val);
        let lhs_ty = cmp.input_field(self.val.lhs);
        let rhs_ty = cmp.input_field(self.val.rhs);
        let offset_ty = FieldTy::BranchOffset;
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}offset: {offset_ty},\n\
            {indent1}lhs: {lhs_ty},\n\
            {indent1}rhs: {rhs_ty},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayEnum<&'_ CmpSelectOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let cmp = self.val.cmp;
        let ident = DisplayIdent::camel(self.val);
        let result_ty = FieldTy::Stack;
        let lhs_ty = cmp.input_field(self.val.lhs);
        let rhs_ty = cmp.input_field(self.val.rhs);
        let val_true = FieldTy::Stack;
        let val_false = FieldTy::Stack;
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}result: {result_ty},\n\
            {indent1}lhs: {lhs_ty},\n\
            {indent1}rhs: {rhs_ty},\n\
            {indent1}val_true: {val_true},\n\
            {indent1}val_false: {val_false},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayEnum<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let ident = DisplayIdent::camel(self.val);
        let result_ty = FieldTy::Stack;
        let (ptr_ty, offset_ty) = match self.val.ptr {
            Input::Stack => {
                let ptr = FieldTy::Stack;
                let offset = match self.val.offset16 {
                    true => FieldTy::Offset16,
                    false => FieldTy::U64,
                };
                (ptr, Some(offset))
            }
            Input::Immediate => (FieldTy::Address, None),
        };
        let offset_field = offset_ty
            .map(|ty| Field::new(Ident::Offset, ty))
            .map(|field| (indent1, field, ",\n"))
            .map(DisplayConcat)
            .display_maybe();
        let memory_field = self
            .val
            .mem0
            .then_some(FieldTy::Memory)
            .map(|ty| Field::new(Ident::Memory, ty))
            .map(|field| (indent1, field, ",\n"))
            .map(DisplayConcat)
            .display_maybe();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}result: {result_ty},\n\
            {indent1}ptr: {ptr_ty},\n\
            {offset_field}\
            {memory_field}\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayEnum<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let ident = DisplayIdent::camel(self.val);
        let ptr_ty = self.val.kind.ptr_ty(self.val.ptr);
        let value_ty = self.val.kind.value_ty(self.val.value);
        let offset_field = self
            .val
            .kind
            .offset_ty(self.val.ptr, self.val.offset16)
            .map(|offset| Field::new(Ident::Offset, offset))
            .map(|field| (indent1, field, ",\n"))
            .map(DisplayConcat)
            .display_maybe();
        let mem_field = self
            .val
            .mem0
            .not()
            .then(|| Field::new(Ident::Memory, FieldTy::Memory))
            .map(|field| (indent1, field, ",\n"))
            .map(DisplayConcat)
            .display_maybe();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}ptr: {ptr_ty},\n\
            {offset_field}\
            {indent1}value: {value_ty},\n\
            {mem_field}\
            {indent0}}},\n\
            ",
        )
    }
}

impl<const N: usize> Display for DisplayEnum<&'_ GenericOp<N>> {
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

impl Display for DisplayEnum<&'_ TableGetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let ident = DisplayIdent::camel(self.val);
        let result_ty = FieldTy::Stack;
        let index_ty = match self.val.index {
            Input::Stack => FieldTy::Stack,
            Input::Immediate => FieldTy::U32,
        };
        let table_ty = FieldTy::Table;
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}result: {result_ty},\n\
            {indent1}index: {index_ty},\n\
            {indent1}table: {table_ty},\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl Display for DisplayEnum<&'_ TableSetOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let ident = DisplayIdent::camel(self.val);
        let index_ty = match self.val.index {
            Input::Stack => FieldTy::Stack,
            Input::Immediate => FieldTy::U32,
        };
        let value_ty = match self.val.value {
            Input::Stack => FieldTy::Stack,
            Input::Immediate => FieldTy::U64,
        };
        let table_ty = FieldTy::Table;
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}table: {table_ty},\n\
            {indent1}index: {index_ty},\n\
            {indent1}value: {value_ty},\n\
            {indent0}}},\n\
            ",
        )
    }
}
