use crate::core::ValType;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct LocalsRegistry {
    groups: Vec<LocalGroup>,
    len_locals: usize,
}

#[derive(Debug, Copy, Clone)]
struct LocalGroup {
    start_idx: usize,
    len: usize,
    ty: ValType,
}
