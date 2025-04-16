use super::*;

pub struct DisplayOpCodeEnum<'a> {
    ctx: &'a Context,
    indent: DisplayIndent,
}

impl<'a> DisplayOpCodeEnum<'a> {
    pub fn new(ctx: &'a Context, indent: DisplayIndent) -> Self {
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

            "impl ::core::marker::Copy for OpCode {}"
            "impl ::core::clone::Clone for OpCode {"
            "    fn clone(&self) -> Self {"
            "        *self"
            "    }"
            "}"
            "impl ::core::cmp::PartialEq for OpCode {"
            "    fn eq(&self, other: &Self) -> bool {"
            "        (*self as ::core::primitive::u16) == (*other as ::core::primitive::u16)"
            "    }"
            "}"
            "impl ::core::cmp::Ord for OpCode {"
            "    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {"
            "        (*self as ::core::primitive::u16).cmp(&(*other as ::core::primitive::u16))"
            "    }"
            "}"
            "impl ::core::cmp::PartialOrd for OpCode {"
            "    fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::core::cmp::Ordering> {"
            "        ::core::option::Option::Some("
            "            <Self as ::core::cmp::Ord>::cmp(self, other)"
            "        )"
            "    }"
            "}"
            "impl ::core::cmp::Eq for OpCode {}"
            "impl crate::GetOpCode for OpCode {"
            "    fn op_code(&self) -> Self {"
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
