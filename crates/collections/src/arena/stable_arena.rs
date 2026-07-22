use super::{ArenaError, ArenaKey, StableVec, StableVecIter, StableVecIterMut};
use core::{
    iter::{Enumerate, FusedIterator, repeat_with},
    marker::PhantomData,
    ops::{Index, IndexMut, Range},
};

/// An append-only [`Arena`] whose entities have stable addresses.
///
/// Mirrors most of the [`Arena`] API but is backed by a [`StableVec`] instead of a `Vec`, so
/// references to stored entities stay valid as more entities are allocated.
///
/// [`Arena`]: super::Arena
#[derive(Debug)]
pub struct StableArena<Key, T> {
    /// The items stored in the arena.
    items: StableVec<T>,
    /// Marker for the compiler to associate the `Key` type.
    marker: PhantomData<Key>,
}

/// [`StableArena`] does not store `Key` therefore it is `Send` without its bound.
unsafe impl<Key, T> Send for StableArena<Key, T> where T: Send {}

/// [`StableArena`] does not store `Key` therefore it is `Sync` without its bound.
unsafe impl<Key, T> Sync for StableArena<Key, T> where T: Sync {}

impl<Key, T> Default for StableArena<Key, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Key, T> PartialEq for StableArena<Key, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.items.eq(&other.items)
    }
}

impl<Key, T> Eq for StableArena<Key, T> where T: Eq {}

impl<Key, T> StableArena<Key, T> {
    /// Creates a new empty entity [`StableArena`].
    pub fn new() -> Self {
        Self {
            items: StableVec::new(),
            marker: PhantomData,
        }
    }

    /// Returns the allocated number of entities.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the arena has not yet allocated entities.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns an iterator over the shared references of the arena entities and their keys.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Key, T> {
        Iter {
            iter: self.items.iter().enumerate(),
            marker: PhantomData,
        }
    }

    /// Returns an iterator over the exclusive references of the arena entities and their keys.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Key, T> {
        IterMut {
            iter: self.items.iter_mut().enumerate(),
            marker: PhantomData,
        }
    }
}

impl<Key, T> StableArena<Key, T>
where
    Key: ArenaKey,
{
    /// Returns the next entity key.
    ///
    /// # Errors
    ///
    /// If there are no more valid keys left for allocation.
    fn next_key(&self) -> Result<Key, ArenaError> {
        Key::from_usize(self.items.len()).ok_or(ArenaError::NotEnoughKeys)
    }

    /// Allocates a new entity and returns its key.
    ///
    /// # Note
    ///
    /// Aborts (instead of returning an error) if the system runs out of memory.
    ///
    /// # Errors
    ///
    /// If there are no more valid keys left for allocation.
    #[inline]
    pub fn alloc(&mut self, entity: T) -> Result<Key, ArenaError> {
        let key = self.next_key()?;
        self.items.push(entity);
        Ok(key)
    }

    /// Allocates `amount` default initialized entities and returns their keys.
    ///
    /// # Note
    ///
    /// Aborts (instead of returning an error) if the system runs out of memory.
    ///
    /// # Errors
    ///
    /// If there are no more valid keys left for allocation.
    #[inline]
    pub fn alloc_many(&mut self, amount: usize) -> Result<Range<Key>, ArenaError>
    where
        T: Default,
    {
        let start = self.next_key()?;
        self.items.extend(repeat_with(T::default).take(amount));
        let end = self.next_key()?;
        Ok(Range { start, end })
    }

    /// Returns a shared reference to the entity at the given key if any.
    ///
    /// # Errors
    ///
    /// If the `key` is out of bounds.
    #[inline]
    pub fn get(&self, key: Key) -> Result<&T, ArenaError> {
        self.items
            .get(key.into_usize())
            .ok_or(ArenaError::KeyOutOfBounds)
    }

    /// Returns an exclusive reference to the entity at the given key if any.
    ///
    /// # Errors
    ///
    /// If the `key` is out of bounds.
    #[inline]
    pub fn get_mut(&mut self, key: Key) -> Result<&mut T, ArenaError> {
        self.items
            .get_mut(key.into_usize())
            .ok_or(ArenaError::KeyOutOfBounds)
    }

    /// Returns exclusive references to the pair of entities at the given keys if any.
    ///
    /// # Errors
    ///
    /// - If `keys[0]` and `keys[1]` refer to the same item, a.k.a. aliasing each other.
    /// - If `keys[0]` or `keys[1]` is out of bounds for the arena.
    #[inline]
    pub fn get_disjoint_mut(&mut self, keys: [Key; 2]) -> Result<[&mut T; 2], ArenaError> {
        let [a, b] = keys;
        self.items
            .get_disjoint_mut([a.into_usize(), b.into_usize()])
    }

    /// Panics with a key out of bounds message.
    #[cold]
    fn panic_index_access(error: ArenaError, len: usize, key: Key) -> ! {
        let key = key.into_usize();
        panic!("failed to access item at {key} of arena with len (= {len}): {error}")
    }
}

impl<Key, T> Index<Key> for StableArena<Key, T>
where
    Key: ArenaKey,
{
    type Output = T;

    #[inline]
    fn index(&self, key: Key) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|error| Self::panic_index_access(error, self.len(), key))
    }
}

impl<Key, T> IndexMut<Key> for StableArena<Key, T>
where
    Key: ArenaKey,
{
    #[inline]
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        let len = self.len();
        self.get_mut(key)
            .unwrap_or_else(|error| Self::panic_index_access(error, len, key))
    }
}

impl<Key, T> FromIterator<T> for StableArena<Key, T> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            items: StableVec::from_iter(iter),
            marker: PhantomData,
        }
    }
}

