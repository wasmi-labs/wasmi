use crate::arena::{GuardedEntity, Index};
use alloc::sync::atomic::{AtomicUsize, Ordering};

/// A unique identifier for an [`Engine`] instance.
///
/// # Note
///
/// Used to protect against invalid entity indices.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EngineIdent(usize);

impl Index for EngineIdent {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

impl EngineIdent {
    /// Returns a new unique [`EngineIdent`].
    pub(super) fn new() -> Self {
        /// A static store index counter.
        static CURRENT_STORE_IDX: AtomicUsize = AtomicUsize::new(0);
        let next_idx = CURRENT_STORE_IDX.fetch_add(1, Ordering::AcqRel);
        Self(next_idx)
    }
}

/// An entity owned by the [`Engine`].
pub type Guarded<Idx> = GuardedEntity<EngineIdent, Idx>;
