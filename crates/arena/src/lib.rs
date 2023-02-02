//! Fast arena allocators for different usage purposes.
//!
//! They cannot deallocate single allocated entities for extra efficiency.
//! These allocators mainly serve as the backbone for an efficient Wasm store
//! implementation.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    clippy::cast_lossless,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::redundant_closure_for_method_calls,
    clippy::type_repetition_in_bounds,
    clippy::inconsistent_struct_constructor,
    clippy::default_trait_access,
    clippy::map_unwrap_or,
    clippy::items_after_statements
)]
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

mod component_vec;
mod dedup;
mod guarded;

#[cfg(test)]
mod tests;

pub use self::{component_vec::ComponentVec, dedup::DedupArena, guarded::GuardedEntity};
use alloc::vec::Vec;
use core::{
    iter::{DoubleEndedIterator, Enumerate, ExactSizeIterator},
    marker::PhantomData,
    ops::{Index, IndexMut},
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

/// `Arena` does not store `Idx` therefore it is `Send` without its bound.
unsafe impl<Idx, T> Send for Arena<Idx, T> where T: Send {}

/// `Arena` does not store `Idx` therefore it is `Sync` without its bound.
unsafe impl<Idx, T> Sync for Arena<Idx, T> where T: Send {}

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
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Returns an iterator over the shared reference of the arena entities.
    pub fn iter(&self) -> Iter<Idx, T> {
        Iter {
            iter: self.entities.iter().enumerate(),
            marker: PhantomData,
        }
    }

    /// Returns an iterator over the exclusive reference of the arena entities.
    pub fn iter_mut(&mut self) -> IterMut<Idx, T> {
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

impl<'a, Idx, T> DoubleEndedIterator for Iter<'a, Idx, T>
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

impl<'a, Idx, T> ExactSizeIterator for Iter<'a, Idx, T>
where
    Idx: ArenaIndex,
{
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

impl<'a, Idx, T> DoubleEndedIterator for IterMut<'a, Idx, T>
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

impl<'a, Idx, T> ExactSizeIterator for IterMut<'a, Idx, T>
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
