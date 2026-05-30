use super::{LocalIdx, StackOperand, StackPos};
use crate::{
    Error,
    ValType,
    core::{RawVal, TypedRawVal},
    engine::translator::{func::layout::StackLayout, utils::required_cells_for_ty},
    ir::{BoundedSlotSpan, Slot, SlotSpan},
};

#[cfg(doc)]
use super::Stack;

/// The location of an operand.
#[derive(Debug, Copy, Clone)]
pub enum Location {
    /// The operand's location is a register.
    Reg(ValType),
    /// The operand's location is a slot.
    Slot(Slot),
}

#[derive(Debug, Copy, Clone)]
pub enum ResolvedOperand<T> {
    /// The operand is a register.
    Reg(ValType),
    /// The operand is located in a [`Slot`].
    Slot(Slot),
    /// The operand is an immediate value of type `T`.
    Immediate(T),
}

impl<T> From<Location> for ResolvedOperand<T> {
    fn from(location: Location) -> Self {
        match location {
            Location::Reg(ty) => Self::Reg(ty),
            Location::Slot(slot) => Self::Slot(slot),
        }
    }
}

impl<T> ResolvedOperand<T> {
    pub fn sort(a: Self, b: Self) -> (Self, Self) {
        match (&a, &b) {
            | (Self::Slot(_), Self::Reg(_))
            | (Self::Immediate(_), Self::Reg(_))
            | (Self::Immediate(_), Self::Slot(_)) => (b, a),
            _ => (a, b),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ResolvedOperand<U> {
        match self {
            Self::Reg(ty) => ResolvedOperand::Reg(ty),
            Self::Slot(slot) => ResolvedOperand::Slot(slot),
            Self::Immediate(value) => ResolvedOperand::Immediate(f(value)),
        }
    }

    pub fn filter_map<U>(self, f: impl FnOnce(T) -> Option<U>) -> Option<ResolvedOperand<U>> {
        self.map(f).transpose()
    }
}

impl<T> ResolvedOperand<Option<T>> {
    /// Transposes a [`ResolvedOperand<Option<T>>`] into an [`Option<ResolvedOperand<T>>`].
    pub fn transpose(self) -> Option<ResolvedOperand<T>> {
        let resolved = match self {
            Self::Reg(ty) => ResolvedOperand::Reg(ty),
            Self::Slot(slot) => ResolvedOperand::Slot(slot),
            Self::Immediate(ok_or_err) => ResolvedOperand::Immediate(ok_or_err?),
        };
        Some(resolved)
    }
}

/// An operand on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub enum Operand {
    /// A register operand.
    Reg(RegOperand),
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
        use StackOperand as Opd;
        match operand {
            Opd::Temp {
                temp_slots,
                ty,
                in_reg,
            } => match in_reg {
                true => Self::reg(stack_pos, temp_slots, ty),
                false => Self::temp(stack_pos, temp_slots, ty),
            },
            Opd::Local {
                temp_slots,
                ty,
                in_reg,
                local_index,
                ..
            } => match in_reg {
                true => Self::reg(stack_pos, temp_slots, ty),
                false => Self::local(temp_slots, local_index, ty),
            },
            Opd::Immediate {
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

    /// Creates a register [`Operand`].
    pub(super) fn reg(stack_pos: StackPos, temp_slots: SlotSpan, ty: ValType) -> Self {
        Self::Reg(RegOperand::new(temp_slots, ty, stack_pos))
    }

    /// Creates a local [`Operand`].
    pub(super) fn local(temp_slots: SlotSpan, local_index: LocalIdx, ty: ValType) -> Self {
        Self::Local(LocalOperand::new(temp_slots, ty, local_index))
    }

    /// Creates a temporary [`Operand`].
    pub(super) fn temp(stack_pos: StackPos, temp_slots: SlotSpan, ty: ValType) -> Self {
        Self::Temp(TempOperand::new(temp_slots, ty, stack_pos))
    }

    /// Creates an immediate [`Operand`].
    pub(super) fn immediate(temp_slots: SlotSpan, ty: ValType, val: RawVal) -> Self {
        Self::Immediate(ImmediateOperand::new(temp_slots, ty, val))
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
            Self::Reg(operand) => operand.temp_slots(),
            Self::Local(operand) => operand.temp_slots(),
            Self::Temp(operand) => operand.temp_slots(),
            Self::Immediate(operand) => operand.temp_slots(),
        }
    }

    /// Returns the type of the [`Operand`].
    pub fn ty(&self) -> ValType {
        match self {
            Self::Reg(operand) => operand.ty(),
            Self::Local(operand) => operand.ty(),
            Self::Temp(operand) => operand.ty(),
            Self::Immediate(operand) => operand.ty(),
        }
    }

    /// Resolves the [`Operand`] into a [`ResolvedOperand<TypedRawVal`].
    ///
    /// This is a convenience wrapper for [`Self::resolve_as`].
    pub fn resolve(&self, layout: &StackLayout) -> Result<ResolvedOperand<TypedRawVal>, Error> {
        self.resolve_as::<TypedRawVal>(layout)
    }

    /// Resolves the [`Operand`] into a [`ResolvedOperand`].
    ///
    /// [`ResolvedOperand`] is a more destructed form which is simpler to handle,
    /// especially in pattern matching contexts. However, in contrast to [`Operand`]
    /// it loses some information during the conversion process.
    pub fn resolve_as<T>(&self, layout: &StackLayout) -> Result<ResolvedOperand<T>, Error>
    where
        T: From<TypedRawVal>,
    {
        let resolved = match self {
            Operand::Reg(operand) => ResolvedOperand::Reg(operand.ty()),
            Operand::Local(operand) => {
                let slot = layout.local_to_slot(operand)?;
                ResolvedOperand::Slot(slot)
            }
            Operand::Temp(operand) => {
                let slot = operand.temp_slots().head();
                ResolvedOperand::Slot(slot)
            }
            Operand::Immediate(operand) => {
                let value = T::from(operand.val());
                ResolvedOperand::Immediate(value)
            }
        };
        Ok(resolved)
    }
}

/// A register operand on the stack.
#[derive(Debug, Copy, Clone)]
#[expect(dead_code)]
pub struct RegOperand {
    /// The temporary [`SlotSpan`] of the local operand.
    temp_slots: SlotSpan,
    /// The type of the operand.
    ty: ValType,
    /// The position of the operand on the operand stack.
    stack_pos: StackPos,
}

impl From<RegOperand> for Operand {
    fn from(operand: RegOperand) -> Self {
        Self::Reg(operand)
    }
}

impl RegOperand {
    /// Creates a new [`RegOperand`] from its parts.
    pub(super) fn new(temp_slots: SlotSpan, ty: ValType, stack_pos: StackPos) -> Self {
        Self {
            temp_slots,
            ty,
            stack_pos,
        }
    }

    /// Returns the type of the [`RegOperand`].
    pub fn ty(&self) -> ValType {
        self.ty
    }

    /// Returns the temporary [`BoundedSlotSpan`] of the [`RegOperand`].
    pub fn temp_slots(&self) -> BoundedSlotSpan {
        let len = required_cells_for_ty(self.ty());
        BoundedSlotSpan::new(self.temp_slots, len)
    }
}

/// A local variable on the stack.
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    pub fn val(&self) -> TypedRawVal {
        TypedRawVal::new(self.ty, self.val)
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
