use core::mem;
use alloc::vec::Vec;

/// A [`Vec`]-like data structure with fast access to the last item.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HeadVec<T> {
    /// The top (or last) item in the [`HeadVec`].
    head: Option<T>,
    /// The rest of the items in the [`HeadVec`] excluding the last item.
    rest: Vec<T>,
}

impl<T> Default for HeadVec<T> {
    #[inline]
    fn default() -> Self {
        Self {
            head: None,
            rest: Vec::new(),
        }
    }
}

impl<T> HeadVec<T> {
    /// Removes all items from the [`HeadVec`].
    #[inline]
    pub fn clear(&mut self) {
        self.head = None;
        self.rest.clear();
    }

    /// Returns the number of items stored in the [`HeadVec`].
    #[inline]
    pub fn len(&self) -> usize {
        match self.head {
            Some(_) => 1 + self.rest.len(),
            None => 0,
        }
    }

    /// Returns `true` if the [`HeadVec`] contains no items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a shared reference to the last item in the [`HeadVec`] if any.
    ///
    /// Returns `None` if the [`HeadVec`] is empty.
    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.head.as_ref()
    }

    /// Returns an exclusive reference to the last item in the [`HeadVec`] if any.
    ///
    /// Returns `None` if the [`HeadVec`] is empty.
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut()
    }

    /// Pushes a new `value` onto the [`HeadVec`].
    #[inline]
    pub fn push(&mut self, value: T) {
        let prev_head = self.head.replace(value);
        if let Some(prev_head) = prev_head {
            self.rest.push(prev_head);
        }
    }

    /// Pops the last `value` from the [`HeadVec`] if any.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        let new_top = self.rest.pop();
        mem::replace(&mut self.head, new_top)
    }
}
