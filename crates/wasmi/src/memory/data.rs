use crate::{
    AsContextMut,
    module::{self, PassiveDataSegmentBytes},
};
use core::convert::AsRef;

define_handle! {
    /// A Wasm data segment reference.
    struct DataSegment(u32) => DataSegmentEntity;
}

impl DataSegment {
    /// Allocates a new active [`DataSegment`] on the store.
    ///
    /// # Errors
    ///
    /// If more than [`u32::MAX`] much linear memory is allocated.
    pub fn new_active(mut ctx: impl AsContextMut) -> Self {
        ctx.as_context_mut()
            .store
            .inner
            .alloc_data_segment(DataSegmentEntity::active())
    }

    /// Allocates a new passive [`DataSegment`] on the store.
    ///
    /// # Errors
    ///
    /// If more than [`u32::MAX`] much linear memory is allocated.
    pub fn new_passive(mut ctx: impl AsContextMut, bytes: PassiveDataSegmentBytes) -> Self {
        ctx.as_context_mut()
            .store
            .inner
            .alloc_data_segment(DataSegmentEntity::passive(bytes))
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
    bytes: Option<PassiveDataSegmentBytes>,
}

impl DataSegmentEntity {
    /// Creates a new active [`DataSegmentEntity`].
    pub fn active() -> Self {
        Self { bytes: None }
    }

    /// Creates a new passive [`DataSegmentEntity`] with its `bytes`.
    pub fn passive(bytes: PassiveDataSegmentBytes) -> Self {
        Self { bytes: Some(bytes) }
    }
}

impl From<&'_ module::DataSegment> for DataSegmentEntity {
    fn from(segment: &'_ module::DataSegment) -> Self {
        Self {
            bytes: segment.passive_data_segment_bytes(),
        }
    }
}

impl DataSegmentEntity {
    /// Returns the bytes of the [`DataSegmentEntity`].
    pub fn bytes(&self) -> &[u8] {
        self.bytes
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or_else(|| &[])
    }

    /// Drops the bytes of the [`DataSegmentEntity`].
    pub fn drop_bytes(&mut self) {
        self.bytes = None;
    }
}
