use super::{utils::ValTy, ImmediateTy};
use crate::instrs::OperandId;
use core::fmt::{self, Display};
use std::{boxed::Box, vec::Vec};

#[derive(Default)]
pub struct Context {
    ops: Vec<Op>,
    pub unary_ops: Vec<OpClass>,
    pub binary_commutative_ops: Vec<OpClass>,
    pub binary_ops: Vec<OpClass>,
    pub load_ops: Vec<OpClass>,
    pub store_ops: Vec<OpClass>,
    pub cmp_branch_commutative_ops: Vec<OpClass>,
    pub cmp_branch_ops: Vec<OpClass>,
}

#[derive(Debug, Copy, Clone)]
pub struct DisplayOpName<'a> {
    name: &'a str,
    suffix: DisplayOpNameSuffix,
    ids: &'a [OperandId],
}

#[derive(Debug, Copy, Clone)]
pub enum DisplayOpNameSuffix {
    None,
    Mem0,
}

impl Display for DisplayOpNameSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Mem0 => write!(f, "Mem0"),
        }
    }
}

impl<'a> DisplayOpName<'a> {
    pub fn new(name: &'a str, suffix: DisplayOpNameSuffix, ids: &'a [OperandId]) -> Self {
        Self { name, suffix, ids }
    }
}

impl<'a> Display for DisplayOpName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ids.is_empty() {
            return write!(f, "{}", self.name);
        }
        write!(f, "{}{}_", self.name, self.suffix)?;
        for id in self.ids {
            write!(f, "{id}")?;
        }
        Ok(())
    }
}

pub struct OpClass {
    pub name: Box<str>,
    pub ty: ValTy,
}

macro_rules! impl_display_name_getter {
    (
        $( fn $name:ident($suffix:expr, $ids:expr); )* $(;)?
    ) => {
        $(
            pub fn $name(&self) -> DisplayOpName {
                DisplayOpName::new(&self.name, $suffix, &$ids)
            }
        )*
    };
}

#[allow(dead_code)]
impl OpClass {
    impl_display_name_getter! {
        // 1 ID
        fn op_r(DisplayOpNameSuffix::None, [OperandId::Reg]);
        fn op_s(DisplayOpNameSuffix::None, [OperandId::Stack]);
        // 2 IDs
        fn op_rr(DisplayOpNameSuffix::None, [OperandId::Reg; 2]);
        fn op_ri(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Immediate]);
        fn op_rs(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Stack]);
        fn op_sr(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Reg]);
        fn op_si(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Immediate]);
        fn op_ss(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Stack]);
        fn op_ir(DisplayOpNameSuffix::None, [OperandId::Immediate, OperandId::Reg]);
        fn op_ii(DisplayOpNameSuffix::None, [OperandId::Immediate, OperandId::Immediate]);
        fn op_is(DisplayOpNameSuffix::None, [OperandId::Immediate, OperandId::Stack]);

        fn op_mem0_rr(DisplayOpNameSuffix::Mem0, [OperandId::Reg, OperandId::Reg]);
        fn op_mem0_ri(DisplayOpNameSuffix::Mem0, [OperandId::Reg, OperandId::Immediate]);
        fn op_mem0_rs(DisplayOpNameSuffix::Mem0, [OperandId::Reg, OperandId::Stack]);
        fn op_mem0_sr(DisplayOpNameSuffix::Mem0, [OperandId::Stack, OperandId::Reg]);
        fn op_mem0_si(DisplayOpNameSuffix::Mem0, [OperandId::Stack, OperandId::Immediate]);
        fn op_mem0_ss(DisplayOpNameSuffix::Mem0, [OperandId::Stack, OperandId::Stack]);
        fn op_mem0_ir(DisplayOpNameSuffix::Mem0, [OperandId::Immediate, OperandId::Reg]);
        fn op_mem0_ii(DisplayOpNameSuffix::Mem0, [OperandId::Immediate, OperandId::Immediate]);
        fn op_mem0_is(DisplayOpNameSuffix::Mem0, [OperandId::Immediate, OperandId::Stack]);

        // 3 IDs
        fn op_rrr(DisplayOpNameSuffix::None, [OperandId::Reg; 3]);
        fn op_rrs(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Reg, OperandId::Stack]);
        fn op_rri(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Reg, OperandId::Immediate]);
        fn op_rsr(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Stack, OperandId::Reg]);
        fn op_rss(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Stack, OperandId::Stack]);
        fn op_rsi(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Stack, OperandId::Immediate]);
        fn op_rir(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Immediate, OperandId::Reg]);
        fn op_ris(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Immediate, OperandId::Stack]);
        fn op_rii(DisplayOpNameSuffix::None, [OperandId::Reg, OperandId::Immediate, OperandId::Immediate]);
        fn op_srr(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Reg, OperandId::Reg]);
        fn op_srs(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Reg, OperandId::Stack]);
        fn op_sri(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Reg, OperandId::Immediate]);
        fn op_ssr(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Stack, OperandId::Reg]);
        fn op_sss(DisplayOpNameSuffix::None, [OperandId::Stack; 3]);
        fn op_ssi(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Stack, OperandId::Immediate]);
        fn op_sir(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Immediate, OperandId::Reg]);
        fn op_sis(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Immediate, OperandId::Stack]);
        fn op_sii(DisplayOpNameSuffix::None, [OperandId::Stack, OperandId::Immediate, OperandId::Immediate]);
    }
}

