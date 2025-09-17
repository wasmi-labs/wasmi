use super::{LocalIdx, Operand, OperandIdx, Reset};
use crate::{
    engine::{
        translator::func::{LocalOperand, TempOperand},
        TranslationError,
    },
    ir::Slot,
    Error,
};

#[cfg(doc)]
use super::Stack;

/// Allows conversion from `Self` to [`LocalIdx`].
///
/// # Note
///
/// This allows to use [`StackLayout::local_to_reg`] with [`LocalIdx`] and [`LocalOperand`].
pub trait IntoLocalIdx {
    /// Converts `self` into [`LocalIdx`].
    fn into_local_idx(self) -> LocalIdx;
}

impl IntoLocalIdx for LocalIdx {
    fn into_local_idx(self) -> LocalIdx {
        self
    }
}

impl IntoLocalIdx for LocalOperand {
    fn into_local_idx(self) -> LocalIdx {
        self.local_index()
    }
}

/// Allows conversion from `Self` to [`OperandIdx`].
///
/// # Note
///
/// This allows to use [`StackLayout::temp_to_reg`] with [`LocalIdx`] and [`TempOperand`].
pub trait IntoOperandIdx {
    /// Converts `self` into [`OperandIdx`].
    fn into_operand_idx(self) -> OperandIdx;
}

impl IntoOperandIdx for OperandIdx {
    fn into_operand_idx(self) -> OperandIdx {
        self
    }
}

impl IntoOperandIdx for TempOperand {
    fn into_operand_idx(self) -> OperandIdx {
        self.operand_index()
    }
}

/// The layout of the [`Stack`].
#[derive(Debug, Default)]
pub struct StackLayout {
    /// The number of locals registered to the function.
    len_locals: usize,
}

impl Reset for StackLayout {
    fn reset(&mut self) {
        self.len_locals = 0;
    }
}

impl StackLayout {
    /// Slot `amount` local variables of common type `ty`.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: usize) -> Result<(), Error> {
        self.len_locals += amount;
        Ok(())
    }

    /// Returns the [`StackSpace`] of the [`Slot`].
    ///
    /// Returns `None` if the [`Slot`] is unknown to the [`Stack`].
    #[must_use]
    pub fn stack_space(&self, slot: Slot) -> StackSpace {
        let index = u16::from(slot);
        if usize::from(index) < self.len_locals {
            return StackSpace::Local;
        }
        StackSpace::Temp
    }

    /// Converts the `operand` into the associated [`Slot`].
    ///
    /// # Note
    ///
    /// Forwards to one of
    ///
    /// - [`StackLayout::local_to_reg`]
    /// - [`StackLayout::temp_to_reg`]
    ///
    /// # Errors
    ///
    /// If the forwarded method returned an error.
    pub fn operand_to_reg(&mut self, operand: Operand) -> Result<Slot, Error> {
        match operand {
            Operand::Local(operand) => self.local_to_reg(operand.local_index()),
            Operand::Temp(operand) => self.temp_to_reg(operand.operand_index()),
            Operand::Immediate(_) => panic!("function local constants have been removed"), // TODO: remove
        }
    }

    /// Converts the local `index` into the associated [`Slot`].
    ///
    /// # Errors
    ///
    /// If `index` cannot be converted into a [`Slot`].
    #[inline]
    pub fn local_to_reg(&self, item: impl IntoLocalIdx) -> Result<Slot, Error> {
        let index = item.into_local_idx();
        debug_assert!(
            (u32::from(index) as usize) < self.len_locals,
            "out of bounds local operand index: {index:?}"
        );
        let Ok(index) = u16::try_from(u32::from(index)) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        Ok(Slot::from(index))
    }

    /// Converts the operand `index` into the associated [`Slot`].
    ///
    /// # Errors
    ///
    /// If `index` cannot be converted into a [`Slot`].
    #[inline]
    pub fn temp_to_reg(&self, item: impl IntoOperandIdx) -> Result<Slot, Error> {
        let index = item.into_operand_idx();
        let index = usize::from(index);
        let Some(index) = index.checked_add(self.len_locals) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        let Ok(index) = u16::try_from(index) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        Ok(Slot::from(index))
    }
}

/// The [`StackSpace`] of a [`Slot`].
#[derive(Debug, Copy, Clone)]
pub enum StackSpace {
    /// Stack slot referring to a local variable.
    Local,
    /// Stack slot referring to a temporary stack operand.
    Temp,
}
