use super::{LocalIdx, OperandIdx, StackOperand};
use crate::{ValType, core::TypedVal};

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
    /// Creates a new [`Operand`] from the given [`StackOperand`] and its [`StackPos`].
    pub(super) fn new(index: StackPos, operand: StackOperand) -> Self {
        match operand {
            StackOperand::Local {
                local_index, ty, ..
            } => Self::local(index, local_index, ty),
            StackOperand::Temp { ty } => Self::temp(index, ty),
            StackOperand::Immediate { ty, val } => Self::immediate(index, TypedVal::new(ty, val)),
        }
    }

    /// Returns `true` if `self` and `other` evaluate to the same value.
    pub fn is_same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Local(lhs), Self::Local(rhs)) => lhs.local_index() == rhs.local_index(),
            (Self::Temp(lhs), Self::Temp(rhs)) => lhs.stack_pos() == rhs.stack_pos(),
            (Self::Immediate(lhs), Self::Immediate(rhs)) => lhs.val() == rhs.val(),
            _ => false,
        }
    }

    /// Creates a local [`Operand`].
    pub(super) fn local(operand_index: StackPos, local_index: LocalIdx, ty: ValType) -> Self {
        Self::Local(LocalOperand {
            stack_pos: operand_index,
            local_index,
            ty,
        })
    }

    /// Creates a temporary [`Operand`].
    pub(super) fn temp(operand_index: StackPos, ty: ValType) -> Self {
        Self::Temp(TempOperand {
            stack_pos: operand_index,
            ty,
        })
    }

    /// Creates an immediate [`Operand`].
    pub(super) fn immediate(operand_index: StackPos, val: TypedVal) -> Self {
        Self::Immediate(ImmediateOperand {
            stack_pos: operand_index,
            val,
        })
    }

    /// Returns `true` if `self` is an [`Operand::Temp`].
    pub fn is_temp(&self) -> bool {
        matches!(self, Self::Temp(_))
    }

    /// Returns the [`StackPos`] of the [`Operand`].
    pub fn stack_pos(&self) -> StackPos {
        match self {
            Self::Local(operand) => operand.stack_pos(),
            Self::Temp(operand) => operand.stack_pos(),
            Self::Immediate(operand) => operand.stack_pos(),
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
    /// The position of the operand on the operand stack.
    stack_pos: StackPos,
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
    /// Returns the stack position of the [`LocalOperand`].
    pub fn stack_pos(&self) -> StackPos {
        self.stack_pos
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
    /// The position of the operand on the operand stack.
    stack_pos: StackPos,
    /// The type of the temporary.
    ty: ValType,
}

impl From<TempOperand> for Operand {
    fn from(operand: TempOperand) -> Self {
        Self::Temp(operand)
    }
}

impl TempOperand {
    /// Returns the stack position of the [`TempOperand`].
    pub fn stack_pos(&self) -> StackPos {
        self.stack_pos
    }

    /// Returns the type of the [`TempOperand`].
    pub fn ty(&self) -> ValType {
        self.ty
    }
}

/// An immediate value on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct ImmediateOperand {
    /// The position of the operand on the operand stack.
    stack_pos: StackPos,
    /// The value and type of the immediate value.
    val: TypedVal,
}

impl From<ImmediateOperand> for Operand {
    fn from(operand: ImmediateOperand) -> Self {
        Self::Immediate(operand)
    }
}

impl ImmediateOperand {
    /// Returns the stack position of the [`ImmediateOperand`].
    pub fn stack_pos(&self) -> StackPos {
        self.stack_pos
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
