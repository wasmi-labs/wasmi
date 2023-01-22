use super::{InitExpr, MemoryIdx};
use crate::errors::ModuleError;
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
    offset: InitExpr,
}

impl ActiveDataSegment {
    /// Returns the Wasm module memory index that is to be initialized.
    pub fn memory_index(&self) -> MemoryIdx {
        self.memory_index
    }

    /// Returns the offset expression of the [`ActiveDataSegment`].
    pub fn offset(&self) -> &InitExpr {
        &self.offset
    }
}

impl TryFrom<wasmparser::DataKind<'_>> for DataSegmentKind {
    type Error = ModuleError;

    fn try_from(data_kind: wasmparser::DataKind<'_>) -> Result<Self, Self::Error> {
        match data_kind {
            wasmparser::DataKind::Active {
                memory_index,
                offset_expr,
            } => {
                let memory_index = MemoryIdx(memory_index);
                let offset = InitExpr::new(offset_expr)?;
                Ok(Self::Active(ActiveDataSegment {
                    memory_index,
                    offset,
                }))
            }
            wasmparser::DataKind::Passive => Ok(Self::Passive),
        }
    }
}

impl TryFrom<wasmparser::Data<'_>> for DataSegment {
    type Error = ModuleError;

    fn try_from(data: wasmparser::Data<'_>) -> Result<Self, Self::Error> {
        let kind = DataSegmentKind::try_from(data.kind)?;
        let bytes = data.data.into();
        Ok(DataSegment { kind, bytes })
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
