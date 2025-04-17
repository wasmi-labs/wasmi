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
        emit!(f, indent =>
            "pub enum Op {"
                variants
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
