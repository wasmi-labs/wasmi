use crate::build::{
    isa::Isa,
    op::{
        BinaryOp,
        BinaryOpKind,
        CmpBranchOp,
        CmpOpKind,
        CmpSelectOp,
        FieldTy,
        Input,
        LoadOp,
        Op,
        StoreOp,
        Ty,
        UnaryOp,
    },
    token::{CamelCase, Ident, SnakeCase},
    IntoMaybe as _,
    Maybe,
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
            Op::CmpBranch(op) => self.map(op).fmt(f),
            Op::CmpSelect(op) => self.map(op).fmt(f),
            Op::Load(op) => self.map(op).fmt(f),
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

impl Display for DisplayEnum<&'_ CmpBranchOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent0 = self.indent;
        let indent1 = indent0.inc();
        let cmp = self.val.cmp;
        let ident = CamelCase(cmp.ident());
        let input_ident = CamelCase(Ident::from(cmp.input_ty()));
        let result_ty = FieldTy::Stack;
        let lhs_ty = cmp.input_field(self.val.lhs);
        let rhs_ty = cmp.input_field(self.val.rhs);
        let result_suffix = CamelCase(Input::Stack);
        let lhs_suffix = SnakeCase(self.val.lhs);
        let rhs_suffix = SnakeCase(self.val.rhs);
        write!(
            f,
            "\
            {indent0}Branch{input_ident}{ident}_S{lhs_suffix}{rhs_suffix} {{\n\
            {indent1}offset: BranchOffset,\n\
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
            {indent0}Select{input_ident}{ident}_S{lhs_suffix}{rhs_suffix} {{\n\
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
        let kind = self.val.kind;
        let ident = DisplayIdent(self.val);
        let result_ty = FieldTy::Stack;
        let result_ident = kind.ident_prefix().map(CamelCase);
        let result_suffix = CamelCase(Input::Stack);
        let ptr_suffix = SnakeCase(self.val.ptr);
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

pub struct DisplayIdent<T>(pub T);

impl Display for DisplayIdent<&'_ LoadOp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.0.kind;
        let ident = CamelCase(kind.ident());
        let result_suffix = CamelCase(Input::Stack);
        let ptr_suffix = SnakeCase(self.0.ptr);
        let ident_prefix = self.0.kind.ident_prefix().map(CamelCase).into_maybe();
        let mem0_ident = self.0.mem0.then(|| "Mem0").unwrap_or_default();
        let offset16_ident = self.0.offset16.then(|| "Offset16").unwrap_or_default();
        write!(
            f,
            "{ident_prefix}{ident}{mem0_ident}{offset16_ident}_{result_suffix}{ptr_suffix}",
        )
    }
}

pub struct DisplayIndented<T> {
    val: T,
    indent: Indent,
}

impl<T> DisplayIndented<T> {
    pub fn new(val: T, indent: Indent) -> Self {
        Self { val, indent }
    }
}

impl<T> Display for DisplayIndented<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let val = &self.val;
        write!(f, "{indent}{val}")
    }
}
