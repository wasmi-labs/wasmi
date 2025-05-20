use super::OperandIdx;
use crate::{core::ValType, engine::TranslationError, Error};
use alloc::vec::Vec;
use core::{cmp, iter};

/// A local variable index.
#[derive(Debug, Copy, Clone)]
pub struct LocalIdx(u32);

impl From<u32> for LocalIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl From<LocalIdx> for u32 {
    fn from(index: LocalIdx) -> Self {
        index.0
    }
}

/// Stores definitions of locals.
#[derive(Debug, Default, Clone)]
pub struct LocalsRegistry {
    /// The types of the first defined local variables.
    tys_first: Vec<ValType>,
    /// The types of the remaining defined local variables.
    tys_remaining: Vec<LocalGroup>,
    /// The first operand for the local on the stack if any.
    first_operands: Vec<Option<OperandIdx>>,
}

impl LocalsRegistry {
    /// Resets `self` for reuse.
    pub fn reset(&mut self) {
        self.tys_first.clear();
        self.tys_remaining.clear();
        self.first_operands.clear();
    }

    /// Returns the number of registered local variables in `self`.
    pub fn len(&self) -> usize {
        self.first_operands.len()
    }

    /// The maximum number of local variables per function.
    const LOCAL_VARIABLES_MAX: usize = 30_000;

    /// The maximum number of local variables in the fast `tys_first` vector.
    const FIRST_TYS_MAX: usize = 100;

    /// Registers `amount` of locals of type `ty` for `self`.
    ///
    /// # Errors
    ///
    /// If too many locals are registered.
    pub fn register(&mut self, amount: u32, ty: ValType) -> Result<(), Error> {
        if amount == 0 {
            return Ok(());
        }
        let Ok(amount) = usize::try_from(amount) else {
            panic!(
                "failed to register {amount} local variables of type {ty:?}: out of bounds `usize`"
            )
        };
        if self.len().saturating_add(amount) > Self::LOCAL_VARIABLES_MAX {
            return Err(Error::from(TranslationError::TooManyFunctionParams));
        }
        let vacant_first = Self::FIRST_TYS_MAX.saturating_sub(self.tys_first.len());
        let push_to_first = cmp::min(vacant_first, amount);
        self.tys_first.extend(iter::repeat_n(ty, push_to_first));
        let remaining_amount = amount - push_to_first;
        let remaining_index = (self.len() + amount - 1) as u32;
        if remaining_amount > 0 {
            self.tys_remaining
                .push(LocalGroup::new(remaining_index, ty));
        }
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

    /// Returns the first operand for this local on the stack if any.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds.
    pub fn first_operand(&self, index: LocalIdx) -> Option<OperandIdx> {
        self.first_operands[Self::local_idx_to_index(index)]
    }

    /// Takes the first operand for this local from the stack if any.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds.
    pub fn take_first_operand(&mut self, index: LocalIdx) -> Option<OperandIdx> {
        let index = Self::local_idx_to_index(index);
        let cell = &mut self.first_operands[index];
        cell.take()
    }

    /// Replaces the first operand for this local on the stack and returns the old one.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds.
    pub fn replace_first_operand(
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

    /// Returns the type of the local variable at `index` if any.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds and does not refer to a local in `self`.
    pub fn ty(&self, index: LocalIdx) -> ValType {
        let index_sz = Self::local_idx_to_index(index);
        match self.tys_first.get(index_sz) {
            Some(ty) => *ty,
            None => self
                .ty_slow(index)
                .unwrap_or_else(|| panic!("out of bounds local index: {index:?}")),
        }
    }

    /// Returns the type of the local variable at `index` if any.
    ///
    /// This is the slow-path for local variables that have been stored in the `remaining` buffer.
    #[cold]
    fn ty_slow(&self, index: LocalIdx) -> Option<ValType> {
        if self.tys_remaining.is_empty() {
            return None;
        }
        match self
            .tys_remaining
            .binary_search_by_key(&index.0, LocalGroup::max_index)
        {
            Err(i) if i == self.tys_remaining.len() => None,
            Ok(i) | Err(i) => Some(self.tys_remaining[i].ty()),
        }
    }
}

/// A local group of one or more locals sharing a common type.
#[derive(Debug, Copy, Clone)]
struct LocalGroup {
    /// The local index of the first local in the group.
    max_index: u32,
    /// The shared type of the locals in the local group.
    ty: ValType,
}

impl LocalGroup {
    /// Creates a new [`LocalGroup`].
    fn new(max_index: u32, ty: ValType) -> Self {
        Self { max_index, ty }
    }

    /// Returns the maximum index of the local variables in the [`LocalGroup`].
    fn max_index(&self) -> u32 {
        self.max_index
    }

    /// Returns the [`ValType`] of the [`LocalGroup`].
    fn ty(&self) -> ValType {
        self.ty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ty_works() {
        let mut locals = LocalsRegistry::default();
        for locals_per_type in [1, 2, 10, 100] {
            locals.reset();
            let tys = [ValType::I32, ValType::I64, ValType::F32, ValType::F64];
            for ty in tys {
                locals.register(locals_per_type, ty).unwrap();
            }
            let locals_per_type = locals_per_type as usize;
            assert_eq!(locals.len(), locals_per_type * tys.len());
            for i in 0..locals.len() {
                assert_eq!(locals.ty(LocalIdx(i as u32)), tys[i / locals_per_type]);
            }
        }
    }

    #[test]
    fn locals_followed_by_groups() {
        let mut locals = LocalsRegistry::default();
        let len_single = [1, 10, 100];
        let len_groups = [1, 10, 100];
        let locals_per_group = [10, 100];
        for len_single in len_single {
            for len_groups in len_groups {
                for locals_per_group in locals_per_group {
                    locals.reset();
                    let len_locals = len_single + (len_groups * locals_per_group);
                    for _ in 0..len_single {
                        locals.register(1, ValType::I32).unwrap();
                    }
                    for _ in 0..len_groups {
                        locals.register(locals_per_group, ValType::I64).unwrap();
                    }
                    for i in 0..len_locals {
                        let ty = match i < len_single {
                            true => ValType::I32,
                            false => ValType::I64,
                        };
                        assert_eq!(locals.ty(LocalIdx(i)), ty);
                    }
                }
            }
        }
    }
}
