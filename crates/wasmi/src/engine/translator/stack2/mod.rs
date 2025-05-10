mod consts;
mod locals;

use self::{consts::ConstRegistry, locals::LocalsRegistry};
use crate::core::{TypedVal, ValType};
use alloc::vec::Vec;
use core::num::NonZeroUsize;

#[derive(Debug, Clone)]
/// The Wasm value stack during translation from Wasm to Wasmi bytecode.
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

/// An [`Operand`] on the [`Stack`].
/// 
/// This is the internal version of [`Operand`] with information that shall remain
/// hidden to the outside.
#[derive(Debug, Copy, Clone)]
enum StackOperand {
    /// A local variable.
    Local {
        /// The index of the local variable.
        index: LocalIdx,
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
    /// The index of the local variable.
    index: LocalIdx,
    /// The type of the local variable.
    ty: ValType,
}

impl LocalOperand {
    /// Returns the index of the local variable.
    pub fn index(self) -> LocalIdx {
        self.index
    }

    /// Returns the type of the local variable.
    pub fn ty(self) -> ValType {
        self.ty
    }
}

/// A temporary on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct TempOperand {
    /// The type of the temporary.
    ty: ValType,
}

impl TempOperand {
    /// Returns the type of the temporary.
    pub fn ty(self) -> ValType {
        self.ty
    }
}

/// An immediate value on the [`Stack`].
#[derive(Debug, Copy, Clone)]
pub struct ImmediateOperand {
    /// The value and type of the immediate value.
    val: TypedVal,
}

impl ImmediateOperand {
    /// Returns the value (and type) of the immediate value.
    pub fn val(self) -> TypedVal {
        self.val
    }

    /// Returns the type of the immediate value.
    pub fn ty(self) -> ValType {
        self.val.ty()
    }
}

/// A local variable index.
#[derive(Debug, Copy, Clone)]
pub struct LocalIdx(usize);
