use super::{br_table::WasmBrTable, BlockType, FunctionTranslator};
use crate::{
    module2::{export::TableIdx, import::FuncTypeIdx, FuncIdx, GlobalIdx},
    ModuleError,
};
use wasmparser::TypeOrFuncType;

impl<'engine, 'parser> FunctionTranslator<'engine, 'parser> {
    /// Translate a Wasm `unreachable` instruction.
    pub fn translate_unreachable(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_unreachable()?;
        Ok(())
    }

    /// Translate a Wasm `nop` (no operation) instruction.
    pub fn translate_nop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    /// Translate a Wasm `block` control flow operator.
    pub fn translate_block(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from(ty)?;
        self.func_builder.translate_block(block_type)?;
        Ok(())
    }

    /// Translate a Wasm `loop` control flow operator.
    pub fn translate_loop(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from(ty)?;
        self.func_builder.translate_loop(block_type)?;
        Ok(())
    }

    /// Translate a Wasm `if` control flow operator.
    pub fn translate_if(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from(ty)?;
        self.func_builder.translate_if(block_type)?;
        Ok(())
    }

    /// Translate a Wasm `else` control flow operator.
    pub fn translate_else(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_else()?;
        Ok(())
    }

    /// Translate a Wasm `end` control flow operator.
    pub fn translate_end(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_end()?;
        Ok(())
    }

    /// Translate a Wasm `br` control flow operator.
    pub fn translate_br(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        self.func_builder.translate_br(relative_depth)?;
        Ok(())
    }

    /// Translate a Wasm `br_if` control flow operator.
    pub fn translate_br_if(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        self.func_builder.translate_br_if(relative_depth)?;
        Ok(())
    }

    /// Translate a Wasm `br_table` control flow operator.
    pub fn translate_br_table(&mut self, br_table: wasmparser::BrTable) -> Result<(), ModuleError> {
        let br_table = WasmBrTable::new(br_table);
        self.func_builder.translate_br_table(&br_table)?;
        Ok(())
    }

    /// Translate a Wasm `return` control flow operator.
    pub fn translate_return(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_return()?;
        Ok(())
    }

    /// Translate a Wasm `call` instruction.
    pub fn translate_call(&mut self, func_idx: u32) -> Result<(), ModuleError> {
        self.func_builder.translate_call(FuncIdx(func_idx))?;
        Ok(())
    }

    /// Translate a Wasm `call_indirect` instruction.
    pub fn translate_call_indirect(
        &mut self,
        func_type_idx: u32,
        table_idx: u32,
    ) -> Result<(), ModuleError> {
        self.func_builder
            .translate_call_indirect(FuncTypeIdx(func_type_idx), TableIdx(table_idx))?;
        Ok(())
    }

    /// Translate a Wasm `drop` instruction.
    pub fn translate_drop(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_drop()?;
        Ok(())
    }

    /// Translate a Wasm `select` instruction.
    pub fn translate_select(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_select()?;
        Ok(())
    }

    /// Translate a Wasm `local.get` instruction.
    pub fn translate_local_get(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.func_builder.translate_local_get(local_idx)?;
        Ok(())
    }

    /// Translate a Wasm `local.set` instruction.
    pub fn translate_local_set(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.func_builder.translate_local_set(local_idx)?;
        Ok(())
    }

    /// Translate a Wasm `local.tee` instruction.
    pub fn translate_local_tee(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        self.func_builder.translate_local_tee(local_idx)?;
        Ok(())
    }

    /// Translate a Wasm `global.get` instruction.
    pub fn translate_global_get(&mut self, global_idx: u32) -> Result<(), ModuleError> {
        self.func_builder
            .translate_global_get(GlobalIdx(global_idx))?;
        Ok(())
    }

    /// Translate a Wasm `global.set` instruction.
    pub fn translate_global_set(&mut self, global_idx: u32) -> Result<(), ModuleError> {
        self.func_builder
            .translate_global_set(GlobalIdx(global_idx))?;
        Ok(())
    }
}
