use super::InstanceEntity;
use crate::{
    ElementSegment,
    Extern,
    ExternType,
    Func,
    Global,
    Memory,
    Module,
    Table,
    collections::Map,
    engine::DedupFuncType,
    memory::DataSegment,
    module::FuncIdx,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};

/// A module instance entity builder.
#[derive(Debug)]
pub struct InstanceEntityBuilder {
    func_types: Arc<[DedupFuncType]>,
    tables: Vec<Table>,
    funcs: Vec<Func>,
    memories: Vec<Memory>,
    globals: Vec<Global>,
    start_fn: Option<FuncIdx>,
    exports: Map<Box<str>, Extern>,
    data_segments: Vec<DataSegment>,
    elem_segments: Vec<ElementSegment>,
}

impl InstanceEntityBuilder {
    /// Creates a new [`InstanceEntityBuilder`] optimized for the [`Module`].
    pub fn new(module: &Module) -> Self {
        fn vec_with_capacity_exact<T>(capacity: usize) -> Vec<T> {
            let mut v = Vec::new();
            v.reserve_exact(capacity);
            v
        }
        let mut len_funcs = module.len_funcs();
        let mut len_globals = module.len_globals();
        let mut len_tables = module.len_tables();
        let mut len_memories = module.len_memories();
        for import in module.imports() {
            match import.ty() {
                ExternType::Func(_) => {
                    len_funcs += 1;
                }
                ExternType::Table(_) => {
                    len_tables += 1;
                }
                ExternType::Memory(_) => {
                    len_memories += 1;
                }
                ExternType::Global(_) => {
                    len_globals += 1;
                }
            }
        }
        Self {
            func_types: module.func_types_cloned(),
            tables: vec_with_capacity_exact(len_tables),
            funcs: vec_with_capacity_exact(len_funcs),
            memories: vec_with_capacity_exact(len_memories),
            globals: vec_with_capacity_exact(len_globals),
            start_fn: None,
            exports: Map::default(),
            data_segments: Vec::new(),
            elem_segments: Vec::new(),
        }
    }

    /// Sets the start function of the built instance.
    ///
    /// # Panics
    ///
    /// If the start function has already been set.
    pub fn set_start(&mut self, start_fn: FuncIdx) {
        match &mut self.start_fn {
            Some(index) => panic!("already set start function {index:?}"),
            None => {
                self.start_fn = Some(start_fn);
            }
        }
    }

    /// Returns the start function index if any.
    pub fn get_start(&self) -> Option<FuncIdx> {
        self.start_fn
    }

    /// Returns the [`Memory`] at the `index`.
    ///
    /// # Panics
    ///
    /// If there is no [`Memory`] at the given `index.
    pub fn get_memory(&self, index: u32) -> Memory {
        self.memories
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Memory` at index: {index}"))
    }

    /// Returns the [`Table`] at the `index`.
    ///
    /// # Panics
    ///
    /// If there is no [`Table`] at the given `index.
    pub fn get_table(&self, index: u32) -> Table {
        self.tables
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Table` at index: {index}"))
    }

    /// Returns the [`Global`] at the `index`.
    ///
    /// # Panics
    ///
    /// If there is no [`Global`] at the given `index.
    pub fn get_global(&self, index: u32) -> Global {
        self.globals
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Global` at index: {index}"))
    }

    /// Returns the function at the `index`.
    ///
    /// # Panics
    ///
    /// If there is no function at the given `index.
    pub fn get_func(&self, index: u32) -> Func {
        self.funcs
            .get(index as usize)
            .copied()
            .unwrap_or_else(|| panic!("missing `Func` at index: {index}"))
    }

    /// Pushes a new [`Memory`] to the [`InstanceEntity`] under construction.
    pub fn push_memory(&mut self, memory: Memory) {
        self.memories.push(memory);
    }

    /// Pushes a new [`Table`] to the [`InstanceEntity`] under construction.
    pub fn push_table(&mut self, table: Table) {
        self.tables.push(table);
    }

    /// Pushes a new [`Global`] to the [`InstanceEntity`] under construction.
    pub fn push_global(&mut self, global: Global) {
        self.globals.push(global);
    }

    /// Pushes a new [`Func`] to the [`InstanceEntity`] under construction.
    pub fn push_func(&mut self, func: Func) {
        self.funcs.push(func);
    }

    /// Pushes a new [`Extern`] under the given `name` to the [`InstanceEntity`] under construction.
    ///
    /// # Panics
    ///
    /// If the name has already been used by an already pushed [`Extern`].
    pub fn push_export(&mut self, name: &str, new_value: Extern) {
        if let Some(old_value) = self.exports.get(name) {
            panic!(
                "tried to register {new_value:?} for name {name} \
                but name is already used by {old_value:?}",
            )
        }
        self.exports.insert(name.into(), new_value);
    }

    /// Pushes the [`DataSegment`] to the [`InstanceEntity`] under construction.
    pub fn push_data_segment(&mut self, segment: DataSegment) {
        self.data_segments.push(segment);
    }

    /// Pushes the [`ElementSegment`] to the [`InstanceEntity`] under construction.
    pub fn push_element_segment(&mut self, segment: ElementSegment) {
        self.elem_segments.push(segment);
    }

    /// Finishes constructing the [`InstanceEntity`].
    pub fn finish(self) -> InstanceEntity {
        InstanceEntity {
            initialized: true,
            func_types: self.func_types,
            tables: self.tables.into(),
            funcs: self.funcs.into(),
            memories: self.memories.into(),
            globals: self.globals.into(),
            exports: self.exports,
            data_segments: self.data_segments.into(),
            elem_segments: self.elem_segments.into(),
        }
    }
}
