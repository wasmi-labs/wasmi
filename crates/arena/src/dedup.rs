use super::{Arena, ArenaIndex, Iter, IterMut};
use alloc::collections::BTreeMap;
use core::ops::{Index, IndexMut};

/// A deduplicating arena allocator with a given index and entity type.
///
/// For performance reasons the arena cannot deallocate single entities.
#[derive(Debug)]
pub struct DedupArena<Idx, T> {
    entity2idx: BTreeMap<T, Idx>,
    entities: Arena<Idx, T>,
}

impl<Idx, T> Default for DedupArena<Idx, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Idx, T> PartialEq for DedupArena<Idx, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.entities.eq(&other.entities)
    }
}

impl<Idx, T> Eq for DedupArena<Idx, T> where T: Eq {}

impl<Idx, T> DedupArena<Idx, T> {
    /// Creates a new empty deduplicating entity arena.
    pub fn new() -> Self {
        Self {
            entity2idx: BTreeMap::new(),
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
    pub fn clear(&mut self) {
        self.entity2idx.clear();
        self.entities.clear();
    }

    /// Returns an iterator over the shared reference of the [`Arena`] entities.
    pub fn iter(&self) -> Iter<Idx, T> {
        self.entities.iter()
    }

    /// Returns an iterator over the exclusive reference of the [`Arena`] entities.
    pub fn iter_mut(&mut self) -> IterMut<Idx, T> {
        self.entities.iter_mut()
    }
}

impl<Idx, T> DedupArena<Idx, T>
where
    Idx: ArenaIndex,
    T: Ord + Clone,
{
    /// Returns the next entity index.
    fn next_index(&self) -> Idx {
        self.entities.next_index()
    }

    /// Allocates a new entity and returns its index.
    ///
    /// # Note
    ///
    /// Only allocates if the entity does not already exist in the [`DedupArena`].
    pub fn alloc(&mut self, entity: T) -> Idx {
        match self.entity2idx.get(&entity) {
            Some(index) => *index,
            None => {
                let index = self.next_index();
                self.entity2idx.insert(entity.clone(), index);
                self.entities.alloc(entity);
                index
            }
        }
    }

    /// Returns a shared reference to the entity at the given index if any.
    #[inline]
    pub fn get(&self, index: Idx) -> Option<&T> {
        self.entities.get(index)
    }

    /// Returns an exclusive reference to the entity at the given index if any.
    #[inline]
    pub fn get_mut(&mut self, index: Idx) -> Option<&mut T> {
        self.entities.get_mut(index)
    }
}

impl<Idx, T> FromIterator<T> for DedupArena<Idx, T>
where
    Idx: ArenaIndex,
    T: Clone + Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let entities = Arena::from_iter(iter);
        let entity2idx = entities
            .iter()
            .map(|(idx, entity)| (entity.clone(), idx))
            .collect::<BTreeMap<_, _>>();
        Self {
            entity2idx,
            entities,
        }
    }
}

impl<'a, Idx, T> IntoIterator for &'a DedupArena<Idx, T>
where
    Idx: ArenaIndex,
{
    type Item = (Idx, &'a T);
    type IntoIter = Iter<'a, Idx, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Idx, T> IntoIterator for &'a mut DedupArena<Idx, T>
where
    Idx: ArenaIndex,
{
    type Item = (Idx, &'a mut T);
    type IntoIter = IterMut<'a, Idx, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<Idx, T> Index<Idx> for DedupArena<Idx, T>
where
    Idx: ArenaIndex,
{
    type Output = T;

    #[inline]
    fn index(&self, index: Idx) -> &Self::Output {
        &self.entities[index]
    }
}

impl<Idx, T> IndexMut<Idx> for DedupArena<Idx, T>
where
    Idx: ArenaIndex,
{
    #[inline]
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.entities[index]
    }
}
