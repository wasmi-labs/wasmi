use crate::core::ValType;
use alloc::vec::Vec;

/// A local variable index.
#[derive(Debug, Copy, Clone)]
pub struct LocalIdx(usize);

#[derive(Debug, Default, Clone)]
pub struct LocalsRegistry {
    groups: Vec<LocalGroup>,
    len_locals: usize,
}

impl LocalsRegistry {
    pub fn ty(&self, local_index: LocalIdx) -> Option<ValType> {
        todo!()
    }
}

#[derive(Debug, Copy, Clone)]
struct LocalGroup {
    start_idx: usize,
    len: usize,
    ty: ValType,
}
