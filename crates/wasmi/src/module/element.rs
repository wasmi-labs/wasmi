use super::{InitExpr, TableIdx};
use crate::{errors::ModuleError, module::utils::WasmiValueType};
use alloc::sync::Arc;
use wasmi_core::ValueType;

/// A table element segment within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct ElementSegment {
    /// The kind of the [`ElementSegment`].
    kind: ElementSegmentKind,
    /// The type of elements of the [`ElementSegment`].
    ty: ValueType,
    /// The items of the [`ElementSegment`].
    items: ElementSegmentItems,
}

/// The items of an [`ElementSegment`].
#[derive(Debug, Clone)]
pub struct ElementSegmentItems {
    exprs: Arc<[InitExpr]>,
}

impl ElementSegmentItems {
    /// Creates new [`ElementSegmentItems`] from the given [`wasmparser::ElementItems`].
    ///
    /// # Panics
    ///
    /// If the given [`wasmparser::ElementItems`] is invalid.
    fn new(items: &wasmparser::ElementItems) -> Self {
        let exprs = match items {
            wasmparser::ElementItems::Functions(items) => items
                .clone()
                .into_iter()
                .map(|item| {
                    item.unwrap_or_else(|error| panic!("failed to parse element item: {error}"))
                })
                .map(InitExpr::new_funcref)
                .collect::<Arc<[_]>>(),
            wasmparser::ElementItems::Expressions(items) => items
                .clone()
                .into_iter()
                .map(|item| {
                    item.unwrap_or_else(|error| panic!("failed to parse element item: {error}"))
                })
                .map(InitExpr::new)
                .collect::<Arc<[_]>>(),
        };
        Self { exprs }
    }

    /// Returns a shared reference to the items of the [`ElementSegmentItems`].
    pub fn items(&self) -> &[InitExpr] {
        &self.exprs
    }
}

/// The kind of a Wasm [`ElementSegment`].
#[derive(Debug)]
pub enum ElementSegmentKind {
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
            wasmparser::ElementKind::Declared => Ok(Self::Declared),
        }
    }
}

impl TryFrom<wasmparser::Element<'_>> for ElementSegment {
    type Error = ModuleError;

    fn try_from(element: wasmparser::Element<'_>) -> Result<Self, Self::Error> {
        assert!(
            element.ty.is_reference_type(),
            "only reftypes are allowed as element types but found: {:?}",
            element.ty
        );
        let kind = ElementSegmentKind::try_from(element.kind)?;
        let ty = WasmiValueType::from(element.ty).into_inner();
        let items = ElementSegmentItems::new(&element.items);
        Ok(ElementSegment { kind, ty, items })
    }
}

impl ElementSegment {
    /// Returns the offset expression of the [`ElementSegment`].
    pub fn kind(&self) -> &ElementSegmentKind {
        &self.kind
    }

    /// Returns the [`ValueType`] of the [`ElementSegment`].
    pub fn ty(&self) -> ValueType {
        self.ty
    }

    /// Returns the element items of the [`ElementSegment`].
    pub fn items_cloned(&self) -> ElementSegmentItems {
        self.items.clone()
    }
}
