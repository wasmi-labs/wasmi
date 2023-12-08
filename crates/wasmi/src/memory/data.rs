use crate::{module, store::Stored, AsContextMut};
use alloc::sync::Arc;
use wasmi_arena::ArenaIndex;

/// A raw index to a data segment entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataSegmentIdx(u32);

impl ArenaIndex for DataSegmentIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as data segment index: {error}")
        });
        Self(value)
    }
}

/// A Wasm data segment reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct DataSegment(Stored<DataSegmentIdx>);

impl DataSegment {
    /// Creates a new linear memory reference.
    pub fn from_inner(stored: Stored<DataSegmentIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub fn as_inner(&self) -> &Stored<DataSegmentIdx> {
        &self.0
    }

    /// Allocates a new [`DataSegment`] on the store.
    ///
    /// # Errors
    ///
    /// If more than [`u32::MAX`] much linear memory is allocated.
    pub fn new(mut ctx: impl AsContextMut, segment: &module::DataSegment) -> Self {
        let entity = DataSegmentEntity::from(segment);
        ctx.as_context_mut().store.inner.alloc_data_segment(entity)
    }
}

/// An instantiated [`DataSegmentEntity`].
///
/// # Note
///
/// With the `bulk-memory` Wasm proposal it is possible to interact
/// with data segments at runtime. Therefore Wasm instances now have
/// a need to have an instantiated representation of data segments.
#[derive(Debug)]
pub struct DataSegmentEntity {
    /// The underlying bytes of the instance data segment.
    ///
    /// # Note
    ///
    /// These bytes are just readable after instantiation.
    /// Using Wasm `data.drop` simply replaces the instance
    /// with an empty one.
    bytes: Option<Arc<[u8]>>,
}

impl From<&'_ module::DataSegment> for DataSegmentEntity {
    fn from(segment: &'_ module::DataSegment) -> Self {
        match segment.kind() {
            module::DataSegmentKind::Passive => Self {
                bytes: Some(segment.clone_bytes()),
            },
            module::DataSegmentKind::Active(_) => Self::empty(),
        }
    }
}

impl DataSegmentEntity {
    /// Create an empty [`DataSegmentEntity`] representing dropped data segments.
    fn empty() -> Self {
        Self { bytes: None }
    }

    /// Returns the bytes of the [`DataSegmentEntity`].
    pub fn bytes(&self) -> &[u8] {
        self.bytes
            .as_ref()
            .map(|bytes| &bytes[..])
            .unwrap_or_else(|| &[])
    }

    /// Drops the bytes of the [`DataSegmentEntity`].
    pub fn drop_bytes(&mut self) {
        self.bytes = None;
    }
}
