use crate::{
    collections::arena::ArenaIndex,
    core::{CoreElementSegment, UntypedVal},
    module,
    store::Stored,
    AsContext,
    AsContextMut,
    Func,
    Global,
};
use alloc::boxed::Box;

/// A raw index to a element segment entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElementSegmentIdx(u32);

impl ArenaIndex for ElementSegmentIdx {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("index {value} is out of bounds as element segment index: {error}")
        });
        Self(value)
    }
}

/// A Wasm data segment reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ElementSegment(Stored<ElementSegmentIdx>);

impl ElementSegment {
    /// Creates a new linear memory reference.
    pub fn from_inner(stored: Stored<ElementSegmentIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub fn as_inner(&self) -> &Stored<ElementSegmentIdx> {
        &self.0
    }

    /// Allocates a new [`ElementSegment`] on the store.
    ///
    /// # Errors
    ///
    /// If more than [`u32::MAX`] much linear memory is allocated.
    pub fn new(
        mut ctx: impl AsContextMut,
        elem: &module::ElementSegment,
        get_func: impl Fn(u32) -> Func,
        get_global: impl Fn(u32) -> Global,
    ) -> Self {
        let get_func = |index| get_func(index).into();
        let get_global = |index| get_global(index).get(&ctx);
        let items: Box<[UntypedVal]> = match elem.kind() {
            module::ElementSegmentKind::Passive | module::ElementSegmentKind::Active(_) => {
                elem
                    .items()
                    .iter()
                    .map(|const_expr| {
                        const_expr.eval_with_context(get_global, get_func).unwrap_or_else(|| {
                            panic!("unexpected failed initialization of constant expression: {const_expr:?}")
                        })
                }).collect()
            }
            module::ElementSegmentKind::Declared => Box::from([]),
        };
        let entity = CoreElementSegment::new(elem.ty(), items);
        ctx.as_context_mut()
            .store
            .inner
            .alloc_element_segment(entity)
    }

    /// Returns the number of items in the [`ElementSegment`].
    pub fn size(&self, ctx: impl AsContext) -> u32 {
        ctx.as_context()
            .store
            .inner
            .resolve_element_segment(self)
            .size()
    }
}
