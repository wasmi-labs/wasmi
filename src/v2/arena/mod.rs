mod dedup;

pub use self::dedup::DedupArena;
use alloc::vec::Vec;
use core::iter::{DoubleEndedIterator, ExactSizeIterator};
use core::marker::PhantomData;
use core::slice;
use core::{iter, ops};

/// An arena allocator with a given index and entity type.
///
/// For performance reasons the arena cannot deallocate single entities.
#[derive(Debug)]
pub struct Arena<Idx, T> {
    entities: Vec<T>,
    __marker: PhantomData<fn() -> Idx>,
}

/// Types that can be used as indices for arenas.
pub trait Index: Copy {
    fn into_usize(self) -> usize;
    fn from_usize(value: usize) -> Self;
}

impl<Idx, T> Default for Arena<Idx, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Idx, T> PartialEq for Arena<Idx, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.entities.eq(&other.entities)
    }
}

impl<Idx, T> Eq for Arena<Idx, T> where T: Eq {}

impl<Idx, T> Arena<Idx, T> {
    /// Creates a new empty entity arena.
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            __marker: PhantomData,
        }
    }

    /// Returns the allocated number of entities.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if the arena has not yet allocated entities.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all entities from the arena.
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Returns an iterator over the shared reference of the arena entities.
    pub fn iter(&self) -> Iter<Idx, T> {
        Iter {
            iter: self.entities.iter().enumerate(),
            __marker: PhantomData,
        }
    }

    /// Returns an iterator over the exclusive reference of the arena entities.
    pub fn iter_mut(&mut self) -> IterMut<Idx, T> {
        IterMut {
            iter: self.entities.iter_mut().enumerate(),
            __marker: PhantomData,
        }
    }
}

impl<Idx, T> Arena<Idx, T>
where
    Idx: Index,
{
    /// Returns the next entity index.
    fn next_index(&self) -> Idx {
        Idx::from_usize(self.entities.len())
    }

    /// Allocates a new entity and returns its index.
    pub fn alloc(&mut self, entity: T) -> Idx {
        let index = self.next_index();
        self.entities.push(entity);
        index
    }

    /// Returns a shared reference to the entity at the given index if any.
    pub fn get(&self, index: Idx) -> Option<&T> {
        self.entities.get(index.into_usize())
    }

    /// Returns an exclusive reference to the entity at the given index if any.
    pub fn get_mut(&mut self, index: Idx) -> Option<&mut T> {
        self.entities.get_mut(index.into_usize())
    }
}

impl<Idx, T> FromIterator<T> for Arena<Idx, T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            entities: Vec::from_iter(iter),
            __marker: PhantomData,
        }
    }
}

impl<'a, Idx, T> IntoIterator for &'a Arena<Idx, T>
where
    Idx: Index,
{
    type IntoIter = Iter<'a, Idx, T>;
    type Item = (Idx, &'a T);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Idx, T> IntoIterator for &'a mut Arena<Idx, T>
where
    Idx: Index,
{
    type IntoIter = IterMut<'a, Idx, T>;
    type Item = (Idx, &'a mut T);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over shared references of arena entities and their indices.
#[derive(Debug)]
pub struct Iter<'a, Idx, T> {
    iter: iter::Enumerate<slice::Iter<'a, T>>,
    __marker: PhantomData<fn() -> Idx>,
}

impl<'a, Idx, T> Iterator for Iter<'a, Idx, T>
where
    Idx: Index,
{
    type Item = (Idx, &'a T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(idx, entity)| (Idx::from_usize(idx), entity))
    }
}

impl<'a, Idx, T> DoubleEndedIterator for Iter<'a, Idx, T>
where
    Idx: Index,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(idx, entity)| (Idx::from_usize(idx), entity))
    }
}

impl<'a, Idx, T> ExactSizeIterator for Iter<'a, Idx, T>
where
    Idx: Index,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// An iterator over exlusive references of arena entities and their indices.
#[derive(Debug)]
pub struct IterMut<'a, Idx, T> {
    iter: iter::Enumerate<slice::IterMut<'a, T>>,
    __marker: PhantomData<fn() -> Idx>,
}

impl<'a, Idx, T> Iterator for IterMut<'a, Idx, T>
where
    Idx: Index,
{
    type Item = (Idx, &'a mut T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(idx, entity)| (Idx::from_usize(idx), entity))
    }
}

impl<'a, Idx, T> DoubleEndedIterator for IterMut<'a, Idx, T>
where
    Idx: Index,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(idx, entity)| (Idx::from_usize(idx), entity))
    }
}

impl<'a, Idx, T> ExactSizeIterator for IterMut<'a, Idx, T>
where
    Idx: Index,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Idx, T> ops::Index<Idx> for Arena<Idx, T>
where
    Idx: Index,
{
    type Output = T;

    fn index(&self, index: Idx) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!(
                "tried to access out of bounds arena entity at {}",
                index.into_usize()
            )
        })
    }
}

impl<Idx, T> ops::IndexMut<Idx> for Arena<Idx, T>
where
    Idx: Index,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.get_mut(index).unwrap_or_else(|| {
            panic!(
                "tried to access out of bounds arena entity at {}",
                index.into_usize()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Index for usize {
        fn into_usize(self) -> usize {
            self
        }

        fn from_usize(value: usize) -> Self {
            value
        }
    }

    fn alloc_arena(entities: &[&'static str]) -> Arena<usize, &'static str> {
        let mut arena = <Arena<usize, &'static str>>::new();
        // Check that the given arena is actually empty.
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        // Fill arena and check invariants while doing so.
        for idx in 0..entities.len() {
            assert!(arena.get(idx).is_none());
        }
        for (n, str) in entities.iter().enumerate() {
            assert_eq!(arena.alloc(str), n);
        }
        // Check state of filled arena.
        assert_eq!(arena.len(), entities.len());
        assert!(!arena.is_empty());
        for (n, str) in entities.iter().enumerate() {
            assert_eq!(arena.get(n), Some(str));
            assert_eq!(&arena[n], str);
        }
        assert_eq!(arena.get(arena.len()), None);
        // Return filled arena.
        arena
    }

    const TEST_ENTITIES: &[&'static str] = &["a", "b", "c", "d"];

    #[test]
    fn alloc_works() {
        alloc_arena(TEST_ENTITIES);
    }

    #[test]
    fn clear_works() {
        let mut arena = alloc_arena(TEST_ENTITIES);
        // Clear the arena and check if all elements are removed.
        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        for idx in 0..arena.len() {
            assert_eq!(arena.get(idx), None);
        }
        assert_eq!(arena.get(arena.len()), None);
    }

    #[test]
    fn iter_works() {
        let arena = alloc_arena(TEST_ENTITIES);
        assert!(arena.iter().eq(TEST_ENTITIES.iter().enumerate()));
    }

    #[test]
    fn from_iter_works() {
        let expected = alloc_arena(TEST_ENTITIES);
        let actual = TEST_ENTITIES.iter().copied().collect::<Arena<_, _>>();
        assert_eq!(actual, expected);
    }
}
