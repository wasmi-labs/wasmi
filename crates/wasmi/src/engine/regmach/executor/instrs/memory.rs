use wasmi_core::Pages;

use super::Executor;
use crate::{
    core::TrapCode,
    engine::{
        bytecode::DataSegmentIdx,
        bytecode2::{Const16, Register},
    },
    error::EntityGrowError,
    store::ResourceLimiterRef,
};

#[cfg(doc)]
use crate::engine::bytecode2::Instruction;

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Executes an [`Instruction::DataDrop`].
    #[inline(always)]
    pub fn execute_data_drop(&mut self, segment_index: DataSegmentIdx) {
        let segment = self
            .cache
            .get_data_segment(self.ctx, segment_index.to_u32());
        self.ctx.resolve_data_segment_mut(&segment).drop_bytes();
        self.next_instr();
    }

    /// Executes an [`Instruction::MemorySize`].
    #[inline(always)]
    pub fn execute_memory_size(&mut self, result: Register) {
        let memory = self.cache.default_memory(self.ctx);
        let size: u32 = self.ctx.resolve_memory(memory).current_pages().into();
        self.set_register(result, size);
        self.next_instr()
    }

    /// Executes an [`Instruction::MemoryGrow`].
    #[inline(always)]
    pub fn execute_memory_grow(
        &mut self,
        result: Register,
        delta: Register,
        resource_limiter: &mut ResourceLimiterRef<'ctx>,
    ) -> Result<(), TrapCode> {
        let delta: u32 = self.get_register_as(delta);
        self.execute_memory_grow_impl(result, delta, resource_limiter)
    }

    /// Executes an [`Instruction::MemoryGrowBy`].
    #[inline(always)]
    pub fn execute_memory_grow_by(
        &mut self,
        result: Register,
        delta: Const16<u32>,
        resource_limiter: &mut ResourceLimiterRef<'ctx>,
    ) -> Result<(), TrapCode> {
        let delta: u32 = delta.into();
        self.execute_memory_grow_impl(result, delta, resource_limiter)
    }

    /// Executes a generic `memory.grow` instruction.
    fn execute_memory_grow_impl(
        &mut self,
        result: Register,
        delta: u32,
        resource_limiter: &mut ResourceLimiterRef<'ctx>,
    ) -> Result<(), TrapCode> {
        if delta == 0 {
            // Case: growing by 0 pages means there is nothing to do
            self.execute_memory_size(result);
            return Ok(());
        }
        let delta = match Pages::new(delta) {
            Some(pages) => pages,
            None => {
                // Cannot grow memory so we push the expected error value.
                self.set_register(result, EntityGrowError::ERROR_CODE);
                return self.try_next_instr();
            }
        };
        let return_value = self.consume_fuel_with(
            |costs| {
                let delta_in_bytes = delta.to_bytes().unwrap_or(0) as u64;
                costs.fuel_for_bytes(delta_in_bytes)
            },
            |this| {
                let memory = this.cache.default_memory(this.ctx);
                let new_pages = this
                    .ctx
                    .resolve_memory_mut(memory)
                    .grow(delta, resource_limiter)
                    .map(u32::from)?;
                // The `memory.grow` operation might have invalidated the cached
                // linear memory so we need to reset it in order for the cache to
                // reload in case it is used again.
                this.cache.reset_default_memory_bytes();
                Ok(new_pages)
            },
        );
        let return_value = match return_value {
            Ok(return_value) => return_value,
            Err(EntityGrowError::InvalidGrow) => EntityGrowError::ERROR_CODE,
            Err(EntityGrowError::TrapCode(trap_code)) => return Err(trap_code),
        };
        self.set_register(result, return_value);
        self.try_next_instr()
    }
}
