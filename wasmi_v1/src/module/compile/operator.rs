use super::{BlockType, FunctionTranslator};
use crate::{
    engine::RelativeDepth,
    module::{export::TableIdx, import::FuncTypeIdx, FuncIdx, GlobalIdx, MemoryIdx},
    ModuleError,
};
use wasmparser::{Ieee32, Ieee64, TypeOrFuncType};

impl<'engine, 'parser> FunctionTranslator<'engine, 'parser> {
    /// Translate a Wasm `nop` (no operation) instruction.
    pub fn translate_nop(&mut self) -> Result<(), ModuleError> {
        // We can simply ignore Wasm `nop` instructions.
        //
        // In most cases they should not be included in well optimized
        // Wasm binaries anyways.
        Ok(())
    }

    /// Translate a Wasm `block` control flow operator.
    pub fn translate_block(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from_wasmparser(ty, self.res)?;
        self.func_builder.translate_block(block_type)?;
        Ok(())
    }

    /// Translate a Wasm `loop` control flow operator.
    pub fn translate_loop(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from_wasmparser(ty, self.res)?;
        self.func_builder.translate_loop(block_type)?;
        Ok(())
    }

    /// Translate a Wasm `if` control flow operator.
    pub fn translate_if(&mut self, ty: TypeOrFuncType) -> Result<(), ModuleError> {
        let block_type = BlockType::try_from_wasmparser(ty, self.res)?;
        self.func_builder.translate_if(block_type)?;
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
        let default = RelativeDepth::from_u32(br_table.default());
        let targets = br_table
            .targets()
            .map(|relative_depth| {
                relative_depth.unwrap_or_else(|error| {
                    panic!(
                        "encountered unexpected invalid relative depth for `br_table` target: {}",
                        error,
                    )
                })
            })
            .map(RelativeDepth::from_u32);
        self.func_builder.translate_br_table(default, targets)?;
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
}

macro_rules! define_translate_fn {
    (
        $(
            $(#[$docs:meta])*
            fn $name:ident();
        )*
    ) => {
        $(
            $( #[$docs] )*
            pub fn $name(&mut self) -> Result<(), ModuleError> {
                self.func_builder.$name()?;
                Ok(())
            }
        )*
    };
}

impl<'engine, 'parser> FunctionTranslator<'engine, 'parser> {
    define_translate_fn! {
        /// Translate a Wasm `unreachable` instruction.
        fn translate_unreachable();
        /// Translate a Wasm `else` control flow operator.
        fn translate_else();
        /// Translate a Wasm `end` control flow operator.
        fn translate_end();
        /// Translate a Wasm `return` control flow operator.
        fn translate_return();
        /// Translate a Wasm `drop` instruction.
        fn translate_drop();
        /// Translate a Wasm `select` instruction.
        fn translate_select();

        /// Translate a Wasm `i32_eqz` instruction.
        fn translate_i32_eqz();
        /// Translate a Wasm `i32_eq` instruction.
        fn translate_i32_eq();
        /// Translate a Wasm `i32_ne` instruction.
        fn translate_i32_ne();
        /// Translate a Wasm `i32_lt` instruction.
        fn translate_i32_lt();
        /// Translate a Wasm `u32_lt` instruction.
        fn translate_u32_lt();
        /// Translate a Wasm `i32_gt` instruction.
        fn translate_i32_gt();
        /// Translate a Wasm `u32_gt` instruction.
        fn translate_u32_gt();
        /// Translate a Wasm `i32_le` instruction.
        fn translate_i32_le();
        /// Translate a Wasm `u32_le` instruction.
        fn translate_u32_le();
        /// Translate a Wasm `i32_ge` instruction.
        fn translate_i32_ge();
        /// Translate a Wasm `u32_ge` instruction.
        fn translate_u32_ge();
        /// Translate a Wasm `i64_eqz` instruction.
        fn translate_i64_eqz();
        /// Translate a Wasm `i64_eq` instruction.
        fn translate_i64_eq();
        /// Translate a Wasm `i64_ne` instruction.
        fn translate_i64_ne();
        /// Translate a Wasm `i64_lt` instruction.
        fn translate_i64_lt();
        /// Translate a Wasm `u64_lt` instruction.
        fn translate_u64_lt();
        /// Translate a Wasm `i64_gt` instruction.
        fn translate_i64_gt();
        /// Translate a Wasm `u64_gt` instruction.
        fn translate_u64_gt();
        /// Translate a Wasm `i64_le` instruction.
        fn translate_i64_le();
        /// Translate a Wasm `u64_le` instruction.
        fn translate_u64_le();
        /// Translate a Wasm `i64_ge` instruction.
        fn translate_i64_ge();
        /// Translate a Wasm `u64_ge` instruction.
        fn translate_u64_ge();
        /// Translate a Wasm `f32_eq` instruction.
        fn translate_f32_eq();
        /// Translate a Wasm `f32_ne` instruction.
        fn translate_f32_ne();
        /// Translate a Wasm `f32_lt` instruction.
        fn translate_f32_lt();
        /// Translate a Wasm `f32_gt` instruction.
        fn translate_f32_gt();
        /// Translate a Wasm `f32_le` instruction.
        fn translate_f32_le();
        /// Translate a Wasm `f32_ge` instruction.
        fn translate_f32_ge();
        /// Translate a Wasm `f64_eq` instruction.
        fn translate_f64_eq();
        /// Translate a Wasm `f64_ne` instruction.
        fn translate_f64_ne();
        /// Translate a Wasm `f64_lt` instruction.
        fn translate_f64_lt();
        /// Translate a Wasm `f64_gt` instruction.
        fn translate_f64_gt();
        /// Translate a Wasm `f64_le` instruction.
        fn translate_f64_le();
        /// Translate a Wasm `f64_ge` instruction.
        fn translate_f64_ge();
        /// Translate a Wasm `i32_clz` instruction.
        fn translate_i32_clz();
        /// Translate a Wasm `i32_ctz` instruction.
        fn translate_i32_ctz();
        /// Translate a Wasm `i32_popcnt` instruction.
        fn translate_i32_popcnt();
        /// Translate a Wasm `i32_add` instruction.
        fn translate_i32_add();
        /// Translate a Wasm `i32_sub` instruction.
        fn translate_i32_sub();
        /// Translate a Wasm `i32_mul` instruction.
        fn translate_i32_mul();
        /// Translate a Wasm `i32_div` instruction.
        fn translate_i32_div();
        /// Translate a Wasm `u32_div` instruction.
        fn translate_u32_div();
        /// Translate a Wasm `i32_rem` instruction.
        fn translate_i32_rem();
        /// Translate a Wasm `u32_rem` instruction.
        fn translate_u32_rem();
        /// Translate a Wasm `i32_and` instruction.
        fn translate_i32_and();
        /// Translate a Wasm `i32_or` instruction.
        fn translate_i32_or();
        /// Translate a Wasm `i32_xor` instruction.
        fn translate_i32_xor();
        /// Translate a Wasm `i32_shl` instruction.
        fn translate_i32_shl();
        /// Translate a Wasm `i32_shr` instruction.
        fn translate_i32_shr();
        /// Translate a Wasm `u32_shr` instruction.
        fn translate_u32_shr();
        /// Translate a Wasm `i32_rotl` instruction.
        fn translate_i32_rotl();
        /// Translate a Wasm `i32_rotr` instruction.
        fn translate_i32_rotr();
        /// Translate a Wasm `i64_clz` instruction.
        fn translate_i64_clz();
        /// Translate a Wasm `i64_ctz` instruction.
        fn translate_i64_ctz();
        /// Translate a Wasm `i64_popcnt` instruction.
        fn translate_i64_popcnt();
        /// Translate a Wasm `i64_add` instruction.
        fn translate_i64_add();
        /// Translate a Wasm `i64_sub` instruction.
        fn translate_i64_sub();
        /// Translate a Wasm `i64_mul` instruction.
        fn translate_i64_mul();
        /// Translate a Wasm `i64_div` instruction.
        fn translate_i64_div();
        /// Translate a Wasm `u64_div` instruction.
        fn translate_u64_div();
        /// Translate a Wasm `i64_rem` instruction.
        fn translate_i64_rem();
        /// Translate a Wasm `u64_rem` instruction.
        fn translate_u64_rem();
        /// Translate a Wasm `i64_and` instruction.
        fn translate_i64_and();
        /// Translate a Wasm `i64_or` instruction.
        fn translate_i64_or();
        /// Translate a Wasm `i64_xor` instruction.
        fn translate_i64_xor();
        /// Translate a Wasm `i64_shl` instruction.
        fn translate_i64_shl();
        /// Translate a Wasm `i64_shr` instruction.
        fn translate_i64_shr();
        /// Translate a Wasm `u64_shr` instruction.
        fn translate_u64_shr();
        /// Translate a Wasm `i64_rotl` instruction.
        fn translate_i64_rotl();
        /// Translate a Wasm `i64_rotr` instruction.
        fn translate_i64_rotr();
        /// Translate a Wasm `f32_abs` instruction.
        fn translate_f32_abs();
        /// Translate a Wasm `f32_neg` instruction.
        fn translate_f32_neg();
        /// Translate a Wasm `f32_ceil` instruction.
        fn translate_f32_ceil();
        /// Translate a Wasm `f32_floor` instruction.
        fn translate_f32_floor();
        /// Translate a Wasm `f32_trunc` instruction.
        fn translate_f32_trunc();
        /// Translate a Wasm `f32_nearest` instruction.
        fn translate_f32_nearest();
        /// Translate a Wasm `f32_sqrt` instruction.
        fn translate_f32_sqrt();
        /// Translate a Wasm `f32_add` instruction.
        fn translate_f32_add();
        /// Translate a Wasm `f32_sub` instruction.
        fn translate_f32_sub();
        /// Translate a Wasm `f32_mul` instruction.
        fn translate_f32_mul();
        /// Translate a Wasm `f32_div` instruction.
        fn translate_f32_div();
        /// Translate a Wasm `f32_min` instruction.
        fn translate_f32_min();
        /// Translate a Wasm `f32_max` instruction.
        fn translate_f32_max();
        /// Translate a Wasm `f32_copysign` instruction.
        fn translate_f32_copysign();
        /// Translate a Wasm `f64_abs` instruction.
        fn translate_f64_abs();
        /// Translate a Wasm `f64_neg` instruction.
        fn translate_f64_neg();
        /// Translate a Wasm `f64_ceil` instruction.
        fn translate_f64_ceil();
        /// Translate a Wasm `f64_floor` instruction.
        fn translate_f64_floor();
        /// Translate a Wasm `f64_trunc` instruction.
        fn translate_f64_trunc();
        /// Translate a Wasm `f64_nearest` instruction.
        fn translate_f64_nearest();
        /// Translate a Wasm `f64_sqrt` instruction.
        fn translate_f64_sqrt();
        /// Translate a Wasm `f64_add` instruction.
        fn translate_f64_add();
        /// Translate a Wasm `f64_sub` instruction.
        fn translate_f64_sub();
        /// Translate a Wasm `f64_mul` instruction.
        fn translate_f64_mul();
        /// Translate a Wasm `f64_div` instruction.
        fn translate_f64_div();
        /// Translate a Wasm `f64_min` instruction.
        fn translate_f64_min();
        /// Translate a Wasm `f64_max` instruction.
        fn translate_f64_max();
        /// Translate a Wasm `f64_copysign` instruction.
        fn translate_f64_copysign();
        /// Translate a Wasm `i32_wrap_i64` instruction.
        fn translate_i32_wrap_i64();
        /// Translate a Wasm `i32_trunc_f32` instruction.
        fn translate_i32_trunc_f32();
        /// Translate a Wasm `u32_trunc_f32` instruction.
        fn translate_u32_trunc_f32();
        /// Translate a Wasm `i32_trunc_f64` instruction.
        fn translate_i32_trunc_f64();
        /// Translate a Wasm `u32_trunc_f64` instruction.
        fn translate_u32_trunc_f64();
        /// Translate a Wasm `i64_extend_i32` instruction.
        fn translate_i64_extend_i32();
        /// Translate a Wasm `u64_extend_i32` instruction.
        fn translate_u64_extend_i32();
        /// Translate a Wasm `i64_trunc_F3` instruction.
        fn translate_i64_trunc_f32();
        /// Translate a Wasm `u64_trunc_F3` instruction.
        fn translate_u64_trunc_f32();
        /// Translate a Wasm `i64_trunc_F6` instruction.
        fn translate_i64_trunc_f64();
        /// Translate a Wasm `u64_trunc_F6` instruction.
        fn translate_u64_trunc_f64();
        /// Translate a Wasm `f32_convert_i32` instruction.
        fn translate_f32_convert_i32();
        /// Translate a Wasm `f32_convert_u32` instruction.
        fn translate_f32_convert_u32();
        /// Translate a Wasm `f32_convert_i64` instruction.
        fn translate_f32_convert_i64();
        /// Translate a Wasm `f32_convert_u64` instruction.
        fn translate_f32_convert_u64();
        /// Translate a Wasm `f32_demote_f64` instruction.
        fn translate_f32_demote_f64();
        /// Translate a Wasm `f64_convert_i32` instruction.
        fn translate_f64_convert_i32();
        /// Translate a Wasm `f64_convert_u32` instruction.
        fn translate_f64_convert_u32();
        /// Translate a Wasm `f64_convert_i64` instruction.
        fn translate_f64_convert_i64();
        /// Translate a Wasm `f64_convert_u64` instruction.
        fn translate_f64_convert_u64();
        /// Translate a Wasm `f64_promote_f32` instruction.
        fn translate_f64_promote_f32();
        /// Translate a Wasm `i32_reinterpret_f32` instruction.
        fn translate_i32_reinterpret_f32();
        /// Translate a Wasm `i64_reinterpret_f64` instruction.
        fn translate_i64_reinterpret_f64();
        /// Translate a Wasm `f32_reinterpret_i32` instruction.
        fn translate_f32_reinterpret_i32();
        /// Translate a Wasm `f64_reinterpret_i64` instruction.
        fn translate_f64_reinterpret_i64();
        /// Translate a Wasm `i32.extend_i8` instruction.
        fn translate_i32_sign_extend8();
        /// Translate a Wasm `i32.extend_i16` instruction.
        fn translate_i32_sign_extend16();
        /// Translate a Wasm `i64.extend_i8` instruction.
        fn translate_i64_sign_extend8();
        /// Translate a Wasm `i64.extend_i16` instruction.
        fn translate_i64_sign_extend16();
        /// Translate a Wasm `i64.extend_i32` instruction.
        fn translate_i64_sign_extend32();
        /// Translate a Wasm `i32.truncate_saturate_f32` instruction.
        fn translate_i32_truncate_saturate_f32();
        /// Translate a Wasm `u32.truncate_saturate_f32` instruction.
        fn translate_u32_truncate_saturate_f32();
        /// Translate a Wasm `i32.truncate_saturate_f64` instruction.
        fn translate_i32_truncate_saturate_f64();
        /// Translate a Wasm `u32.truncate_saturate_f64` instruction.
        fn translate_u32_truncate_saturate_f64();
        /// Translate a Wasm `i64.truncate_saturate_f32` instruction.
        fn translate_i64_truncate_saturate_f32();
        /// Translate a Wasm `u64.truncate_saturate_f32` instruction.
        fn translate_u64_truncate_saturate_f32();
        /// Translate a Wasm `i64.truncate_saturate_f64` instruction.
        fn translate_i64_truncate_saturate_f64();
        /// Translate a Wasm `u64.truncate_saturate_f64` instruction.
        fn translate_u64_truncate_saturate_f64();
    }
}
