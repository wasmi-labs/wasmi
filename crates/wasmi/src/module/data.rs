use super::{ConstExpr, MemoryIdx};
use std::{boxed::Box, sync::Arc};

/// A Wasm [`Module`] data segment.
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct DataSegment {
    inner: DataSegmentInner,
}

/// The inner structure of a [`DataSegment`].
#[derive(Debug)]
pub enum DataSegmentInner {
    /// An active data segment that is initialized upon Wasm module instantiation.
    Active(ActiveDataSegment),
    /// A passive data segment that can be used by some Wasm bulk instructions.
    Passive {
        /// The bytes of the passive data segment.
        bytes: PassiveDataSegmentBytes,
    },
}

/// An active data segment that is initialized upon Wasm module instantiation.
#[derive(Debug)]
pub struct ActiveDataSegment {
    /// The linear memory that is to be initialized with this active segment.
    memory_index: MemoryIdx,
    /// The offset at which the data segment is initialized.
    offset: ConstExpr,
    /// The bytes of the active data segment.
    bytes: Box<[u8]>,
}

/// The bytes of the passive data segment.
#[derive(Debug, Clone)]
pub struct PassiveDataSegmentBytes {
    bytes: Arc<[u8]>,
}

impl AsRef<[u8]> for PassiveDataSegmentBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..]
    }
}

#[test]
fn size_of_data_segment() {
    assert_eq!(core::mem::size_of::<DataSegment>(), 48);
    assert_eq!(core::mem::size_of::<DataSegmentInner>(), 48);
}

impl ActiveDataSegment {
    /// Returns the Wasm module memory index that is to be initialized.
    pub fn memory_index(&self) -> MemoryIdx {
        self.memory_index
    }

    /// Returns the offset expression of the [`ActiveDataSegment`].
    pub fn offset(&self) -> &ConstExpr {
        &self.offset
    }

    /// Returns the bytes of the active data segment.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..]
    }
}

impl From<wasmparser::Data<'_>> for DataSegment {
    fn from(data: wasmparser::Data<'_>) -> Self {
        match data.kind {
            wasmparser::DataKind::Passive => Self {
                inner: DataSegmentInner::Passive {
                    bytes: PassiveDataSegmentBytes {
                        bytes: data.data.into(),
                    },
                },
            },
            wasmparser::DataKind::Active {
                memory_index,
                offset_expr,
            } => {
                let memory_index = MemoryIdx::from(memory_index);
                let offset = ConstExpr::new(offset_expr);
                Self {
                    inner: DataSegmentInner::Active(ActiveDataSegment {
                        memory_index,
                        offset,
                        bytes: data.data.into(),
                    }),
                }
            }
        }
    }
}

impl DataSegment {
    /// Returns the [`ActiveDataSegment`] if this [`DataSegment`] is active.
    pub fn get_active(&self) -> Option<&ActiveDataSegment> {
        match &self.inner {
            DataSegmentInner::Active(segment) => Some(segment),
            DataSegmentInner::Passive { .. } => None,
        }
    }

    /// Returns the bytes of the [`DataSegment`].
    pub fn bytes(&self) -> &[u8] {
        match &self.inner {
            DataSegmentInner::Active(segment) => segment.bytes(),
            DataSegmentInner::Passive { bytes } => bytes.as_ref(),
        }
    }

    /// Returns the bytes of the [`DataSegment`] if passive, otherwise returns `None`.
    pub fn passive_data_segment_bytes(&self) -> Option<PassiveDataSegmentBytes> {
        match &self.inner {
            DataSegmentInner::Active { .. } => None,
            DataSegmentInner::Passive { bytes } => Some(bytes.clone()),
        }
    }
}
