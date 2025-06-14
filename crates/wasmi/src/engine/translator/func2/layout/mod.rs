mod consts;

use self::consts::{ConstRegistry, ConstRegistryIter};
use super::{LocalIdx, Operand, OperandIdx, Reset};
use crate::{
    core::{UntypedVal, ValType},
    engine::TranslationError,
    ir::Reg,
    Error,
};

#[cfg(doc)]
use super::Stack;

/// The layout of the [`Stack`].
#[derive(Debug, Default)]
pub struct StackLayout {
    /// The number of locals registered to the function.
    len_locals: usize,
    /// All function local constants.
    consts: ConstRegistry,
}

impl Reset for StackLayout {
    fn reset(&mut self) {
        self.len_locals = 0;
        self.consts.reset();
    }
}

impl StackLayout {
    /// Register `amount` local variables of common type `ty`.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: u32, _ty: ValType) -> Result<(), Error> {
        self.len_locals += amount as usize;
        Ok(())
    }

    /// Returns the [`StackSpace`] of the [`Reg`].
    ///
    /// Returns `None` if the [`Reg`] is unknown to the [`Stack`].
    #[must_use]
    pub fn stack_space(&self, reg: Reg) -> StackSpace {
        let index = i16::from(reg);
        if index.is_negative() {
            return StackSpace::Const;
        }
        let index = index as u16;
        if usize::from(index) < self.len_locals {
            return StackSpace::Local;
        }
        StackSpace::Temp
    }

    /// Converts the `operand` into the associated [`Reg`].
    ///
    /// # Note
    ///
    /// Forwards to one of
    ///
    /// - [`StackLayout::local_to_reg`]
    /// - [`StackLayout::temp_to_reg`]
    /// - [`StackLayout::const_to_reg`]
    ///
    /// # Errors
    ///
    /// If the forwarded method returned an error.
    pub fn operand_to_reg(&mut self, operand: Operand) -> Result<Reg, Error> {
        match operand {
            Operand::Local(operand) => self.local_to_reg(operand.local_index()),
            Operand::Temp(operand) => self.temp_to_reg(operand.operand_index()),
            Operand::Immediate(operand) => self.const_to_reg(operand.val()),
        }
    }

    /// Converts the local `index` into the associated [`Reg`].
    ///
    /// # Errors
    ///
    /// If `index` cannot be converted into a [`Reg`].
    pub fn local_to_reg(&self, index: LocalIdx) -> Result<Reg, Error> {
        debug_assert!(
            (u32::from(index) as usize) < self.len_locals,
            "out of bounds local operand index: {index:?}"
        );
        let Ok(index) = i16::try_from(u32::from(index)) else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        Ok(Reg::from(index))
    }

    /// Converts the operand `index` into the associated [`Reg`].
    ///
    /// # Errors
    ///
    /// If `index` cannot be converted into a [`Reg`].
    pub fn temp_to_reg(&self, index: OperandIdx) -> Result<Reg, Error> {
        let index = usize::from(index);
        let Some(index) = index.checked_add(self.len_locals) else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        let Ok(index) = i16::try_from(index) else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        Ok(Reg::from(index))
    }

    /// Allocates a function local constant `value`.
    ///
    /// # Errors
    ///
    /// If too many function local constants have been allocated already.
    pub fn const_to_reg(&mut self, value: impl Into<UntypedVal>) -> Result<Reg, Error> {
        self.consts.alloc(value.into())
    }

    /// Returns an iterator yielding all function local constants.
    ///
    /// # Note
    ///
    /// The function local constant are yielded in reverse order of allocation.
    pub fn consts(&self) -> ConstRegistryIter {
        self.consts.iter()
    }
}

/// The [`StackSpace`] of a [`Reg`].
#[derive(Debug, Copy, Clone)]
pub enum StackSpace {
    /// Stack slot referring to a local variable.
    Local,
    /// Stack slot referring to a function local constant value.
    Const,
    /// Stack slot referring to a temporary stack operand.
    Temp,
}

impl StackSpace {
    /// Returns `true` if `self` is [`StackSpace::Local`].
    #[inline]
    pub fn is_local(self) -> bool {
        matches!(self, Self::Local)
    }

    /// Returns `true` if `self` is [`StackSpace::Temp`].
    #[inline]
    pub fn is_temp(self) -> bool {
        matches!(self, Self::Temp)
    }

    /// Returns `true` if `self` is [`StackSpace::Const`].
    #[inline]
    pub fn is_const(self) -> bool {
        matches!(self, Self::Const)
    }
}
