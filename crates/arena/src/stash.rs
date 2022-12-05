use crate::ArenaIndex;
use alloc::vec::Vec;
use core::{
    marker::PhantomData,
    mem::replace,
    ops::{Index, IndexMut},
};

/// A stash arena providing O(1) insertion and indexed deletion.
#[derive(Debug, Default, Clone)]
pub struct StashArena<Idx, T> {
    stash: Stash<T>,
    marker: PhantomData<fn() -> Idx>,
}

impl<Idx, T> StashArena<Idx, T>
where
    Idx: ArenaIndex,
{
    /// Clears the [`StashArena`].
    pub fn clear(&mut self) {
        self.stash.clear()
    }

    /// Returns the amount of values stored in the [`StashArena`].
    pub fn len(&self) -> usize {
        self.stash.len()
    }

    /// Returns `true` if the [`StashArena`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Puts the `value` into the [`StashArena`] and returns a reference to it.
    pub fn put(&mut self, value: T) -> Idx {
        Idx::from_usize(self.stash.put(value).into_index())
    }

    /// Returns the value of the `slot` out of the [`StashArena`] if any.
    ///
    /// This removes the returned value from the [`StashArena`].
    pub fn take(&mut self, index: Idx) -> Option<T> {
        self.stash.take(SlotRef(index.into_usize()))
    }

    /// Returns a shared reference to the value of the `slot` if any.
    pub fn get(&self, index: Idx) -> Option<&T> {
        self.stash.get(SlotRef(index.into_usize()))
    }

    /// Returns a shared reference to the value of the `slot` if any.
    pub fn get_mut(&mut self, index: Idx) -> Option<&mut T> {
        self.stash.get_mut(SlotRef(index.into_usize()))
    }
}

impl<Idx, T> Index<Idx> for StashArena<Idx, T>
where
    Idx: ArenaIndex,
{
    type Output = T;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.stash[SlotRef(index.into_usize())]
    }
}

impl<Idx, T> IndexMut<Idx> for StashArena<Idx, T>
where
    Idx: ArenaIndex,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.stash[SlotRef(index.into_usize())]
    }
}

/// A stash providing O(1) insertion and indexed deletion.
#[derive(Debug, Default, Clone)]
pub struct Stash<T> {
    /// The slots of the stash potentially holding values.
    slots: Vec<Slot<T>>,
    /// The first vacant slot if any.
    first_vacant: Option<SlotRef>,
    /// The number of occupied slots in the stash.
    len_occupied: usize,
}

/// A slot within a [`Stash`].
///
/// Can be either occupied with a value or vacant and referencing the
/// next vacant slot.
#[derive(Debug, Clone)]
enum Slot<T> {
    Vacant { next_vacant: SlotRef },
    Occupied { value: T },
}

/// References a slot in the [`Stash`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SlotRef(usize);

impl SlotRef {
    /// Returns the `usize` index of the [`SlotRef`].
    fn into_index(self) -> usize {
        self.0
    }
}

impl<T> Stash<T> {
    /// Clears the [`Stash`].
    pub fn clear(&mut self) {
        self.slots.clear();
        self.first_vacant = None;
        self.len_occupied = 0;
    }

    /// Returns the amount of values stored in the [`Stash`].
    pub fn len(&self) -> usize {
        self.len_occupied
    }

    /// Returns `true` if the [`Stash`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Asserts that all slots in the [`Stash`] are occupied.
    fn assert_all_slots_occupied(&self) {
        assert_eq!(
            self.len_occupied,
            self.slots.len(),
            "all slots must be occupied"
        );
    }

    /// Puts the `value` into the [`Stash`] and returns a reference to it.
    pub fn put(&mut self, value: T) -> SlotRef {
        let index = match self.first_vacant {
            None => {
                // Case: All slots are occupied which means that we
                //       are allowed to simply append a new slot to the
                //       vector of available slots.
                let index = self.slots.len();
                self.assert_all_slots_occupied();
                self.slots.push(Slot::Occupied { value });
                index
            }
            Some(slot) => {
                // Case: There is at least one vacant slot in the stash
                //       that we can reuse and put the value in.
                let index = slot.into_index();
                match self.slots[index] {
                    Slot::Occupied { .. } => panic!("referenced slot must be vacant"),
                    Slot::Vacant { next_vacant } => {
                        // Update first vacant to the next vacant of the reused
                        // slot. Note that first_vacant is set to `None` in case
                        // `next_vacant` is equal which closes the loop.
                        self.first_vacant = if slot != next_vacant {
                            Some(next_vacant)
                        } else {
                            None
                        };
                        self.first_vacant = Some(next_vacant);
                        self.slots[index] = Slot::Occupied { value };
                        index
                    }
                }
            }
        };
        self.len_occupied += 1;
        SlotRef(index)
    }

