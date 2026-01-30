//! Fast arena data structures specialized for usage in the Wasmi interpreter.
//!
//! They cannot deallocate single allocated entities for extra efficiency.
//! These data structures mainly serve as the backbone for an efficient WebAssembly
//! store, module, instance and engine implementation.

mod component_vec;
mod dedup;

pub use self::{component_vec::ComponentVec, dedup::DedupArena};
use alloc::vec::Vec;
use core::{
    iter::Enumerate,
    marker::PhantomData,
    ops::{Index, IndexMut, Range},
    slice,
};

/// Types that can be used as indices for arenas.
pub trait ArenaIndex: Copy {
    /// Converts the [`ArenaIndex`] into the underlying `usize` value.
    fn into_usize(self) -> usize;
    /// Converts the `usize` value into the associated [`ArenaIndex`].
    fn from_usize(value: usize) -> Self;
}

/// An arena allocator with a given index and entity type.
///
/// For performance reasons the arena cannot deallocate single entities.
#[derive(Debug)]
pub struct Arena<Idx, T> {
    entities: Vec<T>,
    marker: PhantomData<Idx>,
}

/// [`Arena`] does not store `Idx` therefore it is `Send` without its bound.
unsafe impl<Idx, T> Send for Arena<Idx, T> where T: Send {}

/// [`Arena`] does not store `Idx` therefore it is `Sync` without its bound.
unsafe impl<Idx, T> Sync for Arena<Idx, T> where T: Sync {}

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
    pub fn iter(&self) -> Iter<'_, Idx, T> {
        Iter {
            iter: self.entities.iter().enumerate(),
            marker: PhantomData,
        }
    }

    /// Returns an iterator over the exclusive reference of the arena entities.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, Idx, T> {
        IterMut {
            iter: self.entities.iter_mut().enumerate(),
            marker: PhantomData,
        }
    }
}

impl<Idx, T> Arena<Idx, T>
where
    Idx: ArenaIndex,
{
    /// Returns the next entity index.
    fn next_index(&self) -> Idx {
        Idx::from_usize(self.entities.len())
    }

    /// Allocates a new entity and returns its index.
    #[inline]
    pub fn alloc(&mut self, entity: T) -> Idx {
        let index = self.next_index();
        self.entities.push(entity);
        index
    }

    /// Allocates a new default initialized entity and returns its index.
    #[inline]
    pub fn alloc_many(&mut self, amount: usize) -> Range<Idx>
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
    pub fn get(&self, index: Idx) -> Option<&T> {
        self.entities.get(index.into_usize())
    }

    /// Returns an exclusive reference to the entity at the given index if any.
    #[inline]
    pub fn get_mut(&mut self, index: Idx) -> Option<&mut T> {
        self.entities.get_mut(index.into_usize())
    }

    /// Returns an exclusive reference to the pair of entities at the given indices if any.
    ///
    /// Returns `None` if `fst` and `snd` refer to the same entity.
    /// Returns `None` if either `fst` or `snd` is invalid for this [`Arena`].
    #[inline]
    pub fn get_pair_mut(&mut self, fst: Idx, snd: Idx) -> Option<(&mut T, &mut T)> {
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

impl<Idx, T> FromIterator<T> for Arena<Idx, T> {
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

impl<'a, Idx, T> IntoIterator for &'a Arena<Idx, T>
where
    Idx: ArenaIndex,
{
    type Item = (Idx, &'a T);
    type IntoIter = Iter<'a, Idx, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Idx, T> IntoIterator for &'a mut Arena<Idx, T>
where
    Idx: ArenaIndex,
{
    type Item = (Idx, &'a mut T);
    type IntoIter = IterMut<'a, Idx, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over shared references of arena entities and their indices.
#[derive(Debug)]
pub struct Iter<'a, Idx, T> {
    iter: Enumerate<slice::Iter<'a, T>>,
    marker: PhantomData<fn() -> Idx>,
}

impl<'a, Idx, T> Iterator for Iter<'a, Idx, T>
where
    Idx: ArenaIndex,
{
    type Item = (Idx, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(idx, entity)| (Idx::from_usize(idx), entity))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Idx, T> DoubleEndedIterator for Iter<'_, Idx, T>
where
    Idx: ArenaIndex,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(idx, entity)| (Idx::from_usize(idx), entity))
    }
}

impl<Idx, T> ExactSizeIterator for Iter<'_, Idx, T>
where
    Idx: ArenaIndex,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// An iterator over exclusive references of arena entities and their indices.
#[derive(Debug)]
pub struct IterMut<'a, Idx, T> {
    iter: Enumerate<slice::IterMut<'a, T>>,
    marker: PhantomData<fn() -> Idx>,
}

impl<'a, Idx, T> Iterator for IterMut<'a, Idx, T>
where
    Idx: ArenaIndex,
{
    type Item = (Idx, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(idx, entity)| (Idx::from_usize(idx), entity))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Idx, T> DoubleEndedIterator for IterMut<'_, Idx, T>
where
    Idx: ArenaIndex,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(idx, entity)| (Idx::from_usize(idx), entity))
    }
}

impl<Idx, T> ExactSizeIterator for IterMut<'_, Idx, T>
where
    Idx: ArenaIndex,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Idx, T> Arena<Idx, T> {
    /// Panics with an index out of bounds message.
    fn index_out_of_bounds(len: usize, index: usize) -> ! {
        panic!("index out of bounds: the len is {len} but the index is {index}")
    }
}

impl<Idx, T> Index<Idx> for Arena<Idx, T>
where
    Idx: ArenaIndex,
{
    type Output = T;

    #[inline]
    fn index(&self, index: Idx) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| Self::index_out_of_bounds(self.len(), index.into_usize()))
    }
}

impl<Idx, T> IndexMut<Idx> for Arena<Idx, T>
where
    Idx: ArenaIndex,
{
    #[inline]
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        let len = self.len();
        self.get_mut(index)
            .unwrap_or_else(|| Self::index_out_of_bounds(len, index.into_usize()))
    }
}
