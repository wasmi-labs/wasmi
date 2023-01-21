use super::{FuncIdx, InitExpr, TableIdx};
use crate::errors::ModuleError;
use alloc::{boxed::Box, vec::Vec};

/// A table element segment within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct ElementSegment {
    _table_index: TableIdx,
    offset: InitExpr,
    items: Box<[FuncIdx]>,
}

impl TryFrom<wasmparser::Element<'_>> for ElementSegment {
    type Error = ModuleError;

    fn try_from(element: wasmparser::Element<'_>) -> Result<Self, Self::Error> {
        assert_eq!(
            element.ty,
            wasmparser::ValType::FuncRef,
            "wasmi does not support the `reference-types` Wasm proposal"
        );
        let (table_index, offset) = match element.kind {
            wasmparser::ElementKind::Active {
                table_index,
                offset_expr,
            } => {
                let table_index = TableIdx(table_index);
                let offset = InitExpr::try_from(offset_expr)?;
                (table_index, offset)
            }
            wasmparser::ElementKind::Passive => {
                panic!("wasmi does not support the `bulk-memory` Wasm proposal but found passive element segment")
            }
            wasmparser::ElementKind::Declared => {
                panic!("wasmi does not support the `reference-types` Wasm proposal but found declared element segment")
            }
        };
        let items = element
            .items
            .get_items_reader()?
            .into_iter()
            .map(|item| match item? {
                wasmparser::ElementItem::Func(func_idx) => Ok(FuncIdx(func_idx)),
                wasmparser::ElementItem::Expr(expr) => {
                    panic!("wasmi does not support the `bulk-memory` Wasm proposal but found an expression item: {expr:?}")
                }
            })
            .collect::<Result<Vec<_>, ModuleError>>()?
            .into_boxed_slice();
        Ok(ElementSegment {
            _table_index: table_index,
            offset,
            items,
        })
    }
}

impl ElementSegment {
    /// Returns the offset expression of the [`ElementSegment`].
    pub fn offset(&self) -> &InitExpr {
        &self.offset
    }

    /// Returns the element items of the [`ElementSegment`].
    pub fn items(&self) -> &[FuncIdx] {
        &self.items[..]
    }
}