impl Context {
    pub fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn ops(&self) -> &[Op] {
        &self.ops[..]
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OpClassKind {
    None,
    GlobalGet,
    Select,
}

#[derive(Debug, Clone)]
pub struct Op {
    name: Box<str>,
    kind: OpClassKind,
    fields: Vec<Field>,
}

impl Op {
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            kind: OpClassKind::None,
            fields: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &*self.name
    }

    pub fn kind(&self) -> OpClassKind {
        self.kind
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields[..]
    }

    pub fn class_kind(&mut self, kind: OpClassKind) {
        self.kind = kind;
    }

    pub fn push_field(&mut self, name: FieldName, ty: impl Into<FieldTy>) {
        self.fields.push(Field::new(name, ty.into()));
    }

    pub fn result_ty(&self) -> Option<FieldTy> {
        self.fields
            .iter()
            .find(|field| matches!(field.name, FieldName::Result))
            .map(|field| field.ty)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Field {
    pub name: FieldName,
    pub ty: FieldTy,
}

impl Field {
    pub fn new(name: impl Into<FieldName>, ty: impl Into<FieldTy>) -> Self {
        Self {
            name: name.into(),
            ty: ty.into(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FieldName {
    Result,
    Condition,
    Value,
    Lhs,
    Rhs,
    Ptr,
    Offset,
    Input,
    Index,
    DstIndex,
    SrcIndex,
    DstTable,
    SrcTable,
    DstMemory,
    SrcMemory,
    Len,
    LenTargets,
    LenValues,
    LenParams,
    LenResults,
    Delta,
    Address,
    Memory,
    Table,
    Global,
    Func,
    Data,
    Elem,
    Code,
    Fuel,
}

#[macro_export]
#[rustfmt::skip]
macro_rules! ident_to_field_name {
    (result) => { FieldName::Result };
    (results) => { FieldName::Results };
    (condition) => { FieldName::Condition };
    (value) => { FieldName::Value };
    (lhs) => { FieldName::Lhs };
    (rhs) => { FieldName::Rhs };
    (ptr) => { FieldName::Ptr };
    (offset) => { FieldName::Offset };
    (input) => { FieldName::Input };
    (index) => { FieldName::Index };
    (dst_index) => { FieldName::DstIndex };
    (src_index) => { FieldName::SrcIndex };
    (dst_table) => { FieldName::DstTable };
    (src_table) => { FieldName::SrcTable };
    (dst_memory) => { FieldName::DstMemory };
    (src_memory) => { FieldName::SrcMemory };
    (len) => { FieldName::Len };
    (len_targets) => { FieldName::LenTargets };
    (len_values) => { FieldName::LenValues };
    (len_params) => { FieldName::LenParams };
    (len_results) => { FieldName::LenResults };
    (delta) => { FieldName::Delta };
    (address) => { FieldName::Address };
    (memory) => { FieldName::Memory };
    (table) => { FieldName::Table };
    (global) => { FieldName::Global };
    (func) => { FieldName::Func };
    (data) => { FieldName::Data };
    (elem) => { FieldName::Elem };
    (code) => { FieldName::Code };
    (fuel) => { FieldName::Fuel };
    ($error:ident) => {{
        compile_error!(concat!(
            "invalid field name identifier: ",
            stringify!($error)
        ))
    }};
}

#[derive(Debug, Copy, Clone)]
pub enum FieldTy {
    Reg,
    Stack,
    Immediate(ImmediateTy),
}

impl From<ImmediateTy> for FieldTy {
    fn from(imm_ty: ImmediateTy) -> Self {
        Self::Immediate(imm_ty)
    }
}
