use super::FieldTy;
use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone)]
#[expect(dead_code)] // TODO: remove
pub enum ImmediateTy {
    U32,
    U64,
    Usize,
    I32,
    I64,
    Isize,
    F32,
    F64,
    Global,
    Func,
    WasmFunc,
    Memory,
    Table,
    Data,
    Elem,
    Address,
    Offset,
    BranchOffset,
    TrapCode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operand {
    Reg,
    Stack,
    Immediate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperandId {
    Reg,
    Stack,
    Immediate,
}

impl Display for OperandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            OperandId::Reg => 'R',
            OperandId::Stack => 'S',
            OperandId::Immediate => 'I',
        };
        write!(f, "{repr}")
    }
}

impl Operand {
    pub fn id(&self) -> OperandId {
        match self {
            Self::Reg => OperandId::Reg,
            Self::Stack => OperandId::Stack,
            Self::Immediate => OperandId::Immediate,
        }
    }

    pub fn ty(&self, type_: impl Into<Option<ValTy>>) -> FieldTy {
        match self {
            Operand::Reg => FieldTy::Reg,
            Operand::Stack => FieldTy::Stack,
            Operand::Immediate => {
                let imm_ty = match type_.into().unwrap() {
                    ValTy::I32 => ImmediateTy::I32,
                    ValTy::I64 => ImmediateTy::I64,
                    ValTy::F32 => ImmediateTy::F32,
                    ValTy::F64 => ImmediateTy::F64,
                };
                FieldTy::Immediate(imm_ty)
            }
        }
    }

    pub fn is_reg(&self) -> bool {
        matches!(self, Self::Reg)
    }

    pub fn is_stack(&self) -> bool {
        matches!(self, Self::Stack)
    }

    pub fn is_imm(&self) -> bool {
        matches!(self, Self::Immediate)
    }
}

#[derive(Copy, Clone)]
pub enum ValTy {
    I32,
    I64,
    F32,
    F64,
}

impl From<ValTy> for ImmediateTy {
    fn from(ty: ValTy) -> Self {
        match ty {
            ValTy::I32 => ImmediateTy::I32,
            ValTy::I64 => ImmediateTy::I64,
            ValTy::F32 => ImmediateTy::F32,
            ValTy::F64 => ImmediateTy::F64,
        }
    }
}

impl Display for ValTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::I32 => "I32",
            Self::I64 => "I64",
            Self::F32 => "F32",
            Self::F64 => "F64",
        };
        write!(f, "{str}")
    }
}

#[macro_export]
macro_rules! op {
    (
        name: $name:expr,
        $( kind: $kind:expr, )?
        fields: [
            $(
                $field_name:ident: $field_ty:expr
            ),*
            $(,)?
        ]
        $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut instr = Op::new($name);
        $(
            instr.class_kind($kind);
        )?
        $(
            instr.push_field(ident_to_field_name!($field_name), $field_ty);
        )*
        instr
    }};
}
