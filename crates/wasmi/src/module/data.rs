use crate::Error;
use super::{ConstExpr, MemoryIdx};
use core::slice;
use std::{boxed::Box, sync::Arc, vec::Vec};

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
    /// End-index of the data segments bytes buffer.
    end: usize,
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
    assert_eq!(core::mem::size_of::<DataSegment>(), 40);
    assert_eq!(core::mem::size_of::<DataSegmentInner>(), 40);
}

impl DataSegment {
    /// Returns the bytes of the [`DataSegment`] if passive, otherwise returns `None`.
    pub fn passive_data_segment_bytes(&self) -> Option<PassiveDataSegmentBytes> {
        match &self.inner {
            DataSegmentInner::Active { .. } => None,
            DataSegmentInner::Passive { bytes } => Some(bytes.clone()),
        }
    }
}

/// Stores all data segments and their associated data.
#[derive(Debug)]
pub struct DataSegments {
    /// All data segments.
    segments: Box<[DataSegment]>,
    /// All bytes from all active data segments.
    bytes: Box<[u8]>,
}

impl DataSegments {
    pub fn build() -> DataSegmentsBuilder {
        DataSegmentsBuilder {
            segments: Vec::new(),
            bytes: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct DataSegmentsBuilder {
    segments: Vec<DataSegment>,
    bytes: Vec<u8>,
}

impl DataSegmentsBuilder {
    pub fn reserve(&mut self, count: usize) {
        assert!(
            self.segments.capacity() == 0,
            "must not reserve multiple times"
        );
        self.segments.reserve(count);
    }

    pub fn push_data_segment(&mut self, segment: wasmparser::Data) -> Result<(), Error> {
        match segment.kind {
            wasmparser::DataKind::Passive => {
                self.segments.push(DataSegment {
                    inner: DataSegmentInner::Passive {
                        bytes: PassiveDataSegmentBytes {
                            bytes: segment.data.into(),
                        },
                    },
                });
            }
            wasmparser::DataKind::Active {
                memory_index,
                offset_expr,
            } => {
                let memory_index = MemoryIdx::from(memory_index);
                let offset = ConstExpr::new(offset_expr);
                self.bytes.extend_from_slice(segment.data);
                let end = self.bytes.len();
                self.segments.push(DataSegment {
                    inner: DataSegmentInner::Active(ActiveDataSegment {
                        memory_index,
                        offset,
                        end,
                    }),
                });
            }
        }
        Ok(())
    }

    pub fn finish(self) -> DataSegments {
        DataSegments {
            segments: self.segments.into(),
            bytes: self.bytes.into(),
        }
    }
}

impl<'a> IntoIterator for &'a DataSegments {
    type Item = InitDataSegment<'a>;
    type IntoIter = InitDataSegmentIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        InitDataSegmentIter {
            segments: self.segments.iter(),
            bytes: &self.bytes[..],
            start: 0,
        }
    }
}

#[derive(Debug)]
pub struct InitDataSegmentIter<'a> {
    segments: slice::Iter<'a, DataSegment>,
    bytes: &'a [u8],
    start: usize,
}

impl<'a> Iterator for InitDataSegmentIter<'a> {
    type Item = InitDataSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let segment = self.segments.next()?;
        match &segment.inner {
            DataSegmentInner::Active(segment) => {
                let end = segment.end;
                let bytes = &self.bytes[self.start..end];
                self.start = end;
                Some(InitDataSegment::Active {
                    memory_index: segment.memory_index(),
                    offset: segment.offset(),
                    bytes,
                })
            }
            DataSegmentInner::Passive { bytes } => Some(InitDataSegment::Passive {
                bytes: bytes.clone(),
            }),
        }
    }
}

/// Iterated-over [`DataSegment`] when instantiating a [`Module`].
pub enum InitDataSegment<'a> {
    Active {
        /// The linear memory that is to be initialized with this active segment.
        memory_index: MemoryIdx,
        /// The offset at which the data segment is initialized.
        offset: &'a ConstExpr,
        /// The bytes of the active data segment.
        bytes: &'a [u8],
    },
    Passive {
        bytes: PassiveDataSegmentBytes,
    },
}