impl<'a, Key, T> IntoIterator for &'a StableArena<Key, T>
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

impl<'a, Key, T> IntoIterator for &'a mut StableArena<Key, T>
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

/// An iterator over shared references of arena entities and their keys.
#[derive(Debug)]
pub struct Iter<'a, Key, T> {
    iter: Enumerate<StableVecIter<'a, T>>,
    marker: PhantomData<fn() -> Key>,
}

impl<'a, Key, T> Iterator for Iter<'a, Key, T>
where
    Key: ArenaKey,
{
    type Item = (Key, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (key, entity) = self.iter.next()?;
        let Some(key) = Key::from_usize(key) else {
            unreachable!("arena can only contain valid keys")
        };
        Some((key, entity))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Key, T> DoubleEndedIterator for Iter<'_, Key, T>
where
    Key: ArenaKey,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let (key, entity) = self.iter.next_back()?;
        let Some(key) = Key::from_usize(key) else {
            unreachable!("arena can only contain valid keys")
        };
        Some((key, entity))
    }
}

impl<Key, T> ExactSizeIterator for Iter<'_, Key, T>
where
    Key: ArenaKey,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Key, T> FusedIterator for Iter<'_, Key, T> where Key: ArenaKey {}

/// An iterator over exclusive references of arena entities and their keys.
#[derive(Debug)]
pub struct IterMut<'a, Key, T> {
    iter: Enumerate<StableVecIterMut<'a, T>>,
    marker: PhantomData<fn() -> Key>,
}

impl<'a, Key, T> Iterator for IterMut<'a, Key, T>
where
    Key: ArenaKey,
{
    type Item = (Key, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (key, entity) = self.iter.next()?;
        let Some(key) = Key::from_usize(key) else {
            unreachable!("arena can only contain valid keys")
        };
        Some((key, entity))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Key, T> DoubleEndedIterator for IterMut<'_, Key, T>
where
    Key: ArenaKey,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let (key, entity) = self.iter.next_back()?;
        let Some(key) = Key::from_usize(key) else {
            unreachable!("arena can only contain valid keys")
        };
        Some((key, entity))
    }
}

impl<Key, T> ExactSizeIterator for IterMut<'_, Key, T>
where
    Key: ArenaKey,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Key, T> FusedIterator for IterMut<'_, Key, T> where Key: ArenaKey {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{vec, vec::Vec};

    type Arena = StableArena<usize, i32>;

    #[test]
    fn alloc_and_get() {
        let mut arena = Arena::new();
        let keys: Vec<usize> = (0..100).map(|i| arena.alloc(i).unwrap()).collect();
        assert_eq!(keys, (0..100).collect::<Vec<_>>());
        assert_eq!(arena.len(), 100);
        for key in 0..100 {
            assert_eq!(arena.get(key).unwrap(), &(key as i32));
        }
        assert!(matches!(arena.get(100), Err(ArenaError::KeyOutOfBounds)));
    }

    #[test]
    fn stable_addresses() {
        let mut arena = Arena::new();
        arena.alloc(0).unwrap();
        let ptr: *const i32 = arena.get(0).unwrap();
        for i in 1..1000 {
            arena.alloc(i).unwrap();
        }
        assert_eq!(ptr, arena.get(0).unwrap() as *const i32);
    }

    #[test]
    fn alloc_many() {
        let mut arena = StableArena::<usize, i32>::new();
        let range = arena.alloc_many(10).unwrap();
        assert_eq!(range, (0..10));
        assert_eq!(arena.len(), 10);
        assert!(arena.iter().all(|(_key, &value)| value == 0));
    }

    #[test]
    fn get_mut_and_pair() {
        let mut arena: Arena = (0..10).collect();
        *arena.get_mut(3).unwrap() = 30;
        assert_eq!(arena.get(3).unwrap(), &30);
        let [a, b] = arena.get_disjoint_mut([1, 8]).unwrap();
        *a = -1;
        *b = -8;
        assert_eq!((arena[1], arena[8]), (-1, -8));
        assert!(matches!(
            arena.get_disjoint_mut([2, 2]),
            Err(ArenaError::AliasingPairAccess)
        ));
        assert!(matches!(
            arena.get_disjoint_mut([0, 10]),
            Err(ArenaError::KeyOutOfBounds)
        ));
    }

    #[test]
    fn iter_yields_keys_and_is_double_ended() {
        let arena: Arena = (0..5).collect();
        let forward: Vec<(usize, i32)> = arena.iter().map(|(k, &v)| (k, v)).collect();
        assert_eq!(forward, vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
        let backward: Vec<(usize, i32)> = arena.iter().rev().map(|(k, &v)| (k, v)).collect();
        assert_eq!(backward, vec![(4, 4), (3, 3), (2, 2), (1, 1), (0, 0)]);
        assert_eq!(arena.iter().len(), 5);
    }

    #[test]
    fn iter_mut_updates_in_place() {
        let mut arena: Arena = (0..10).collect();
        for (key, value) in &mut arena {
            *value += key as i32;
        }
        let collected: Vec<i32> = arena.iter().map(|(_k, &v)| v).collect();
        assert_eq!(collected, (0..10).map(|i| i * 2).collect::<Vec<_>>());
    }

    #[test]
    fn index_panics_out_of_bounds() {
        let arena: Arena = (0..3).collect();
        assert!(std::panic::catch_unwind(|| arena[5]).is_err());
    }

    #[test]
    fn eq() {
        let a: Arena = (0..10).collect();
        let b: Arena = (0..10).collect();
        let c: Arena = (0..9).collect();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
