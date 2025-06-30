use alloc::vec::Vec;
use core::{slice, str};

/// Wasm custom sections.
#[derive(Default, Debug)]
pub struct CustomSections {
    inner: CustomSectionsInner,
}

impl CustomSections {
    /// Returns an iterator over the [`CustomSection`]s stored in `self`.
    #[inline]
    pub fn iter(&self) -> CustomSectionsIter<'_> {
        self.inner.iter()
    }
}

/// A builder for [`CustomSections`].
#[derive(Default, Debug)]
pub struct CustomSectionsBuilder {
    inner: CustomSectionsInner,
}

impl CustomSectionsBuilder {
    /// Pushes a new custom section segment to the [`CustomSectionsBuilder`].
    #[inline]
    pub fn push(&mut self, name: &str, data: &[u8]) {
        self.inner.push(name, data);
    }

    /// Finalize construction of the [`CustomSections`].
    #[inline]
    pub fn finish(self) -> CustomSections {
        CustomSections { inner: self.inner }
    }
}

/// Internal representation of [`CustomSections`].
#[derive(Debug, Default)]
pub struct CustomSectionsInner {
    /// The name and data lengths of each Wasm custom section.
    items: Vec<CustomSectionInner>,
    /// The combined name and data of all Wasm custom sections.
    names_and_data: Vec<u8>,
}

/// Internal representation of a Wasm [`CustomSection`].
#[derive(Debug, Copy, Clone)]
pub struct CustomSectionInner {
    /// The length in bytes of the Wasm custom section name.
    len_name: usize,
    /// The length in bytes of the Wasm custom section data.
    len_data: usize,
}

impl CustomSectionsInner {
    /// Pushes a new custom section segment to the [`CustomSectionsBuilder`].
    #[inline]
    pub fn push(&mut self, name: &str, data: &[u8]) {
        let name_bytes = name.as_bytes();
        self.names_and_data.extend_from_slice(name_bytes);
        self.names_and_data.extend_from_slice(data);
        self.items.push(CustomSectionInner {
            len_name: name_bytes.len(),
            len_data: data.len(),
        })
    }

    /// Returns an iterator over the [`CustomSection`]s stored in `self`.
    #[inline]
    pub fn iter(&self) -> CustomSectionsIter<'_> {
        CustomSectionsIter {
            items: self.items.iter(),
            names_and_data: &self.names_and_data[..],
        }
    }
}

/// A Wasm custom section.
#[derive(Debug)]
pub struct CustomSection<'a> {
    /// The name of the custom section.
    name: &'a str,
    /// The undecoded data of the custom section.
    data: &'a [u8],
}

impl<'a> CustomSection<'a> {
    /// Returns the name or identifier of the [`CustomSection`].
    #[inline]
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns a shared reference to the data of the [`CustomSection`].
    #[inline]
    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

/// An iterator over the custom sections of a Wasm module.
#[derive(Debug)]
pub struct CustomSectionsIter<'a> {
    items: slice::Iter<'a, CustomSectionInner>,
    names_and_data: &'a [u8],
}

impl<'a> Iterator for CustomSectionsIter<'a> {
    type Item = CustomSection<'a>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.next()?;
        let names_and_data = self.names_and_data;
        let (name, names_and_data) = names_and_data.split_at(item.len_name);
        let (data, names_and_data) = names_and_data.split_at(item.len_data);
        self.names_and_data = names_and_data;
        // Safety: We encoded this part of the data buffer from the bytes of a string previously.
        let name = unsafe { str::from_utf8_unchecked(name) };
        Some(CustomSection { name, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut builder = CustomSectionsBuilder::default();
        builder.push("A", b"first");
        builder.push("B", b"second");
        builder.push("C", b"third");
        builder.push("", b"fourth"); // empty name
        builder.push("E", &[]); // empty data
        let custom_sections = builder.finish();
        let mut iter = custom_sections.iter();
        assert_eq!(
            iter.next().map(|s| (s.name(), s.data())),
            Some(("A", &b"first"[..]))
        );
        assert_eq!(
            iter.next().map(|s| (s.name(), s.data())),
            Some(("B", &b"second"[..]))
        );
        assert_eq!(
            iter.next().map(|s| (s.name(), s.data())),
            Some(("C", &b"third"[..]))
        );
        assert_eq!(
            iter.next().map(|s| (s.name(), s.data())),
            Some(("", &b"fourth"[..]))
        );
        assert_eq!(
            iter.next().map(|s| (s.name(), s.data())),
            Some(("E", &b""[..]))
        );
        assert_eq!(iter.next().map(|s| (s.name(), s.data())), None);
    }
}
