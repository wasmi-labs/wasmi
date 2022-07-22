use super::GlobalIdx;
use crate::{
    engine::DedupFuncType,
    Engine,
    FuncType,
    GlobalType,
    MemoryType,
    Module,
    ModuleError,
    TableType,
};
use alloc::boxed::Box;
use core::slice::Iter as SliceIter;

/// The index of a function declaration within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug, Copy, Clone)]
pub struct FuncIdx(pub(super) u32);

impl FuncIdx {
    /// Returns the [`FuncIdx`] as `u32`.
    pub fn into_u32(self) -> u32 {
        self.0
    }

    /// Returns the [`FuncIdx`] as `usize`.
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }
}

/// The index of a table declaration within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug, Copy, Clone)]
pub struct TableIdx(pub(super) u32);

impl TableIdx {
    /// Returns the [`TableIdx`] as `u32`.
    pub fn into_u32(self) -> u32 {
        self.0
    }
}

/// The index of a linear memory declaration within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug, Copy, Clone)]
pub struct MemoryIdx(pub(super) u32);

impl MemoryIdx {
    /// Returns the [`MemoryIdx`] as `u32`.
    pub fn into_u32(self) -> u32 {
        self.0
    }
}

/// An export definition within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct Export {
    /// The name under which the export definition is exported.
    field: Box<str>,
    /// The external item of the export definition.
    external: External,
}

impl TryFrom<wasmparser::Export<'_>> for Export {
    type Error = ModuleError;

    fn try_from(export: wasmparser::Export<'_>) -> Result<Self, Self::Error> {
        let field = export.field.into();
        let external = (export.kind, export.index).try_into()?;
        Ok(Export { field, external })
    }
}

impl Export {
    /// Returns the field name of the [`Export`].
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Returns the [`External`] item of the [`Export`].
    pub fn external(&self) -> External {
        self.external
    }
}

/// An external item of an [`Export`] definition within a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug, Copy, Clone)]
pub enum External {
    /// An exported function and its index witihn the [`Module`].
    ///
    /// [`Module`]: [`super::Module`]
    Func(FuncIdx),
    /// An exported table and its index witihn the [`Module`].
    ///
    /// [`Module`]: [`super::Module`]
    Table(TableIdx),
    /// An exported linear memory and its index witihn the [`Module`].
    ///
    /// [`Module`]: [`super::Module`]
    Memory(MemoryIdx),
    /// An exported global variable and its index witihn the [`Module`].
    ///
    /// [`Module`]: [`super::Module`]
    Global(GlobalIdx),
}

impl TryFrom<(wasmparser::ExternalKind, u32)> for External {
    type Error = ModuleError;

    fn try_from((kind, index): (wasmparser::ExternalKind, u32)) -> Result<Self, Self::Error> {
        match kind {
            wasmparser::ExternalKind::Function => Ok(External::Func(FuncIdx(index))),
            wasmparser::ExternalKind::Table => Ok(External::Table(TableIdx(index))),
            wasmparser::ExternalKind::Memory => Ok(External::Memory(MemoryIdx(index))),
            wasmparser::ExternalKind::Global => Ok(External::Global(GlobalIdx(index))),
            wasmparser::ExternalKind::Tag
            | wasmparser::ExternalKind::Type
            | wasmparser::ExternalKind::Module
            | wasmparser::ExternalKind::Instance => Err(ModuleError::unsupported(kind)),
        }
    }
}

/// An iterator over the exports of a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct ModuleExportsIter<'module> {
    exports: SliceIter<'module, Export>,
    engine: &'module Engine,
    funcs: &'module [DedupFuncType],
    tables: &'module [TableType],
    memories: &'module [MemoryType],
    globals: &'module [GlobalType],
}

/// An item exported from a [`Module`].
#[derive(Debug)]
pub struct ExportItem<'module> {
    name: &'module str,
    kind: ExportItemKind,
}

impl<'module> ExportItem<'module> {
    /// Returns the name of the exported item.
    pub fn name(&self) -> &'module str {
        self.name
    }

    /// Returns the kind of the exported item.
    pub fn kind(&self) -> &ExportItemKind {
        &self.kind
    }
}

/// The kind of an item exported from a [`Module`].
#[derive(Debug, Clone)]
pub enum ExportItemKind {
    /// An exported function of a [`Module`].
    Func(FuncType),
    /// An exported table of a [`Module`].
    Table(TableType),
    /// An exported linear memory of a [`Module`].
    Memory(MemoryType),
    /// An exported global variable of a [`Module`].
    Global(GlobalType),
}

impl<'module> ModuleExportsIter<'module> {
    /// Creates a new [`ModuleExportsIter`] from the given [`Module`].
    pub(super) fn new(module: &'module Module) -> Self {
        Self {
            exports: module.exports.iter(),
            engine: &module.engine,
            funcs: &module.funcs,
            tables: &module.tables,
            memories: &module.memories,
            globals: &module.globals,
        }
    }
}

impl<'module> Iterator for ModuleExportsIter<'module> {
    type Item = ExportItem<'module>;

    fn next(&mut self) -> Option<Self::Item> {
        self.exports.next().map(|export| {
            let name = export.field();
            let kind = match export.external() {
                External::Func(index) => {
                    let dedup = self.funcs[index.into_usize()];
                    let func_type = self.engine.resolve_func_type(dedup, Clone::clone);
                    ExportItemKind::Func(func_type)
                }
                External::Table(index) => {
                    let table_type = self.tables[index.into_u32() as usize];
                    ExportItemKind::Table(table_type)
                }
                External::Memory(index) => {
                    let memory_type = self.memories[index.into_u32() as usize];
                    ExportItemKind::Memory(memory_type)
                }
                External::Global(index) => {
                    let global_type = self.globals[index.into_usize()];
                    ExportItemKind::Global(global_type)
                }
            };
            ExportItem { name, kind }
        })
    }
}
