use crate::{
    AsContext,
    AsContextMut,
    Func,
    Global,
    core::{CoreElementSegment, UntypedRef},
    module,
    store::Stored,
};
use alloc::boxed::Box;

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
        elem: &module::ElementSegment,
        get_func: impl Fn(u32) -> Func,
        get_global: impl Fn(u32) -> Global,
    ) -> Self {
        let get_func = |index| get_func(index).into();
        let get_global = |index| get_global(index).get(&ctx);
        let items: Box<[UntypedRef]> = match elem.kind() {
            module::ElementSegmentKind::Passive | module::ElementSegmentKind::Active(_) => {
                elem
                    .items()
                    .iter()
                    .map(|const_expr| {
                        let Some(init) = const_expr.eval_with_context(get_global, get_func) else {
                            panic!("unexpected failed initialization of constant expression: {const_expr:?}")
                        };
                        UntypedRef::from(init)
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
        ctx.as_context().store.inner.resolve_element(self).size()
    }
}
