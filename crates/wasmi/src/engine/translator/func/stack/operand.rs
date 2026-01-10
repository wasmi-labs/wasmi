use super::{LocalIdx, OperandIdx, StackOperand};
use crate::{ValType, core::TypedVal, engine::translator::func::encoder::BytePos};

#[cfg(doc)]
use super::Stack;

/// An operand on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub enum Operand {
    /// A local variable operand.
    Local(LocalOperand),
    /// A temporary operand.
    Temp(TempOperand),
    /// An immediate value operand.
    Immediate(ImmediateOperand),
}

impl Operand {
    /// Creates a new [`Operand`] from the given [`StackOperand`] and its [`OperandIdx`].
    pub(super) fn new(index: OperandIdx, operand: StackOperand) -> Self {
        match operand {
            StackOperand::Local {
                local_index, ty, ..
            } => Self::local(index, local_index, ty),
            StackOperand::Temp { ty } => Self::temp(index, ty),
            StackOperand::Immediate { val } => Self::immediate(index, val),
        }
    }

    /// Returns `true` if `self` and `other` evaluate to the same value.
    pub fn is_same(&self, other: &Self) -> bool {
        match (self, other) {
            (Operand::Local(lhs), Operand::Local(rhs)) => lhs.local_index() == rhs.local_index(),
            (Operand::Temp(lhs), Operand::Temp(rhs)) => lhs.operand_index() == rhs.operand_index(),
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => lhs.val() == rhs.val(),
            _ => false,
        }
    }

    /// Creates a local [`Operand`].
    pub(super) fn local(operand_index: OperandIdx, local_index: LocalIdx, ty: ValType) -> Self {
        Self::Local(LocalOperand {
            operand_index,
            local_index,
            ty,
        })
    }

    /// Creates a temporary [`Operand`].
    pub(super) fn temp(operand_index: OperandIdx, ty: ValType) -> Self {
        Self::Temp(TempOperand { operand_index, ty })
    }

    /// Creates an immediate [`Operand`].
    pub(super) fn immediate(operand_index: OperandIdx, val: TypedVal) -> Self {
        Self::Immediate(ImmediateOperand { operand_index, val })
    }

    /// Returns `true` if `self` is an [`Operand::Temp`].
    pub fn is_temp(&self) -> bool {
        matches!(self, Self::Temp(_))
    }

    /// Returns the [`OperandIdx`] of the [`Operand`].
    pub fn index(&self) -> OperandIdx {
        match self {
            Operand::Local(operand) => operand.operand_index(),
            Operand::Temp(operand) => operand.operand_index(),
            Operand::Immediate(operand) => operand.operand_index(),
        }
    }

    /// Returns the type of the [`Operand`].
    pub fn ty(&self) -> ValType {
        match self {
            Self::Local(operand) => operand.ty(),
            Self::Temp(operand) => operand.ty(),
            Self::Immediate(operand) => operand.ty(),
        }
    }
}

/// A local variable on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct LocalOperand {
    /// The index of the operand.
    operand_index: OperandIdx,
    /// The index of the local variable.
    local_index: LocalIdx,
    /// The type of the local variable.
    ty: ValType,
}

impl From<LocalOperand> for Operand {
    fn from(operand: LocalOperand) -> Self {
        Self::Local(operand)
    }
}

impl LocalOperand {
    /// Returns the operand index of the [`LocalOperand`].
    pub fn operand_index(&self) -> OperandIdx {
        self.operand_index
    }

    /// Returns the index of the [`LocalOperand`].
    pub fn local_index(&self) -> LocalIdx {
        self.local_index
    }

    /// Returns the type of the [`LocalOperand`].
    pub fn ty(&self) -> ValType {
        self.ty
    }
}

/// A temporary on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct TempOperand {
    /// The index of the operand.
    operand_index: OperandIdx,
    /// The type of the temporary.
    ty: ValType,
}

impl From<TempOperand> for Operand {
    fn from(operand: TempOperand) -> Self {
        Self::Temp(operand)
    }
}

impl TempOperand {
    /// Returns the operand index of the [`TempOperand`].
    pub fn operand_index(&self) -> OperandIdx {
        self.operand_index
    }

    /// Returns the type of the [`TempOperand`].
    pub fn ty(&self) -> ValType {
        self.ty
    }
}

/// An immediate value on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct ImmediateOperand {
    /// The index of the operand.
    operand_index: OperandIdx,
    /// The value and type of the immediate value.
    val: TypedVal,
}

impl From<ImmediateOperand> for Operand {
    fn from(operand: ImmediateOperand) -> Self {
        Self::Immediate(operand)
    }
}

impl ImmediateOperand {
    /// Returns the operand index of the [`ImmediateOperand`].
    pub fn operand_index(&self) -> OperandIdx {
        self.operand_index
    }

    /// Returns the immediate value (and its type) of the [`ImmediateOperand`].
    pub fn val(&self) -> TypedVal {
        self.val
    }

    /// Returns the type of the [`ImmediateOperand`].
    pub fn ty(&self) -> ValType {
        self.val.ty()
    }
}

impl AsRef<Operand> for Operand {
    fn as_ref(&self) -> &Operand {
        self
    }
}
