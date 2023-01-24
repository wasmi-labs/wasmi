use super::{FuncIdx, InitExpr, TableIdx};
use crate::errors::ModuleError;
use alloc::sync::Arc;

/// A table element segment within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct ElementSegment {
    /// The kind of the [`ElementSegment`].
    kind: ElementSegmentKind,
    /// The items of the [`ElementSegment`].
    items: Arc<[Option<FuncIdx>]>,
}

/// The kind of a Wasm [`ElementSegment`].
#[derive(Debug)]
pub enum ElementSegmentKind {
    /// A passive [`ElementSegment`] from the `bulk-memory` Wasm proposal.
    Passive,
    /// An active [`ElementSegment`].
    Active(ActiveElementSegment),
}

/// An active Wasm element segment.
#[derive(Debug)]
pub struct ActiveElementSegment {
    /// The index of the Wasm table that is to be initialized.
    table_index: TableIdx,
    /// The offset where the Wasm table is to be initialized.
    offset: InitExpr,
}

impl ActiveElementSegment {
    /// Returns the Wasm module table index that is to be initialized.
    pub fn table_index(&self) -> TableIdx {
        self.table_index
    }

    /// Returns the offset expression of the [`ActiveElementSegment`].
    pub fn offset(&self) -> &InitExpr {
        &self.offset
    }
}

impl TryFrom<wasmparser::ElementKind<'_>> for ElementSegmentKind {
    type Error = ModuleError;

    fn try_from(element_kind: wasmparser::ElementKind<'_>) -> Result<Self, Self::Error> {
        match element_kind {
            wasmparser::ElementKind::Active {
                table_index,
                offset_expr,
            } => {
                let table_index = TableIdx(table_index);
                let offset = InitExpr::new(offset_expr);
                Ok(Self::Active(ActiveElementSegment {
                    table_index,
                    offset,
                }))
            }
            wasmparser::ElementKind::Passive => Ok(Self::Passive),
            wasmparser::ElementKind::Declared => {
                panic!("wasmi does not support the `reference-types` Wasm proposal but found declared element segment")
            }
        }
    }
}

impl TryFrom<wasmparser::Element<'_>> for ElementSegment {
    type Error = ModuleError;

    fn try_from(element: wasmparser::Element<'_>) -> Result<Self, Self::Error> {
        assert_eq!(
            element.ty,
            wasmparser::ValType::FuncRef,
            "wasmi does not support the `reference-types` Wasm proposal"
        );
        let kind = ElementSegmentKind::try_from(element.kind)?;
        let items = element
            .items
            .get_items_reader()?
            .into_iter()
            .map(|item| {
                let func_ref = match item? {
                    wasmparser::ElementItem::Func(func_idx) => Some(FuncIdx(func_idx)),
                    wasmparser::ElementItem::Expr(expr) => InitExpr::new(expr).into_elemexpr(),
                };
                <Result<_, ModuleError>>::Ok(func_ref)
            })
            .collect::<Result<Arc<[_]>, _>>()?;
        Ok(ElementSegment { kind, items })
    }
}

impl ElementSegment {
    /// Returns the offset expression of the [`ElementSegment`].
    pub fn kind(&self) -> &ElementSegmentKind {
        &self.kind
    }

    /// Returns the element items of the [`ElementSegment`].
    pub fn items(&self) -> &[Option<FuncIdx>] {
        &self.items[..]
    }

    /// Clone the underlying items of the [`ElementSegment`].
    pub fn clone_items(&self) -> Arc<[Option<FuncIdx>]> {
        self.items.clone()
    }
}
