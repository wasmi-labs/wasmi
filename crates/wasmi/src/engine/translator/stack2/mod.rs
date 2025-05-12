#![expect(unused_variables, dead_code)]

mod consts;
mod locals;

use self::{
    consts::ConstRegistry,
    locals::{LocalIdx, LocalsRegistry},
};
use crate::{
    core::{TypedVal, ValType},
    ir::Reg,
    Error,
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

impl Stack {
    /// Register `amount` local variables of common type `ty`.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: u32, ty: ValType) -> Result<(), Error> {
        todo!()
    }

    /// Finish registration of local variables.
    ///
    /// # Errors
    ///
    /// If the current [`StackPhase`] is not [`StackPhase::DefineLocals`].
    pub fn finish_register_locals(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// Pushes a local variable with index `local_idx` to the [`Stack`].
    ///
    /// # Errors
    ///
    /// - If the current [`StackPhase`] is not [`StackPhase::Translation`].
    /// - If too many operands have been pushed onto the [`Stack`].
    /// - If the local with `local_idx` does not exist.
    pub fn push_local(&mut self, local_idx: u32) -> Result<OperandIdx, Error> {
        todo!()
    }

    /// Pushes a temporary with type `ty` on the [`Stack`].
    ///
    /// # Errors
    ///
    /// - If the current [`StackPhase`] is not [`StackPhase::Translation`].
    /// - If too many operands have been pushed onto the [`Stack`].
    pub fn push_temp(&mut self, ty: ValType) -> Result<OperandIdx, Error> {
        todo!()
    }

    /// Pushes an immediate `value` on the [`Stack`].
    ///
    /// # Errors
    ///
    /// - If the current [`StackPhase`] is not [`StackPhase::Translation`].
    /// - If too many operands have been pushed onto the [`Stack`].
    pub fn push_immediate(&mut self, value: impl Into<TypedVal>) -> Result<OperandIdx, Error> {
        todo!()
    }

    /// Pops the top-most [`Operand`] from the [`Stack`].
    ///
    /// Returns `None` if the [`Stack`] is empty.
    pub fn pop(&mut self) -> Option<Operand> {
        let operand = self.operands.pop()?;
        let index = OperandIdx::from(self.operands.len());
        Some(Operand::new(index, operand, &self.locals))
    }

    /// Pops the two top-most [`Operand`] from the [`Stack`].
    ///
    /// - Returns `None` if the [`Stack`] is empty.
    /// - The last returned [`Operand`] is the top-most one.
    pub fn pop2(&mut self) -> Option<(Operand, Operand)> {
        let [o1, o2] = self.pop_some::<2>()?;
        Some((o1, o2))
    }

    /// Pops the three top-most [`Operand`] from the [`Stack`].
    ///
    /// - Returns `None` if the [`Stack`] is empty.
    /// - The last returned [`Operand`] is the top-most one.
    pub fn pop3(&mut self) -> Option<(Operand, Operand, Operand)> {
        let [o1, o2, o3] = self.pop_some::<3>()?;
        Some((o1, o2, o3))
    }

    /// Pops the top-most `N` [`Operand`]s from the [`Stack`].
    ///
    /// - Returns `None` if the [`Stack`] is empty.
    /// - The last returned [`Operand`] is the top-most one.
    fn pop_some<const N: usize>(&mut self) -> Option<[Operand; N]> {
        todo!()
    }

    /// Converts the [`Operand`] at `index` to a [`Reg`] if possible.
    ///
    /// # Errors
    ///
    /// If the translator ran out of [`Reg`]s for the function.
    pub fn operand_to_reg(&mut self, index: OperandIdx) -> Result<Reg, Error> {
        todo!()
    }

    /// Returns the [`RegisterSpace`] of the [`Reg`].
    ///
    /// Returns `None` if the [`Reg`] is unknown to the [`Stack`].
    pub fn reg_space(&self, reg: Reg) -> Option<RegisterSpace> {
        todo!()
    }
}

/// The [`RegisterSpace`] of a [`Reg`].
#[derive(Debug, Copy, Clone)]
pub enum RegisterSpace {
    /// Register referring to a local variable.
    Local,
    /// Register referring to a function local constant value.
    Const,
    /// Register referring to a temporary stack operand.
    Temp,
}

impl RegisterSpace {
    /// Returns `true` if `self` is [`RegisterSpace::Local`].
    pub fn is_local(self) -> bool {
        matches!(self, Self::Local)
    }

    /// Returns `true` if `self` is [`RegisterSpace::Temp`].
    pub fn is_temp(self) -> bool {
        matches!(self, Self::Temp)
    }

    /// Returns `true` if `self` is [`RegisterSpace::Const`].
    pub fn is_const(self) -> bool {
        matches!(self, Self::Const)
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
