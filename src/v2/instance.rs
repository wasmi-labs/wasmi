use super::{Index, Stored};

/// A raw index to a module instance entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceIdx(usize);

impl Index for InstanceIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

/// A module instance entitiy.
#[derive(Debug)]
pub struct InstanceEntity {}

/// A Wasm module instance reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Instance(Stored<InstanceIdx>);
