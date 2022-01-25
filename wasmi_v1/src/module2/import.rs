use crate::{GlobalType, MemoryType, ModuleError, TableType};
use wasmparser::ImportSectionEntryType;

/// A [`Module`] import item.
#[derive(Debug)]
pub struct Import {
    /// The name of the [`Module`] that defines the imported item.
    module: Box<str>,
    /// The optional name of the imported item within the [`Module`] namespace.
    field: Option<Box<str>>,
    /// The kind of the imported item.
    kind: ImportKind,
}

impl TryFrom<wasmparser::Import<'_>> for Import {
    type Error = ModuleError;

    fn try_from(import: wasmparser::Import) -> Result<Self, Self::Error> {
        let kind = match import.ty {
            ImportSectionEntryType::Function(func_type) => {
                Ok(ImportKind::Function(FuncTypeIdx(func_type)))
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
            module: module.into(),
            field: field.map(Into::into),
            kind,
        }
    }

    /// Returns the name of the [`Module`] that defines the imported item.
    pub fn module_name(&self) -> &str {
        &self.module
    }

    /// Returns the optional name of the imported item within the [`Module`] namespace.
    pub fn field_name(&self) -> Option<&str> {
        self.field.as_deref()
    }

    /// Returns the kind of the imported item and its associated data.
    pub fn kind(&self) -> &ImportKind {
        &self.kind
    }
}

/// The kind of a [`Module`] import.
#[derive(Debug)]
pub enum ImportKind {
    /// An imported function.
    Function(FuncTypeIdx),
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
#[derive(Debug, Copy, Clone)]
pub struct FuncTypeIdx(pub(super) u32);

impl FuncTypeIdx {
    /// Returns the [`FuncTypeIdx`] as `usize`.
    ///
    /// # Note
    ///
    /// This is mostly useful for indexing into buffers.
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }
}
