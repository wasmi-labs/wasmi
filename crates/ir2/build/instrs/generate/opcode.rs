use super::{Context, DisplayFileHeader, DisplayIndent, Op};
use std::fmt::{self, Display};

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
        let display_variants = DisplayOpCodeEnumVariants::new(self.ctx.ops(), indent.inc());
        let impl_operator_code =
            DisplayOpEnumImplOperatorCode::new(self.ctx.ops(), indent.inc_by(3));
        let op_mod_impl_operator_code = DisplayOpModImplOperatorCode::new(self.ctx.ops(), indent);
        let display_header = DisplayFileHeader;
        emit!(f, indent =>
            display_header

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
            "impl crate::OperatorCode for OpCode {"
            "    fn op_code(&self) -> Self {"
            "        *self"
            "    }"
            "}"
            "impl crate::OperatorCode for crate::Op {"
            "    fn op_code(&self) -> crate::OpCode {"
            "        match self {"
                        impl_operator_code
            "        }"
            "    }"
            "}"
            op_mod_impl_operator_code
        );
        Ok(())
    }
}

pub struct DisplayOpCodeEnumVariants<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpCodeEnumVariants<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }

    fn emit(
        &self,
        f: &mut fmt::Formatter,
        op: &Op,
        index: usize,
        indent: DisplayIndent,
    ) -> fmt::Result {
        let name = op.name();
        write!(f, "{indent}{name} = {index}_u16,")
    }
}

impl Display for DisplayOpCodeEnumVariants<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        self.emit(f, first, 0, self.indent)?;
        for (index, op) in rest.iter().enumerate() {
            writeln!(f)?;
            self.emit(f, op, index + 1, self.indent)?;
        }
        Ok(())
    }
}

pub struct DisplayOpEnumImplOperatorCode<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumImplOperatorCode<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }
}

impl Display for DisplayOpEnumImplOperatorCode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        DisplayOpEnumImplOperatorCodeForVariant::new(first, self.indent).fmt(f)?;
        for instr in rest {
            writeln!(f)?;
            DisplayOpEnumImplOperatorCodeForVariant::new(instr, self.indent).fmt(f)?;
        }
        Ok(())
    }
}

pub struct DisplayOpEnumImplOperatorCodeForVariant<'a> {
    op: &'a Op,
    indent: DisplayIndent,
}

impl<'a> DisplayOpEnumImplOperatorCodeForVariant<'a> {
    fn new(op: &'a Op, indent: DisplayIndent) -> Self {
        Self { op, indent }
    }
}

impl Display for DisplayOpEnumImplOperatorCodeForVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let name = self.op.name();
        write!(f, "{indent}Self::{name} {{ .. }} => crate::OpCode::{name},")?;
        Ok(())
    }
}

pub struct DisplayOpModImplOperatorCode<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpModImplOperatorCode<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }

    fn emit_op(&self, f: &mut fmt::Formatter, op: &Op) -> fmt::Result {
        let indent = self.indent;
        let name = op.name();
        write!(
            f,
            "\
            {indent}impl crate::OperatorCode for crate::op::{name} {{\n\
            {indent}    fn op_code(&self) -> crate::OpCode {{\n\
            {indent}        crate::OpCode::{name}\n\
            {indent}    }}\n\
            {indent}}}\
            "
        )
    }
}

impl Display for DisplayOpModImplOperatorCode<'_> {
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
