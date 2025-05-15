use super::OperandIdx;
use crate::{core::ValType, Error};
use alloc::vec::Vec;
use core::{cmp::Ordering, iter};

/// A local variable index.
#[derive(Debug, Copy, Clone)]
pub struct LocalIdx(usize);

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

    /// Returns the first operand for this local on the stack if any.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds.
    fn first_operand(&self, index: LocalIdx) -> Option<OperandIdx> {
        let Some(cell) = self.first_operands.get(index.0) else {
            panic!("`first_operand`: out of bounds local index: {index:?}")
        };
        *cell
    }

    /// Takes the first operand for this local from the stack if any.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds.
    fn take_first_operand(&mut self, index: LocalIdx) -> Option<OperandIdx> {
        let Some(cell) = self.first_operands.get_mut(index.0) else {
            panic!("`first_operand`: out of bounds local index: {index:?}")
        };
        cell.take()
    }

    /// Returns an exclusive reference to the first operand for this local on the stack.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds.
    fn set_first_operand(
        &mut self,
        index: LocalIdx,
        first_operand: OperandIdx,
    ) -> Option<OperandIdx> {
        let Some(cell) = self.first_operands.get_mut(index.0) else {
            panic!("`first_operand`: out of bounds local index: {index:?}")
        };
        cell.replace(first_operand)
    }

    /// Returns the type of the `local_index` if any.
    ///
    /// Returns `None` if `local_index` does not refer to any local in `self`.
    pub fn ty(&self, local_index: LocalIdx) -> Option<ValType> {
        if let Some(first_group) = self.first_group {
            if local_index.0 >= first_group {
                return self.ty_slow(local_index);
            }
        }
        self.ty_fast(local_index)
    }

    /// Returns the [`ValType`] of the local at `local_index`.
    ///
    /// # Note
    ///
    /// This is the fast version used for locals with indices
    /// smaller than the first local group.
    #[inline]
    fn ty_fast(&self, local_index: LocalIdx) -> Option<ValType> {
        self.groups.get(local_index.0).map(LocalGroup::ty)
    }

    /// Returns the [`ValType`] of the local at `local_index`.
    ///
    /// # Note
    ///
    /// This is the slow version used for locals with indices
    /// equal to or greater than the first local group.
    #[cold]
    fn ty_slow(&self, local_index: LocalIdx) -> Option<ValType> {
        let Some(first_group) = self.first_group else {
            unreachable!("cannot use `ty_slow` with `first_group` being `None`");
        };
        let groups = &self.groups[first_group..];
        if groups.is_empty() {
            return None;
        }
        let local_index = local_index.0;
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
                assert_eq!(locals.ty(LocalIdx(i)), Some(tys[i / locals_per_type]));
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
            assert_eq!(locals.ty(LocalIdx(i as usize)), Some(ty));
        }
    }
}
