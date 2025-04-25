use super::super::{DisplayFileHeader, DisplayIndent};
use crate::instrs::{instrs::ImmediateTy, OpClass};
use core::{fmt, fmt::Display};

pub struct DisplayStoreOperatorImpls<'a> {
    ops: &'a [OpClass],
    indent: DisplayIndent,
}

impl<'a> DisplayStoreOperatorImpls<'a> {
    pub fn new(ops: &'a [OpClass], indent: DisplayIndent) -> Self {
        Self { ops, indent }
    }

    fn emit(&self, f: &mut fmt::Formatter, op: &OpClass) -> fmt::Result {
        let indent = self.indent;
        let name = &*op.name;
        let op_mem0_rr = op.op_mem0_rr();
        let op_mem0_ri = op.op_mem0_ri();
        let op_mem0_rs = op.op_mem0_rs();
        let op_mem0_sr = op.op_mem0_sr();
        let op_mem0_si = op.op_mem0_si();
        let op_mem0_ss = op.op_mem0_ss();
        let op_mem0_ir = op.op_mem0_ir();
        let op_mem0_ii = op.op_mem0_ii();
        let op_mem0_is = op.op_mem0_is();
        let op_ss = op.op_ss();
        let op_si = op.op_si();
        let op_is = op.op_is();
        let op_ii = op.op_ii();
        let ty = ImmediateTy::from(op.ty);
        let (op_mem0_rr, op_mem0_rr_impl) = match ty {
            ImmediateTy::F32 | ImmediateTy::F64 => {
                let alias = format!("crate::op::{op_mem0_rr}");
                let expr = "::core::option::Option::Some(Self::OpMem0Rr { ptr, offset, value })";
                (alias, expr)
            }
            ImmediateTy::I32 | ImmediateTy::I64 => {
                let alias = "crate::utils::NoOp".into();
                let expr = "::core::option::Option::None";
                (alias, expr)
            }
            _ => unreachable!(),
        };
        write!(
            f,
            "\
            {indent}pub enum {name} {{}}\n\
            {indent}impl crate::StoreOperator for {name} {{\n\
            {indent}    const NAME: &'static ::core::primitive::str = \"{name}\";\n\
            {indent}    type Imm = {ty};\n\
            {indent}    type OpMem0Rr = {op_mem0_rr};\n\
            {indent}    type OpMem0Ri = crate::op::{op_mem0_ri};\n\
            {indent}    type OpMem0Rs = crate::op::{op_mem0_rs};\n\
            {indent}    type OpMem0Sr = crate::op::{op_mem0_sr};\n\
            {indent}    type OpMem0Si = crate::op::{op_mem0_si};\n\
            {indent}    type OpMem0Ss = crate::op::{op_mem0_ss};\n\
            {indent}    type OpMem0Ir = crate::op::{op_mem0_ir};\n\
            {indent}    type OpMem0Ii = crate::op::{op_mem0_ii};\n\
            {indent}    type OpMem0Is = crate::op::{op_mem0_is};\n\
            {indent}    type OpSs = crate::op::{op_ss};\n\
            {indent}    type OpSi = crate::op::{op_si};\n\
            {indent}    type OpIs = crate::op::{op_is};\n\
            {indent}    type OpIi = crate::op::{op_ii};\n\
            {indent}    fn make_mem0_rr(ptr: crate::Reg, offset: crate::Offset, value: crate::Reg) -> ::core::option::Option<Self::OpMem0Rr> {{\n\
            {indent}        {op_mem0_rr_impl}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_rs(ptr: crate::Reg, offset: crate::Offset, value: crate::Stack) -> Self::OpMem0Rs {{\n\
            {indent}        Self::OpMem0Rs {{ ptr, offset, value }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_ri(ptr: crate::Reg, offset: crate::Offset, value: Self::Imm) -> Self::OpMem0Ri {{\n\
            {indent}        Self::OpMem0Ri {{ ptr, offset, value }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_sr(ptr: crate::Stack, offset: crate::Offset, value: crate::Reg) -> Self::OpMem0Sr {{\n\
            {indent}        Self::OpMem0Sr {{ ptr, offset, value }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_ss(ptr: crate::Stack, offset: crate::Offset, value: crate::Stack) -> Self::OpMem0Ss {{\n\
            {indent}        Self::OpMem0Ss {{ ptr, offset, value }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_si(ptr: crate::Stack, offset: crate::Offset, value: Self::Imm) -> Self::OpMem0Si {{\n\
            {indent}        Self::OpMem0Si {{ ptr, offset, value }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_ir(address: crate::Address, value: crate::Reg) -> Self::OpMem0Ir {{\n\
            {indent}        Self::OpMem0Ir {{ address, value }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_is(address: crate::Address, value: crate::Stack) -> Self::OpMem0Is {{\n\
            {indent}        Self::OpMem0Is {{ address, value }}\n\
            {indent}    }}\n\
            {indent}    fn make_mem0_ii(address: crate::Address, value: Self::Imm) -> Self::OpMem0Ii {{\n\
            {indent}        Self::OpMem0Ii {{ address, value }}\n\
            {indent}    }}\n\
            {indent}    fn make_ss(ptr: crate::Stack, offset: crate::Offset, value: crate::Stack, memory: crate::Memory) -> Self::OpSs {{\n\
            {indent}        Self::OpSs {{ ptr, value, offset, memory }}\n\
            {indent}    }}\n\
            {indent}    fn make_si(ptr: crate::Stack, offset: crate::Offset, value: Self::Imm, memory: crate::Memory) -> Self::OpSi {{\n\
            {indent}        Self::OpSi {{ ptr, value, offset, memory }}\n\
            {indent}    }}\n\
            {indent}    fn make_is(address: crate::Address, value: crate::Stack, memory: crate::Memory) -> Self::OpIs {{\n\
            {indent}        Self::OpIs {{ address, value, memory }}\n\
            {indent}    }}\n\
            {indent}    fn make_ii(address: crate::Address, value: Self::Imm, memory: crate::Memory) -> Self::OpIi {{\n\
            {indent}        Self::OpIi {{ address, value, memory }}\n\
            {indent}    }}\n\
            {indent}}}\
            "
        )
    }
}

impl Display for DisplayStoreOperatorImpls<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_header = DisplayFileHeader;
        emit!(f, self.indent =>
            display_header
            "#![expect(unused_variables)]"
            ""
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
