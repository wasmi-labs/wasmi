use super::{LocalIdx, StackOperand, StackPos};
use crate::{
    ValType,
    core::{RawVal, TypedVal},
    engine::translator::utils::required_cells_for_ty,
    ir::{BoundedSlotSpan, SlotSpan},
};

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
                temp_slots,
                ..
            } => Self::local(temp_slots, local_index, ty),
            StackOperand::Temp { ty, temp_slots, .. } => Self::temp(stack_pos, temp_slots, ty),
            StackOperand::Immediate {
                ty,
                temp_slots,
                val,
                ..
            } => Self::immediate(temp_slots, ty, val),
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
    pub(super) fn local(temp_slots: SlotSpan, local_index: LocalIdx, ty: ValType) -> Self {
        Self::Local(LocalOperand {
            temp_slots,
            ty,
            local_index,
        })
    }

    /// Creates a temporary [`Operand`].
    pub(super) fn temp(stack_pos: StackPos, temp_slots: SlotSpan, ty: ValType) -> Self {
        Self::Temp(TempOperand {
            temp_slots,
            ty,
            stack_pos,
        })
    }

    /// Creates an immediate [`Operand`].
    pub(super) fn immediate(temp_slots: SlotSpan, ty: ValType, val: RawVal) -> Self {
        Self::Immediate(ImmediateOperand {
            temp_slots,
            ty,
            val,
        })
    }

    /// Returns `true` if `self` is an [`Operand::Temp`].
    pub fn is_temp(&self) -> bool {
        matches!(self, Self::Temp(_))
    }

    /// Returns the temporary [`BoundedSlotSpan`] of the [`Operand`].
    ///
    /// # Note
    ///
    /// This is required to copy an span of operand to its temporary [`BoundedSlotSpan`].
    pub fn temp_slots(&self) -> BoundedSlotSpan {
        match self {
            Self::Local(operand) => operand.temp_slots(),
            Self::Temp(operand) => operand.temp_slots(),
            Self::Immediate(operand) => operand.temp_slots(),
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
    /// The temporary [`SlotSpan`] of the local operand.
    temp_slots: SlotSpan,
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
    /// Creates a new [`LocalOperand`] from its parts.
    pub(super) fn new(temp_slots: SlotSpan, ty: ValType, local_index: LocalIdx) -> Self {
        Self {
            temp_slots,
            ty,
            local_index,
        }
    }

    /// Returns the temporary [`BoundedSlotSpan`] of the [`LocalOperand`].
    pub fn temp_slots(&self) -> BoundedSlotSpan {
        let len = required_cells_for_ty(self.ty());
        BoundedSlotSpan::new(self.temp_slots, len)
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
    /// The temporary [`SlotSpan`] of the local operand.
    temp_slots: SlotSpan,
    /// The type of the temporary.
    ty: ValType,
    /// The position of the operand on the operand stack.
    stack_pos: StackPos,
}

impl From<TempOperand> for Operand {
    fn from(operand: TempOperand) -> Self {
        Self::Temp(operand)
    }
}

impl TempOperand {
    /// Creates a new [`TempOperand`] from its parts.
    pub(super) fn new(temp_slots: SlotSpan, ty: ValType, stack_pos: StackPos) -> Self {
        Self {
            temp_slots,
            ty,
            stack_pos,
        }
    }

    /// Returns the stack position of the [`TempOperand`].
    fn stack_pos(&self) -> StackPos {
        self.stack_pos
    }

    /// Returns the temporary [`BoundedSlotSpan`] of the [`TempOperand`].
    pub fn temp_slots(&self) -> BoundedSlotSpan {
        let len = required_cells_for_ty(self.ty());
        BoundedSlotSpan::new(self.temp_slots, len)
    }

    /// Returns the type of the [`TempOperand`].
    pub fn ty(&self) -> ValType {
        self.ty
    }
}

/// An immediate value on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct ImmediateOperand {
    /// The temporary [`SlotSpan`] of the local operand.
    temp_slots: SlotSpan,
    /// The type of the immediate value.
    ty: ValType,
    /// The value of the immediate value.
    val: RawVal,
}

impl From<ImmediateOperand> for Operand {
    fn from(operand: ImmediateOperand) -> Self {
        Self::Immediate(operand)
    }
}

impl ImmediateOperand {
    /// Creates a new [`ImmediateOperand`] from its parts.
    pub(super) fn new(temp_slots: SlotSpan, ty: ValType, val: RawVal) -> Self {
        Self {
            temp_slots,
            ty,
            val,
        }
    }

    /// Returns the temporary [`Slot`](crate::ir::BoundedSlotSpan) of the [`ImmediateOperand`].
    pub fn temp_slots(&self) -> BoundedSlotSpan {
        let len = required_cells_for_ty(self.ty());
        BoundedSlotSpan::new(self.temp_slots, len)
    }

    /// Returns the immediate value (and its type) of the [`ImmediateOperand`].
    pub fn val(&self) -> TypedVal {
        TypedVal::new(self.ty, self.val)
    }

    /// Returns the type of the [`ImmediateOperand`].
    pub fn ty(&self) -> ValType {
        self.ty
    }
}

impl AsRef<Operand> for Operand {
    fn as_ref(&self) -> &Operand {
        self
    }
}
