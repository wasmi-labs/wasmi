use super::{Context, Field, FieldName, FieldTy, ImmediateTy, Instr};
use std::{
    fmt::{self, Display, Write as _},
    write,
    writeln,
};

pub fn generate_instrs(ctx: &Context) {
    let mut code = String::new();
    write!(code, "{}", ctx).unwrap();
    std::fs::write("src/instr/mod.rs", code).unwrap();
}

macro_rules! emit {
    (
        $f:expr, $indent:expr =>
        $( $line:expr )*
    ) => {{
        $(
            write!($f, "{}{}\n", $indent, $line)?;
        )*
    }};
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = DisplayIndent(0);
        emit!(f, indent =>
            "use super::*;"
            ""
        );
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
        let display_variants = DisplayOpCodeEnumVariants::new(self.ctx.instrs(), indent.inc());
        emit!(f, indent =>
            "#[repr(u16)]"
            "pub enum OpCode {"
                display_variants
            "}"

            "impl Copy for OpCode {}"
            "impl Clone for OpCode {"
            "    fn clone(&self) -> Self {"
            "        *self"
            "    }"
            "}"
        );
        Ok(())
    }
}

pub struct DisplayOpCodeEnumVariants<'a> {
    instrs: &'a [Instr],
    indent: DisplayIndent,
}

impl<'a> DisplayOpCodeEnumVariants<'a> {
    fn new(instrs: &'a [Instr], indent: DisplayIndent) -> Self {
        Self { instrs, indent }
    }
}

impl Display for DisplayOpCodeEnumVariants<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.instrs.split_first() else {
            return Ok(());
        };
        DisplayOpCodeEnumVariant::new(first, self.indent).fmt(f)?;
        for instr in rest {
            writeln!(f)?;
            DisplayOpCodeEnumVariant::new(instr, self.indent).fmt(f)?;
        }
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
        write!(f, "{indent}{name},")?;
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
        let variants = DisplayOpEnumVariants::new(self.ctx.instrs(), indent.inc());
        emit!(f, indent =>
            "pub enum Op {"
                variants
            "}"
        );
        Ok(())
    }
}

pub struct DisplayOpEnumVariants<'a> {
    instrs: &'a [Instr],
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumVariants<'a> {
    fn new(instrs: &'a [Instr], indent: DisplayIndent) -> Self {
        Self { instrs, indent }
    }
}

impl Display for DisplayOpEnumVariants<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.instrs.split_first() else {
            return Ok(());
        };
        DisplayOpEnumVariant::new(first, self.indent).fmt(f)?;
        for instr in rest {
            writeln!(f)?;
            DisplayOpEnumVariant::new(instr, self.indent).fmt(f)?;
        }
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
        let fields = DisplayOpEnumVariantFields::new(self.instr.fields(), indent.inc());
        let name = self.instr.name();
        if self.instr.fields().is_empty() {
            return writeln!(f, "{indent}{name},");
        }
        write!(
            f,
            "\
            {indent}{name} {{\n\
            {fields}\n\
            {indent}}},\
            "
        )?;
        Ok(())
    }
}

pub struct DisplayOpEnumVariantFields<'a> {
    fields: &'a [Field],
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumVariantFields<'a> {
    fn new(fields: &'a [Field], indent: DisplayIndent) -> Self {
        Self { fields, indent }
    }
}

impl Display for DisplayOpEnumVariantFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.fields.split_first() else {
            return Ok(());
        };
        DisplayOpEnumVariantField::new(first, self.indent).fmt(f)?;
        for field in rest {
            writeln!(f)?;
            DisplayOpEnumVariantField::new(field, self.indent).fmt(f)?;
        }
        Ok(())
    }
}

pub struct DisplayOpEnumVariantField<'a> {
    field: &'a Field,
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumVariantField<'a> {
    fn new(field: &'a Field, indent: DisplayIndent) -> Self {
        Self { field, indent }
    }
}

impl Display for DisplayOpEnumVariantField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let field_name = self.field.name;
        let field_ty = self.field.ty;
        write!(f, "{indent}{field_name}: {field_ty},")?;
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

impl Display for ImmediateTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::Usize => "usize",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::Isize => "isize",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Global => "Global",
            Self::Func => "Func",
            Self::WasmFunc => "WasmFunc",
            Self::Memory => "Memory",
            Self::Table => "Table",
            Self::Data => "Data",
            Self::Elem => "Elem",
            Self::Address => "Address",
            Self::Offset => "Offset",
            Self::BranchOffset => "BranchOffset",
        };
        write!(f, "{str}")
    }
}
