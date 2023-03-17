use super::{ConstExpr, MemoryIdx};
use alloc::sync::Arc;

/// A Wasm [`Module`] data segment.
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct DataSegment {
    /// The kind of the data segment.
    kind: DataSegmentKind,
    /// The bytes of the data segment.
    bytes: Arc<[u8]>,
}

/// The kind of a Wasm module [`DataSegment`].
#[derive(Debug)]
pub enum DataSegmentKind {
    /// A passive data segment from the `bulk-memory` Wasm proposal.
    Passive,
    /// An active data segment that is initialized upon module instantiation.
    Active(ActiveDataSegment),
}

/// An active data segment.
#[derive(Debug)]
pub struct ActiveDataSegment {
    /// The linear memory that is to be initialized with this active segment.
    memory_index: MemoryIdx,
    /// The offset at which the data segment is initialized.
    offset: ConstExpr,
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
}

impl From<wasmparser::DataKind<'_>> for DataSegmentKind {
    fn from(data_kind: wasmparser::DataKind<'_>) -> Self {
        match data_kind {
            wasmparser::DataKind::Active {
                memory_index,
                offset_expr,
            } => {
                let memory_index = MemoryIdx::from(memory_index);
                let offset = ConstExpr::new(offset_expr);
                Self::Active(ActiveDataSegment {
                    memory_index,
                    offset,
                })
            }
            wasmparser::DataKind::Passive => Self::Passive,
        }
    }
}

impl From<wasmparser::Data<'_>> for DataSegment {
    fn from(data: wasmparser::Data<'_>) -> Self {
        let kind = DataSegmentKind::from(data.kind);
        let bytes = data.data.into();
        Self { kind, bytes }
    }
}

impl DataSegment {
    /// Returns the [`DataSegmentKind`] of the [`DataSegment`].
    pub fn kind(&self) -> &DataSegmentKind {
        &self.kind
    }

    /// Returns the bytes of the [`DataSegment`].
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Clone the underlying bytes of the [`DataSegment`].
    pub fn clone_bytes(&self) -> Arc<[u8]> {
        self.bytes.clone()
    }
}
