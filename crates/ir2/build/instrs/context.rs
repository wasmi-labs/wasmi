use super::{utils::ValTy, ImmediateTy};
use std::{boxed::Box, vec::Vec};

#[derive(Default)]
pub struct Context {
    ops: Vec<Op>,
    pub unary_ops: Vec<UnaryOp>,
    pub binary_commutative_ops: Vec<BinaryOp>,
}

pub struct UnaryOp {
    pub name: Box<str>,
}

pub struct BinaryOp {
    pub name: Box<str>,
    pub input_ty: ValTy,
}

impl Context {
    pub fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn ops(&self) -> &[Op] {
        &self.ops[..]
    }
}

#[derive(Debug, Clone)]
pub struct Op {
    name: Box<str>,
    fields: Vec<Field>,
}

impl Op {
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &*self.name
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields[..]
    }

    pub fn push_field(&mut self, name: FieldName, ty: impl Into<FieldTy>) {
        self.fields.push(Field::new(name, ty.into()));
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
