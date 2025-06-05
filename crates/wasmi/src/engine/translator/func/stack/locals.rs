use crate::{ir::Reg, Error};
use alloc::{
    collections::{btree_map, BTreeMap},
    vec::Vec,
};
use core::mem;

#[cfg(doc)]
use super::ProviderStack;

/// The index of a `local.get` on the [`ProviderStack`].
pub type StackIndex = usize;

/// The index of an entry in the [`LocalRefs`] data structure.
type EntryIndex = usize;

/// Data structure to store local indices on the compilation stack for large stacks.
///
/// # Note
///
/// - The main purpose is to provide efficient implementations to preserve locals on
///   the compilation stack for single local and mass local preservation.
/// - Also this data structure is critical to not be attackable by malicious actors
///   when operating on very large stacks or local variable quantities.
#[derive(Debug, Default)]
pub struct LocalRefs {
    /// The last local added to [`LocalRefs`] per local variable if any.
    locals_last: BTreeMap<Reg, EntryIndex>,
    /// The entries of the [`LocalRefs`] data structure.
    entries: LocalRefsEntries,
}

/// The entries of the [`LocalRefs`] data structure.
///
/// # Note
///
/// This type mostly exists to gracefully resolve some borrow-checking issues
/// when operating on parts of the fields of the [`LocalRefs`] while `locals_last`
/// is borrowed.
#[derive(Debug, Default)]
pub struct LocalRefsEntries {
    /// The index of the next free (vacant) entry.
    next_free: Option<EntryIndex>,
    /// All entries of the [`LocalRefs`] data structure.
    entries: Vec<LocalRefEntry>,
}

impl LocalRefsEntries {
    /// Resets the [`LocalRefs`].
    pub fn reset(&mut self) {
        self.next_free = None;
        self.entries.clear();
    }

    /// Returns the next free [`EntryIndex`] for reusing vacant entries.
    #[inline]
    pub fn next_free(&self) -> Option<EntryIndex> {
        self.next_free
    }

    /// Returns the next [`EntryIndex`] for the next new non-reused entry.
    #[inline]
    pub fn next_index(&self) -> EntryIndex {
        self.entries.len()
    }

    /// Pushes an occupied entry to the [`LocalRefsEntries`].
    #[inline]
    pub fn push_occupied(&mut self, slot: StackIndex, prev: Option<EntryIndex>) -> EntryIndex {
        let index = self.next_index();
        self.entries.push(LocalRefEntry::Occupied { slot, prev });
        index
    }

    /// Reuses the vacant entry at `index` for a new occupied entry.
    ///
    /// # Panics
    ///
    /// If the entry at `index` is not vacant.
    #[inline]
    pub fn reuse_vacant(&mut self, index: EntryIndex, slot: StackIndex, prev: Option<EntryIndex>) {
        let old_entry = mem::replace(
            &mut self.entries[index],
            LocalRefEntry::Occupied { slot, prev },
        );
        self.next_free = match old_entry {
            LocalRefEntry::Vacant { next_free } => next_free,
            occupied @ LocalRefEntry::Occupied { .. } => {
                panic!("tried to reuse occupied entry at index {index}: {occupied:?}")
            }
        };
    }

    /// Removes the entry at the given `index`.
    ///
    /// Returns the entry index of the next entry in the list and the
    /// [`StackIndex`] associated to the removed entry.
    #[inline]
    fn remove_entry(&mut self, index: EntryIndex) -> (Option<EntryIndex>, StackIndex) {
        let next_free = self.next_free();
        let old_entry = mem::replace(
            &mut self.entries[index],
            LocalRefEntry::Vacant { next_free },
        );
        let LocalRefEntry::Occupied { prev, slot } = old_entry else {
            panic!("expected occupied entry but found vacant: {old_entry:?}");
        };
        self.next_free = Some(index);
        (prev, slot)
    }
}

/// An entry representing a local variable on the compilation stack or a vacant entry.
#[derive(Debug, Copy, Clone)]
enum LocalRefEntry {
    Vacant {
        /// The next free slot of the [`LocalRefs`] data structure.
        next_free: Option<EntryIndex>,
    },
    Occupied {
        /// The slot index of the local variable on the compilation stack.
        slot: StackIndex,
        /// The next [`LocalRefEntry`] referencing the same local variable if any.
        prev: Option<EntryIndex>,
    },
}

impl LocalRefs {
    /// Resets the [`LocalRefs`].
    pub fn reset(&mut self) {
        self.locals_last.clear();
        self.entries.reset();
    }

