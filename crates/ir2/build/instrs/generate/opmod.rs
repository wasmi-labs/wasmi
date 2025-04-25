use super::{Context, DisplayFields, DisplayFileHeader, DisplayIndent, Field, Op, Visibility};
use std::fmt::{self, Display};

pub struct DisplayOpMod<'a> {
    ctx: &'a Context,
    indent: DisplayIndent,
}

impl<'a> DisplayOpMod<'a> {
    pub fn new(ctx: &'a Context, indent: DisplayIndent) -> Self {
        Self { ctx, indent }
    }
}

impl Display for DisplayOpMod<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let display_instrs = DisplayOpModInstrs::new(self.ctx.ops(), indent);
        let display_header = DisplayFileHeader;
        emit!(f, indent =>
            display_header
            "#![allow(unused_variables)]"
            ""
            display_instrs
        );
        Ok(())
    }
}

pub struct DisplayOpModInstrs<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpModInstrs<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }
}

impl Display for DisplayOpModInstrs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        DisplayOpModInstr::new(first, self.indent).fmt(f)?;
        for instr in rest {
            writeln!(f)?;
            DisplayOpModInstr::new(instr, self.indent).fmt(f)?;
        }
        Ok(())
    }
}

pub struct DisplayOpModInstr<'a> {
    op: &'a Op,
    indent: DisplayIndent,
}

impl<'a> DisplayOpModInstr<'a> {
    fn new(op: &'a Op, indent: DisplayIndent) -> Self {
        Self { op, indent }
    }
}

impl Display for DisplayOpModInstr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let fields = DisplayFields::new(self.op.fields(), indent.inc(), Visibility::Pub);
        let from_impl = DisplayOpModFromImpl::new(self.op, indent);
        let name = self.op.name();
        match self.op.fields().is_empty() {
            true => writeln!(
                f,
                "\
                {indent}#[repr(C, packed)]\n\
                {indent}pub struct {name};\
                "
            )?,
            false => writeln!(
                f,
                "\
                {indent}#[repr(C, packed)]\n\
                {indent}pub struct {name} {{\n\
                {fields}\n\
                {indent}}}\
                "
            )?,
        }
        write!(
            f,
            "\
            {indent}impl ::core::marker::Copy for {name} {{}}\n\
            {indent}impl ::core::clone::Clone for {name} {{\n\
            {indent}    fn clone(&self) -> Self {{\n\
            {indent}        *self\n\
            {indent}    }}\n\
            {indent}}}\n\
            {from_impl}\
            "
        )?;
        Ok(())
    }
}

pub struct DisplayOpModFromImpl<'a> {
    op: &'a Op,
    indent: DisplayIndent,
}

impl<'a> DisplayOpModFromImpl<'a> {
    fn new(op: &'a Op, indent: DisplayIndent) -> Self {
        Self { op, indent }
    }
}

impl Display for DisplayOpModFromImpl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let fields = DisplayOpModFromImplFields::new(self.op.fields(), indent.inc_by(3));
        let name = self.op.name();
        write!(
            f,
            "\
            {indent}impl ::core::convert::From<{name}> for crate::Op {{\n\
            {indent}    fn from(op: {name}) -> Self {{\n\
            {indent}        Self::{name} {{\n\
                                {fields}\n\
            {indent}        }}\n\
            {indent}    }}\n\
            {indent}}}\
            "
        )?;
        Ok(())
    }
}

pub struct DisplayOpModFromImplFields<'a> {
    fields: &'a [Field],
    indent: DisplayIndent,
}

impl<'a> DisplayOpModFromImplFields<'a> {
    fn new(fields: &'a [Field], indent: DisplayIndent) -> Self {
        Self { fields, indent }
    }
}

impl Display for DisplayOpModFromImplFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.fields.split_first() else {
            return Ok(());
        };
        DisplayOpModFromImplField::new(first, self.indent).fmt(f)?;
        for instr in rest {
            writeln!(f)?;
            DisplayOpModFromImplField::new(instr, self.indent).fmt(f)?;
        }
        Ok(())
    }
}

pub struct DisplayOpModFromImplField<'a> {
    field: &'a Field,
    indent: DisplayIndent,
}

impl<'a> DisplayOpModFromImplField<'a> {
    fn new(field: &'a Field, indent: DisplayIndent) -> Self {
        Self { field, indent }
    }
}

impl Display for DisplayOpModFromImplField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let name = self.field.name;
        write!(f, "{indent}{name}: op.{name},")?;
        Ok(())
    }
}
