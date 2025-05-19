use super::OperandIdx;
use crate::{core::ValType, Error};
use alloc::vec::Vec;
use core::{cmp::Ordering, iter};

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
    /// Groups of local variables sharing a common type.
    groups: Vec<LocalGroup>,
    /// The index of the first local variable in a group of more than 1 locals.
    ///
    /// # Note
    ///
    /// All local indices that are smaller than this can be queried faster.
    first_group: Option<usize>,
    /// The first operand for the local on the stack if any.
    first_operands: Vec<Option<OperandIdx>>,
}

impl LocalsRegistry {
    /// Resets `self` for reuse.
    pub fn reset(&mut self) {
        self.groups.clear();
        self.first_group = None;
        self.first_operands.clear();
    }

    /// Returns the number of registered local variables in `self`.
    pub fn len(&self) -> usize {
        self.first_operands.len()
    }

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
        let first_local = self.len();
        if amount > 1 {
            self.first_group.get_or_insert(first_local);
        }
        let last_local = first_local + amount;
        self.groups.push(LocalGroup {
            first_local,
            last_local,
            ty,
        });
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
        let index = Self::local_idx_to_index(index);
        let Some(cell) = self.first_operands.get(index) else {
            panic!("`first_operand`: out of bounds local index: {index:?}")
        };
        *cell
    }

    /// Takes the first operand for this local from the stack if any.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds.
    pub fn take_first_operand(&mut self, index: LocalIdx) -> Option<OperandIdx> {
        let index = Self::local_idx_to_index(index);
        let Some(cell) = self.first_operands.get_mut(index) else {
            panic!("`first_operand`: out of bounds local index: {index:?}")
        };
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
        let Some(cell) = self.first_operands.get_mut(index) else {
            panic!("`first_operand`: out of bounds local index: {index:?}")
        };
        match first_operand {
            Some(first_operand) => cell.replace(first_operand),
            None => cell.take(),
        }
    }

    /// Returns the type of the `index` if any.
    ///
    /// Returns `None` if `index` does not refer to any local in `self`.
    pub fn ty(&self, index: LocalIdx) -> Option<ValType> {
        let index = Self::local_idx_to_index(index);
        if let Some(first_group) = self.first_group {
            if index >= first_group {
                return self.ty_slow(index);
            }
        }
        self.ty_fast(index)
    }

    /// Returns the [`ValType`] of the local at `local_index`.
    ///
    /// # Note
    ///
    /// This is the fast version used for locals with indices
    /// smaller than the first local group.
    #[inline]
    fn ty_fast(&self, local_index: usize) -> Option<ValType> {
        self.groups.get(local_index).map(LocalGroup::ty)
    }

    /// Returns the [`ValType`] of the local at `local_index`.
    ///
    /// # Note
    ///
    /// This is the slow version used for locals with indices
    /// equal to or greater than the first local group.
    #[cold]
    fn ty_slow(&self, local_index: usize) -> Option<ValType> {
        let Some(first_group) = self.first_group else {
            unreachable!("cannot use `ty_slow` with `first_group` being `None`");
        };
        let groups = &self.groups[first_group..];
        if groups.is_empty() {
            return None;
        }
        let index = groups
            .binary_search_by(|group| {
                if local_index < group.first_local {
                    Ordering::Greater
                } else if local_index > group.last_local {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .ok()?;
        let ty = groups[index].ty();
        Some(ty)
    }
}

/// A local group of one or more locals sharing a common type.
#[derive(Debug, Copy, Clone)]
struct LocalGroup {
    /// The local index of the first local in the group.
    first_local: usize,
    /// The local index right after the last local in the group.
    last_local: usize,
    /// The shared type of the locals in the local group.
    ty: ValType,
}

impl LocalGroup {
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
        for locals_per_type in [1, 2, 10] {
            let mut locals = LocalsRegistry::default();
            let tys = [ValType::I32, ValType::I64, ValType::F32, ValType::F64];
            for ty in tys {
                locals.register(locals_per_type, ty).unwrap();
            }
            let locals_per_type = locals_per_type as usize;
            assert_eq!(locals.len(), locals_per_type * tys.len());
            for i in 0..locals.len() {
                assert_eq!(
                    locals.ty(LocalIdx(i as u32)),
                    Some(tys[i / locals_per_type])
                );
            }
        }
    }

    #[test]
    fn locals_followed_by_groups() {
        let mut locals = LocalsRegistry::default();
        let len_single = 10;
        let len_groups = 10;
        let locals_per_group = 100;
        let len_locals = len_single + len_groups * locals_per_group;
        for i in 0..len_single {
            locals.register(1, ValType::I32).unwrap();
        }
        for i in 0..len_groups {
            locals.register(locals_per_group, ValType::I64).unwrap();
        }
        for i in 0..len_locals {
            let ty = match i < len_single {
                true => ValType::I32,
                false => ValType::I64,
            };
            assert_eq!(locals.ty(LocalIdx(i)), Some(ty));
        }
    }
}
