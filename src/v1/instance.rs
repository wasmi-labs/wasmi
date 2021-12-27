#![allow(dead_code)] // TODO: remove

use core::ops::Deref;

use super::{AsContext, Extern, Func, Global, Index, Memory, Signature, Stored, Table};
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

/// A raw index to a module instance entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceIdx(usize);

impl Index for InstanceIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

/// A module instance entity.
#[derive(Debug)]
pub struct InstanceEntity {
    signatures: Vec<Signature>,
    tables: Vec<Table>,
    funcs: Vec<Func>,
    memories: Vec<Memory>,
    globals: Vec<Global>,
    exports: BTreeMap<String, Extern>,
}

impl InstanceEntity {
    /// Creates a new [`InstanceEntityBuilder`].
    pub(crate) fn build() -> InstanceEntityBuilder {
        InstanceEntityBuilder {
            instance: Self {
                signatures: Vec::default(),
                tables: Vec::default(),
                funcs: Vec::default(),
                memories: Vec::default(),
                globals: Vec::default(),
                exports: BTreeMap::default(),
            },
        }
    }

    /// Returns the linear memory at the `index` if any.
    pub(crate) fn get_memory(&self, index: u32) -> Option<Memory> {
        self.memories.get(index as usize).copied()
    }

    /// Returns the table at the `index` if any.
    pub(crate) fn get_table(&self, index: u32) -> Option<Table> {
        self.tables.get(index as usize).copied()
    }

    /// Returns the global variable at the `index` if any.
    pub(crate) fn get_global(&self, index: u32) -> Option<Global> {
        self.globals.get(index as usize).copied()
    }

    /// Returns the function at the `index` if any.
    pub(crate) fn get_func(&self, index: u32) -> Option<Func> {
        self.funcs.get(index as usize).copied()
    }

    /// Returns the signature at the `index` if any.
    pub(crate) fn get_signature(&self, index: u32) -> Option<Signature> {
        self.signatures.get(index as usize).copied()
    }

    /// Returns the value exported to the given `name` if any.
    pub(crate) fn get_export(&self, name: &str) -> Option<Extern> {
        self.exports.get(name).copied()
    }
}

/// A module instance entitiy builder.
#[derive(Debug)]
pub struct InstanceEntityBuilder {
    /// The [`InstanceEntity`] under construction.
    instance: InstanceEntity,
}

impl InstanceEntityBuilder {
    /// Pushes a new [`Memory`] to the [`InstanceEntity`] under construction.
    pub(crate) fn push_memory(&mut self, memory: Memory) {
        self.instance.memories.push(memory);
    }

    /// Pushes a new [`Table`] to the [`InstanceEntity`] under construction.
    pub(crate) fn push_table(&mut self, table: Table) {
        self.instance.tables.push(table);
    }

    /// Pushes a new [`Global`] to the [`InstanceEntity`] under construction.
    pub(crate) fn push_global(&mut self, global: Global) {
        self.instance.globals.push(global);
    }

    /// Pushes a new [`Func`] to the [`InstanceEntity`] under construction.
    pub(crate) fn push_func(&mut self, func: Func) {
        self.instance.funcs.push(func);
    }

    /// Pushes a new [`Signature`] to the [`InstanceEntity`] under construction.
    pub(crate) fn push_signature(&mut self, signature: Signature) {
        self.instance.signatures.push(signature);
    }

    /// Pushes a new [`Extern`] under the given `name` to the [`InstanceEntity`] under construction.
    ///
    /// # Panics
    ///
    /// If the name has already been used by an already pushed [`Extern`].
    pub(crate) fn push_export(&mut self, name: &str, new_value: Extern) {
        if let Some(old_value) = self.instance.exports.get(name) {
            panic!(
                "tried to register {:?} for name {} but name is already used by {:?}",
                new_value, name, old_value,
            )
        }
        self.instance.exports.insert(name.to_string(), new_value);
    }

    /// Finishes constructing the [`InstanceEntity`].
    pub(crate) fn finish(self) -> InstanceEntity {
        self.instance
    }
}

impl Deref for InstanceEntityBuilder {
    type Target = InstanceEntity;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

/// A Wasm module instance reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Instance(Stored<InstanceIdx>);

impl Instance {
    /// Creates a new stored instance reference.
    ///
    /// # Note
    ///
    /// This API is primarily used by the [`Store`] itself.
    ///
    /// [`Store`]: [`crate::v1::Store`]
    pub(super) fn from_inner(stored: Stored<InstanceIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn into_inner(self) -> Stored<InstanceIdx> {
        self.0
    }

    /// Returns the linear memory at the `index` if any.
    pub(crate) fn get_memory(&self, store: impl AsContext, index: u32) -> Option<Memory> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_memory(index)
    }

    /// Returns the table at the `index` if any.
    pub(crate) fn get_table(&self, store: impl AsContext, index: u32) -> Option<Table> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_table(index)
    }

    /// Returns the global variable at the `index` if any.
    pub(crate) fn get_global(&self, store: impl AsContext, index: u32) -> Option<Global> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_global(index)
    }

    /// Returns the function at the `index` if any.
    pub(crate) fn get_func(&self, store: impl AsContext, index: u32) -> Option<Func> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_func(index)
    }

    /// Returns the signature at the `index` if any.
    pub(crate) fn get_signature(&self, store: impl AsContext, index: u32) -> Option<Signature> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_signature(index)
    }

    /// Returns the value exported to the given `name` if any.
    pub(crate) fn get_export(&self, store: impl AsContext, name: &str) -> Option<Extern> {
        store
            .as_context()
            .store
            .resolve_instance(*self)
            .get_export(name)
    }
}
