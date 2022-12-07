use crate::ArenaIndex;

/// A guarded entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GuardedEntity<GuardIdx, EntityIdx> {
    guard_idx: GuardIdx,
    entity_idx: EntityIdx,
}

impl<GuardIdx, EntityIdx> GuardedEntity<GuardIdx, EntityIdx> {
    /// Creates a new [`GuardedEntity`].
    pub fn new(guard_idx: GuardIdx, entity_idx: EntityIdx) -> Self {
        Self {
            guard_idx,
            entity_idx,
        }
    }
}

impl<GuardIdx, EntityIdx> GuardedEntity<GuardIdx, EntityIdx>
where
    GuardIdx: ArenaIndex,
    EntityIdx: ArenaIndex,
{
    /// Returns the entity index of the [`GuardedEntity`].
    ///
    /// Return `None` if the `guard_index` does not match.
    pub fn entity_index(&self, guard_index: GuardIdx) -> Option<EntityIdx> {
        if self.guard_idx.into_usize() != guard_index.into_usize() {
            return None;
        }
        Some(self.entity_idx)
    }
}
