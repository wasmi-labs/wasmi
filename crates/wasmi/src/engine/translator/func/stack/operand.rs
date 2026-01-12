use super::{LocalIdx, StackPos, StackOperand};
use crate::{ValType, core::TypedVal, ir::Slot};

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
    pub(super) fn new(stack_pos: StackPos, operand: StackOperand) -> Self {
        match operand {
            StackOperand::Local {
                local_index,
                ty,
                temp_slot,
                ..
            } => Self::local(stack_pos, temp_slot, local_index, ty),
            StackOperand::Temp { ty, temp_slot, .. } => Self::temp(stack_pos, temp_slot, ty),
            StackOperand::Immediate {
                ty, temp_slot, val, ..
            } => Self::immediate(stack_pos, temp_slot, TypedVal::new(ty, val)),
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
    pub(super) fn local(
        stack_pos: StackPos,
        temp_slot: Slot,
        local_index: LocalIdx,
        ty: ValType,
    ) -> Self {
        Self::Local(LocalOperand {
            stack_pos,
            temp_slot,
            ty,
            local_index,
        })
    }

    /// Creates a temporary [`Operand`].
    pub(super) fn temp(stack_pos: StackPos, temp_slot: Slot, ty: ValType) -> Self {
        Self::Temp(TempOperand {
            stack_pos,
            temp_slot,
            ty,
        })
    }

    /// Creates an immediate [`Operand`].
    pub(super) fn immediate(stack_pos: StackPos, temp_slot: Slot, val: TypedVal) -> Self {
        Self::Immediate(ImmediateOperand {
            stack_pos,
            temp_slot,
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

    /// Returns the temporary [`Slot`](crate::ir::Slot) of the [`Operand`].
    pub fn temp_slot(&self) -> Slot {
        match self {
            Self::Local(operand) => operand.temp_slot(),
            Self::Temp(operand) => operand.temp_slot(),
            Self::Immediate(operand) => operand.temp_slot(),
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
    /// The temporary [`Slot`] of the local operand.
    temp_slot: Slot,
    /// The type of the local variable.
    ty: ValType,
    /// The index of the local variable.
    local_index: LocalIdx,
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

    /// Returns the temporary [`Slot`](crate::ir::Slot) of the [`LocalOperand`].
    pub fn temp_slot(&self) -> Slot {
        self.temp_slot
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
    /// The temporary [`Slot`] of the local operand.
    temp_slot: Slot,
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

    /// Returns the temporary [`Slot`](crate::ir::Slot) of the [`TempOperand`].
    pub fn temp_slot(&self) -> Slot {
        self.temp_slot
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
    /// The temporary [`Slot`] of the local operand.
    temp_slot: Slot,
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

    /// Returns the temporary [`Slot`](crate::ir::Slot) of the [`ImmediateOperand`].
    pub fn temp_slot(&self) -> Slot {
        self.temp_slot
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
