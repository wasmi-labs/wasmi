use super::{ConstExpr, MemoryIdx};
use crate::Error;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::slice;

/// A Wasm [`Module`] data segment.
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct DataSegment {
    pub(crate) inner: DataSegmentInner,
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
    pub(crate) memory_index: MemoryIdx,
    /// The offset at which the data segment is initialized.
    pub(crate) offset: ConstExpr,
    /// Number of bytes of the active data segment.
    pub(crate) len: u32,
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

    /// Returns the number of bytes of the [`ActiveDataSegment`] as `usize`.
    pub fn len(&self) -> usize {
        self.len as usize
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

impl PassiveDataSegmentBytes {
    pub(crate) fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            bytes: Arc::from(vec),
        }
    }
}

#[test]
fn size_of_data_segment() {
    assert!(core::mem::size_of::<DataSegment>() <= 32);
    assert!(core::mem::size_of::<DataSegmentInner>() <= 32);
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
    pub(crate) segments: Box<[DataSegment]>,
    /// All bytes from all active data segments.
    ///
    /// # Note
    ///
    /// We deliberately do not use `Box<[u8]>` here because it is not possible
    /// to properly pre-reserve space for the bytes and thus finishing construction
    /// of the [`DataSegments`] would highly likely reallocate and mass-copy
    /// which we prevent by simply using a `Vec<u8>` instead.
    pub(crate) bytes: Vec<u8>,
}

impl DataSegments {
    /// Creates a new [`DataSegmentsBuilder`].
    pub fn build() -> DataSegmentsBuilder {
        DataSegmentsBuilder {
            segments: Vec::new(),
            bytes: Vec::new(),
        }
    }
}

/// Builds up a [`DataSegments`] instance.
#[derive(Debug)]
pub struct DataSegmentsBuilder {
    /// All active or passive data segments built-up so far.
    segments: Vec<DataSegment>,
    /// The bytes of all active data segments.
    bytes: Vec<u8>,
}

impl DataSegmentsBuilder {
    /// Creates a DataSegmentsBuilder from an existing DataSegments.
    pub fn from_data_segments(data_segments: DataSegments) -> Self {
        DataSegmentsBuilder {
            segments: data_segments.segments.into(),
            bytes: data_segments.bytes,
        }
    }
    /// Reserves space for at least `additional` new [`DataSegments`].
    pub fn reserve(&mut self, count: usize) {
        assert!(
            self.segments.capacity() == 0,
            "must not reserve multiple times"
        );
        self.segments.reserve(count);
    }

    /// Pushes another [`DataSegment`] to the [`DataSegmentsBuilder`].
    ///
    /// # Panics
    ///
    /// If an active data segment has too many bytes.
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
                let len = u32::try_from(segment.data.len()).unwrap_or_else(|_x| {
                    panic!("data segment has too many bytes: {}", segment.data.len())
                });
                self.bytes.extend_from_slice(segment.data);
                self.segments.push(DataSegment {
                    inner: DataSegmentInner::Active(ActiveDataSegment {
                        memory_index,
                        offset,
                        len,
                    }),
                });
            }
        }
        Ok(())
    }

    pub fn finish(self) -> DataSegments {
        DataSegments {
            segments: self.segments.into(),
            bytes: self.bytes,
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
        }
    }
}

/// Iterator over the [`DataSegment`]s and their associated bytes.
#[derive(Debug)]
pub struct InitDataSegmentIter<'a> {
    segments: slice::Iter<'a, DataSegment>,
    bytes: &'a [u8],
}

impl<'a> Iterator for InitDataSegmentIter<'a> {
    type Item = InitDataSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let segment = self.segments.next()?;
        match &segment.inner {
            DataSegmentInner::Active(segment) => {
                let (bytes, rest) = self.bytes.split_at(segment.len());
                self.bytes = rest;
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
///
/// [`Module`]: crate::Module
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
