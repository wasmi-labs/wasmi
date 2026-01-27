use super::{LocalIdx, Operand, Reset};
use crate::{
    Error,
    ValType,
    engine::{
        TranslationError,
        translator::{func::LocalOperand, utils::required_cells_for_ty},
    },
    ir::{BoundedSlotSpan, Slot, SlotSpan},
};
use alloc::vec::Vec;

#[cfg(doc)]
use super::Stack;

/// Allows conversion from `Self` to [`LocalIdx`].
///
/// # Note
///
/// This allows to use [`StackLayout::local_to_slot`] with [`LocalIdx`] and [`LocalOperand`].
pub trait IntoLocalIdx: Copy {
    /// Converts `self` into [`LocalIdx`].
    fn into_local_idx(self) -> LocalIdx;
}

impl IntoLocalIdx for LocalIdx {
    #[inline]
    fn into_local_idx(self) -> LocalIdx {
        self
    }
}

impl IntoLocalIdx for LocalOperand {
    #[inline]
    fn into_local_idx(self) -> LocalIdx {
        self.local_index()
    }
}

impl IntoLocalIdx for &'_ LocalOperand {
    #[inline]
    fn into_local_idx(self) -> LocalIdx {
        self.local_index()
    }
}

pub trait LocalValType {
    fn ty(self) -> ValType;
}

impl LocalValType for LocalOperand {
    fn ty(self) -> ValType {
        LocalOperand::ty(&self)
    }
}

impl LocalValType for &'_ LocalOperand {
    fn ty(self) -> ValType {
        LocalOperand::ty(self)
    }
}

/// The layout of the [`Stack`] for local operands.
#[derive(Debug, Default)]
pub struct StackLayout {
    /// The cell offsets of each local variable of the function.
    local_offsets: Vec<u16>,
    /// The minimum cell offset for temporary operands.
    ///
    /// # Note
    ///
    /// This offset is always the smallest offset greater than all local offsets.
    min_temp_offset: u16,
}

impl Reset for StackLayout {
    fn reset(&mut self) {
        self.local_offsets.clear();
        self.min_temp_offset = 0;
    }
}

impl StackLayout {
    /// Returns the number of locals registers to `self`.
    fn len_locals(&self) -> usize {
        self.local_offsets.len()
    }

    /// Slot `amount` local variables of common type `ty`.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: usize, ty: ValType) -> Result<(), Error> {
        let cells_per_local = required_cells_for_ty(ty);
        let err_too_many_slots = || Error::from(TranslationError::AllocatedTooManySlots);
        let delta_offset = amount
            .checked_mul(usize::from(cells_per_local))
            .ok_or_else(err_too_many_slots)?;
        usize::from(self.min_temp_offset)
            .checked_add(delta_offset)
            .filter(|&new_max| new_max <= usize::from(u16::MAX))
            .ok_or_else(err_too_many_slots)?;
        for _ in 0..amount {
            self.local_offsets.push(self.min_temp_offset);
            self.min_temp_offset += cells_per_local;
        }
        Ok(())
    }

    /// Returns the [`StackSpace`] of the [`Slot`].
    ///
    /// Returns `None` if the [`Slot`] is unknown to the [`Stack`].
    #[must_use]
    pub fn stack_space(&self, slot: Slot) -> StackSpace {
        let index = u16::from(slot);
        if index < self.min_temp_offset {
            return StackSpace::Local;
        }
        StackSpace::Temp
    }

    /// Converts the `operand` into the associated [`Slot`].
    ///
    /// # Note
    ///
    /// Forwards to [`StackLayout::local_to_slot`] if possible.
    ///
    ///
    /// # Errors
    ///
    /// If the forwarded method returned an error.
    ///
    /// # Panics
    ///
    /// If `operand` is an [`ImmediateOperand`].
    ///
    /// [`ImmediateOperand`]: crate::engine::translator::func::ImmediateOperand
    pub fn operand_to_slot(&mut self, operand: Operand) -> Result<Slot, Error> {
        match operand {
            Operand::Local(operand) => self.local_to_slot(operand),
            Operand::Temp(operand) => Ok(operand.temp_slots().head()),
            Operand::Immediate(operand) => {
                panic!("cannot convert `ImmediateOperand` to stack `Slot` but got: {operand:?}")
            }
        }
    }

    /// Converts the local `index` into the associated [`Slot`].
    ///
    /// # Errors
    ///
    /// If `index` cannot be converted into a [`Slot`].
    #[inline]
    pub fn local_to_slot(&self, item: impl IntoLocalIdx) -> Result<Slot, Error> {
        // TODO: replace usage with `local_to_slots` method
        let index = item.into_local_idx();
        debug_assert!(
            (u32::from(index) as usize) < self.len_locals(),
            "out of bounds local operand index: {index:?}"
        );
        let Ok(index) = u16::try_from(u32::from(index)) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        let offset = self.local_offsets[usize::from(index)];
        Ok(Slot::from(offset))
    }

    /// Converts the local `index` into the associated [`Slot`].
    ///
    /// # Errors
    ///
    /// If `index` cannot be converted into a [`Slot`].
    #[inline]
    pub fn local_to_slots<L>(&self, item: L) -> Result<BoundedSlotSpan, Error>
    where
        L: IntoLocalIdx + LocalValType,
    {
        let head = self.local_to_slot(item)?;
        let len = required_cells_for_ty(item.ty());
        Ok(BoundedSlotSpan::new(SlotSpan::new(head), len))
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
