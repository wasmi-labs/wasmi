use super::GlobalIdx;
use crate::ModuleError;

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

    /// Returns the [`TableIdx`] as `usize`.
    pub fn into_usize(self) -> usize {
        self.0 as usize
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

    /// Returns the [`MemoryIdx`] as `usize`.
    pub fn into_usize(self) -> usize {
        self.0 as usize
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
