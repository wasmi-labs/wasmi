use crate::arena::ArenaError;
use alloc::boxed::Box;
use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

/// The base-2 logarithm of the first bucket's item count.
const LEN_BUCKET0_LOG2: usize = 5;

/// The number of items in the first bucket.
const LEN_BUCKET0: usize = 1 << LEN_BUCKET0_LOG2;

/// The maximum number of buckets, capping total capacity at `2^32 - 32` items.
///
/// Also keeps the largest bucket size (`1 << 31`) within a 32-bit `usize`.
const MAX_BUCKETS: usize = 27;

/// An append-only vector that hands out index handles with stable element addresses.
///
/// Once pushed, an item never moves, so raw pointers obtained from [`get`](StableVec::get) stay
/// valid for the vector's lifetime. The bucket array is stored inline; wrap the whole
/// [`StableVec`] in a `Box` to move it off the stack.
pub struct StableVec<T> {
    /// The total number of items across all `buckets`.
    len: usize,
    /// Thin pointers to each bucket's heap allocation, or `None` if not yet allocated.
    ///
    /// Bucket `n` is allocated for `size_of_bucket_at(n)` items; only its first `len - start` slots
    /// (where `start = first_index_of_bucket(n)`) are initialized. Length and capacity are inferred,
    /// never stored.
    buckets: [Option<NonNull<T>>; MAX_BUCKETS],
    /// Marks that `self` owns its `T`s (for drop-check and variance).
    marker: PhantomData<T>,
}

// Safety: a `StableVec<T>` uniquely owns its `T`s like a `Box<[T]>`, so it is `Send`/`Sync`
//         exactly when `T` is.
unsafe impl<T: Send> Send for StableVec<T> {}
unsafe impl<T: Sync> Sync for StableVec<T> {}

impl<T> Default for StableVec<T> {
    #[inline]
    fn default() -> Self {
        Self {
            buckets: [const { None }; MAX_BUCKETS],
            len: 0,
            marker: PhantomData,
        }
    }
}

impl<T> StableVec<T> {
    /// Creates a new empty [`StableVec`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of items stored in the [`StableVec`].
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the [`StableVec`] contains no items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends `value` to the [`StableVec`] and returns its index.
    ///
    /// # Panics
    ///
    /// If the vector is full (`2^32 - 32` items); unreachable in practice as allocation fails first.
    #[inline]
    pub fn push(&mut self, value: T) -> usize {
        let index = self.len;
        let (bucket_index, slot) = Self::locate(index);
        let ptr = match self.buckets[bucket_index] {
            Some(ptr) => ptr,
            None => {
                let ptr = Self::alloc_bucket(Self::size_of_bucket_at(bucket_index));
                self.buckets[bucket_index] = Some(ptr);
                ptr
            }
        };
        // Safety: `slot` is in bounds for this bucket and currently uninitialized.
        unsafe { ptr.as_ptr().add(slot).write(value) };
        self.len += 1;
        index
    }

