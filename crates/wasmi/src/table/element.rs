use crate::{
    collections::arena::ArenaIndex,
    core::{UntypedVal, ValType},
    module,
    store::Stored,
    AsContext,
    AsContextMut,
    Func,
    FuncRef,
    Global,
    Val,
};
use std::boxed::Box;

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
        let entity = ElementSegmentEntity::new(elem, get_func, get_global);
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

/// An instantiated [`ElementSegmentEntity`].
///
/// # Note
///
/// With the `bulk-memory` Wasm proposal it is possible to interact
/// with element segments at runtime. Therefore Wasm instances now have
/// a need to have an instantiated representation of data segments.
#[derive(Debug)]
pub struct ElementSegmentEntity {
    /// The [`ValType`] of elements of this [`ElementSegmentEntity`].
    ty: ValType,
    /// Pre-resolved untyped items of the Wasm element segment.
    items: Box<[UntypedVal]>,
}

impl ElementSegmentEntity {
    pub fn new(
        elem: &'_ module::ElementSegment,
        get_func: impl Fn(u32) -> FuncRef,
        get_global: impl Fn(u32) -> Val,
    ) -> Self {
        let ty = elem.ty();
        match elem.kind() {
            module::ElementSegmentKind::Passive | module::ElementSegmentKind::Active(_) => {
                let items = elem
                    .items()
                    .iter()
                    .map(|const_expr| {
                        const_expr.eval_with_context(&get_global, &get_func).unwrap_or_else(|| {
                            panic!("unexpected failed initialization of constant expression: {const_expr:?}")
                        })
                }).collect::<Box<[_]>>();
                Self { ty, items }
            }
            module::ElementSegmentKind::Declared => Self::empty(ty),
        }
    }

    /// Create an empty [`ElementSegmentEntity`] representing dropped element segments.
    fn empty(ty: ValType) -> Self {
        Self {
            ty,
            items: [].into(),
        }
    }

    /// Returns the [`ValType`] of elements in the [`ElementSegmentEntity`].
    pub fn ty(&self) -> ValType {
        self.ty
    }

    /// Returns the number of items in the [`ElementSegment`].
    pub fn size(&self) -> u32 {
        self.items().len() as u32
    }

    /// Returns the items of the [`ElementSegmentEntity`].
    pub fn items(&self) -> &[UntypedVal] {
        &self.items[..]
    }

    /// Drops the items of the [`ElementSegmentEntity`].
    pub fn drop_items(&mut self) {
        self.items = [].into();
    }
}