    /// Registers an `amount` of function inputs or local variables.
    ///
    /// # Errors
    ///
    /// If too many registers have been registered.
    pub fn register_locals(&mut self, _amount: u32) {
        // Nothing to do here.
    }

    /// Updates the last index for `local` to `index` and returns the previous last index.
    fn update_last(&mut self, index: EntryIndex, local: Reg) -> Option<EntryIndex> {
        match self.locals_last.entry(local) {
            btree_map::Entry::Vacant(entry) => {
                entry.insert(index);
                None
            }
            btree_map::Entry::Occupied(mut entry) => {
                let prev = *entry.get();
                entry.insert(index);
                Some(prev)
            }
        }
    }

    /// Pushes the stack index of a `local.get` on the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// If the `local` index is out of bounds.
    pub fn push_at(&mut self, local: Reg, slot: StackIndex) {
        match self.entries.next_free() {
            Some(index) => {
                let prev = self.update_last(index, local);
                self.entries.reuse_vacant(index, slot, prev);
            }
            None => {
                let index = self.entries.next_index();
                let prev = self.update_last(index, local);
                let pushed = self.entries.push_occupied(slot, prev);
                debug_assert_eq!(pushed, index);
            }
        };
    }

    /// Returns `true` if `self` is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.locals_last.is_empty()
    }

    /// Reset `self` if `self` is empty.
    #[inline]
    fn reset_if_empty(&mut self) {
        if self.is_empty() {
            self.entries.reset();
        }
    }

    /// Pops the stack index of a `local.get` on the [`ProviderStack`].
    ///
    /// # Panics
    ///
    /// - If the `local` index is out of bounds.
    /// - If there is no `local.get` stack index on the stack.
    pub fn pop_at(&mut self, local: Reg) -> StackIndex {
        let btree_map::Entry::Occupied(mut last) = self.locals_last.entry(local) else {
            panic!("missing stack index for local on the provider stack: {local:?}")
        };
        let index = *last.get();
        let (prev, slot) = self.entries.remove_entry(index);
        match prev {
            Some(prev) => last.insert(prev),
            None => last.remove(),
        };
        self.reset_if_empty();
        slot
    }

    /// Drains all local indices of the `local` variable on the [`ProviderStack`].
    ///
    /// # Note
    ///
    /// Calls `f` with the index of each local on the [`ProviderStack`] that matches `local`.
    pub fn drain_at(
        &mut self,
        local: Reg,
        f: impl FnMut(StackIndex) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let Some(last) = self.locals_last.remove(&local) else {
            return Ok(());
        };
        self.drain_list_at(last, f)?;
        self.reset_if_empty();
        Ok(())
    }

    /// Drains all local indices on the [`ProviderStack`].
    ///
    /// # Note
    ///
    /// Calls `f` with the pair of local and its index of each local on the [`ProviderStack`].
    pub fn drain_all(
        &mut self,
        mut f: impl FnMut(Reg, StackIndex) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let local_last = mem::take(&mut self.locals_last);
        for (local, last) in &local_last {
            let local = *local;
            self.drain_list_at(*last, |index| f(local, index))?;
        }
        self.locals_last = local_last;
        self.locals_last.clear();
        self.entries.reset();
        Ok(())
    }

    /// Drains the list of locals starting at `index` at the entries array.
    #[inline]
    fn drain_list_at(
        &mut self,
        index: EntryIndex,
        mut f: impl FnMut(StackIndex) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut last = Some(index);
        while let Some(index) = last {
            let (prev, slot) = self.entries.remove_entry(index);
            last = prev;
            f(slot)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg(index: i16) -> Reg {
        Reg::from(index)
    }

    #[test]
    fn push_pop_works() {
        let mut locals = LocalRefs::default();
        locals.push_at(reg(0), 2);
        locals.push_at(reg(0), 4);
        locals.push_at(reg(1), 6);
        locals.push_at(reg(2), 8);
        locals.push_at(reg(5), 10);
        locals.push_at(reg(1), 12);
        locals.push_at(reg(0), 14);
        assert_eq!(locals.pop_at(reg(0)), 14);
        assert_eq!(locals.pop_at(reg(0)), 4);
        assert_eq!(locals.pop_at(reg(0)), 2);
        assert_eq!(locals.pop_at(reg(1)), 12);
        assert_eq!(locals.pop_at(reg(1)), 6);
        assert_eq!(locals.pop_at(reg(2)), 8);
        assert_eq!(locals.pop_at(reg(5)), 10);
    }
}