    /// Returns a shared reference to the item at `index`, or `None` if out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let (bucket_index, slot) = Self::locate(index);
        // Safety: `index < len` implies the bucket is allocated and `slot` is initialized.
        let ptr = unsafe { self.buckets[bucket_index].unwrap_unchecked() };
        Some(unsafe { &*ptr.as_ptr().add(slot) })
    }

    /// Returns an exclusive reference to the item at `index`, or `None` if out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        let (bucket_index, slot) = Self::locate(index);
        // Safety: `index < len` implies the bucket is allocated and `slot` is initialized.
        let ptr = unsafe { self.buckets[bucket_index].unwrap_unchecked() };
        Some(unsafe { &mut *ptr.as_ptr().add(slot) })
    }

    /// Returns a raw pointer to the item at `index`, or `None` if out of bounds.
    ///
    /// Unlike [`get_mut`](StableVec::get_mut), this never forms an intermediate `&mut T`, so the
    /// pointer carries the bucket allocation's own provenance.
    #[inline]
    pub fn get_mut_ptr(&mut self, index: usize) -> Option<NonNull<T>> {
        if index >= self.len {
            return None;
        }
        let (bucket_index, slot) = Self::locate(index);
        // Safety: `index < len` implies the bucket is allocated and `slot` is initialized.
        let ptr = unsafe { self.buckets[bucket_index].unwrap_unchecked() };
        Some(unsafe { NonNull::new_unchecked(ptr.as_ptr().add(slot)) })
    }

    /// Returns exclusive references to the items at `a` and `b`.
    ///
    /// # Errors
    ///
    /// - If `indices[0]` and `indices[1]` refer to the same item, a.k.a. aliasing each other.
    /// - If `indices[0]` or `indices[1]` is out of bounds for the arena.
    #[inline]
    pub fn get_disjoint_mut(&mut self, indices: [usize; 2]) -> Result<[&mut T; 2], ArenaError> {
        let [a, b] = indices;
        if a == b {
            return Err(ArenaError::AliasingPairAccess);
        }
        if a >= self.len || b >= self.len {
            return Err(ArenaError::KeyOutOfBounds);
        }
        let (ba, sa) = Self::locate(a);
        let (bb, sb) = Self::locate(b);
        // Safety: `a != b` and both are in bounds, so the two slots are initialized and disjoint;
        //         the produced exclusive references therefore never alias.
        unsafe {
            let pa = self.buckets[ba].unwrap_unchecked().as_ptr().add(sa);
            let pb = self.buckets[bb].unwrap_unchecked().as_ptr().add(sb);
            Ok([&mut *pa, &mut *pb])
        }
    }

    /// Returns an iterator over the items in insertion order.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            vector: self,
            front: 0,
            back: self.len,
        }
    }

    /// Returns an iterator over exclusive references to the items in insertion order.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let back = self.len;
        IterMut {
            vector: self,
            front: 0,
            back,
        }
    }

    /// Allocates an uninitialized bucket for `size` items and returns a thin pointer to it.
    #[inline]
    fn alloc_bucket(size: usize) -> NonNull<T> {
        let bucket = Box::<[T]>::new_uninit_slice(size);
        let ptr = Box::into_raw(bucket).cast::<T>();
        // Safety: `Box::into_raw` never returns a null pointer.
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// Maps a global `index` to its `(bucket, slot)` position.
    #[inline]
    fn locate(index: usize) -> (usize, usize) {
        let j = index as u64 + LEN_BUCKET0 as u64;
        let msb = 63 - j.leading_zeros() as usize; // floor(log2(j))
        let bucket = msb - LEN_BUCKET0_LOG2;
        let slot = (j - (1u64 << msb)) as usize;
        (bucket, slot)
    }

    /// Returns the number of items stored in the bucket at index `n`.
    #[inline]
    const fn size_of_bucket_at(n: usize) -> usize {
        1usize << (LEN_BUCKET0_LOG2 + n)
    }

    /// Returns the global index of the first item stored in the bucket at index `n`.
    #[inline]
    const fn first_index_of_bucket(n: usize) -> usize {
        Self::size_of_bucket_at(n) - LEN_BUCKET0
    }
}

impl<T> Drop for StableVec<T> {
    fn drop(&mut self) {
        let len = self.len;
        for (n, slot) in self.buckets.iter_mut().enumerate() {
            let Some(ptr) = slot.take() else {
                break; // buckets are populated in order, so the rest are empty
            };
            let cap = Self::size_of_bucket_at(n);
            let filled = len.saturating_sub(Self::first_index_of_bucket(n)).min(cap);
            // Safety: drop the initialized items, then free the whole allocation as `MaybeUninit`
            //         (which does not drop) to avoid a double-drop. `cap` matches the allocation.
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut(ptr.as_ptr(), filled));
                let raw = ptr::slice_from_raw_parts_mut(ptr.as_ptr().cast::<MaybeUninit<T>>(), cap);
                drop(Box::from_raw(raw));
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for StableVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Clone> Clone for StableVec<T> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: PartialEq> PartialEq for StableVec<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for StableVec<T> {}

impl<T: PartialOrd> PartialOrd for StableVec<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord> Ord for StableVec<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: Hash> Hash for StableVec<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Mirrors the `[T]`/`Vec<T>` hash: length prefix followed by the items.
        self.len.hash(state);
        for item in self {
            item.hash(state);
        }
    }
}

impl<T> FromIterator<T> for StableVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vector = Self::new();
        vector.extend(iter);
        vector
    }
}

