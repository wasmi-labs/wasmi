//! Fast arena data structures specialized for usage in the Wasmi interpreter.
//!
//! They cannot deallocate single allocated entities for extra efficiency.
//! These data structures mainly serve as the backbone for an efficient WebAssembly
//! store, module, instance and engine implementation.

mod component_vec;
mod dedup;
mod error;

pub use self::{component_vec::ComponentVec, dedup::DedupArena};
use alloc::vec::Vec;
use core::{
    iter::Enumerate,
    marker::PhantomData,
    ops::{Index, IndexMut, Range},
    slice,
};

/// Types that can be used as indices for arenas.
pub trait ArenaKey: Copy {
    /// Converts the [`ArenaKey`] into the underlying `usize` value.
    fn into_usize(self) -> usize;
    /// Converts the `usize` value into the associated [`ArenaKey`].
    fn from_usize(value: usize) -> Self;
}

/// An arena allocator with a given index and entity type.
///
/// For performance reasons the arena cannot deallocate single entities.
#[derive(Debug)]
pub struct Arena<Key, T> {
    entities: Vec<T>,
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
        self.entities.eq(&other.entities)
    }
}

impl<Key, T> Eq for Arena<Key, T> where T: Eq {}

impl<Key, T> Arena<Key, T> {
    /// Creates a new empty entity [`Arena`].
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Returns the allocated number of entities.
    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if the arena has not yet allocated entities.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all entities from the arena.
    #[inline]
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Returns an iterator over the shared reference of the arena entities.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Key, T> {
        Iter {
            iter: self.entities.iter().enumerate(),
            marker: PhantomData,
        }
    }

    /// Returns an iterator over the exclusive reference of the arena entities.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Key, T> {
        IterMut {
            iter: self.entities.iter_mut().enumerate(),
            marker: PhantomData,
        }
    }
}

impl<Key, T> Arena<Key, T>
where
    Key: ArenaKey,
{
    /// Returns the next entity index.
    fn next_index(&self) -> Key {
        Key::from_usize(self.entities.len())
    }

    /// Allocates a new entity and returns its index.
    #[inline]
    pub fn alloc(&mut self, entity: T) -> Key {
        let index = self.next_index();
        self.entities.push(entity);
        index
    }

    /// Allocates a new default initialized entity and returns its index.
    #[inline]
    pub fn alloc_many(&mut self, amount: usize) -> Range<Key>
    where
        T: Default,
    {
        let start = self.next_index();
        self.entities
            .extend(core::iter::repeat_with(T::default).take(amount));
        let end = self.next_index();
        Range { start, end }
    }

    /// Returns a shared reference to the entity at the given index if any.
    #[inline]
    pub fn get(&self, index: Key) -> Option<&T> {
        self.entities.get(index.into_usize())
    }

    /// Returns an exclusive reference to the entity at the given index if any.
    #[inline]
    pub fn get_mut(&mut self, index: Key) -> Option<&mut T> {
        self.entities.get_mut(index.into_usize())
    }

    /// Returns an exclusive reference to the pair of entities at the given indices if any.
    ///
    /// Returns `None` if `fst` and `snd` refer to the same entity.
    /// Returns `None` if either `fst` or `snd` is invalid for this [`Arena`].
    #[inline]
    pub fn get_pair_mut(&mut self, fst: Key, snd: Key) -> Option<(&mut T, &mut T)> {
        let fst_index = fst.into_usize();
        let snd_index = snd.into_usize();
        if fst_index == snd_index {
            return None;
        }
        if fst_index > snd_index {
            let (fst, snd) = self.get_pair_mut(snd, fst)?;
            return Some((snd, fst));
        }
        // At this point we know that fst_index < snd_index.
        let (fst_set, snd_set) = self.entities.split_at_mut(snd_index);
        let fst = fst_set.get_mut(fst_index)?;
        let snd = snd_set.get_mut(0)?;
        Some((fst, snd))
    }
}

impl<Key, T> FromIterator<T> for Arena<Key, T> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            entities: Vec::from_iter(iter),
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
        self.iter
            .next()
            .map(|(idx, entity)| (Key::from_usize(idx), entity))
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
        self.iter
            .next()
            .map(|(idx, entity)| (Key::from_usize(idx), entity))
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
        self.iter
            .next()
            .map(|(idx, entity)| (Key::from_usize(idx), entity))
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
        self.iter
            .next()
            .map(|(idx, entity)| (Key::from_usize(idx), entity))
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

impl<Key, T> Arena<Key, T> {
    /// Panics with an index out of bounds message.
    fn index_out_of_bounds(len: usize, index: usize) -> ! {
        panic!("index out of bounds: the len is {len} but the index is {index}")
    }
}

impl<Key, T> Index<Key> for Arena<Key, T>
where
    Key: ArenaKey,
{
    type Output = T;

    #[inline]
    fn index(&self, index: Key) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| Self::index_out_of_bounds(self.len(), index.into_usize()))
    }
}

impl<Key, T> IndexMut<Key> for Arena<Key, T>
where
    Key: ArenaKey,
{
    #[inline]
    fn index_mut(&mut self, index: Key) -> &mut Self::Output {
        let len = self.len();
        self.get_mut(index)
            .unwrap_or_else(|| Self::index_out_of_bounds(len, index.into_usize()))
    }
}