    /// Returns the value of the `slot` out of the [`Stash`] if any.
    ///
    /// This removes the returned value from the [`Stash`].
    pub fn take(&mut self, slot: SlotRef) -> Option<T> {
        let next_vacant = self.first_vacant.unwrap_or(slot);
        let entry = self.slots.get_mut(slot.into_index())?;
        match replace(entry, Slot::Vacant { next_vacant }) {
            Slot::Vacant { next_vacant } => {
                // If the slot was already vacant we need to undo the changes
                // done via the call to `replace` because they were optimistic
                // for the occupied case.
                *entry = Slot::Vacant { next_vacant };
                None
            }
            Slot::Occupied { value } => {
                self.len_occupied -= 1;
                if self.is_empty() {
                    // Case: The stash is empty after this operation so we can
                    //       reset the underlying linked list and slots to make
                    //       future insertions more efficient.
                    self.first_vacant = None;
                    self.slots.clear();
                } else {
                    self.first_vacant = Some(slot);
                }
                Some(value)
            }
        }
    }

    /// Returns a shared reference to the value of the `slot` if any.
    pub fn get(&self, slot: SlotRef) -> Option<&T> {
        match self.slots.get(slot.into_index())? {
            Slot::Vacant { .. } => None,
            Slot::Occupied { value } => Some(value),
        }
    }

    /// Returns a shared reference to the value of the `slot` if any.
    pub fn get_mut(&mut self, slot: SlotRef) -> Option<&mut T> {
        match self.slots.get_mut(slot.into_index())? {
            Slot::Vacant { .. } => None,
            Slot::Occupied { value } => Some(value),
        }
    }
}

impl<T> Index<SlotRef> for Stash<T> {
    type Output = T;

    fn index(&self, index: SlotRef) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("unexpected vacant slot at index {}", index.into_index()))
    }
}

impl<T> IndexMut<SlotRef> for Stash<T> {
    fn index_mut(&mut self, index: SlotRef) -> &mut Self::Output {
        self.get_mut(index)
            .unwrap_or_else(|| panic!("unexpected vacant slot at index {}", index.into_index()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_works() {
        let mut stash = <Stash<i32>>::default();
        assert!(stash.is_empty());
        assert_eq!(stash.len(), 0);
        assert_eq!(stash.get(SlotRef(0)), None);
        assert_eq!(stash.get_mut(SlotRef(0)), None);
        assert_eq!(stash.take(SlotRef(0)), None);
    }

    #[test]
    fn get_works() {
        let mut stash = <Stash<i32>>::default();
        let mut values = [10, 20, 30, 40, 50];
        let slots = values.map(|value| stash.put(value));
        assert!(!stash.is_empty());
        assert_eq!(stash.len(), 5);

        // Stash::{get, get_mut} and Index[Mut] works
        for i in 0..values.len() {
            assert_eq!(slots[i], SlotRef(i));
            assert_eq!(stash.get(slots[i]), Some(&values[i]));
            assert_eq!(stash.get_mut(slots[i]), Some(&mut values[i]));
            assert_eq!(&stash[slots[i]], &values[i]);
            assert_eq!(&mut stash[slots[i]], &mut values[i]);
        }
    }

    #[test]
    fn put_take_works() {
        let mut stash = <Stash<i32>>::default();
        let values = [10, 20, 30, 40, 50];
        let slots = values.map(|value| stash.put(value));
        // Take in order
        let mut tmp = stash.clone();
        for i in 0..values.len() {
            assert_eq!(tmp.take(slots[i]), Some(values[i]));
            assert_eq!(tmp.take(slots[i]), None);
        }
        assert!(tmp.is_empty());
        // Take in reverse order
        let mut tmp = stash.clone();
        for i in (0..values.len()).rev() {
            assert_eq!(tmp.take(slots[i]), Some(values[i]));
            assert_eq!(tmp.take(slots[i]), None);
        }
        assert!(tmp.is_empty());
        // Take one, put one: check if slot will be reused immediately
        let mut tmp = stash.clone();
        for i in 0..values.len() {
            let rev_i = values.len() - i - 1;
            assert_eq!(tmp.take(slots[i]), Some(values[i]));
            tmp.put(values[rev_i]);
            assert_eq!(tmp[slots[i]], values[rev_i]);
        }
        // Remove all but first, then refill in reverse order
        let mut tmp = stash.clone();
        for i in 1..values.len() {
            assert_eq!(tmp.take(slots[i]), Some(values[i]));
        }
        assert_eq!(tmp.len(), 1);
        for i in (1..values.len()).rev() {
            assert_eq!(tmp.put(values[i]), slots[i]);
        }
        for i in 0..values.len() {
            assert_eq!(tmp[slots[i]], values[i]);
        }
    }
}