impl<T> Extend<T> for StableVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<'a, T> IntoIterator for &'a StableVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut StableVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over the items of a [`StableVec`] in insertion order.
#[derive(Debug)]
pub struct Iter<'a, T> {
    /// The iterated [`StableVec`].
    vector: &'a StableVec<T>,
    /// The next index yielded from the front.
    front: usize,
    /// One past the next index yielded from the back.
    back: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        let item = self.vector.get(self.front);
        self.front += 1;
        item
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        self.back -= 1;
        self.vector.get(self.back)
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}
impl<'a, T> FusedIterator for Iter<'a, T> {}

/// An iterator over exclusive references to the items of a [`StableVec`] in insertion order.
#[derive(Debug)]
pub struct IterMut<'a, T> {
    /// The iterated [`StableVec`].
    vector: &'a mut StableVec<T>,
    /// The next index yielded from the front.
    front: usize,
    /// One past the next index yielded from the back.
    back: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        let index = self.front;
        self.front += 1;
        // Safety: `index < back <= len`, so the slot is initialized. Reborrowing through a raw
        //         pointer detaches the returned reference from `self`; distinct calls yield
        //         distinct indices, so the handed-out references never alias.
        let item: *mut T = unsafe { self.vector.get_mut(index).unwrap_unchecked() };
        Some(unsafe { &mut *item })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        self.back -= 1;
        // Safety: see `Iterator::next`.
        let item: *mut T = unsafe { self.vector.get_mut(self.back).unwrap_unchecked() };
        Some(unsafe { &mut *item })
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {}
impl<'a, T> FusedIterator for IterMut<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn empty() {
        let vector = StableVec::<i32>::new();
        assert_eq!(vector.len(), 0);
        assert!(vector.is_empty());
        assert_eq!(vector.get(0), None);
    }

    #[test]
    fn push_and_get() {
        let mut vector = StableVec::new();
        // Push enough items to span several buckets (32, 64, 128 -> boundaries at 32 and 96).
        let n = 200;
        for i in 0..n {
            assert_eq!(vector.push(i), i);
        }
        assert_eq!(vector.len(), n);
        assert!(!vector.is_empty());
        for i in 0..n {
            assert_eq!(vector.get(i), Some(&i));
        }
        assert_eq!(vector.get(n), None);
    }

    #[test]
    fn locate_boundaries() {
        // First bucket holds items [0, 32).
        assert_eq!(StableVec::<()>::locate(0), (0, 0));
        assert_eq!(StableVec::<()>::locate(31), (0, 31));
        // Second bucket holds items [32, 96).
        assert_eq!(StableVec::<()>::locate(32), (1, 0));
        assert_eq!(StableVec::<()>::locate(95), (1, 63));
        // Third bucket holds items [96, 224).
        assert_eq!(StableVec::<()>::locate(96), (2, 0));
        assert_eq!(StableVec::<()>::locate(223), (2, 127));
    }

    #[test]
    fn size_of_bucket_at() {
        assert_eq!(StableVec::<()>::size_of_bucket_at(0), 32);
        assert_eq!(StableVec::<()>::size_of_bucket_at(1), 64);
        assert_eq!(StableVec::<()>::size_of_bucket_at(2), 128);
    }

    #[test]
    fn pointers_are_stable() {
        let mut vector = StableVec::new();
        // Fill the first bucket and capture the address of its first and last item.
        for i in 0..32 {
            vector.push(i);
        }
        let ptr0: *const i32 = vector.get(0).unwrap();
        let ptr31: *const i32 = vector.get(31).unwrap();
        // Push far more items, forcing several new bucket allocations.
        for i in 32..1000 {
            vector.push(i);
        }
        // The previously captured pointers must still be valid and unchanged.
        assert_eq!(ptr0, vector.get(0).unwrap() as *const i32);
        assert_eq!(ptr31, vector.get(31).unwrap() as *const i32);
        // Safety: the vector still owns these items and their addresses did not change.
        assert_eq!(unsafe { *ptr0 }, 0);
        assert_eq!(unsafe { *ptr31 }, 31);
    }

    #[test]
    fn get_mut_and_iter() {
        let mut vector = StableVec::new();
        for i in 0..100 {
            vector.push(i);
        }
        for i in 0..100 {
            *vector.get_mut(i).unwrap() *= 2;
        }
        let collected: Vec<usize> = vector.iter().copied().collect();
        let expected: Vec<usize> = (0..100).map(|i| i * 2).collect();
        assert_eq!(collected, expected);
        assert_eq!(vector.get_mut(100), None);
    }

    #[test]
    fn iter_double_ended_and_exact_size() {
        let mut vector = StableVec::new();
        for i in 0..100 {
            vector.push(i);
        }
        // `ExactSizeIterator::len` shrinks as items are taken from both ends.
        let mut it = vector.iter();
        assert_eq!(it.len(), 100);
        assert_eq!(it.next(), Some(&0));
        assert_eq!(it.next_back(), Some(&99));
        assert_eq!(it.len(), 98);
        // Meeting in the middle from alternating ends yields the full range in order.
        let front: Vec<usize> = vector.iter().step_by(1).copied().collect();
        let reversed: Vec<usize> = vector.iter().rev().copied().collect();
        assert_eq!(front, (0..100).collect::<Vec<_>>());
        assert_eq!(reversed, (0..100).rev().collect::<Vec<_>>());
    }

    #[test]
    fn from_iter_extend_and_ref_into_iter() {
        let mut vector: StableVec<usize> = (0..50).collect();
        vector.extend(50..100);
        // `&StableVec` iterates like `iter`.
        let collected: Vec<usize> = (&vector).into_iter().copied().collect();
        assert_eq!(collected, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn clone_and_eq() {
        let a: StableVec<usize> = (0..100).collect();
        let b = a.clone();
        assert_eq!(a, b);
        let mut c = a.clone();
        c.push(100);
        assert_ne!(a, c);
        let d: StableVec<usize> = (0..99).collect();
        assert_ne!(a, d);
    }

    #[test]
    fn ord_is_lexicographic() {
        let a: StableVec<i32> = [1, 2, 3].into_iter().collect();
        let b: StableVec<i32> = [1, 2, 4].into_iter().collect();
        let c: StableVec<i32> = [1, 2].into_iter().collect();
        assert!(a < b);
        assert!(c < a); // prefix is less
        assert_eq!(a.cmp(&a.clone()), Ordering::Equal);
    }

    #[test]
    fn hash_matches_for_equal_vectors() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        fn hash_of(vector: &StableVec<usize>) -> u64 {
            let mut hasher = DefaultHasher::new();
            vector.hash(&mut hasher);
            hasher.finish()
        }
        let a: StableVec<usize> = (0..100).collect();
        let b = a.clone();
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn iter_mut_updates_in_place() {
        let mut vector: StableVec<usize> = (0..100).collect();
        for item in &mut vector {
            *item *= 2;
        }
        // `IterMut` is also double-ended.
        let reversed: Vec<usize> = vector.iter_mut().rev().map(|item| *item).collect();
        assert_eq!(reversed, (0..100).rev().map(|i| i * 2).collect::<Vec<_>>());
    }

    #[test]
    fn get_pair_mut_disjoint() {
        let mut vector: StableVec<usize> = (0..100).collect();
        let [a, b] = vector.get_disjoint_mut([10, 90]).unwrap();
        *a = 111;
        *b = 999;
        assert_eq!(vector.get(10), Some(&111));
        assert_eq!(vector.get(90), Some(&999));
        assert!(matches!(
            vector.get_disjoint_mut([5, 5]),
            Err(ArenaError::AliasingPairAccess)
        ));
        assert!(matches!(
            vector.get_disjoint_mut([5, 100]),
            Err(ArenaError::KeyOutOfBounds)
        ));
    }

    #[test]
    fn iter_empty() {
        let vector = StableVec::<i32>::new();
        assert_eq!(vector.iter().next(), None);
        assert_eq!(vector.iter().count(), 0);
    }

    #[test]
    fn debug() {
        let mut vector = StableVec::new();
        vector.push(1);
        vector.push(2);
        vector.push(3);
        assert_eq!(std::format!("{vector:?}"), "[1, 2, 3]");
    }

    #[test]
    fn drops_all_items() {
        use std::rc::Rc;
        let counter = Rc::new(());
        let mut vector = StableVec::new();
        // Span multiple buckets with a partially-filled last bucket.
        for _ in 0..100 {
            vector.push(Rc::clone(&counter));
        }
        assert_eq!(Rc::strong_count(&counter), 101);
        drop(vector);
        assert_eq!(Rc::strong_count(&counter), 1);
    }
}
