use super::{
    Context,
    DisplayFields,
    DisplayFieldsPattern,
    DisplayFileHeader,
    DisplayIndent,
    Op,
    Visibility,
};
use std::fmt::{self, Display};

pub struct DisplayOpEnum<'a> {
    ctx: &'a Context,
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnum<'a> {
    pub fn new(ctx: &'a Context, indent: DisplayIndent) -> Self {
        Self { ctx, indent }
    }
}

impl Display for DisplayOpEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let variants = DisplayOpEnumVariants::new(self.ctx.ops(), indent.inc());
        let impl_encode_params =
            DisplayOpEnumImplEncodeForVariants::new(self.ctx.ops(), indent.inc_by(3));
        let display_header = DisplayFileHeader;
        emit!(f, indent =>
            display_header
            "#[repr(u16)]"
            "pub enum Op {"
                variants
            "}"
            "impl ::core::marker::Copy for Op {}"
            "impl ::core::clone::Clone for Op {"
            "    fn clone(&self) -> Self {"
            "        *self"
            "    }"
            "}"
            "impl Op {"
            "    /// Encodes `self` without its [`OpCode`][crate::OpCode]."
            "    pub fn encode_params("
            "        &self,"
            "        encoder: &mut crate::CopyEncoder,"
            "    ) -> ::core::result::Result<::core::primitive::usize, crate::EncoderError> {"
            "        match *self {"
                        impl_encode_params
            "        }"
            "    }"
            "}"
        );
        Ok(())
    }
}

pub struct DisplayOpEnumVariants<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumVariants<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }
}

impl Display for DisplayOpEnumVariants<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        DisplayOpEnumVariant::new(first, 0, self.indent).fmt(f)?;
        for (index, op) in rest.iter().enumerate() {
            writeln!(f)?;
            DisplayOpEnumVariant::new(op, index + 1, self.indent).fmt(f)?;
        }
        Ok(())
    }
}

pub struct DisplayOpEnumVariant<'a> {
    op: &'a Op,
    index: usize,
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumVariant<'a> {
    fn new(op: &'a Op, index: usize, indent: DisplayIndent) -> Self {
        Self { op, index, indent }
    }
}

impl Display for DisplayOpEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let fields = DisplayFields::new(self.op.fields(), indent.inc(), Visibility::Default);
        let name = self.op.name();
        let index = self.index;
        if self.op.fields().is_empty() {
            return write!(f, "{indent}{name} = {index}_u16,");
        }
        write!(
            f,
            "\
            {indent}{name} {{\n\
            {fields}\n\
            {indent}}} = {index}_u16,\
            "
        )?;
        Ok(())
    }
}

pub struct DisplayOpEnumImplEncodeForVariants<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumImplEncodeForVariants<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }

    fn emit_op(&self, f: &mut fmt::Formatter, op: &Op) -> fmt::Result {
        let indent = self.indent;
        let fields = DisplayFieldsPattern::new(op.fields());
        let name = op.name();
        write!(
            f,
            "\
            {indent}Self::{name} {{ {fields} }} => {{\n\
            {indent}    encoder.encode(crate::op::{name} {{ {fields} }})\n\
            {indent}}}\
            "
        )?;
        Ok(())
    }
}

impl Display for DisplayOpEnumImplEncodeForVariants<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        self.emit_op(f, first)?;
        for op in rest {
            writeln!(f)?;
            self.emit_op(f, op)?;
        }
        Ok(())
    }
}
