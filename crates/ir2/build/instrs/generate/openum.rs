use super::*;

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
        let variants = DisplayOpEnumVariants::new(self.ctx.instrs(), indent.inc());
        let impl_encode_for_variants =
            DisplayOpEnumImplEncodeForVariants::new(self.ctx.instrs(), indent.inc().inc().inc());
        emit!(f, indent =>
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
            "    /// Encodes [`Op`] allowing customization of its [`OpCode`] encoding."
            "    ///"
            "    /// This is useful to allow both direct and indirect dispatch techniques."
            "    pub fn encode_as<T: Copy>("
            "        &self,"
            "        encoder: &mut crate::CopyEncoder,"
            "        f: impl ::core::ops::Fn(crate::OpCode) -> T"
            "    ) -> ::core::result::Result<(), crate::EncoderError> {"
            "        match *self {"
                        impl_encode_for_variants
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
        DisplayOpEnumVariant::new(first, self.indent).fmt(f)?;
        for instr in rest {
            writeln!(f)?;
            DisplayOpEnumVariant::new(instr, self.indent).fmt(f)?;
        }
        Ok(())
    }
}

pub struct DisplayOpEnumVariant<'a> {
    op: &'a Op,
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumVariant<'a> {
    fn new(op: &'a Op, indent: DisplayIndent) -> Self {
        Self { op, indent }
    }
}

impl Display for DisplayOpEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let fields = DisplayFields::new(self.op.fields(), indent.inc(), Visibility::Default);
        let name = self.op.name();
        if self.op.fields().is_empty() {
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

pub struct DisplayOpEnumImplEncodeForVariants<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumImplEncodeForVariants<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }
}

impl Display for DisplayOpEnumImplEncodeForVariants<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        DisplayOpEnumImplEncodeForVariant::new(first, self.indent).fmt(f)?;
        for instr in rest {
            writeln!(f)?;
            DisplayOpEnumImplEncodeForVariant::new(instr, self.indent).fmt(f)?;
        }
        Ok(())
    }
}

pub struct DisplayOpEnumImplEncodeForVariant<'a> {
    op: &'a Op,
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumImplEncodeForVariant<'a> {
    fn new(op: &'a Op, indent: DisplayIndent) -> Self {
        Self { op, indent }
    }
}

impl Display for DisplayOpEnumImplEncodeForVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let fields = DisplayFieldsPattern::new(self.op.fields());
        let name = self.op.name();
        if self.op.fields().is_empty() {
            return writeln!(
                f,
                "{indent}Self::{name} => {{ encoder.encode(f(crate::OpCode::{name})) }} "
            );
        }
        write!(
            f,
            "\
            {indent}Self::{name} {{ {fields} }} => {{\n\
            {indent}    encoder.encode(f(crate::OpCode::{name}))?;\n\
            {indent}    encoder.encode(crate::op::{name} {{ {fields} }} )\n\
            {indent}}}\
            "
        )?;
        Ok(())
    }
}

pub struct DisplayFieldsPattern<'a> {
    fields: &'a [Field],
}

impl<'a> DisplayFieldsPattern<'a> {
    fn new(fields: &'a [Field]) -> Self {
        Self { fields }
    }
}

impl Display for DisplayFieldsPattern<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.fields.split_first() else {
            return Ok(());
        };
        write!(f, "{}", first.name)?;
        for field in rest {
            write!(f, ", {}", field.name)?;
        }
        Ok(())
    }
}
