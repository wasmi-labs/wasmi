use super::super::{DisplayFileHeader, DisplayIndent};
use crate::instrs::OpClass;
use core::{fmt, fmt::Display};

pub struct DisplayLoadOperatorImpls<'a> {
    ops: &'a [OpClass],
    indent: DisplayIndent,
}

impl<'a> DisplayLoadOperatorImpls<'a> {
    pub fn new(ops: &'a [OpClass], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }

    fn emit(&self, f: &mut fmt::Formatter, op: &OpClass) -> fmt::Result {
        let indent = self.indent;
        let name = &*op.name;
        let op_mem0_ri = op.op_mem0_ri();
        let op_mem0_rr = op.op_mem0_rr();
        let op_mem0_rs = op.op_mem0_rs();
        let op_mem0_si = op.op_mem0_si();
        let op_mem0_sr = op.op_mem0_sr();
        let op_mem0_ss = op.op_mem0_ss();
        let op_ri = op.op_ri();
        let op_rr = op.op_rr();
        let op_rs = op.op_rs();
        write!(
            f,
            "\
            {indent}pub enum {name} {{}}\n\
            {indent}impl crate::LoadOperator for {name} {{\n\
            {indent}    const NAME: &'static ::core::primitive::str = \"{name}\";\n\
            {indent}    type OpMem0Ri = crate::op::{op_mem0_ri};\n\
            {indent}    type OpMem0Rr = crate::op::{op_mem0_rr};\n\
            {indent}    type OpMem0Rs = crate::op::{op_mem0_rs};\n\
            {indent}    type OpMem0Si = crate::op::{op_mem0_si};\n\
            {indent}    type OpMem0Sr = crate::op::{op_mem0_sr};\n\
            {indent}    type OpMem0Ss = crate::op::{op_mem0_ss};\n\
            {indent}    type OpRi = crate::op::{op_ri};\n\
            {indent}    type OpRr = crate::op::{op_rr};\n\
            {indent}    type OpRs = crate::op::{op_rs};\n\
            {indent}    fn make_mem0_ri(result: crate::Reg, address: crate::Address) -> Self::OpMem0Ri {{\n\
            {indent}        Self::OpMem0Ri {{ result, address }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_rr(result: crate::Reg, ptr: crate::Reg, offset: crate::Offset) -> Self::OpMem0Rr {{\n\
            {indent}        Self::OpMem0Rr {{ result, ptr, offset }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_rs(result: crate::Reg, ptr: crate::Stack, offset: crate::Offset) -> Self::OpMem0Rs {{\n\
            {indent}        Self::OpMem0Rs {{ result, ptr, offset }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_si(result: crate::Stack, address: crate::Address) -> Self::OpMem0Si {{\n\
            {indent}        Self::OpMem0Si {{ result, address }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_sr(result: crate::Stack, ptr: crate::Reg, offset: crate::Offset) -> Self::OpMem0Sr {{\n\
            {indent}        Self::OpMem0Sr {{ result, ptr, offset }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_ss(result: crate::Stack, ptr: crate::Stack, offset: crate::Offset) -> Self::OpMem0Ss {{\n\
            {indent}        Self::OpMem0Ss {{ result, ptr, offset }}\n\
            {indent}    }}\n\
            {indent}    fn make_ri(result: crate::Reg, address: crate::Address, memory: crate::Memory) -> Self::OpRi {{\n\
            {indent}        Self::OpRi {{ result, address, memory }}\n\
            {indent}    }}\n\
            {indent}    fn make_rr(result: crate::Reg, ptr: crate::Reg, offset: crate::Offset, memory: crate::Memory) -> Self::OpRr {{\n\
            {indent}        Self::OpRr {{ result, ptr, memory, offset }}\n\
            {indent}    }}\n\
            {indent}    fn make_rs(result: crate::Reg, ptr: crate::Stack, offset: crate::Offset, memory: crate::Memory) -> Self::OpRs {{\n\
            {indent}        Self::OpRs {{ result, ptr, memory, offset }}\n\
            {indent}    }}\n\
            {indent}}}\
            "
        )
    }
}

impl Display for DisplayLoadOperatorImpls<'_> {
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
