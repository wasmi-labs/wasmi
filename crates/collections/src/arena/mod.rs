//! Fast arena data structures specialized for usage in the Wasmi interpreter.
//!
//! They cannot deallocate single allocated entities for extra efficiency.
//! These data structures mainly serve as the backbone for an efficient WebAssembly
//! store, module, instance and engine implementation.

mod component_vec;
mod dedup;
mod error;

pub use self::{component_vec::ComponentVec, dedup::DedupArena, error::ArenaError};
use alloc::vec::Vec;
use core::{
    iter::{self, Enumerate},
    marker::PhantomData,
    ops::{Index, IndexMut, Range},
    slice,
};

/// Types that can be used as indices for arenas.
pub trait ArenaKey: Copy {
    /// Converts the [`ArenaKey`] into the underlying `usize` value.
    fn into_usize(self) -> usize;
    /// Converts the `usize` value into the associated [`ArenaKey`].
    ///
    /// Returns `None` if `Self` cannot represent `value`.
    fn from_usize(value: usize) -> Option<Self>;
}

/// An arena allocator with a given key and entity type.
///
/// For performance reasons the arena cannot deallocate single entities.
#[derive(Debug)]
pub struct Arena<Key, T> {
    /// The items stored in the arena.
    items: Vec<T>,
    /// Marker for the compiler to associate the `Key` type.
    marker: PhantomData<Key>,
}

/// [`Arena`] does not store `Key` therefore it is `Send` without its bound.
unsafe impl<Key, T> Send for Arena<Key, T> where T: Send {}

/// [`Arena`] does not store `Key` therefore it is `Sync` without its bound.
unsafe impl<Key, T> Sync for Arena<Key, T> where T: Sync {}

impl<Key, T> Default for Arena<Key, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Key, T> PartialEq for Arena<Key, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.items.eq(&other.items)
    }
}

impl<Key, T> Eq for Arena<Key, T> where T: Eq {}

impl<Key, T> Arena<Key, T> {
    /// Creates a new empty entity [`Arena`].
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
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
        self.len() == 0
    }

    /// Clears all entities from the arena.
    #[inline]
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Returns an iterator over the shared reference of the arena entities.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Key, T> {
        Iter {
            iter: self.items.iter().enumerate(),
            marker: PhantomData,
        }
    }

    /// Returns an iterator over the exclusive reference of the arena entities.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Key, T> {
        IterMut {
            iter: self.items.iter_mut().enumerate(),
            marker: PhantomData,
        }
    }
}

