use alloc::boxed::Box;
use core::{
    fmt,
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

/// An append-only arena that hands out index handles with stable element addresses.
///
/// Once pushed, an item never moves, so raw pointers obtained from [`get`](StableArena::get) stay
/// valid for the arena's lifetime. The bucket array is stored inline; wrap the whole
/// [`StableArena`] in a `Box` to move it off the stack.
pub struct StableArena<T> {
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

// Safety: a `StableArena<T>` uniquely owns its `T`s like a `Box<[T]>`, so it is `Send`/`Sync`
//         exactly when `T` is.
unsafe impl<T: Send> Send for StableArena<T> {}
unsafe impl<T: Sync> Sync for StableArena<T> {}

impl<T> Default for StableArena<T> {
    #[inline]
    fn default() -> Self {
        Self {
            buckets: [const { None }; MAX_BUCKETS],
            len: 0,
            marker: PhantomData,
        }
    }
}

impl<T> StableArena<T> {
    /// Creates a new empty [`StableArena`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of items stored in the [`StableArena`].
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the [`StableArena`] contains no items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends `value` to the [`StableArena`] and returns its index.
    ///
    /// # Panics
    ///
    /// If the arena is full (`2^32 - 32` items); unreachable in practice as allocation fails first.
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

    /// Returns an iterator over the items in insertion order.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            arena: self,
            front: 0,
            back: self.len,
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

impl<T> Drop for StableArena<T> {
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

impl<T: fmt::Debug> fmt::Debug for StableArena<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// An iterator over the items of a [`StableArena`] in insertion order.
pub struct Iter<'a, T> {
    /// The iterated [`StableArena`].
    arena: &'a StableArena<T>,
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
        let item = self.arena.get(self.front);
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
        self.arena.get(self.back)
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}
impl<'a, T> FusedIterator for Iter<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn empty() {
        let arena = StableArena::<i32>::new();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        assert_eq!(arena.get(0), None);
    }

    #[test]
    fn push_and_get() {
        let mut arena = StableArena::new();
        // Push enough items to span several buckets (32, 64, 128 -> boundaries at 32 and 96).
        let n = 200;
        for i in 0..n {
            assert_eq!(arena.push(i), i);
        }
        assert_eq!(arena.len(), n);
        assert!(!arena.is_empty());
        for i in 0..n {
            assert_eq!(arena.get(i), Some(&i));
        }
        assert_eq!(arena.get(n), None);
    }

    #[test]
    fn locate_boundaries() {
        // First bucket holds items [0, 32).
        assert_eq!(StableArena::<()>::locate(0), (0, 0));
        assert_eq!(StableArena::<()>::locate(31), (0, 31));
        // Second bucket holds items [32, 96).
        assert_eq!(StableArena::<()>::locate(32), (1, 0));
        assert_eq!(StableArena::<()>::locate(95), (1, 63));
        // Third bucket holds items [96, 224).
        assert_eq!(StableArena::<()>::locate(96), (2, 0));
        assert_eq!(StableArena::<()>::locate(223), (2, 127));
    }

    #[test]
    fn size_of_bucket_at() {
        assert_eq!(StableArena::<()>::size_of_bucket_at(0), 32);
        assert_eq!(StableArena::<()>::size_of_bucket_at(1), 64);
        assert_eq!(StableArena::<()>::size_of_bucket_at(2), 128);
    }

    #[test]
    fn pointers_are_stable() {
        let mut arena = StableArena::new();
        // Fill the first bucket and capture the address of its first and last item.
        for i in 0..32 {
            arena.push(i);
        }
        let ptr0: *const i32 = arena.get(0).unwrap();
        let ptr31: *const i32 = arena.get(31).unwrap();
        // Push far more items, forcing several new bucket allocations.
        for i in 32..1000 {
            arena.push(i);
        }
        // The previously captured pointers must still be valid and unchanged.
        assert_eq!(ptr0, arena.get(0).unwrap() as *const i32);
        assert_eq!(ptr31, arena.get(31).unwrap() as *const i32);
        // Safety: the arena still owns these items and their addresses did not change.
        assert_eq!(unsafe { *ptr0 }, 0);
        assert_eq!(unsafe { *ptr31 }, 31);
    }

    #[test]
    fn get_mut_and_iter() {
        let mut arena = StableArena::new();
        for i in 0..100 {
            arena.push(i);
        }
        for i in 0..100 {
            *arena.get_mut(i).unwrap() *= 2;
        }
        let collected: Vec<usize> = arena.iter().copied().collect();
        let expected: Vec<usize> = (0..100).map(|i| i * 2).collect();
        assert_eq!(collected, expected);
        assert_eq!(arena.get_mut(100), None);
    }

    #[test]
    fn iter_double_ended_and_exact_size() {
        let mut arena = StableArena::new();
        for i in 0..100 {
            arena.push(i);
        }
        // `ExactSizeIterator::len` shrinks as items are taken from both ends.
        let mut it = arena.iter();
        assert_eq!(it.len(), 100);
        assert_eq!(it.next(), Some(&0));
        assert_eq!(it.next_back(), Some(&99));
        assert_eq!(it.len(), 98);
        // Meeting in the middle from alternating ends yields the full range in order.
        let front: Vec<usize> = arena.iter().step_by(1).copied().collect();
        let reversed: Vec<usize> = arena.iter().rev().copied().collect();
        assert_eq!(front, (0..100).collect::<Vec<_>>());
        assert_eq!(reversed, (0..100).rev().collect::<Vec<_>>());
    }

    #[test]
    fn iter_empty() {
        let arena = StableArena::<i32>::new();
        assert_eq!(arena.iter().next(), None);
        assert_eq!(arena.iter().count(), 0);
    }

    #[test]
    fn debug() {
        let mut arena = StableArena::new();
        arena.push(1);
        arena.push(2);
        arena.push(3);
        assert_eq!(std::format!("{arena:?}"), "[1, 2, 3]");
    }

    #[test]
    fn drops_all_items() {
        use std::rc::Rc;
        let counter = Rc::new(());
        let mut arena = StableArena::new();
        // Span multiple buckets with a partially-filled last bucket.
        for _ in 0..100 {
            arena.push(Rc::clone(&counter));
        }
        assert_eq!(Rc::strong_count(&counter), 101);
        drop(arena);
        assert_eq!(Rc::strong_count(&counter), 1);
    }
}
