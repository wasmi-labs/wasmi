use super::{DisplayFileHeader, DisplayIndent, Op};
use crate::instrs::{Context, FieldName};
use core::{fmt, fmt::Display};

pub struct DisplayOpResult<'a> {
    ctx: &'a Context,
    indent: DisplayIndent,
}

impl<'a> DisplayOpResult<'a> {
    pub fn new(ctx: &'a Context, indent: DisplayIndent) -> Self {
        Self { ctx, indent }
    }
}

impl Display for DisplayOpResult<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent;
        let display_impl = DisplayOpResultImpl::new(self.ctx.ops(), indent.inc_by(3));
        let display_impls = DisplayOpResultImpls::new(self.ctx.ops(), indent);
        let display_header = DisplayFileHeader;
        emit!(f, indent =>
            display_header
            "impl crate::OperatorResult for crate::Op {"
            "    fn operator_result(&self) -> ::core::option::Option<crate::OpResult> {"
            "        match self {"
                         display_impl
            "            _ => ::core::option::Option::None,"
            "        }"
            "    }"
            "}"
            display_impls
        );
        Ok(())
    }
}

pub struct DisplayOpResultImpl<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpResultImpl<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }

    fn emit(&self, f: &mut fmt::Formatter, op: &Op) -> fmt::Result {
        let Some(field) = op
            .fields()
            .iter()
            .find(|field| matches!(field.name, FieldName::Result))
        else {
            return Ok(());
        };
        let indent = self.indent;
        let name = op.name();
        let ty = field.ty;
        write!(
            f,
            "\
            {indent}Self::{name} {{ result, .. }} => ::core::option::Option::Some(\n\
            {indent}    <crate::OpResult as ::core::convert::From<{ty}>>::from(*result)\n\
            {indent}),\n\
            "
        )
    }
}

impl Display for DisplayOpResultImpl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        self.emit(f, first)?;
        for op in rest {
            self.emit(f, op)?;
        }
        Ok(())
    }
}

pub struct DisplayOpResultImpls<'a> {
    ops: &'a [Op],
    indent: DisplayIndent,
}

impl<'a> DisplayOpResultImpls<'a> {
    fn new(ops: &'a [Op], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }

    fn emit(&self, f: &mut fmt::Formatter, op: &Op) -> fmt::Result {
        let indent = self.indent;
        let name = op.name();
        write!(
            f,
            "\
            {indent}impl crate::OperatorResult for crate::op::{name} {{\
            "
        )?;
        if let Some(field) = op
            .fields()
            .iter()
            .find(|field| matches!(field.name, FieldName::Result))
        {
            let ty = field.ty;
            write!(f, "\n\
                {indent}    fn operator_result(&self) -> ::core::option::Option<crate::OpResult> {{\n\
                {indent}        ::core::option::Option::Some(\n\
                {indent}            <crate::OpResult as ::core::convert::From<{ty}>>::from(self.result)\n\
                {indent}        )\n\
                {indent}    }}\n\
                {indent}\
                "
            )?;
        }
        write!(f, "}}\n")?;
        Ok(())
    }
}

impl Display for DisplayOpResultImpls<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        self.emit(f, first)?;
        for op in rest {
            self.emit(f, op)?;
        }
        Ok(())
    }
}
