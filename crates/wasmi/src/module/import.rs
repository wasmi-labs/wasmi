use crate::{GlobalType, MemoryType, ModuleError, TableType};
use alloc::boxed::Box;
use core::fmt::{self, Display};
use wasmparser::TypeRef;

/// A [`Module`] import item.
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub struct Import {
    /// The name of the imported item.
    name: ImportName,
    /// The type of the imported item.
    kind: ExternTypeIdx,
}

/// The name or namespace of an imported item.
#[derive(Debug, Clone)]
pub struct ImportName {
    /// The name of the [`Module`] that defines the imported item.
    ///
    /// [`Module`]: [`super::Module`]
    module: Box<str>,
    /// The name of the imported item within the [`Module`] namespace.
    ///
    /// [`Module`]: [`super::Module`]
    field: Box<str>,
}

impl Display for ImportName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let module_name = &*self.module;
        let field_name = &*self.field;
        write!(f, "{module_name}::{field_name}")
    }
}

impl ImportName {
    /// Creates a new [`Import`] item.
    pub fn new(module: &str, field: &str) -> Self {
        Self {
            module: module.into(),
            field: field.into(),
        }
    }

    /// Returns the name of the [`Module`] that defines the imported item.
    ///
    /// [`Module`]: [`super::Module`]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// Returns the name of the imported item within the [`Module`] namespace.
    ///
    /// [`Module`]: [`super::Module`]
    pub fn name(&self) -> &str {
        &self.field
    }
}

impl TryFrom<wasmparser::Import<'_>> for Import {
    type Error = ModuleError;

    fn try_from(import: wasmparser::Import) -> Result<Self, Self::Error> {
        let kind = match import.ty {
            TypeRef::Func(func_type) => Ok(ExternTypeIdx::Func(FuncTypeIdx(func_type))),
            TypeRef::Table(table_type) => table_type.try_into().map(ExternTypeIdx::Table),
            TypeRef::Memory(memory_type) => memory_type.try_into().map(ExternTypeIdx::Memory),
            TypeRef::Global(global_type) => global_type.try_into().map(ExternTypeIdx::Global),
            TypeRef::Tag(_) => Err(ModuleError::unsupported(import)),
        }?;
        Ok(Self::new(import.module, import.name, kind))
    }
}

impl Import {
    /// Creates a new [`Import`] item.
    pub fn new(module: &str, field: &str, kind: ExternTypeIdx) -> Self {
        Self {
            name: ImportName::new(module, field),
            kind,
        }
    }

    /// Splits the [`Import`] into its raw parts.
    ///
    /// # Note
    ///
    /// This allows to reuse some allocations in certain cases.
    pub fn into_name_and_type(self) -> (ImportName, ExternTypeIdx) {
        (self.name, self.kind)
    }
}

/// The kind of a [`Module`] import.
///
/// [`Module`]: [`super::Module`]
#[derive(Debug)]
pub enum ExternTypeIdx {
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
pub struct FuncTypeIdx(pub(crate) u32);

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
