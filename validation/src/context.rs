use crate::Error;
use alloc::vec::Vec;
use parity_wasm::elements::{
    BlockType,
    FunctionType,
    GlobalType,
    MemoryType,
    TableType,
    ValueType,
};

#[derive(Default, Debug)]
pub struct ModuleContext {
    pub memories: Vec<MemoryType>,
    pub tables: Vec<TableType>,
    pub globals: Vec<GlobalType>,
    pub types: Vec<FunctionType>,
    pub func_type_indexes: Vec<u32>,
}

impl ModuleContext {
    pub fn memories(&self) -> &[MemoryType] {
        &self.memories
    }

    pub fn tables(&self) -> &[TableType] {
        &self.tables
    }

    pub fn globals(&self) -> &[GlobalType] {
        &self.globals
    }

    pub fn types(&self) -> &[FunctionType] {
        &self.types
    }

    pub fn func_type_indexes(&self) -> &[u32] {
        &self.func_type_indexes
    }

    pub fn require_memory(&self, idx: u32) -> Result<(), Error> {
        if self.memories().get(idx as usize).is_none() {
            return Err(Error(format!("Memory at index {} doesn't exists", idx)));
        }
        Ok(())
    }

    pub fn require_table(&self, idx: u32) -> Result<&TableType, Error> {
        self.tables()
            .get(idx as usize)
            .ok_or_else(|| Error(format!("Table at index {} doesn't exists", idx)))
    }

    pub fn require_function(&self, idx: u32) -> Result<(&[ValueType], BlockType), Error> {
        let ty_idx = self
            .func_type_indexes()
            .get(idx as usize)
            .ok_or_else(|| Error(format!("Function at index {} doesn't exists", idx)))?;
        self.require_function_type(*ty_idx)
    }

    pub fn require_function_type(&self, idx: u32) -> Result<(&[ValueType], BlockType), Error> {
        let ty = self
            .types()
            .get(idx as usize)
            .ok_or_else(|| Error(format!("Type at index {} doesn't exists", idx)))?;

        let params = ty.params();
        let return_ty = ty
            .results()
            .first()
            .map(|vty| BlockType::Value(*vty))
            .unwrap_or(BlockType::NoResult);
        Ok((params, return_ty))
    }

    pub fn require_global(&self, idx: u32, mutability: Option<bool>) -> Result<&GlobalType, Error> {
        let global = self
            .globals()
            .get(idx as usize)
            .ok_or_else(|| Error(format!("Global at index {} doesn't exists", idx)))?;

        if let Some(expected_mutable) = mutability {
            if expected_mutable && !global.is_mutable() {
                return Err(Error(format!("Expected global {} to be mutable", idx)));
            }
            if !expected_mutable && global.is_mutable() {
                return Err(Error(format!("Expected global {} to be immutable", idx)));
            }
        }
        Ok(global)
    }
}

#[derive(Default)]
pub struct ModuleContextBuilder {
    memories: Vec<MemoryType>,
    tables: Vec<TableType>,
    globals: Vec<GlobalType>,
    types: Vec<FunctionType>,
    func_type_indexes: Vec<u32>,
}

impl ModuleContextBuilder {
    pub fn new() -> ModuleContextBuilder {
        ModuleContextBuilder::default()
    }

    pub fn push_memory(&mut self, memory: MemoryType) {
        self.memories.push(memory);
    }

    pub fn push_table(&mut self, table: TableType) {
        self.tables.push(table);
    }

    pub fn push_global(&mut self, global: GlobalType) {
        self.globals.push(global);
    }

    pub fn set_types(&mut self, types: Vec<FunctionType>) {
        self.types = types;
    }

    pub fn push_func_type_index(&mut self, func_type_index: u32) {
        self.func_type_indexes.push(func_type_index);
    }

    pub fn build(self) -> ModuleContext {
        let ModuleContextBuilder {
            memories,
            tables,
            globals,
            types,
            func_type_indexes,
        } = self;

        ModuleContext {
            memories,
            tables,
            globals,
            types,
            func_type_indexes,
        }
    }
}
