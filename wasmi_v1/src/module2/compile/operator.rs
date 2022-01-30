use super::{br_table::WasmBrTable, BlockType, FunctionTranslator};
use crate::{
    module2::{export::TableIdx, import::FuncTypeIdx, FuncIdx, GlobalIdx, MemoryIdx},
    ModuleError,
};
use wasmparser::{Ieee32, Ieee64, TypeOrFuncType};

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

    /// Translate a Wasm `i32.load` instruction.
    pub fn translate_i32_load(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder.translate_i32_load(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.load` instruction.
    pub fn translate_i64_load(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder.translate_i64_load(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `f32.load` instruction.
    pub fn translate_f32_load(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder.translate_f32_load(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `f64.load` instruction.
    pub fn translate_f64_load(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder.translate_f64_load(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i32.load_i8` instruction.
    pub fn translate_i32_load_i8(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i32_load_i8(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i32.load_u8` instruction.
    pub fn translate_i32_load_u8(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i32_load_u8(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i32.load_i16` instruction.
    pub fn translate_i32_load_i16(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i32_load_i16(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i32.load_u16` instruction.
    pub fn translate_i32_load_u16(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i32_load_u16(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.load_i8` instruction.
    pub fn translate_i64_load_i8(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_load_i8(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.load_u8` instruction.
    pub fn translate_i64_load_u8(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_load_u8(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.load_i16` instruction.
    pub fn translate_i64_load_i16(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_load_i16(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.load_u16` instruction.
    pub fn translate_i64_load_u16(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_load_u16(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.load_i32` instruction.
    pub fn translate_i64_load_i32(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_load_i32(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.load_u32` instruction.
    pub fn translate_i64_load_u32(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_load_u32(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i32.store` instruction.
    pub fn translate_i32_store(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder.translate_i32_store(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.store` instruction.
    pub fn translate_i64_store(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder.translate_i64_store(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `f32.store` instruction.
    pub fn translate_f32_store(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder.translate_f32_store(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `f64.store` instruction.
    pub fn translate_f64_store(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder.translate_f64_store(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i32.store_i8` instruction.
    pub fn translate_i32_store_i8(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i32_store_i8(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i32.store_i16` instruction.
    pub fn translate_i32_store_i16(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i32_store_i16(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.store_i8` instruction.
    pub fn translate_i64_store_i8(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_store_i8(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.store_i16` instruction.
    pub fn translate_i64_store_i16(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_store_i16(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `i64.store_i32` instruction.
    pub fn translate_i64_store_i32(
        &mut self,
        memarg: wasmparser::MemoryImmediate,
    ) -> Result<(), ModuleError> {
        let memory_idx = MemoryIdx(memarg.memory);
        let offset = memarg.offset as u32;
        self.func_builder
            .translate_i64_store_i32(memory_idx, offset)?;
        Ok(())
    }

    /// Translate a Wasm `memory.size` instruction.
    pub fn translate_memory_size(
        &mut self,
        memory_idx: u32,
        _memory_byte: u8,
    ) -> Result<(), ModuleError> {
        self.func_builder
            .translate_memory_size(MemoryIdx(memory_idx))?;
        Ok(())
    }

    /// Translate a Wasm `memory.grow` instruction.
    pub fn translate_memory_grow(
        &mut self,
        memory_idx: u32,
        _memory_byte: u8,
    ) -> Result<(), ModuleError> {
        self.func_builder
            .translate_memory_grow(MemoryIdx(memory_idx))?;
        Ok(())
    }

    /// Translate a Wasm `i32.const` instruction.
    pub fn translate_i32_const(&mut self, value: i32) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_const(value)?;
        Ok(())
    }
    /// Translate a Wasm `i64.const` instruction.
    pub fn translate_i64_const(&mut self, value: i64) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_const(value)?;
        Ok(())
    }
    /// Translate a Wasm `f32.const` instruction.
    pub fn translate_f32_const(&mut self, value: Ieee32) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_const(value.bits().into())?;
        Ok(())
    }
    /// Translate a Wasm `f64.const` instruction.
    pub fn translate_f64_const(&mut self, value: Ieee64) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_const(value.bits().into())?;
        Ok(())
    }

    /// Translate a Wasm `i32_eqz` instruction.
    pub fn translate_i32_eqz(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_eqz()?;
        Ok(())
    }

    /// Translate a Wasm `i32_eq` instruction.
    pub fn translate_i32_eq(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_eq()?;
        Ok(())
    }

    /// Translate a Wasm `i32_ne` instruction.
    pub fn translate_i32_ne(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_ne()?;
        Ok(())
    }

    /// Translate a Wasm `i32_lt` instruction.
    pub fn translate_i32_lt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_lt()?;
        Ok(())
    }

    /// Translate a Wasm `u32_lt` instruction.
    pub fn translate_u32_lt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_lt()?;
        Ok(())
    }

    /// Translate a Wasm `i32_gt` instruction.
    pub fn translate_i32_gt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_gt()?;
        Ok(())
    }

    /// Translate a Wasm `u32_gt` instruction.
    pub fn translate_u32_gt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_gt()?;
        Ok(())
    }

    /// Translate a Wasm `i32_le` instruction.
    pub fn translate_i32_le(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_le()?;
        Ok(())
    }

    /// Translate a Wasm `u32_le` instruction.
    pub fn translate_u32_le(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_le()?;
        Ok(())
    }

    /// Translate a Wasm `i32_ge` instruction.
    pub fn translate_i32_ge(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_ge()?;
        Ok(())
    }

    /// Translate a Wasm `u32_ge` instruction.
    pub fn translate_u32_ge(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_ge()?;
        Ok(())
    }

    /// Translate a Wasm `i64_eqz` instruction.
    pub fn translate_i64_eqz(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_eqz()?;
        Ok(())
    }

    /// Translate a Wasm `i64_eq` instruction.
    pub fn translate_i64_eq(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_eq()?;
        Ok(())
    }

    /// Translate a Wasm `i64_ne` instruction.
    pub fn translate_i64_ne(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_ne()?;
        Ok(())
    }

    /// Translate a Wasm `i64_lt` instruction.
    pub fn translate_i64_lt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_lt()?;
        Ok(())
    }

    /// Translate a Wasm `u64_lt` instruction.
    pub fn translate_u64_lt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_lt()?;
        Ok(())
    }

    /// Translate a Wasm `i64_gt` instruction.
    pub fn translate_i64_gt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_gt()?;
        Ok(())
    }

    /// Translate a Wasm `u64_gt` instruction.
    pub fn translate_u64_gt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_gt()?;
        Ok(())
    }

    /// Translate a Wasm `i64_le` instruction.
    pub fn translate_i64_le(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_le()?;
        Ok(())
    }

    /// Translate a Wasm `u64_le` instruction.
    pub fn translate_u64_le(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_le()?;
        Ok(())
    }

    /// Translate a Wasm `i64_ge` instruction.
    pub fn translate_i64_ge(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_ge()?;
        Ok(())
    }

    /// Translate a Wasm `u64_ge` instruction.
    pub fn translate_u64_ge(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_ge()?;
        Ok(())
    }

    /// Translate a Wasm `f32_eq` instruction.
    pub fn translate_f32_eq(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_eq()?;
        Ok(())
    }

    /// Translate a Wasm `f32_ne` instruction.
    pub fn translate_f32_ne(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_ne()?;
        Ok(())
    }

    /// Translate a Wasm `f32_lt` instruction.
    pub fn translate_f32_lt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_lt()?;
        Ok(())
    }

    /// Translate a Wasm `f32_gt` instruction.
    pub fn translate_f32_gt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_gt()?;
        Ok(())
    }

    /// Translate a Wasm `f32_le` instruction.
    pub fn translate_f32_le(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_le()?;
        Ok(())
    }

    /// Translate a Wasm `f32_ge` instruction.
    pub fn translate_f32_ge(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_ge()?;
        Ok(())
    }

    /// Translate a Wasm `f64_eq` instruction.
    pub fn translate_f64_eq(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_eq()?;
        Ok(())
    }

    /// Translate a Wasm `f64_ne` instruction.
    pub fn translate_f64_ne(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_ne()?;
        Ok(())
    }

    /// Translate a Wasm `f64_lt` instruction.
    pub fn translate_f64_lt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_lt()?;
        Ok(())
    }

    /// Translate a Wasm `f64_gt` instruction.
    pub fn translate_f64_gt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_gt()?;
        Ok(())
    }

    /// Translate a Wasm `f64_le` instruction.
    pub fn translate_f64_le(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_le()?;
        Ok(())
    }

    /// Translate a Wasm `f64_ge` instruction.
    pub fn translate_f64_ge(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_ge()?;
        Ok(())
    }

    /// Translate a Wasm `i32_clz` instruction.
    pub fn translate_i32_clz(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_clz()?;
        Ok(())
    }

    /// Translate a Wasm `i32_ctz` instruction.
    pub fn translate_i32_ctz(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_ctz()?;
        Ok(())
    }

    /// Translate a Wasm `i32_popcnt` instruction.
    pub fn translate_i32_popcnt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_popcnt()?;
        Ok(())
    }

    /// Translate a Wasm `i32_add` instruction.
    pub fn translate_i32_add(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_add()?;
        Ok(())
    }

    /// Translate a Wasm `i32_sub` instruction.
    pub fn translate_i32_sub(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_sub()?;
        Ok(())
    }

    /// Translate a Wasm `i32_mul` instruction.
    pub fn translate_i32_mul(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_mul()?;
        Ok(())
    }

    /// Translate a Wasm `i32_div` instruction.
    pub fn translate_i32_div(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_div()?;
        Ok(())
    }

    /// Translate a Wasm `u32_div` instruction.
    pub fn translate_u32_div(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_div()?;
        Ok(())
    }

    /// Translate a Wasm `i32_remS` instruction.
    pub fn translate_i32_remS(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_remS()?;
        Ok(())
    }

    /// Translate a Wasm `u32_rem` instruction.
    pub fn translate_u32_rem(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_rem()?;
        Ok(())
    }

    /// Translate a Wasm `i32_and` instruction.
    pub fn translate_i32_and(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_and()?;
        Ok(())
    }

    /// Translate a Wasm `i32_or` instruction.
    pub fn translate_i32_or(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_or()?;
        Ok(())
    }

    /// Translate a Wasm `i32_xor` instruction.
    pub fn translate_i32_xor(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_xor()?;
        Ok(())
    }

    /// Translate a Wasm `i32_shl` instruction.
    pub fn translate_i32_shl(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_shl()?;
        Ok(())
    }

    /// Translate a Wasm `i32_shr` instruction.
    pub fn translate_i32_shr(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_shr()?;
        Ok(())
    }

    /// Translate a Wasm `u32_shr` instruction.
    pub fn translate_u32_shr(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_shr()?;
        Ok(())
    }

    /// Translate a Wasm `i32_rotl` instruction.
    pub fn translate_i32_rotl(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_rotl()?;
        Ok(())
    }

    /// Translate a Wasm `i32_rotr` instruction.
    pub fn translate_i32_rotr(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_rotr()?;
        Ok(())
    }

    /// Translate a Wasm `i64_clz` instruction.
    pub fn translate_i64_clz(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_clz()?;
        Ok(())
    }

    /// Translate a Wasm `i64_ctz` instruction.
    pub fn translate_i64_ctz(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_ctz()?;
        Ok(())
    }

    /// Translate a Wasm `i64_popcnt` instruction.
    pub fn translate_i64_popcnt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_popcnt()?;
        Ok(())
    }

    /// Translate a Wasm `i64_add` instruction.
    pub fn translate_i64_add(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_add()?;
        Ok(())
    }

    /// Translate a Wasm `i64_sub` instruction.
    pub fn translate_i64_sub(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_sub()?;
        Ok(())
    }

    /// Translate a Wasm `i64_mul` instruction.
    pub fn translate_i64_mul(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_mul()?;
        Ok(())
    }

    /// Translate a Wasm `i64_div` instruction.
    pub fn translate_i64_div(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_div()?;
        Ok(())
    }

    /// Translate a Wasm `u64_div` instruction.
    pub fn translate_u64_div(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_div()?;
        Ok(())
    }

    /// Translate a Wasm `i64_rem` instruction.
    pub fn translate_i64_rem(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_rem()?;
        Ok(())
    }

    /// Translate a Wasm `u64_rem` instruction.
    pub fn translate_u64_rem(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_rem()?;
        Ok(())
    }

    /// Translate a Wasm `i64_and` instruction.
    pub fn translate_i64_and(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_and()?;
        Ok(())
    }

    /// Translate a Wasm `i64_or` instruction.
    pub fn translate_i64_or(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_or()?;
        Ok(())
    }

    /// Translate a Wasm `i64_xor` instruction.
    pub fn translate_i64_xor(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_xor()?;
        Ok(())
    }

    /// Translate a Wasm `i64_shl` instruction.
    pub fn translate_i64_shl(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_shl()?;
        Ok(())
    }

    /// Translate a Wasm `i64_shrS` instruction.
    pub fn translate_i64_shrS(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_shrS()?;
        Ok(())
    }

    /// Translate a Wasm `u64_shr` instruction.
    pub fn translate_u64_shr(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_shr()?;
        Ok(())
    }

    /// Translate a Wasm `i64_rotl` instruction.
    pub fn translate_i64_rotl(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_rotl()?;
        Ok(())
    }

    /// Translate a Wasm `i64_rotr` instruction.
    pub fn translate_i64_rotr(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_rotr()?;
        Ok(())
    }

    /// Translate a Wasm `f32_abs` instruction.
    pub fn translate_f32_abs(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_abs()?;
        Ok(())
    }

    /// Translate a Wasm `f32_neg` instruction.
    pub fn translate_f32_neg(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_neg()?;
        Ok(())
    }

    /// Translate a Wasm `f32_ceil` instruction.
    pub fn translate_f32_ceil(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_ceil()?;
        Ok(())
    }

    /// Translate a Wasm `f32_floor` instruction.
    pub fn translate_f32_floor(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_floor()?;
        Ok(())
    }

    /// Translate a Wasm `f32_trunc` instruction.
    pub fn translate_f32_trunc(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_trunc()?;
        Ok(())
    }

    /// Translate a Wasm `f32_nearest` instruction.
    pub fn translate_f32_nearest(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_nearest()?;
        Ok(())
    }

    /// Translate a Wasm `f32_sqrt` instruction.
    pub fn translate_f32_sqrt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_sqrt()?;
        Ok(())
    }

    /// Translate a Wasm `f32_add` instruction.
    pub fn translate_f32_add(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_add()?;
        Ok(())
    }

    /// Translate a Wasm `f32_sub` instruction.
    pub fn translate_f32_sub(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_sub()?;
        Ok(())
    }

    /// Translate a Wasm `f32_mul` instruction.
    pub fn translate_f32_mul(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_mul()?;
        Ok(())
    }

    /// Translate a Wasm `f32_div` instruction.
    pub fn translate_f32_div(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_div()?;
        Ok(())
    }

    /// Translate a Wasm `f32_min` instruction.
    pub fn translate_f32_min(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_min()?;
        Ok(())
    }

    /// Translate a Wasm `f32_max` instruction.
    pub fn translate_f32_max(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_max()?;
        Ok(())
    }

    /// Translate a Wasm `f32_copysign` instruction.
    pub fn translate_f32_copysign(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_copysign()?;
        Ok(())
    }

    /// Translate a Wasm `f64_abs` instruction.
    pub fn translate_f64_abs(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_abs()?;
        Ok(())
    }

    /// Translate a Wasm `f64_neg` instruction.
    pub fn translate_f64_neg(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_neg()?;
        Ok(())
    }

    /// Translate a Wasm `f64_ceil` instruction.
    pub fn translate_f64_ceil(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_ceil()?;
        Ok(())
    }

    /// Translate a Wasm `f64_floor` instruction.
    pub fn translate_f64_floor(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_floor()?;
        Ok(())
    }

    /// Translate a Wasm `f64_trunc` instruction.
    pub fn translate_f64_trunc(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_trunc()?;
        Ok(())
    }

    /// Translate a Wasm `f64_nearest` instruction.
    pub fn translate_f64_nearest(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_nearest()?;
        Ok(())
    }

    /// Translate a Wasm `f64_sqrt` instruction.
    pub fn translate_f64_sqrt(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_sqrt()?;
        Ok(())
    }

    /// Translate a Wasm `f64_add` instruction.
    pub fn translate_f64_add(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_add()?;
        Ok(())
    }

    /// Translate a Wasm `f64_sub` instruction.
    pub fn translate_f64_sub(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_sub()?;
        Ok(())
    }

    /// Translate a Wasm `f64_mul` instruction.
    pub fn translate_f64_mul(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_mul()?;
        Ok(())
    }

    /// Translate a Wasm `f64_div` instruction.
    pub fn translate_f64_div(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_div()?;
        Ok(())
    }

    /// Translate a Wasm `f64_min` instruction.
    pub fn translate_f64_min(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_min()?;
        Ok(())
    }

    /// Translate a Wasm `f64_max` instruction.
    pub fn translate_f64_max(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_max()?;
        Ok(())
    }

    /// Translate a Wasm `f64_copysign` instruction.
    pub fn translate_f64_copysign(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_copysign()?;
        Ok(())
    }

    /// Translate a Wasm `i32_wrap_i64` instruction.
    pub fn translate_i32_wrap_i64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_wrap_i64()?;
        Ok(())
    }

    /// Translate a Wasm `i32_trunc_f32` instruction.
    pub fn translate_i32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_trunc_f32()?;
        Ok(())
    }

    /// Translate a Wasm `u32_trunc_f32` instruction.
    pub fn translate_u32_trunc_f32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_trunc_f32()?;
        Ok(())
    }

    /// Translate a Wasm `i32_trunc_f64` instruction.
    pub fn translate_i32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_trunc_f64()?;
        Ok(())
    }

    /// Translate a Wasm `u32_trunc_f64` instruction.
    pub fn translate_u32_trunc_f64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u32_trunc_f64()?;
        Ok(())
    }

    /// Translate a Wasm `i64_extend_i32` instruction.
    pub fn translate_i64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_extend_i32()?;
        Ok(())
    }

    /// Translate a Wasm `u64_extend_i32` instruction.
    pub fn translate_u64_extend_i32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_extend_i32()?;
        Ok(())
    }

    /// Translate a Wasm `i64_trunc_F3` instruction.
    pub fn translate_i64_trunc_F3(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_trunc_F3()?;
        Ok(())
    }

    /// Translate a Wasm `u64_trunc_F3` instruction.
    pub fn translate_u64_trunc_F3(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_trunc_F3()?;
        Ok(())
    }

    /// Translate a Wasm `i64_trunc_F6` instruction.
    pub fn translate_i64_trunc_F6(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_trunc_F6()?;
        Ok(())
    }

    /// Translate a Wasm `u64_trunc_F6` instruction.
    pub fn translate_u64_trunc_F6(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_u64_trunc_F6()?;
        Ok(())
    }

    /// Translate a Wasm `f32_convert_i32` instruction.
    pub fn translate_f32_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_convert_i32()?;
        Ok(())
    }

    /// Translate a Wasm `f32_convert_u32` instruction.
    pub fn translate_f32_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_convert_u32()?;
        Ok(())
    }

    /// Translate a Wasm `f32_convert_i64` instruction.
    pub fn translate_f32_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_convert_i64()?;
        Ok(())
    }

    /// Translate a Wasm `f32_convert_u64` instruction.
    pub fn translate_f32_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_convert_u64()?;
        Ok(())
    }

    /// Translate a Wasm `f32_demote_f64` instruction.
    pub fn translate_f32_demote_f64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_demote_f64()?;
        Ok(())
    }

    /// Translate a Wasm `f64_convert_i32` instruction.
    pub fn translate_f64_convert_i32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_convert_i32()?;
        Ok(())
    }

    /// Translate a Wasm `f64_convert_u32` instruction.
    pub fn translate_f64_convert_u32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_convert_u32()?;
        Ok(())
    }

    /// Translate a Wasm `f64_convert_i64` instruction.
    pub fn translate_f64_convert_i64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_convert_i64()?;
        Ok(())
    }

    /// Translate a Wasm `f64_convert_u64` instruction.
    pub fn translate_f64_convert_u64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_convert_u64()?;
        Ok(())
    }

    /// Translate a Wasm `f64_promote_f32` instruction.
    pub fn translate_f64_promote_f32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_promote_f32()?;
        Ok(())
    }

    /// Translate a Wasm `i32_reinterpret_f32` instruction.
    pub fn translate_i32_reinterpret_f32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i32_reinterpret_f32()?;
        Ok(())
    }

    /// Translate a Wasm `i64_reinterpret_f64` instruction.
    pub fn translate_i64_reinterpret_f64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_i64_reinterpret_f64()?;
        Ok(())
    }

    /// Translate a Wasm `f32_reinterpret_i32` instruction.
    pub fn translate_f32_reinterpret_i32(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f32_reinterpret_i32()?;
        Ok(())
    }

    /// Translate a Wasm `f64_reinterpret_i64` instruction.
    pub fn translate_f64_reinterpret_i64(&mut self) -> Result<(), ModuleError> {
        self.func_builder.translate_f64_reinterpret_i64()?;
        Ok(())
    }
}
