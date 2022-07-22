use crate::ModuleError;
use alloc::{boxed::Box, vec::Vec};

use super::{FuncIdx, InitExpr, TableIdx};

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
        if !matches!(element.ty, wasmparser::Type::FuncRef) {
            return Err(ModuleError::unsupported(element.ty));
        }
        let (table_index, offset) = match element.kind {
            wasmparser::ElementKind::Active {
                table_index,
                init_expr,
            } => {
                let table_index = TableIdx(table_index);
                let offset = InitExpr::try_from(init_expr)?;
                (table_index, offset)
            }
            wasmparser::ElementKind::Passive | wasmparser::ElementKind::Declared => {
                return Err(ModuleError::unsupported(
                    "encountered unsupported passive or declared element segment",
                ))
            }
        };
        let items = element
            .items
            .get_items_reader()?
            .into_iter()
            .map(|item| match item? {
                wasmparser::ElementItem::Func(func_idx) => Ok(FuncIdx(func_idx)),
                wasmparser::ElementItem::Expr(_) => {
                    unreachable!("encountered unexpected init expression for element item")
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
