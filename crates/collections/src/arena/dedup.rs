use super::{Arena, ArenaError, ArenaKey, Iter, IterMut};
use crate::{Map, map};
use core::{
    hash::Hash,
    ops::{Index, IndexMut},
};

/// A deduplicating arena allocator with a given index and entity type.
///
/// For performance reasons the arena cannot deallocate single entities.
#[derive(Debug)]
pub struct DedupArena<Key, T> {
    entity2idx: Map<T, Key>,
    entities: Arena<Key, T>,
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
        self.entities.eq(&other.entities)
    }
}

impl<Key, T> Eq for DedupArena<Key, T> where T: Eq {}

impl<Key, T> DedupArena<Key, T> {
    /// Creates a new empty deduplicating entity arena.
    #[inline]
    pub fn new() -> Self {
        Self {
            entity2idx: Map::new(),
            entities: Arena::new(),
        }
    }

    /// Returns the allocated number of entities.
    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if the [`Arena`] has not yet allocated entities.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all entities from the arena.
    #[inline]
    pub fn clear(&mut self) {
        self.entity2idx.clear();
        self.entities.clear();
    }

    /// Returns an iterator over the shared reference of the [`Arena`] entities.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Key, T> {
        self.entities.iter()
    }

    /// Returns an iterator over the exclusive reference of the [`Arena`] entities.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Key, T> {
        self.entities.iter_mut()
    }
}

impl<Key, T> DedupArena<Key, T>
where
    Key: ArenaKey,
    T: Hash + Ord + Clone,
{
    /// Allocates a new entity and returns its index.
    ///
    /// # Note
    ///
    /// Only allocates if the entity does not already exist in the [`DedupArena`].
    pub fn alloc(&mut self, item: T) -> Result<Key, ArenaError> {
        match self.entity2idx.entry(item.clone()) {
            map::Entry::Occupied(entry) => {
                let key = *entry.get();
                Ok(key)
            }
            map::Entry::Vacant(entry) => {
                let key = self.entities.next_key()?;
                self.entities.alloc(item)?;
                entry.insert(key);
                Ok(key)
            }
        }
    }

    /// Returns a shared reference to the entity at the given key if any.
    #[inline]
    pub fn get(&self, key: Key) -> Result<&T, ArenaError> {
        self.entities.get(key)
    }

    /// Returns an exclusive reference to the entity at the given key if any.
    #[inline]
    pub fn get_mut(&mut self, key: Key) -> Result<&mut T, ArenaError> {
        self.entities.get_mut(key)
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
        let entities = Arena::from_iter(iter);
        let entity2idx = entities
            .iter()
            .map(|(idx, entity)| (entity.clone(), idx))
            .collect::<Map<_, _>>();
        Self {
            entity2idx,
            entities,
        }
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
        &self.entities[key]
    }
}

impl<Key, T> IndexMut<Key> for DedupArena<Key, T>
where
    Key: ArenaKey,
{
    #[inline]
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        &mut self.entities[key]
    }
}
