use crate::{core::Pages, store::StoreInner, Store};

use super::Executor;
use crate::{
    core::TrapCode,
    engine::{
        bytecode::{Const16, DataSegmentIdx, Instruction, Register},
        code_map::InstructionPtr,
    },
    error::EntityGrowError,
    store::ResourceLimiterRef,
    Error,
};

impl<'engine> Executor<'engine> {
    /// Returns the [`Instruction::DataSegmentIdx`] parameter for an [`Instruction`].
    fn fetch_data_segment_index(&self, offset: usize) -> DataSegmentIdx {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::DataSegmentIdx(segment_index) => segment_index,
            _ => unreachable!("expected an Instruction::DataSegmentIdx instruction word"),
        }
    }

    /// Executes an [`Instruction::DataDrop`].
    #[inline(always)]
    pub fn execute_data_drop(&mut self, store: &mut StoreInner, segment_index: DataSegmentIdx) {
        let segment = self.get_data_segment(store, segment_index);
        store.resolve_data_segment_mut(&segment).drop_bytes();
        self.next_instr();
    }

    /// Executes an [`Instruction::MemorySize`].
    #[inline(always)]
    pub fn execute_memory_size(&mut self, store: &StoreInner, result: Register) {
        let memory = self.get_default_memory(store);
        let size: u32 = store.resolve_memory(&memory).current_pages().into();
        self.set_register(result, size);
        self.next_instr()
    }

    /// Executes an [`Instruction::MemoryGrow`].
    #[inline(always)]
    pub fn execute_memory_grow<T>(
        &mut self,
        store: &mut Store<T>,
        result: Register,
        delta: Register,
    ) -> Result<(), Error> {
        let delta: u32 = self.get_register_as(delta);
        let (store, mut resource_limiter) = store.store_inner_and_resource_limiter_ref();
        self.execute_memory_grow_impl(store, result, delta, &mut resource_limiter)
    }

    /// Executes an [`Instruction::MemoryGrowBy`].
    #[inline(always)]
    pub fn execute_memory_grow_by<T>(
        &mut self,
        store: &mut Store<T>,
        result: Register,
        delta: Const16<u32>,
    ) -> Result<(), Error> {
        let delta: u32 = delta.into();
        let (store, mut resource_limiter) = store.store_inner_and_resource_limiter_ref();
        self.execute_memory_grow_impl(store, result, delta, &mut resource_limiter)
    }

    /// Executes a generic `memory.grow` instruction.
    fn execute_memory_grow_impl<'store>(
        &mut self,
        store: &'store mut StoreInner,
        result: Register,
        delta: u32,
        resource_limiter: &mut ResourceLimiterRef<'store>,
    ) -> Result<(), Error> {
        if delta == 0 {
            // Case: growing by 0 pages means there is nothing to do
            self.execute_memory_size(store, result);
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
        let memory = self.get_default_memory(store);
        let (memory, fuel) = store.resolve_memory_and_fuel_mut(&memory);
        let return_value = memory
            .grow(delta, Some(fuel), resource_limiter)
            .map(u32::from);
        let return_value = match return_value {
            Ok(return_value) => {
                // The `memory.grow` operation might have invalidated the cached
                // linear memory so we need to reset it in order for the cache to
                // reload in case it is used again.
                let instance = Self::instance(&self.stack.calls);
                self.memory.update(store, instance);
                return_value
            }
            Err(EntityGrowError::InvalidGrow) => EntityGrowError::ERROR_CODE,
            Err(EntityGrowError::TrapCode(trap_code)) => return Err(Error::from(trap_code)),
        };
        self.set_register(result, return_value);
        self.try_next_instr()
    }

    /// Executes an [`Instruction::MemoryCopy`].
    #[inline(always)]
    pub fn execute_memory_copy(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        src: Register,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryCopyTo`].
    #[inline(always)]
    pub fn execute_memory_copy_to(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Register,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryCopyFrom`].
    #[inline(always)]
    pub fn execute_memory_copy_from(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        src: Const16<u32>,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryCopyFromTo`].
    #[inline(always)]
    pub fn execute_memory_copy_from_to(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryCopyExact`].
    #[inline(always)]
    pub fn execute_memory_copy_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        src: Register,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryCopyToExact`].
    #[inline(always)]
    pub fn execute_memory_copy_to_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Register,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryCopyFromExact`].
    #[inline(always)]
    pub fn execute_memory_copy_from_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryCopyFromToExact`].
    #[inline(always)]
    pub fn execute_memory_copy_from_to_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes a generic `memory.copy` instruction.
    #[inline(never)]
    fn execute_memory_copy_impl(
        &mut self,
        store: &mut StoreInner,
        dst_index: u32,
        src_index: u32,
        len: u32,
    ) -> Result<(), Error> {
        let src_index = src_index as usize;
        let dst_index = dst_index as usize;
        // Safety: The Wasmi executor keep track of the current Wasm instance
        //         being used and properly updates the cached linear memory
        //         whenever needed.
        let memory = unsafe { self.memory.data_mut() };
        // These accesses just perform the bounds checks required by the Wasm spec.
        memory
            .get(src_index..)
            .and_then(|memory| memory.get(..len as usize))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        memory
            .get(dst_index..)
            .and_then(|memory| memory.get(..len as usize))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        store
            .fuel_mut()
            .consume_fuel_if(|costs| costs.fuel_for_bytes(u64::from(len)))?;
        memory.copy_within(src_index..src_index.wrapping_add(len as usize), dst_index);
        self.try_next_instr()
    }

    /// Executes an [`Instruction::MemoryFill`].
    #[inline(always)]
    pub fn execute_memory_fill(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        value: Register,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let value: u8 = self.get_register_as(value);
        let len: u32 = self.get_register_as(len);
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillAt`].
    #[inline(always)]
    pub fn execute_memory_fill_at(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        value: Register,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let value: u8 = self.get_register_as(value);
        let len: u32 = self.get_register_as(len);
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillImm`].
    #[inline(always)]
    pub fn execute_memory_fill_imm(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        value: u8,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let len: u32 = self.get_register_as(len);
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillAtImm`].
    #[inline(always)]
    pub fn execute_memory_fill_at_imm(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        value: u8,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let len: u32 = self.get_register_as(len);
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillExact`].
    #[inline(always)]
    pub fn execute_memory_fill_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        value: Register,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let value: u8 = self.get_register_as(value);
        let len: u32 = len.into();
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillAtExact`].
    #[inline(always)]
    pub fn execute_memory_fill_at_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        value: Register,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let value: u8 = self.get_register_as(value);
        let len: u32 = len.into();
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillImmExact`].
    #[inline(always)]
    pub fn execute_memory_fill_imm_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        value: u8,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let len: u32 = len.into();
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillAtImmExact`].
    #[inline(always)]
    pub fn execute_memory_fill_at_imm_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        value: u8,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let len: u32 = len.into();
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes a generic `memory.fill` instruction.
    #[inline(never)]
    fn execute_memory_fill_impl(
        &mut self,
        store: &mut StoreInner,
        dst: u32,
        value: u8,
        len: u32,
    ) -> Result<(), Error> {
        let dst = dst as usize;
        let len = len as usize;
        // Safety: The Wasmi executor keep track of the current Wasm instance
        //         being used and properly updates the cached linear memory
        //         whenever needed.
        let memory = unsafe { self.memory.data_mut() };
        let slice = memory
            .get_mut(dst..)
            .and_then(|memory| memory.get_mut(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        store
            .fuel_mut()
            .consume_fuel_if(|costs| costs.fuel_for_bytes(len as u64))?;
        slice.fill(value);
        self.try_next_instr()
    }

    /// Executes an [`Instruction::MemoryInit`].
    #[inline(always)]
    pub fn execute_memory_init(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        src: Register,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryInitTo`].
    #[inline(always)]
    pub fn execute_memory_init_to(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Register,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryInitFrom`].
    #[inline(always)]
    pub fn execute_memory_init_from(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        src: Const16<u32>,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryInitFromTo`].
    #[inline(always)]
    pub fn execute_memory_init_from_to(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Register,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryInitExact`].
    #[inline(always)]
    pub fn execute_memory_init_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        src: Register,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryInitToExact`].
    #[inline(always)]
    pub fn execute_memory_init_to_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Register,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryInitFromExact`].
    #[inline(always)]
    pub fn execute_memory_init_from_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Register,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryInitFromToExact`].
    #[inline(always)]
    pub fn execute_memory_init_from_to_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes a generic `memory.init` instruction.
    #[inline(never)]
    fn execute_memory_init_impl(
        &mut self,
        store: &mut StoreInner,
        dst: u32,
        src: u32,
        len: u32,
    ) -> Result<(), Error> {
        let dst_index = dst as usize;
        let src_index = src as usize;
        let len = len as usize;
        let data_index: DataSegmentIdx = self.fetch_data_segment_index(1);
        let (data, fuel) =
            store.resolve_data_and_fuel_mut(&self.get_data_segment(store, data_index));
        // Safety: The Wasmi executor keep track of the current Wasm instance
        //         being used and properly updates the cached linear memory
        //         whenever needed.
        let memory = unsafe { self.memory.data_mut() };
        let memory = memory
            .get_mut(dst_index..)
            .and_then(|memory| memory.get_mut(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        let data = data
            .bytes()
            .get(src_index..)
            .and_then(|data| data.get(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        fuel.consume_fuel_if(|costs| costs.fuel_for_bytes(len as u64))?;
        memory.copy_from_slice(data);
        self.try_next_instr_at(2)
    }
}
