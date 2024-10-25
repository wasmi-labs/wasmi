use core::slice;

/// Names of exported Wasm objects from a fuzzed Wasm module.
#[derive(Debug, Default)]
pub struct ModuleExports {
    /// Names of exported functions.
    funcs: StringSequence,
    /// Names of exported global variables.
    globals: StringSequence,
    /// Names of exported linear memories.
    memories: StringSequence,
    /// Names of exported tables.
    tables: StringSequence,
}

impl ModuleExports {
    /// Pushes an exported function `name` to `self`.
    pub(crate) fn push_func(&mut self, name: &str) {
        self.funcs.push(name);
    }

    /// Pushes an exported global `name` to `self`.
    pub(crate) fn push_global(&mut self, name: &str) {
        self.globals.push(name);
    }

    /// Pushes an exported memory `name` to `self`.
    pub(crate) fn push_memory(&mut self, name: &str) {
        self.memories.push(name);
    }

    /// Pushes an exported table `name` to `self`.
    pub(crate) fn push_table(&mut self, name: &str) {
        self.tables.push(name);
    }

    /// Returns an iterator yielding the names of the exported Wasm functions.
    pub fn funcs(&self) -> StringSequenceIter {
        self.funcs.iter()
    }

    /// Returns an iterator yielding the names of the exported Wasm globals.
    pub fn globals(&self) -> StringSequenceIter {
        self.globals.iter()
    }

    /// Returns an iterator yielding the names of the exported Wasm memories.
    pub fn memories(&self) -> StringSequenceIter {
        self.memories.iter()
    }

    /// Returns an iterator yielding the names of the exported Wasm tables.
    pub fn tables(&self) -> StringSequenceIter {
        self.tables.iter()
    }
}

/// An append-only sequence of strings.
#[derive(Debug, Default)]
pub struct StringSequence {
    /// The underlying sequence of strings.
    strings: Vec<Box<str>>,
}

impl StringSequence {
    /// Pushes another string `s` to `self`.
    pub fn push(&mut self, s: &str) {
        self.strings.push(Box::from(s));
    }

    /// Returns an iterator over the strings in `self`.
    ///
    /// The iterator yields the strings in order of their insertion.
    pub fn iter(&self) -> StringSequenceIter {
        StringSequenceIter {
            iter: self.strings.iter(),
        }
    }
}

/// An iterator yielding the strings of a [`StringSequence`].
#[derive(Debug)]
pub struct StringSequenceIter<'a> {
    /// The underlying iterator over strings.
    iter: slice::Iter<'a, Box<str>>,
}

impl<'a> Iterator for StringSequenceIter<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|s| &**s)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl ExactSizeIterator for StringSequenceIter<'_> {}
