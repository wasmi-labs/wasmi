use crate::{AsContext, Extern, ExternType, Func, Global, Memory, Table};
use alloc::{boxed::Box, collections::btree_map};
use core::iter::FusedIterator;

/// An exported WebAssembly value.
///
/// This type is primarily accessed from the [`Instance::exports`] method
/// and describes what names and items are exported from a Wasm [`Instance`].
#[derive(Debug, Clone)]
pub struct Export<'instance> {
    /// The name of the exported item.
    name: &'instance str,
    /// The definition of the exported item.
    definition: Extern,
}

impl<'instance> Export<'instance> {
    /// Creates a new [`Export`] with the given `name` and `definition`.
    pub(crate) fn new(name: &'instance str, definition: Extern) -> Export<'instance> {
        Self { name, definition }
    }

    /// Returns the name by which this export is known.
    pub fn name(&self) -> &'instance str {
        self.name
    }

    /// Return the [`ExternType`] of this export.
    ///
    /// # Panics
    ///
    /// If `ctx` does not own this [`Export`].
    pub fn ty(&self, ctx: impl AsContext) -> ExternType {
        self.definition.ty(ctx)
    }

    /// Consume this [`Export`] and return the underlying [`Extern`].
    pub fn into_extern(self) -> Extern {
        self.definition
    }

    /// Returns the underlying [`Func`], if the [`Export`] is a function or `None` otherwise.
    pub fn into_func(self) -> Option<Func> {
        self.definition.into_func()
    }

    /// Returns the underlying [`Table`], if the [`Export`] is a table or `None` otherwise.
    pub fn into_table(self) -> Option<Table> {
        self.definition.into_table()
    }

    /// Returns the underlying [`Memory`], if the [`Export`] is a linear memory or `None` otherwise.
    pub fn into_memory(self) -> Option<Memory> {
        self.definition.into_memory()
    }

    /// Returns the underlying [`Global`], if the [`Export`] is a global variable or `None` otherwise.
    pub fn into_global(self) -> Option<Global> {
        self.definition.into_global()
    }
}

/// An iterator over the [`Extern`] declarations of an [`Instance`].
#[derive(Debug)]
pub struct ExportsIter<'instance> {
    iter: btree_map::Iter<'instance, Box<str>, Extern>,
}

impl<'instance> ExportsIter<'instance> {
    /// Creates a new [`ExportsIter`].
    pub(super) fn new(iter: btree_map::Iter<'instance, Box<str>, Extern>) -> Self {
        Self { iter }
    }

    /// Prepares an item to match the expected iterator `Item` signature.
    #[allow(clippy::borrowed_box)]
    fn convert_item((name, export): (&'instance Box<str>, &'instance Extern)) -> Export {
        Export::new(name, *export)
    }
}

impl<'instance> Iterator for ExportsIter<'instance> {
    type Item = Export<'instance>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::convert_item)
    }
}

impl DoubleEndedIterator for ExportsIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::convert_item)
    }
}

impl ExactSizeIterator for ExportsIter<'_> {}
impl FusedIterator for ExportsIter<'_> {}
