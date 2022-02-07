use core::fmt::{self, Display};

use crate::{GlobalType, MemoryType, ModuleError, TableType};
use wasmparser::ImportSectionEntryType;

/// A [`Module`] import item.
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct Import {
    /// The name of the imported item.
    name: ImportName,
    /// The kind of the imported item.
    kind: ImportKind,
}

/// The name or namespace of an imported item.
#[derive(Debug, Clone)]
pub struct ImportName {
    /// The name of the [`Module`] that defines the imported item.
    ///
    /// [`Module`]: [`super::Module`]
    module: Box<str>,
    /// The optional name of the imported item within the [`Module`] namespace.
    ///
    /// [`Module`]: [`super::Module`]
    field: Option<Box<str>>,
}

impl Display for ImportName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let module_name = &*self.module;
        if let Some(field_name) = self.field.as_deref() {
            write!(f, "{}::{}", module_name, field_name)
        } else {
            write!(f, "{}", module_name)
        }
    }
}

impl ImportName {
    /// Creates a new [`Import`] item.
    pub fn new(module: &str, field: Option<&str>) -> Self {
        Self {
            module: module.into(),
            field: field.map(Into::into),
        }
    }

    /// Returns the name of the [`Module`] that defines the imported item.
    ///
    /// [`Module`]: [`super::Module`]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// Returns the optional name of the imported item within the [`Module`] namespace.
    ///
    /// [`Module`]: [`super::Module`]
    pub fn field(&self) -> Option<&str> {
        self.field.as_deref()
    }
}

impl TryFrom<wasmparser::Import<'_>> for Import {
    type Error = ModuleError;

    fn try_from(import: wasmparser::Import) -> Result<Self, Self::Error> {
        let kind = match import.ty {
            ImportSectionEntryType::Function(func_type) => {
                Ok(ImportKind::Func(FuncTypeIdx(func_type)))
            }
            ImportSectionEntryType::Table(table_type) => {
                table_type.try_into().map(ImportKind::Table)
            }
            ImportSectionEntryType::Memory(memory_type) => {
                memory_type.try_into().map(ImportKind::Memory)
            }
            ImportSectionEntryType::Global(global_type) => {
                global_type.try_into().map(ImportKind::Global)
            }
            ImportSectionEntryType::Tag(_)
            | ImportSectionEntryType::Module(_)
            | ImportSectionEntryType::Instance(_) => Err(ModuleError::unsupported(import)),
        }?;
        Ok(Self::new(import.module, import.field, kind))
    }
}

impl Import {
    /// Creates a new [`Import`] item.
    pub fn new(module: &str, field: Option<&str>, kind: ImportKind) -> Self {
        Self {
            name: ImportName::new(module, field),
            kind,
        }
    }

    /// Returns the name of the imported item.
    pub fn name(&self) -> &ImportName {
        &self.name
    }

    /// Returns the kind of the imported item and its associated data.
    pub fn kind(&self) -> &ImportKind {
        &self.kind
    }

    /// Splits the [`Import`] into its raw parts.
    ///
    /// # Note
    ///
    /// This allows to reuse some allocations in certain cases.
    pub fn into_name_and_kind(self) -> (ImportName, ImportKind) {
        (self.name, self.kind)
    }
}

/// The kind of a [`Module`] import.
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub enum ImportKind {
    /// An imported function.
    Func(FuncTypeIdx),
    /// An imported table.
    Table(TableType),
    /// An imported linear memory.
    Memory(MemoryType),
    /// An imported global variable.
    Global(GlobalType),
}

/// A [`FuncType`] index.
///
/// # Note
///
/// This generally refers to a [`FuncType`] within the same [`Module`]
/// and is used by both function declarations and function imports.
///
/// [`Module`]: [`super::Module`]
/// [`FuncType`]: [`crate::FuncType`]
#[derive(Debug, Copy, Clone)]
pub struct FuncTypeIdx(pub(super) u32);

impl FuncTypeIdx {
    /// Returns the [`FuncTypeIdx`] as `u32`.
    ///
    /// # Note
    ///
    /// This is mostly useful for indexing into buffers.
    pub fn into_u32(self) -> u32 {
        self.0
    }

    /// Returns the [`FuncTypeIdx`] as `usize`.
    ///
    /// # Note
    ///
    /// This is mostly useful for indexing into buffers.
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }
}
