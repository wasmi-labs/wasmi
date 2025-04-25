use super::super::{DisplayFileHeader, DisplayIndent, ImmediateTy};
use crate::instrs::OpClass;
use core::{fmt, fmt::Display};

pub struct DisplayCmpBranchCommutativeOperatorImpls<'a> {
    ops: &'a [OpClass],
    indent: DisplayIndent,
}

impl<'a> DisplayCmpBranchCommutativeOperatorImpls<'a> {
    pub fn new(ops: &'a [OpClass], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }

    fn emit(&self, f: &mut fmt::Formatter, op: &OpClass) -> fmt::Result {
        let indent = self.indent;
        let name = &*op.name;
        let op_ri = op.op_ri();
        let op_rs = op.op_rs();
        let op_si = op.op_si();
        let op_ss = op.op_ss();
        let imm = ImmediateTy::from(op.ty);
        write!(
            f,
            "\
            {indent}pub enum {name} {{}}\n\
            {indent}impl crate::CmpBranchCommutativeOperator for {name} {{\n\
            {indent}    const NAME: &'static ::core::primitive::str = \"{name}\";\n\
            {indent}    type Imm = {imm};\n\
            {indent}    type OpRi = crate::op::{op_ri};\n\
            {indent}    type OpRs = crate::op::{op_rs};\n\
            {indent}    type OpSi = crate::op::{op_si};\n\
            {indent}    type OpSs = crate::op::{op_ss};\n\
            {indent}    fn make_ri(lhs: crate::Reg, rhs: Self::Imm, offset: crate::BranchOffset) -> Self::OpRi {{\n\
            {indent}        Self::OpRi {{ lhs, rhs, offset }}\n\
            {indent}    }}\n\
            {indent}    fn make_rs(lhs: crate::Reg, rhs: crate::Stack, offset: crate::BranchOffset) -> Self::OpRs {{\n\
            {indent}        Self::OpRs {{ lhs, rhs, offset }}\n\
            {indent}    }}\n\
            {indent}    fn make_si(lhs: crate::Stack, rhs: Self::Imm, offset: crate::BranchOffset) -> Self::OpSi {{\n\
            {indent}        Self::OpSi {{ lhs, rhs, offset }}\n\
            {indent}    }}\n\
            {indent}    fn make_ss(lhs: crate::Stack, rhs: crate::Stack, offset: crate::BranchOffset) -> Self::OpSs {{\n\
            {indent}        Self::OpSs {{ lhs, rhs, offset }}\n\
            {indent}    }}\n\
            {indent}}}\
            "
        )
    }
}

impl Display for DisplayCmpBranchCommutativeOperatorImpls<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_header = DisplayFileHeader;
        emit!(f, self.indent =>
            display_header
        );
        let Some((first, rest)) = self.ops.split_first() else {
            return Ok(());
        };
        self.emit(f, first)?;
        for op in rest {
            writeln!(f)?;
            self.emit(f, op)?;
        }
        writeln!(f)?;
        Ok(())
    }
}
