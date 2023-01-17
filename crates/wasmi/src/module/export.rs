use super::GlobalIdx;
use crate::{
    engine::DedupFuncType,
    Engine,
    ExternType,
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
pub struct FuncIdx(pub(crate) u32);

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
pub struct TableIdx(pub(crate) u32);

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
pub struct MemoryIdx(pub(crate) u32);

impl MemoryIdx {
    /// Returns the [`MemoryIdx`] as `u32`.
    pub fn into_u32(self) -> u32 {
        self.0
    }
}

/// A descriptor of a [`Module`] export definition.
///
/// [`Module`]: [`crate::Module`]
#[derive(Debug)]
pub struct ModuleExport {
    /// The name under which the export definition is exported.
    field: Box<str>,
    /// The external item of the export definition.
    external: ExternIdx,
}

impl TryFrom<wasmparser::Export<'_>> for ModuleExport {
    type Error = ModuleError;

    fn try_from(export: wasmparser::Export<'_>) -> Result<Self, Self::Error> {
        let field = export.name.into();
        let external = (export.kind, export.index).try_into()?;
        Ok(ModuleExport { field, external })
    }
}

impl ModuleExport {
    /// Returns the field name of the [`ModuleExport`].
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Returns the [`ExternIdx`] item of the [`ModuleExport`].
    pub fn idx(&self) -> ExternIdx {
        self.external
    }
}

/// An external item of an [`ExportType`] definition within a [`Module`].
///
/// [`Module`]: [`crate::Module`]
#[derive(Debug, Copy, Clone)]
pub enum ExternIdx {
    /// An exported function and its index within the [`Module`].
    ///
    /// [`Module`]: [`super::Module`]
    Func(FuncIdx),
    /// An exported table and its index within the [`Module`].
    ///
    /// [`Module`]: [`super::Module`]
    Table(TableIdx),
    /// An exported linear memory and its index within the [`Module`].
    ///
    /// [`Module`]: [`super::Module`]
    Memory(MemoryIdx),
    /// An exported global variable and its index within the [`Module`].
    ///
    /// [`Module`]: [`super::Module`]
    Global(GlobalIdx),
}

impl TryFrom<(wasmparser::ExternalKind, u32)> for ExternIdx {
    type Error = ModuleError;

    fn try_from((kind, index): (wasmparser::ExternalKind, u32)) -> Result<Self, Self::Error> {
        match kind {
            wasmparser::ExternalKind::Func => Ok(ExternIdx::Func(FuncIdx(index))),
            wasmparser::ExternalKind::Table => Ok(ExternIdx::Table(TableIdx(index))),
            wasmparser::ExternalKind::Memory => Ok(ExternIdx::Memory(MemoryIdx(index))),
            wasmparser::ExternalKind::Global => Ok(ExternIdx::Global(GlobalIdx(index))),
            wasmparser::ExternalKind::Tag => Err(ModuleError::unsupported(kind)),
        }
    }
}

/// An iterator over the exports of a [`Module`].
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct ModuleExportsIter<'module> {
    exports: SliceIter<'module, ModuleExport>,
    engine: &'module Engine,
    funcs: &'module [DedupFuncType],
    tables: &'module [TableType],
    memories: &'module [MemoryType],
    globals: &'module [GlobalType],
}

/// A descriptor for an exported WebAssembly value of a [`Module`].
///
/// This type is primarily accessed from the [`Module::exports`] method and describes
/// what names are exported from a Wasm [`Module`] and the type of the item that is exported.
#[derive(Debug)]
pub struct ExportType<'module> {
    name: &'module str,
    ty: ExternType,
}

impl<'module> ExportType<'module> {
    /// Returns the name by which the export is known.
    pub fn name(&self) -> &'module str {
        self.name
    }

    /// Returns the type of the exported item.
    pub fn ty(&self) -> &ExternType {
        &self.ty
    }
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
    type Item = ExportType<'module>;

    fn next(&mut self) -> Option<Self::Item> {
        self.exports.next().map(|export| {
            let name = export.field();
            let ty = match export.idx() {
                ExternIdx::Func(index) => {
                    let dedup = self.funcs[index.into_usize()];
                    let func_type = self.engine.resolve_func_type(dedup, Clone::clone);
                    ExternType::Func(func_type)
                }
                ExternIdx::Table(index) => {
                    let table_type = self.tables[index.into_u32() as usize];
                    ExternType::Table(table_type)
                }
                ExternIdx::Memory(index) => {
                    let memory_type = self.memories[index.into_u32() as usize];
                    ExternType::Memory(memory_type)
                }
                ExternIdx::Global(index) => {
                    let global_type = self.globals[index.into_usize()];
                    ExternType::Global(global_type)
                }
            };
            ExportType { name, ty }
        })
    }
}
