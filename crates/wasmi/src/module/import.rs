use crate::{GlobalType, MemoryType, TableType};
use alloc::boxed::Box;
use core::fmt::{self, Display};

#[cfg(feature = "parser")]
use super::utils::FromWasmparser as _;
#[cfg(feature = "parser")]
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

#[cfg(feature = "parser")]
impl From<wasmparser::Import<'_>> for Import {
    fn from(import: wasmparser::Import) -> Self {
        let kind = match import.ty {
            TypeRef::Func(ty) => ExternTypeIdx::Func(ty.into()),
            TypeRef::Table(ty) => ExternTypeIdx::Table(TableType::from_wasmparser(ty)),
            TypeRef::Memory(ty) => ExternTypeIdx::Memory(MemoryType::from_wasmparser(ty)),
            TypeRef::Global(ty) => ExternTypeIdx::Global(GlobalType::from_wasmparser(ty)),
            TypeRef::Tag(tag) => panic!(
                "wasmi does not support the `exception-handling` Wasm proposal but found: {tag:?}"
            ),
        };
        Self::new(import.module, import.name, kind)
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
pub struct FuncTypeIdx(u32);

impl From<u32> for FuncTypeIdx {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl FuncTypeIdx {
    /// Returns the inner `u32` index of the [`FuncTypeIdx`].
    pub fn into_u32(self) -> u32 {
        self.0
    }
}
