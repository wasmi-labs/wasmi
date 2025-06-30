use crate::{UntypedVal, ValType};
use alloc::boxed::Box;

/// A Wasm [`ElementSegment`].
#[derive(Debug)]
pub struct ElementSegment {
    /// The [`ValType`] of elements of this [`ElementSegment`].
    ty: ValType,
    /// Pre-resolved items of the Wasm element segment.
    items: Box<[UntypedVal]>,
}

impl ElementSegment {
    /// Creates a new [`ElementSegment`].
    ///
    /// # Panics
    ///
    /// If the length of `items` exceeds `u32`.
    pub fn new<I>(ty: ValType, items: I) -> Self
    where
        I: IntoIterator<Item = UntypedVal>,
    {
        let items: Box<[UntypedVal]> = items.into_iter().collect();
        assert!(
            u32::try_from(items.len()).is_ok(),
            "element segment has too many items: {}",
            items.len()
        );
        Self { ty, items }
    }

    /// Returns `self` as [`ElementSegmentRef`].
    pub fn as_ref(&self) -> ElementSegmentRef<'_> {
        ElementSegmentRef::from(self)
    }

    /// Returns the [`ValType`] of elements in the [`ElementSegment`].
    pub fn ty(&self) -> ValType {
        self.ty
    }

    /// Returns the items of the [`ElementSegment`].
    pub fn items(&self) -> &[UntypedVal] {
        &self.items[..]
    }

    /// Returns the number of items in the [`ElementSegment`].
    pub fn size(&self) -> u32 {
        self.as_ref().size()
    }

    /// Drops the items of the [`ElementSegment`].
    pub fn drop_items(&mut self) {
        self.items = [].into();
    }
}

/// A shared reference to a Wasm [`ElementSegment`].
#[derive(Debug, Copy, Clone)]
pub struct ElementSegmentRef<'a> {
    /// The [`ValType`] of elements of this [`ElementSegment`].
    ty: ValType,
    /// The items of the Wasm element segment.
    items: &'a [UntypedVal],
}

impl<'a> From<&'a ElementSegment> for ElementSegmentRef<'a> {
    fn from(element: &'a ElementSegment) -> Self {
        Self {
            ty: element.ty(),
            items: element.items(),
        }
    }
}

impl<'a> ElementSegmentRef<'a> {
    /// Returns the [`ValType`] of elements in the [`ElementSegment`].
    pub fn ty(self) -> ValType {
        self.ty
    }

    /// Returns the items of the [`ElementSegment`].
    pub fn items(self) -> &'a [UntypedVal] {
        self.items
    }

    /// Returns the number of items in the [`ElementSegment`].
    pub fn size(self) -> u32 {
        let len = self.items().len();
        u32::try_from(len).unwrap_or_else(|_| {
            panic!("element segments are ensured to have at most 2^32 items but found: {len}")
        })
    }
}