impl<Key, T> Arena<Key, T>
where
    Key: ArenaKey,
{
    /// Returns the next entity index.
    ///
    /// # Errors
    ///
    /// If there are no more valid keys left for allocation.
    fn next_key(&self) -> Result<Key, ArenaError> {
        Key::from_usize(self.items.len()).ok_or(ArenaError::NotEnoughKeys)
    }

    /// Allocates a new entity and returns its index.
    ///
    /// # Errors
    ///
    /// - If there are no more valid keys left for allocation.
    /// - If the system ran out of heap memory.
    #[inline]
    pub fn alloc(&mut self, entity: T) -> Result<Key, ArenaError> {
        let key = self.next_key()?;
        self.items
            .try_reserve(1)
            .map_err(|_| ArenaError::OutOfSystemMemory)?;
        self.items.push(entity);
        Ok(key)
    }

    /// Allocates `amount` default initialized entities and returns their keys.
    ///
    /// # Errors
    ///
    /// - If there are no more valid keys left for allocation.
    /// - If the system ran out of heap memory.
    #[inline]
    pub fn alloc_many(&mut self, amount: usize) -> Result<Range<Key>, ArenaError>
    where
        T: Default,
    {
        let start = self.next_key()?;
        self.items
            .try_reserve(amount)
            .map_err(|_| ArenaError::OutOfSystemMemory)?;
        self.items
            .extend(iter::repeat_with(T::default).take(amount));
        let end = self.next_key()?;
        Ok(Range { start, end })
    }

    /// Returns a shared reference to the entity at the given key if any.
    ///
    /// # Errors
    ///
    /// - If the `key` is out of bounds.
    /// - If the `key` is invalid.
    #[inline]
    pub fn get(&self, key: Key) -> Result<&T, ArenaError> {
        let key = key.into_usize();
        self.items.get(key).ok_or(ArenaError::OutOfBoundsKey)
    }

    /// Returns an exclusive reference to the entity at the given key if any.
    ///
    /// # Errors
    ///
    /// - If the `key` is out of bounds.
    /// - If the `key` is invalid.
    #[inline]
    pub fn get_mut(&mut self, key: Key) -> Result<&mut T, ArenaError> {
        let key = key.into_usize();
        self.items.get_mut(key).ok_or(ArenaError::OutOfBoundsKey)
    }

    /// Returns an exclusive reference to the pair of entities at the given indices if any.
    ///
    /// Returns `None` if `fst` and `snd` refer to the same entity.
    /// Returns `None` if either `fst` or `snd` is invalid for this [`Arena`].
    #[inline]
    pub fn get_pair_mut(&mut self, fst: Key, snd: Key) -> Result<(&mut T, &mut T), ArenaError> {
        let fst_key = fst.into_usize();
        let snd_key = snd.into_usize();
        if fst_key == snd_key {
            return Err(ArenaError::AliasingPairAccess);
        }
        if fst_key > snd_key {
            let (fst, snd) = self.get_pair_mut(snd, fst)?;
            return Ok((snd, fst));
        }
        debug_assert!(fst_key < snd_key);
        let Some((fst_set, snd_set)) = self.items.split_at_mut_checked(snd_key) else {
            return Err(ArenaError::OutOfBoundsKey);
        };
        let fst = &mut fst_set[fst_key];
        let snd = &mut snd_set[0];
        Ok((fst, snd))
    }
}

impl<Key, T> FromIterator<T> for Arena<Key, T> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            items: Vec::from_iter(iter),
            marker: PhantomData,
        }
    }
}

impl<'a, Key, T> IntoIterator for &'a Arena<Key, T>
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

impl<'a, Key, T> IntoIterator for &'a mut Arena<Key, T>
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

/// An iterator over shared references of arena entities and their indices.
#[derive(Debug)]
pub struct Iter<'a, Key, T> {
    iter: Enumerate<slice::Iter<'a, T>>,
    marker: PhantomData<fn() -> Key>,
}

impl<'a, Key, T> Iterator for Iter<'a, Key, T>
where
    Key: ArenaKey,
{
    type Item = (Key, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(key, entity)| {
            let Some(key) = Key::from_usize(key) else {
                unreachable!("arena can only contain valid keys")
            };
            (key, entity)
        })
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
        self.iter.next().map(|(key, entity)| {
            let Some(key) = Key::from_usize(key) else {
                unreachable!("arena can only contain valid keys")
            };
            (key, entity)
        })
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

/// An iterator over exclusive references of arena entities and their indices.
#[derive(Debug)]
pub struct IterMut<'a, Key, T> {
    iter: Enumerate<slice::IterMut<'a, T>>,
    marker: PhantomData<fn() -> Key>,
}

impl<'a, Key, T> Iterator for IterMut<'a, Key, T>
where
    Key: ArenaKey,
{
    type Item = (Key, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(key, entity)| {
            let Some(key) = Key::from_usize(key) else {
                unreachable!("arena can only contain valid keys")
            };
            (key, entity)
        })
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
        self.iter.next().map(|(key, entity)| {
            let Some(key) = Key::from_usize(key) else {
                unreachable!("arena can only contain valid keys")
            };
            (key, entity)
        })
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

impl<Key, T> Arena<Key, T>
where
    Key: ArenaKey,
{
    /// Panics with an key out of bounds message.
    #[cold]
    fn panic_index_access(error: ArenaError, len: usize, key: Key) -> ! {
        let key = key.into_usize();
        panic!("failed to access item at {key} of arena with len (= {len}): {error}")
    }
}

impl<Key, T> Index<Key> for Arena<Key, T>
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

impl<Key, T> IndexMut<Key> for Arena<Key, T>
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
