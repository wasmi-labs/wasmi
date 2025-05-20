use super::{LocalIdx, LocalsRegistry, OperandIdx, StackOperand};
use crate::{
    core::{TypedVal, ValType},
    engine::Instr,
};

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
    pub(super) fn new(index: OperandIdx, operand: StackOperand, locals: &LocalsRegistry) -> Self {
        match operand {
            StackOperand::Local { local_index, .. } => Self::local(index, local_index, locals),
            StackOperand::Temp { ty, instr } => Self::temp(index, ty, instr),
            StackOperand::Immediate { val } => Self::immediate(index, val),
        }
    }

    /// Creates a local [`Operand`].
    pub(super) fn local(
        operand_index: OperandIdx,
        local_index: LocalIdx,
        locals: &LocalsRegistry,
    ) -> Self {
        let Some(ty) = locals.ty(local_index) else {
            panic!("failed to query type of local at: {local_index:?}");
        };
        Self::Local(LocalOperand {
            operand_index,
            local_index,
            ty,
        })
    }

    /// Creates a temporary [`Operand`].
    pub(super) fn temp(operand_index: OperandIdx, ty: ValType, instr: Option<Instr>) -> Self {
        Self::Temp(TempOperand {
            operand_index,
            ty,
            instr,
        })
    }

    /// Creates an immediate [`Operand`].
    pub(super) fn immediate(operand_index: OperandIdx, val: TypedVal) -> Self {
        Self::Immediate(ImmediateOperand { operand_index, val })
    }

    /// Returns `true` if `self` is an [`Operand::Local`].
    pub fn is_local(self) -> bool {
        matches!(self, Self::Local(_))
    }

    /// Returns `true` if `self` is an [`Operand::Temp`].
    pub fn is_temp(self) -> bool {
        matches!(self, Self::Temp(_))
    }

    /// Returns `true` if `self` is an [`Operand::Immediate`].
    pub fn is_immediate(self) -> bool {
        matches!(self, Self::Immediate(_))
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
    pub fn ty(self) -> ValType {
        match self {
            Self::Local(local_operand) => local_operand.ty(),
            Self::Temp(temp_operand) => temp_operand.ty(),
            Self::Immediate(immediate_operand) => immediate_operand.ty(),
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
    /// The instruction which created this [`TempOperand`] as its result.
    instr: Option<Instr>,
}

impl TempOperand {
    /// Returns the operand index of the [`TempOperand`].
    pub fn operand_index(&self) -> OperandIdx {
        self.operand_index
    }

    /// Returns the type of the [`TempOperand`].
    pub fn ty(self) -> ValType {
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

impl ImmediateOperand {
    /// Returns the operand index of the [`ImmediateOperand`].
    pub fn operand_index(&self) -> OperandIdx {
        self.operand_index
    }

    /// Returns the immediate value (and its type) of the [`ImmediateOperand`].
    pub fn val(self) -> TypedVal {
        self.val
    }

    /// Returns the type of the [`ImmediateOperand`].
    pub fn ty(self) -> ValType {
        self.val.ty()
    }
}
