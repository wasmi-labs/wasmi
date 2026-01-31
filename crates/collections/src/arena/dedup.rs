use super::{Arena, ArenaError, ArenaKey, Iter, IterMut};
use crate::{Map, map};
use core::{
    hash::Hash,
    ops::{Index, IndexMut},
};

/// A deduplicating arena allocator with a given key and item type.
///
/// For performance reasons the arena cannot deallocate single items.
#[derive(Debug)]
pub struct DedupArena<Key, T> {
    /// Allows to track existence and key of a contained item.
    item2key: Map<T, Key>,
    /// The contained items of the deduplicating arena.
    items: Arena<Key, T>,
}

impl<Key, T> Default for DedupArena<Key, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<Key, T> PartialEq for DedupArena<Key, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.items.eq(&other.items)
    }
}

impl<Key, T> Eq for DedupArena<Key, T> where T: Eq {}

impl<Key, T> DedupArena<Key, T> {
    /// Creates a new empty deduplicating item arena.
    #[inline]
    pub fn new() -> Self {
        Self {
            item2key: Map::new(),
            items: Arena::new(),
        }
    }

    /// Returns the allocated number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the [`Arena`] has not yet allocated items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all items from the arena.
    #[inline]
    pub fn clear(&mut self) {
        self.item2key.clear();
        self.items.clear();
    }

    /// Returns an iterator over the shared reference of the [`Arena`] items.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Key, T> {
        self.items.iter()
    }

    /// Returns an iterator over the exclusive reference of the [`Arena`] items.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Key, T> {
        self.items.iter_mut()
    }
}

impl<Key, T> DedupArena<Key, T>
where
    Key: ArenaKey,
    T: Hash + Ord + Clone,
{
    /// Allocates a new item and returns its index.
    ///
    /// # Note
    ///
    /// Only allocates if the item does not already exist in the [`DedupArena`].
    ///
    /// # Errors
    ///
    /// - If there are no more valid keys left for allocation.
    /// - If the system ran out of heap memory.
    pub fn alloc(&mut self, item: T) -> Result<Key, ArenaError> {
        match self.item2key.entry(item.clone()) {
            map::Entry::Occupied(entry) => {
                let key = *entry.get();
                Ok(key)
            }
            map::Entry::Vacant(entry) => {
                let key = self.items.next_key()?;
                self.items.alloc(item)?;
                entry.insert(key);
                Ok(key)
            }
        }
    }

    /// Returns a shared reference to the item at the given key if any.
    ///
    /// # Errors
    ///
    /// - If the `key` is out of bounds.
    /// - If the `key` is invalid.
    #[inline]
    pub fn get(&self, key: Key) -> Result<&T, ArenaError> {
        self.items.get(key)
    }

    /// Returns an exclusive reference to the item at the given key if any.
    ///
    /// # Errors
    ///
    /// - If the `key` is out of bounds.
    /// - If the `key` is invalid.
    #[inline]
    pub fn get_mut(&mut self, key: Key) -> Result<&mut T, ArenaError> {
        self.items.get_mut(key)
    }
}

impl<Key, T> FromIterator<T> for DedupArena<Key, T>
where
    Key: ArenaKey,
    T: Hash + Clone + Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let items = Arena::from_iter(iter);
        let item2key = items
            .iter()
            .map(|(key, item)| (item.clone(), key))
            .collect::<Map<_, _>>();
        Self { item2key, items }
    }
}

impl<'a, Key, T> IntoIterator for &'a DedupArena<Key, T>
where
    Key: ArenaKey,
{
    type Item = (Key, &'a T);
    type IntoIter = Iter<'a, Key, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Key, T> IntoIterator for &'a mut DedupArena<Key, T>
where
    Key: ArenaKey,
{
    type Item = (Key, &'a mut T);
    type IntoIter = IterMut<'a, Key, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<Key, T> Index<Key> for DedupArena<Key, T>
where
    Key: ArenaKey,
{
    type Output = T;

    #[inline]
    fn index(&self, key: Key) -> &Self::Output {
        &self.items[key]
    }
}

impl<Key, T> IndexMut<Key> for DedupArena<Key, T>
where
    Key: ArenaKey,
{
    #[inline]
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        &mut self.items[key]
    }
}
