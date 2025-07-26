use super::{OperandIdx, Reset};
use crate::{engine::translator::func2::LocalIdx, Error};
use alloc::vec::Vec;
use core::iter;

/// Store the index of the first occurence on the stack for every local variable.
#[derive(Debug, Default)]
pub struct LocalsHead {
    /// The index of the first occurence of every local variable.
    first_operands: Vec<Option<OperandIdx>>,
}

impl Reset for LocalsHead {
    fn reset(&mut self) {
        self.first_operands.clear();
    }
}

impl LocalsHead {
    /// Registers `amount` of locals.
    ///
    /// # Errors
    ///
    /// If too many locals are registered.
    pub fn register(&mut self, amount: usize) -> Result<(), Error> {
        self.first_operands.extend(iter::repeat_n(None, amount));
        Ok(())
    }

    /// Converts `index` into a `usize` value.
    fn local_idx_to_index(index: LocalIdx) -> usize {
        let index = u32::from(index);
        let Ok(index) = usize::try_from(index) else {
            panic!("out of bounds `LocalIdx`: {index}")
        };
        index
    }

    /// Replaces the first operand for this local on the stack and returns the old one.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds.
    pub fn replace_first(
        &mut self,
        index: LocalIdx,
        first_operand: Option<OperandIdx>,
    ) -> Option<OperandIdx> {
        let index = Self::local_idx_to_index(index);
        let cell = &mut self.first_operands[index];
        match first_operand {
            Some(first_operand) => cell.replace(first_operand),
            None => cell.take(),
        }
    }
}
