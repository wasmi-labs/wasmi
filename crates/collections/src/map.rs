//! Type definitions for a default map.

use core::{borrow::Borrow, hash::Hash, iter::FusedIterator, ops::Index};
use std::fmt::Debug;

#[cfg(not(feature = "no-hash-maps"))]
mod detail {
    use crate::hash;
    use hashbrown::{hash_map, HashMap};

    pub type MapImpl<K, V> = HashMap<K, V, hash::RandomState>;
    pub type EntryImpl<'a, K, V> = hash_map::Entry<'a, K, V, hash::RandomState>;
    pub type OccupiedEntryImpl<'a, K, V> = hash_map::OccupiedEntry<'a, K, V, hash::RandomState>;
    pub type VacantEntryImpl<'a, K, V> = hash_map::VacantEntry<'a, K, V, hash::RandomState>;
    pub type IterImpl<'a, K, V> = hash_map::Iter<'a, K, V>;
    pub type IterMutImpl<'a, K, V> = hash_map::IterMut<'a, K, V>;
    pub type IntoIterImpl<K, V> = hash_map::IntoIter<K, V>;
    pub type KeysImpl<'a, K, V> = hash_map::Keys<'a, K, V>;
    pub type ValuesImpl<'a, K, V> = hash_map::Values<'a, K, V>;
    pub type ValuesMutImpl<'a, K, V> = hash_map::ValuesMut<'a, K, V>;
}

#[cfg(feature = "no-hash-maps")]
mod detail {
    use std::collections::{btree_map, BTreeMap};

    pub type MapImpl<K, V> = BTreeMap<K, V>;
    pub type EntryImpl<'a, K, V> = btree_map::Entry<'a, K, V>;
    pub type OccupiedEntryImpl<'a, K, V> = btree_map::OccupiedEntry<'a, K, V>;
    pub type VacantEntryImpl<'a, K, V> = btree_map::VacantEntry<'a, K, V>;
    pub type IterImpl<'a, K, V> = btree_map::Iter<'a, K, V>;
    pub type IterMutImpl<'a, K, V> = btree_map::IterMut<'a, K, V>;
    pub type IntoIterImpl<K, V> = btree_map::IntoIter<K, V>;
    pub type KeysImpl<'a, K, V> = btree_map::Keys<'a, K, V>;
    pub type ValuesImpl<'a, K, V> = btree_map::Values<'a, K, V>;
    pub type ValuesMutImpl<'a, K, V> = btree_map::ValuesMut<'a, K, V>;
}

/// A default key-value mapping.
///
/// Provides an API compatible with both [`HashMap`] and [`BTreeMap`].
///
/// [`HashMap`]: hashbrown::HashMap
/// [`BTreeMap`]: std::collections::BTreeMap
#[derive(Debug, Clone)]
pub struct Map<K, V> {
    inner: detail::MapImpl<K, V>,
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self {
            inner: detail::MapImpl::default(),
        }
    }
}

impl<K, V> Map<K, V> {
    /// Creates a new empty [`Map`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the [`Map`], removing all elements.
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Returns the number of elements in the [`Map`].
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the [`Map`] contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an iterator that yields the items in the [`Map`].
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.inner.iter(),
        }
    }

    /// Returns a mutable iterator that yields the items in the [`Map`].
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            inner: self.inner.iter_mut(),
        }
    }

    /// Returns an iterator that yields the keys in the [`Map`].
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            inner: self.inner.keys(),
        }
    }

    /// Returns an iterator that yields the values in the [`Map`].
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            inner: self.inner.values(),
        }
    }

    /// Returns a mutable iterator that yields the values in the [`Map`].
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            inner: self.inner.values_mut(),
        }
    }
}

impl<K, V> Map<K, V>
where
    K: Hash + Eq + Ord,
{
    /// Reserves capacity for at least `additional` more elements to be inserted in the [`Map`].
    pub fn reserve(&mut self, additional: usize) {
        #[cfg(not(feature = "no-hash-maps"))]
        self.inner.reserve(additional);
        #[cfg(feature = "no-hash-maps")]
        let _ = additional;
    }

    /// Returns true if `key` is contains in the [`Map`].
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord,
    {
        self.inner.contains_key(key)
    }

    /// Returns a reference to the value corresponding to the `key`.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord,
    {
        self.inner.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord,
    {
        self.inner.get_mut(key)
    }

    /// Inserts a key-value pair into the [`Map`].
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// Removes a key from the [`Map`], returning the value at the key if the key was previously in the map.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + Ord,
    {
        self.inner.remove(key)
    }

    /// Gets the given key's corresponding entry in the [`Map`] for in-place manipulation.
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        match self.inner.entry(key) {
            detail::EntryImpl::Occupied(entry) => Entry::Occupied(OccupiedEntry { inner: entry }),
            detail::EntryImpl::Vacant(entry) => Entry::Vacant(VacantEntry { inner: entry }),
        }
    }
}

