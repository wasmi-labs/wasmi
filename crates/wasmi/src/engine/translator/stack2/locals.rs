use crate::{core::ValType, Error};
use alloc::vec::Vec;

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
    /// The total number of registered locals.
    len_locals: usize,
}

impl LocalsRegistry {
    /// Resets `self` for reuse.
    pub fn reset(&mut self) {
        self.groups.clear();
        self.first_group = None;
        self.len_locals = 0;
    }

    /// Registers `amount` of locals of type `ty` for `self`.
    ///
    /// # Errors
    ///
    /// If too many locals are registered.
    pub fn register(&mut self, amount: usize, ty: ValType) -> Result<(), Error> {
        if amount == 0 {
            return Ok(());
        }
        let first_local = self.len_locals;
        if amount > 1 {
            self.first_group.get_or_insert(first_local);
        }
        self.groups.push(LocalGroup { first_local, ty });
        self.len_locals += amount;
        Ok(())
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
        // slow path using binary search
        todo!()
    }
}

/// A local group of one or more locals sharing a common type.
#[derive(Debug, Copy, Clone)]
struct LocalGroup {
    /// The local index of the first local in the group.
    first_local: usize,
    /// The shared type of the locals in the local group.
    ty: ValType,
}

impl LocalGroup {
    /// Returns the [`ValType`] of the [`LocalGroup`].
    fn ty(&self) -> ValType {
        self.ty
    }
}
