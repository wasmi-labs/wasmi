mod utils;

use self::utils::{DisplayConcat, DisplaySequence, IntoDisplayMaybe as _};
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
    token::{CamelCase, Ident, SnakeCase},
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
        let result_suffix = CamelCase(Input::Stack);
        let value_suffix = SnakeCase(Input::Stack);
        let result_field = FieldTy::Stack;
        let value_field = FieldTy::Stack;
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        write!(
            f,
            "\
            {indent0}{result_ident}{ident}_{result_suffix}{value_suffix} {{\n\
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
        let result_suffix = CamelCase(Input::Stack);
        let value_suffix = SnakeCase(Input::Stack);
        let result_field = FieldTy::Stack;
        let value_field = FieldTy::Stack;
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        write!(
            f,
            "\
            {indent0}{result_ident}{ident}{input_ident}_{result_suffix}{value_suffix} {{\n\
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
        let ident_prefix = CamelCase(kind.ident_prefix());
        let result_ty = FieldTy::Stack;
        let lhs_ty = kind.lhs_field(self.val.lhs);
        let rhs_ty = kind.rhs_field(self.val.rhs);
        let result_suffix = CamelCase(Input::Stack);
        let lhs_suffix = SnakeCase(self.val.lhs);
        let rhs_suffix = SnakeCase(self.val.rhs);
        write!(
            f,
            "\
            {indent0}{ident_prefix}{ident}_{result_suffix}{lhs_suffix}{rhs_suffix} {{\n\
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
        let branch = CamelCase(Ident::Branch);
        let ident = CamelCase(cmp.ident());
        let input_ident = CamelCase(Ident::from(cmp.input_ty()));
        let lhs_ty = cmp.input_field(self.val.lhs);
        let rhs_ty = cmp.input_field(self.val.rhs);
        let offset_ty = FieldTy::BranchOffset;
        let result_suffix = CamelCase(Input::Stack);
        let lhs_suffix = SnakeCase(self.val.lhs);
        let rhs_suffix = SnakeCase(self.val.rhs);
        write!(
            f,
            "\
            {indent0}{branch}{input_ident}{ident}_{result_suffix}{lhs_suffix}{rhs_suffix} {{\n\
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
        let select = CamelCase(Ident::Select);
        let ident = CamelCase(cmp.ident());
        let input_ident = CamelCase(Ident::from(cmp.input_ty()));
        let result_ty = FieldTy::Stack;
        let lhs_ty = cmp.input_field(self.val.lhs);
        let rhs_ty = cmp.input_field(self.val.rhs);
        let result_suffix = CamelCase(Input::Stack);
        let lhs_suffix = SnakeCase(self.val.lhs);
        let rhs_suffix = SnakeCase(self.val.rhs);
        let val_true = FieldTy::Stack;
        let val_false = FieldTy::Stack;
        write!(
            f,
            "\
            {indent0}{select}{input_ident}{ident}_{result_suffix}{lhs_suffix}{rhs_suffix} {{\n\
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
        let ident = DisplayIdent(self.val);
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
        let mem_ty = match self.val.mem0 {
            false => Some(FieldTy::Memory),
            true => None,
        };
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}result: {result_ty},\n\
            {indent1}ptr: {ptr_ty},\n\
            ",
        )?;
        if let Some(offset) = offset_ty {
            write!(
                f,
                "\
                    {indent1}offset: {offset},\n\
                "
            )?;
        }
        if let Some(mem) = mem_ty {
            write!(
                f,
                "\
                    {indent1}mem: {mem},\n\
                "
            )?;
        }
        write!(
            f,
            "\
            {indent0}}},\n\
            ",
        )?;
        Ok(())
    }
}

impl Display for DisplayEnum<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let ident = DisplayIdent(self.val);
        let ptr_ty = self.val.kind.ptr_ty(self.val.ptr);
        let value_ty = self.val.kind.value_ty(self.val.value);
        let offset_field = self
            .val
            .kind
            .offset_ty(self.val.ptr, self.val.offset16)
            .map(|offset| Field::new(Ident::Offset, offset))
            .map(|field| (indent1, field))
            .map(DisplayConcat::from)
            .display_maybe();
        let mem_field = self
            .val
            .mem0
            .not()
            .then(|| Field::new(Ident::Memory, FieldTy::Memory))
            .map(|field| (indent1, field))
            .map(DisplayConcat::from)
            .display_maybe();
        write!(
            f,
            "\
            {indent0}{ident} {{\n\
            {indent1}ptr: {ptr_ty},\n\
            {offset_field}\n\
            {indent1}value: {value_ty},\n\
            {mem_field}\n\
            {indent0}}},\n\
            ",
        )
    }
}

impl<const N: usize> Display for DisplayEnum<&'_ GenericOp<N>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let ident = CamelCase(self.val.ident);
        let fields = DisplaySequence(
            self.val
                .fields
                .into_iter()
                .map(move |field| (indent1, field, "\n"))
                .map(DisplayConcat::from),
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
        let ident = CamelCase(Ident::TableGet);
        let result_ty = FieldTy::Stack;
        let index_ty = match self.val.index {
            Input::Stack => FieldTy::Stack,
            Input::Immediate => FieldTy::U32,
        };
        let table_ty = FieldTy::Table;
        let result_suffix = CamelCase(Input::Stack);
        let index_suffix = SnakeCase(self.val.index);
        write!(
            f,
            "\
            {indent0}{ident}_{result_suffix}{index_suffix} {{\n\
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
        let ident = CamelCase(Ident::TableSet);
        let index_ty = match self.val.index {
            Input::Stack => FieldTy::Stack,
            Input::Immediate => FieldTy::U32,
        };
        let value_ty = match self.val.value {
            Input::Stack => FieldTy::Stack,
            Input::Immediate => FieldTy::U64,
        };
        let table_ty = FieldTy::Table;
        let index_suffix = CamelCase(self.val.index);
        let value_suffix = SnakeCase(self.val.value);
        write!(
            f,
            "\
            {indent0}{ident}_{index_suffix}{value_suffix} {{\n\
            {indent1}table: {table_ty},\n\
            {indent1}index: {index_ty},\n\
            {indent1}value: {value_ty},\n\
            {indent0}}},\n\
            ",
        )
    }
}

pub struct DisplayIdent<T>(pub T);

impl Display for DisplayIdent<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.0.kind;
        let ident = CamelCase(kind.ident());
        let result_suffix = CamelCase(Input::Stack);
        let ptr_suffix = SnakeCase(self.0.ptr);
        let ident_prefix = self.0.kind.ident_prefix().map(CamelCase).display_maybe();
        let mem0_ident = self.0.mem0.then_some("Mem0").display_maybe();
        let offset16_ident = self.0.offset16.then_some("Offset16").display_maybe();
        write!(
            f,
            "{ident_prefix}{ident}{mem0_ident}{offset16_ident}_{result_suffix}{ptr_suffix}",
        )
    }
}

impl Display for DisplayIdent<&'_ StoreOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.0.kind;
        let ident = CamelCase(kind.ident());
        let ptr_suffix = CamelCase(self.0.ptr);
        let value_suffix = SnakeCase(self.0.value);
        let ident_prefix = self.0.kind.ident_prefix().map(CamelCase).display_maybe();
        let mem0_ident = self.0.mem0.then_some("Mem0").display_maybe();
        let offset16_ident = self.0.offset16.then_some("Offset16").display_maybe();
        write!(
            f,
            "{ident_prefix}{ident}{mem0_ident}{offset16_ident}_{ptr_suffix}{value_suffix}",
        )
    }
}
