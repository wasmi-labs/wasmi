mod consts;
mod locals;

use self::{consts::ConstRegistry, locals::LocalsRegistry};
use crate::core::{TypedVal, ValType};
use alloc::vec::Vec;
use core::num::NonZeroUsize;

#[derive(Debug, Clone)]
pub struct ValueStack {
    operands: Vec<StackOperand>,
    consts: ConstRegistry,
    locals: LocalsRegistry,
    first_local: Option<OperandIdx>,
    last_local: Option<OperandIdx>,
    max_stack_height: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct OperandIdx(NonZeroUsize);

impl From<OperandIdx> for usize {
    fn from(value: OperandIdx) -> Self {
        value.0.get()
    }
}

impl From<usize> for OperandIdx {
    fn from(value: usize) -> Self {
        let Some(operand_idx) = NonZeroUsize::new(value.wrapping_add(1)) else {
            panic!("out of bounds `OperandIdx`: {value}")
        };
        Self(operand_idx)
    }
}

#[derive(Debug, Copy, Clone)]
enum StackOperand {
    Local {
        index: LocalIdx,
        prev_local: Option<OperandIdx>,
        next_local: Option<OperandIdx>,
    },
    Temp {
        ty: ValType,
    },
    Immediate {
        val: TypedVal,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum Operand {
    Local(LocalOperand),
    Temp(TempOperand),
    Immediate(ImmediateOperand),
}

impl Operand {
    pub fn is_local(self) -> bool {
        matches!(self, Self::Local(_))
    }

    pub fn is_temp(self) -> bool {
        matches!(self, Self::Temp(_))
    }

    pub fn is_immediate(self) -> bool {
        matches!(self, Self::Immediate(_))
    }

    pub fn ty(self) -> ValType {
        match self {
            Self::Local(local_operand) => local_operand.ty(),
            Self::Temp(temp_operand) => temp_operand.ty(),
            Self::Immediate(immediate_operand) => immediate_operand.ty(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LocalOperand {
    index: LocalIdx,
    ty: ValType,
}

impl LocalOperand {
    pub fn index(self) -> LocalIdx {
        self.index
    }

    pub fn ty(self) -> ValType {
        self.ty
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TempOperand {
    ty: ValType,
}

impl TempOperand {
    pub fn ty(self) -> ValType {
        self.ty
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ImmediateOperand {
    val: TypedVal,
}

impl ImmediateOperand {
    pub fn val(self) -> TypedVal {
        self.val
    }

    pub fn ty(self) -> ValType {
        self.val.ty()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LocalIdx(usize);
