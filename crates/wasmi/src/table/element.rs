use crate::{
    AsContext,
    AsContextMut,
    RefType,
    core::{CoreElementSegment, RawRef},
    store::Stored,
};

define_handle! {
    /// A Wasm data segment reference.
    struct ElementSegment(u32, Stored) => CoreElementSegment;
}

impl ElementSegment {
    /// Allocates a new [`ElementSegment`] on the store.
    ///
    /// # Errors
    ///
    /// If more than [`u32::MAX`] much linear memory is allocated.
    pub fn new(
        mut ctx: impl AsContextMut,
        ty: RefType,
        items: impl IntoIterator<Item = RawRef>,
    ) -> Self {
        let entity = CoreElementSegment::new(ty, items);
        ctx.as_context_mut()
            .store
            .inner
            .alloc_element_segment(entity)
    }

    /// Returns the number of items in the [`ElementSegment`].
    pub fn size(&self, ctx: impl AsContext) -> u32 {
        ctx.as_context().store.inner.resolve_element(self).size()
    }
}
