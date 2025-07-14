use super::{ConstExpr, TableIdx};
use crate::ValType;
use alloc::boxed::Box;

/// A table element segment within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct ElementSegment {
    pub(crate) kind: ElementSegmentKind,
    /// The type of elements of the [`ElementSegment`].
    pub(crate) ty: ValType,
    pub(crate) items: Box<[ConstExpr]>,
}

/// The kind of a Wasm [`ElementSegment`].
#[derive(Debug)]
pub(crate) enum ElementSegmentKind {
    /// A passive [`ElementSegment`] from the `bulk-memory` Wasm proposal.
    Passive,
    /// An active [`ElementSegment`].
    Active(ActiveElementSegment),
    /// A declared [`ElementSegment`] from the `reference-types` Wasm proposal.
    Declared,
}

/// An active Wasm element segment.
#[derive(Debug)]
pub struct ActiveElementSegment {
    /// The index of the Wasm table that is to be initialized.
    pub(crate) table_index: TableIdx,
    /// The offset where the Wasm table is to be initialized.
    pub(crate) offset: ConstExpr,
}

impl ActiveElementSegment {
    /// Returns the Wasm module table index that is to be initialized.
    pub fn table_index(&self) -> TableIdx {
        self.table_index
    }

    /// Returns the offset expression of the [`ActiveElementSegment`].
    pub fn offset(&self) -> &ConstExpr {
        &self.offset
    }
}

impl From<wasmparser::ElementKind<'_>> for ElementSegmentKind {
    fn from(element_kind: wasmparser::ElementKind<'_>) -> Self {
        match element_kind {
            wasmparser::ElementKind::Active {
                table_index,
                offset_expr,
            } => {
                let table_index = TableIdx::from(table_index.unwrap_or(0));
                let offset = ConstExpr::new(offset_expr);
                Self::Active(ActiveElementSegment {
                    table_index,
                    offset,
                })
            }
            wasmparser::ElementKind::Passive => Self::Passive,
            wasmparser::ElementKind::Declared => Self::Declared,
        }
    }
}

impl From<wasmparser::Element<'_>> for ElementSegment {
    fn from(element: wasmparser::Element<'_>) -> Self {
        let kind = ElementSegmentKind::from(element.kind);
        let (items, ty) = match element.items {
            wasmparser::ElementItems::Functions(items) => {
                let items = items
                    .into_iter()
                    .map(|item| {
                        item.unwrap_or_else(|error| panic!("failed to parse element item: {error}"))
                    })
                    .map(ConstExpr::new_funcref)
                    .collect::<Box<[_]>>();
                (items, ValType::FuncRef)
            }
            wasmparser::ElementItems::Expressions(ref_ty, items) => {
                let ty = match ref_ty {
                    ty if ty.is_func_ref() => ValType::FuncRef,
                    ty if ty.is_extern_ref() => ValType::ExternRef,
                    _ => panic!("unsupported Wasm reference type"),
                };
                let items = items
                    .into_iter()
                    .map(|item| {
                        item.unwrap_or_else(|error| panic!("failed to parse element item: {error}"))
                    })
                    .map(ConstExpr::new)
                    .collect::<Box<[_]>>();
                (items, ty)
            }
        };
        Self { kind, ty, items }
    }
}

impl ElementSegment {
    /// Returns the offset expression of the [`ElementSegment`].
    pub(crate) fn kind(&self) -> &ElementSegmentKind {
        &self.kind
    }

    /// Returns the [`ValType`] of the [`ElementSegment`].
    pub fn ty(&self) -> ValType {
        self.ty
    }

    /// Returns the element items of the [`ElementSegment`].
    pub fn items(&self) -> &[ConstExpr] {
        &self.items[..]
    }
}
