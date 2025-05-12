mod consts;
mod locals;

use crate::core::{TypedVal, ValType};
use self::{
    consts::ConstRegistry,
    locals::{LocalIdx, LocalsRegistry},
};
use alloc::vec::Vec;
use core::num::NonZeroUsize;

/// The Wasm value stack during translation from Wasm to Wasmi bytecode.
#[derive(Debug, Default, Clone)]
pub struct Stack {
    /// The stack of operands.
    operands: Vec<StackOperand>,
    /// All function local constants.
    consts: ConstRegistry,
    /// All function parameters and locals and their types.
    locals: LocalsRegistry,
    /// The index of the first [`StackOperand::Local`] on the [`Stack`].
    first_local: Option<OperandIdx>,
    /// The index of the last [`StackOperand::Local`] on the [`Stack`].
    last_local: Option<OperandIdx>,
    /// The maximum size of the [`Stack`].
    max_stack_height: usize,
    /// The current phase of the [`Stack`].
    phase: StackPhase,
}

/// The current phase of the [`Stack`].
#[derive(Debug, Default, Copy, Clone)]
pub enum StackPhase {
    /// Phase that allows to define local variables.
    #[default]
    DefineLocals,
    /// Phase that allows to manipulate the stack and allocate function local constants.
    Translation,
    /// Phase after finishing translation.
    ///
    /// In this phase state changes are no longer allowed.
    /// Only resetting the [`Stack`] is allowed in order to restart the phase cycle.
    Finish,
}

impl StackPhase {
    /// Resets the [`StackPhase`] to the [`StackPhase::DefineLocals`] phase.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Ensures that the current phase is [`StackPhase::DefineLocals`].
    pub fn assert_define_locals(&self) {
        assert!(matches!(self, Self::DefineLocals));
    }

    /// Turns the current phase into [`StackPhase::Translation`].
    ///
    /// # Panics
    ///
    /// If the current phase is incompatible with this phase shift.
    pub fn assert_translation(&mut self) {
        assert!(matches!(self, Self::DefineLocals | Self::Translation));
        *self = Self::Translation;
    }

    /// Turns the current phase into [`StackPhase::Finish`].
    ///
    /// # Panics
    ///
    /// If the current phase is incompatible with this phase shift.
    pub fn assert_finish(&mut self) {
        assert!(matches!(self, Self::Translation | Self::Finish));
        *self = Self::Finish;
    }
}

/// A [`StackOperand`] or [`Operand`] index on the [`Stack`].
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

/// An [`Operand`] on the [`Stack`].
///
/// This is the internal version of [`Operand`] with information that shall remain
/// hidden to the outside.
#[derive(Debug, Copy, Clone)]
enum StackOperand {
    /// A local variable.
    Local {
        /// The index of the local variable.
        local_index: LocalIdx,
        /// The previous [`StackOperand::Local`] on the [`Stack`].
        prev_local: Option<OperandIdx>,
        /// The next [`StackOperand::Local`] on the [`Stack`].
        next_local: Option<OperandIdx>,
    },
    /// A temporary value on the [`Stack`].
    Temp {
        /// The type of the temporary value.
        ty: ValType,
    },
    /// An immediate value on the [`Stack`].
    Immediate {
        /// The value (and type) of the immediate value.
        val: TypedVal,
    },
}

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
    fn new(operand_index: OperandIdx, operand: StackOperand, locals: &LocalsRegistry) -> Self {
        match operand {
            StackOperand::Local { local_index, .. } => {
                let Some(ty) = locals.ty(local_index) else {
                    panic!("failed to query type of local at: {local_index:?}");
                };
                Self::Local(LocalOperand {
                    operand_index,
                    local_index,
                    ty,
                })
            }
            StackOperand::Temp { ty } => Self::Temp(TempOperand { operand_index, ty }),
            StackOperand::Immediate { val } => {
                Self::Immediate(ImmediateOperand { operand_index, val })
            }
        }
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
