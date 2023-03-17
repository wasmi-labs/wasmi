use crate::{
    module,
    module::{ConstExpr, ElementSegmentItems},
    store::Stored,
    AsContext,
    AsContextMut,
};
use wasmi_arena::ArenaIndex;
use wasmi_core::ValueType;

/// A raw index to a element segment entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElementSegmentIdx(u32);

impl ArenaIndex for ElementSegmentIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as element segment index: {error}")
        });
        Self(value)
    }
}

/// A Wasm data segment reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ElementSegment(Stored<ElementSegmentIdx>);

impl ElementSegment {
    /// Creates a new linear memory reference.
    pub fn from_inner(stored: Stored<ElementSegmentIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub fn as_inner(&self) -> &Stored<ElementSegmentIdx> {
        &self.0
    }

    /// Allocates a new [`ElementSegment`] on the store.
    ///
    /// # Errors
    ///
    /// If more than [`u32::MAX`] much linear memory is allocated.
    pub fn new(mut ctx: impl AsContextMut, segment: &module::ElementSegment) -> Self {
        let entity = ElementSegmentEntity::from(segment);
        ctx.as_context_mut()
            .store
            .inner
            .alloc_element_segment(entity)
    }

    /// Returns the number of items in the [`ElementSegment`].
    pub fn size(&self, ctx: impl AsContext) -> u32 {
        ctx.as_context()
            .store
            .inner
            .resolve_element_segment(self)
            .size()
    }

    /// Drops the items of the [`ElementSegment`].
    pub fn drop_items(&self, mut ctx: impl AsContextMut) {
        ctx.as_context_mut()
            .store
            .inner
            .resolve_element_segment_mut(self)
            .drop_items()
    }
}

/// An instantiated [`ElementSegmentEntity`].
///
/// # Note
///
/// With the `bulk-memory` Wasm proposal it is possible to interact
/// with element segments at runtime. Therefore Wasm instances now have
/// a need to have an instantiated representation of data segments.
#[derive(Debug)]
pub struct ElementSegmentEntity {
    /// The [`ValueType`] of elements of this [`ElementSegmentEntity`].
    ty: ValueType,
    /// The underlying items of the instance element segment.
    ///
    /// # Note
    ///
    /// These items are just readable after instantiation.
    /// Using Wasm `elem.drop` simply replaces the instance
    /// with an empty one.
    items: Option<ElementSegmentItems>,
}

impl From<&'_ module::ElementSegment> for ElementSegmentEntity {
    fn from(segment: &'_ module::ElementSegment) -> Self {
        let ty = segment.ty();
        match segment.kind() {
            module::ElementSegmentKind::Passive | module::ElementSegmentKind::Active(_) => Self {
                ty,
                items: Some(segment.items_cloned()),
            },
            module::ElementSegmentKind::Declared => Self::empty(ty),
        }
    }
}

impl ElementSegmentEntity {
    /// Create an empty [`ElementSegmentEntity`] representing dropped element segments.
    fn empty(ty: ValueType) -> Self {
        Self { ty, items: None }
    }

    /// Returns the [`ValueType`] of elements in the [`ElementSegmentEntity`].
    pub fn ty(&self) -> ValueType {
        self.ty
    }

    /// Returns the number of items in the [`ElementSegment`].
    pub fn size(&self) -> u32 {
        self.items().len() as u32
    }

    /// Returns the items of the [`ElementSegmentEntity`].
    pub fn items(&self) -> &[ConstExpr] {
        self.items
            .as_ref()
            .map(ElementSegmentItems::items)
            .unwrap_or(&[])
    }

    /// Drops the items of the [`ElementSegmentEntity`].
    pub fn drop_items(&mut self) {
        self.items = None;
    }
}
