use super::{Context, FieldName, FieldTy, Instr};
use std::fmt::{self, Display};

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = DisplayIndent(0);
        DisplayOpEnum::new(self, indent).fmt(f)?;
        DisplayOpCodeEnum::new(self, indent).fmt(f)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DisplayIndent(usize);

impl DisplayIndent {
    pub fn inc(self) -> Self {
        Self(self.0 + 1)
    }
}

impl Display for DisplayIndent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, "    ")?;
        }
        Ok(())
    }
}

pub struct DisplayOpCodeEnum<'a> {
    ctx: &'a Context,
    indent: DisplayIndent,
}

impl<'a> DisplayOpCodeEnum<'a> {
    fn new(ctx: &'a Context, indent: DisplayIndent) -> Self {
        Self { ctx, indent }
    }
}

impl Display for DisplayOpCodeEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        std::writeln!(f, "{indent}pub enum OpCode {{")?;
        for instr in self.ctx.instrs() {
            DisplayOpCodeEnumVariant::new(instr, indent.inc()).fmt(f)?;
        }
        std::writeln!(f, "{indent}}}")?;
        Ok(())
    }
}

pub struct DisplayOpCodeEnumVariant<'a> {
    instr: &'a Instr,
    indent: DisplayIndent,
}

impl<'a> DisplayOpCodeEnumVariant<'a> {
    fn new(instr: &'a Instr, indent: DisplayIndent) -> Self {
        Self { instr, indent }
    }
}

impl Display for DisplayOpCodeEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let name = self.instr.name();
        writeln!(f, "{indent}{name},")?;
        Ok(())
    }
}

pub struct DisplayOpEnum<'a> {
    ctx: &'a Context,
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnum<'a> {
    fn new(ctx: &'a Context, indent: DisplayIndent) -> Self {
        Self { ctx, indent }
    }
}

impl Display for DisplayOpEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        std::writeln!(f, "{indent}use super::*;")?;
        std::writeln!(f)?;
        std::writeln!(f, "{indent}pub enum Op {{")?;
        for instr in self.ctx.instrs() {
            DisplayOpEnumVariant::new(instr, indent.inc()).fmt(f)?;
        }
        std::writeln!(f, "{indent}}}")?;
        Ok(())
    }
}

pub struct DisplayOpEnumVariant<'a> {
    instr: &'a Instr,
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumVariant<'a> {
    fn new(instr: &'a Instr, indent: DisplayIndent) -> Self {
        Self { instr, indent }
    }
}

impl Display for DisplayOpEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let field_indent = indent.inc();
        let name = self.instr.name();
        if self.instr.fields().is_empty() {
            return std::writeln!(f, "{indent}{name},");
        }
        std::writeln!(f, "{indent}{name} {{")?;
        for field in self.instr.fields() {
            let field_name = field.name;
            let field_ty = field.ty;
            std::writeln!(f, "{field_indent}{field_name}: {field_ty},")?;
        }
        std::writeln!(f, "{indent}}},")?;
        Ok(())
    }
}

impl Display for FieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Result => "result",
            Self::Condition => "condition",
            Self::Value => "value",
            Self::Lhs => "lhs",
            Self::Rhs => "rhs",
            Self::Ptr => "ptr",
            Self::Offset => "offset",
            Self::Input => "input",
            Self::Index => "index",
            Self::DstIndex => "dst_index",
            Self::SrcIndex => "src_index",
            Self::DstTable => "dst_table",
            Self::SrcTable => "src_table",
            Self::DstMemory => "dst_memory",
            Self::SrcMemory => "src_memory",
            Self::Len => "len",
            Self::LenTargets => "len_targets",
            Self::LenValues => "len_values",
            Self::LenParams => "len_params",
            Self::LenResults => "len_results",
            Self::Delta => "delta",
            Self::Address => "address",
            Self::Memory => "memory",
            Self::Table => "table",
            Self::Global => "global",
            Self::Func => "func",
            Self::Data => "data",
            Self::Elem => "elem",
        };
        write!(f, "{str}")
    }
}

impl Display for FieldTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Reg => "Reg",
            Self::Stack => "Stack",
            Self::Immediate(imm) => return imm.fmt(f),
        };
        write!(f, "{str}")
    }
}