impl<K, Q, V> Index<&Q> for Map<K, V>
where
    K: Borrow<Q> + Hash + Eq + Ord,
    Q: ?Sized + Hash + Eq + Ord,
{
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        &self.inner[key]
    }
}

impl<K, V> Extend<(K, V)> for Map<K, V>
where
    K: Eq + Hash + Ord,
{
    fn extend<Iter: IntoIterator<Item = (K, V)>>(&mut self, iter: Iter) {
        self.inner.extend(iter)
    }
}

/// A view into a single entry in a [`Map`], which may either be vacant or occupied.
///
/// This enum is constructed from the entry method on [`Map`].
#[derive(Debug)]
pub enum Entry<'a, K: Ord, V> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Ord,
{
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the [`Entry`] by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Ensures a value is in the [`Entry`] by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a reference to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is
    /// unnecessary, unlike with `.or_insert_with(|| ... )`.
    #[inline]
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    /// Returns a reference to this [`Entry`]'s key.
    pub fn key(&self) -> &K {
        match *self {
            Self::Occupied(ref entry) => entry.key(),
            Self::Vacant(ref entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied [`Entry`] before any
    /// potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Self::Occupied(mut entry) => {
                f(entry.get_mut());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Ord,
    V: Default,
{
    /// Ensures a value is in the [`Entry`] by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

/// A view into an occupied entry in a [`Map`].
///
/// It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, K, V> {
    inner: detail::OccupiedEntryImpl<'a, K, V>,
}

impl<'a, K, V> Debug for OccupiedEntry<'a, K, V>
where
    K: Debug + Ord + 'a,
    V: Debug + 'a,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Ord + 'a,
    V: 'a,
{
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        self.inner.key()
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        self.inner.get()
    }

    /// Gets a mutable reference to the value in the entry.
    pub fn get_mut(&mut self) -> &mut V {
        self.inner.get_mut()
    }

    /// Sets the value of the entry with the [`OccupiedEntry`]'s key, and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        self.inner.insert(value)
    }

    /// Converts the [`OccupiedEntry`] into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    pub fn into_mut(self) -> &'a mut V {
        self.inner.into_mut()
    }

    /// Take ownership of the key and value from the [`Map`].
    pub fn remove_entry(self) -> (K, V) {
        self.inner.remove_entry()
    }

    /// Takes the value of the entry out of the [`Map`], and returns it.
    pub fn remove(self) -> V {
        self.inner.remove()
    }
}

/// A view into a vacant entry in a [`Map`].
///
/// It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, K, V> {
    inner: detail::VacantEntryImpl<'a, K, V>,
}

impl<'a, K, V> Debug for VacantEntry<'a, K, V>
where
    K: Debug + Ord + 'a,
    V: Debug + 'a,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Ord + 'a,
    V: 'a,
{
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        self.inner.key()
    }

    /// Take ownership of the key.
    pub fn into_key(self) -> K {
        self.inner.into_key()
    }

    /// Sets the value of the entry with the [`VacantEntry`]'s key, and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash,
    {
        self.inner.insert(value)
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V>
where
    K: Hash + Eq + Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        Self {
            inner: <detail::MapImpl<K, V>>::from_iter(iter),
        }
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the items of a [`Map`].
#[derive(Debug, Clone)]
pub struct Iter<'a, K, V> {
    inner: detail::IterImpl<'a, K, V>,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for Iter<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for Iter<'a, K, V> {}

impl<'a, K: 'a, V: 'a> IntoIterator for &'a mut Map<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over the mutable items of a [`Map`].
#[derive(Debug)]
pub struct IterMut<'a, K, V> {
    inner: detail::IterMutImpl<'a, K, V>,
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for IterMut<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for IterMut<'a, K, V> {}

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter(),
        }
    }
}

/// An iterator over the owned items of an [`Map`].
#[derive(Debug)]
pub struct IntoIter<K, V> {
    inner: detail::IntoIterImpl<K, V>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

/// An iterator over the keys of a [`Map`].
#[derive(Debug, Clone)]
pub struct Keys<'a, K, V> {
    inner: detail::KeysImpl<'a, K, V>,
}

impl<'a, K: 'a, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, K: 'a, V> ExactSizeIterator for Keys<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K: 'a, V> FusedIterator for Keys<'a, K, V> {}

/// An iterator over the values of a [`Map`].
#[derive(Debug, Clone)]
pub struct Values<'a, K, V> {
    inner: detail::ValuesImpl<'a, K, V>,
}

impl<'a, K, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, K, V: 'a> ExactSizeIterator for Values<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K, V: 'a> FusedIterator for Values<'a, K, V> {}

/// An mutable iterator over the values of a [`Map`].
#[derive(Debug)]
pub struct ValuesMut<'a, K, V> {
    inner: detail::ValuesMutImpl<'a, K, V>,
}

impl<'a, K, V: 'a> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, K, V: 'a> ExactSizeIterator for ValuesMut<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K, V: 'a> FusedIterator for ValuesMut<'a, K, V> {}
