use core::slice;
use std::{boxed::Box, vec::Vec};

/// Wasm custom sections.
#[derive(Default, Debug)]
pub struct CustomSections {
    items: Vec<CustomSection>,
}

impl CustomSections {
    /// Returns an iterator over the [`CustomSection`]s stored in `self`.
    #[inline]
    pub fn iter(&self) -> CustomSectionsIter {
        CustomSectionsIter {
            iter: self.items.iter(),
        }
    }
}

/// A builder for [`CustomSections`].
#[derive(Default, Debug)]
pub struct CustomSectionsBuilder {
    items: Vec<CustomSection>,
}

impl CustomSectionsBuilder {
    /// Pushes a new custom section segment to the [`CustomSectionsBuilder`].
    #[inline]
    pub fn push(&mut self, name: &str, data: &[u8]) {
        self.items.push(CustomSection {
            name: name.into(),
            data: data.into(),
        })
    }

    /// Finalize construction of the [`CustomSections`].
    #[inline]
    pub fn finish(self) -> CustomSections {
        CustomSections { items: self.items }
    }
}

/// The data of a Wasm custom section.
#[derive(Debug)]
pub struct CustomSection {
    /// The name of the custom section.
    name: Box<str>,
    /// The undecoded data of the custom section.
    data: Box<[u8]>,
}

impl CustomSection {
    /// Returns the name or identifier of the [`CustomSection`].
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a shared reference to the data of the [`CustomSection`].
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// An iterator over the custom sections of a Wasm module.
#[derive(Debug)]
pub struct CustomSectionsIter<'a> {
    iter: slice::Iter<'a, CustomSection>,
}

impl<'a> Iterator for CustomSectionsIter<'a> {
    type Item = &'a CustomSection;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
