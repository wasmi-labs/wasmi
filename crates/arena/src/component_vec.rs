use crate::ArenaIndex;
use alloc::vec::Vec;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Index, IndexMut},
};

/// Stores components for entities backed by a [`Vec`].
pub struct ComponentVec<Idx, T> {
    components: Vec<Option<T>>,
    marker: PhantomData<fn() -> Idx>,
}

/// [`ComponentVec`] does not store `Idx` therefore it is `Send` without its bound.
unsafe impl<Idx, T> Send for ComponentVec<Idx, T> where T: Send {}

/// [`ComponentVec`] does not store `Idx` therefore it is `Sync` without its bound.
unsafe impl<Idx, T> Sync for ComponentVec<Idx, T> where T: Send {}

impl<Idx, T> Debug for ComponentVec<Idx, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentVec")
            .field("components", &DebugComponents(&self.components))
            .finish()
    }
}

struct DebugComponents<'a, T>(&'a [Option<T>]);

impl<'a, T> Debug for DebugComponents<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        let components = self
            .0
            .iter()
            .enumerate()
            .filter_map(|(n, component)| component.as_ref().map(|c| (n, c)));
        for (idx, component) in components {
            map.entry(&idx, component);
        }
        map.finish()
    }
}

impl<Idx, T> Default for ComponentVec<Idx, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Idx, T> PartialEq for ComponentVec<Idx, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.components.eq(&other.components)
    }
}

impl<Idx, T> Eq for ComponentVec<Idx, T> where T: Eq {}

impl<Idx, T> ComponentVec<Idx, T> {
    /// Creates a new empty [`ComponentVec`].
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Clears all components from the [`ComponentVec`].
    pub fn clear(&mut self) {
        self.components.clear();
    }
}

impl<Idx, T> ComponentVec<Idx, T>
where
    Idx: ArenaIndex,
{
    /// Sets the `component` for the entity at `index`.
    ///
    /// Returns the old component of the same entity if any.
    pub fn set(&mut self, index: Idx, component: T) -> Option<T> {
        let index = index.into_usize();
        if index >= self.components.len() {
            // The underlying vector does not have enough capacity
            // and is required to be enlarged.
            self.components.resize_with(index + 1, || None);
        }
        self.components[index].replace(component)
    }

    /// Unsets the component for the entity at `index` and returns it if any.
    pub fn unset(&mut self, index: Idx) -> Option<T> {
        self.components
            .get_mut(index.into_usize())
            .and_then(Option::take)
    }

    /// Returns a shared reference to the component at the `index` if any.
    ///
    /// Returns `None` if no component is stored under the `index`.
    #[inline]
    pub fn get(&self, index: Idx) -> Option<&T> {
        self.components
            .get(index.into_usize())
            .and_then(Option::as_ref)
    }

    /// Returns an exclusive reference to the component at the `index` if any.
    ///
    /// Returns `None` if no component is stored under the `index`.
    #[inline]
    pub fn get_mut(&mut self, index: Idx) -> Option<&mut T> {
        self.components
            .get_mut(index.into_usize())
            .and_then(Option::as_mut)
    }
}

impl<Idx, T> Index<Idx> for ComponentVec<Idx, T>
where
    Idx: ArenaIndex,
{
    type Output = T;

    #[inline]
    fn index(&self, index: Idx) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("missing component at index: {}", index.into_usize()))
    }
}

impl<Idx, T> IndexMut<Idx> for ComponentVec<Idx, T>
where
    Idx: ArenaIndex,
{
    #[inline]
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.get_mut(index)
            .unwrap_or_else(|| panic!("missing component at index: {}", index.into_usize()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Add `n` components and perform checks along the way.
    fn add_components(vec: &mut ComponentVec<usize, String>, n: usize) {
        for i in 0..n {
            let mut str = format!("{i}");
            assert!(vec.get(i).is_none());
            assert!(vec.get_mut(i).is_none());
            assert!(vec.set(i, str.clone()).is_none());
            assert_eq!(vec.get(i), Some(&str));
            assert_eq!(vec.get_mut(i), Some(&mut str));
            assert_eq!(&vec[i], &str);
            assert_eq!(&mut vec[i], &mut str);
        }
    }

    #[test]
    fn it_works() {
        let mut vec = <ComponentVec<usize, String>>::new();
        let n = 10;
        add_components(&mut vec, n);
        // Remove components in reverse order for fun.
        // Check if components have been removed properly.
        for i in (0..n).rev() {
            let str = format!("{i}");
            assert_eq!(vec.unset(i), Some(str));
            assert!(vec.get(i).is_none());
            assert!(vec.get_mut(i).is_none());
        }
    }

    #[test]
    fn clear_works() {
        let mut vec = <ComponentVec<usize, String>>::new();
        let n = 10;
        add_components(&mut vec, n);
        // Clear component vec and check if components have been removed properly.
        vec.clear();
        for i in 0..n {
            assert!(vec.get(i).is_none());
            assert!(vec.get_mut(i).is_none());
        }
    }

    #[test]
    fn debug_works() {
        let mut vec = <ComponentVec<usize, String>>::new();
        add_components(&mut vec, 4);
        {
            let debug_str = format!("{vec:?}");
            let expected_str = "\
                ComponentVec { components: {0: \"0\", 1: \"1\", 2: \"2\", 3: \"3\"} }\
            ";
            assert_eq!(debug_str, expected_str);
        }
        {
            let debug_str = format!("{vec:#?}");
            let expected_str = "\
                ComponentVec {\n    \
                    components: {\n        \
                        0: \"0\",\n        \
                        1: \"1\",\n        \
                        2: \"2\",\n        \
                        3: \"3\",\n    \
                    },\n}\
            ";
            assert_eq!(debug_str, expected_str);
        }
    }
}
