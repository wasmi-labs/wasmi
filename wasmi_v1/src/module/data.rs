use super::{InitExpr, MemoryIdx, ModuleError};
use alloc::boxed::Box;

/// A linear memory data segment within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct DataSegment {
    _memory_index: MemoryIdx,
    offset: InitExpr,
    data: Box<[u8]>,
}

impl TryFrom<wasmparser::Data<'_>> for DataSegment {
    type Error = ModuleError;

    fn try_from(data: wasmparser::Data<'_>) -> Result<Self, Self::Error> {
        let (memory_index, offset) = match data.kind {
            wasmparser::DataKind::Active {
                memory_index,
                init_expr,
            } => {
                let memory_index = MemoryIdx(memory_index);
                let offset = InitExpr::try_from(init_expr)?;
                (memory_index, offset)
            }
            wasmparser::DataKind::Passive => {
                return Err(ModuleError::unsupported(
                    "encountered unsupported passive data segment",
                ))
            }
        };
        let data = data.data.into();
        Ok(DataSegment {
            _memory_index: memory_index,
            offset,
            data,
        })
    }
}

impl DataSegment {
    /// Returns the offset expression of the [`DataSegment`].
    pub fn offset(&self) -> &InitExpr {
        &self.offset
    }

    /// Returns the element items of the [`DataSegment`].
    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}
