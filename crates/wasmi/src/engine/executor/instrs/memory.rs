use super::{Executor, InstructionPtr};
use crate::{
    core::{MemoryError, ResourceLimiterRef, TrapCode},
    engine::{utils::unreachable_unchecked, ResumableOutOfFuelError},
    ir::{
        index::{Data, Memory},
        Const16,
        Const32,
        Instruction,
        Reg,
    },
    store::{PrunedStore, StoreInner},
    Error,
};

impl Executor<'_> {
    /// Returns the [`Instruction::MemoryIndex`] parameter for an [`Instruction`].
    fn fetch_memory_index(&self, offset: usize) -> Memory {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::MemoryIndex { index } => index,
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::MemoryIndex`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::MemoryIndex` but found: {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Returns the [`Instruction::DataIndex`] parameter for an [`Instruction`].
    fn fetch_data_segment_index(&self, offset: usize) -> Data {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::DataIndex { index } => index,
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::DataIndex`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::DataIndex` but found: {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Executes an [`Instruction::DataDrop`].
    pub fn execute_data_drop(&mut self, store: &mut StoreInner, segment_index: Data) {
        let segment = self.get_data_segment(segment_index);
        store.resolve_data_segment_mut(&segment).drop_bytes();
        self.next_instr();
    }

    /// Executes an [`Instruction::MemorySize`].
    pub fn execute_memory_size(&mut self, store: &StoreInner, result: Reg, memory: Memory) {
        self.execute_memory_size_impl(store, result, memory);
        self.next_instr()
    }

    /// Underlying implementation of [`Instruction::MemorySize`].
    fn execute_memory_size_impl(&mut self, store: &StoreInner, result: Reg, memory: Memory) {
        let memory = self.get_memory(memory);
        let size = store.resolve_memory(&memory).size();
        self.set_register(result, size);
    }

    /// Executes an [`Instruction::MemoryGrow`].
    pub fn execute_memory_grow(
        &mut self,
        store: &mut PrunedStore,
        result: Reg,
        delta: Reg,
    ) -> Result<(), Error> {
        let delta: u64 = self.get_register_as(delta);
        let (store, mut resource_limiter) = store.store_inner_and_resource_limiter_ref();
        self.execute_memory_grow_impl(store, result, delta, &mut resource_limiter)
    }

    /// Executes an [`Instruction::MemoryGrowImm`].
    pub fn execute_memory_grow_imm(
        &mut self,
        store: &mut PrunedStore,
        result: Reg,
        delta: Const32<u64>,
    ) -> Result<(), Error> {
        let (store, mut resource_limiter) = store.store_inner_and_resource_limiter_ref();
        let delta = u64::from(delta);
        self.execute_memory_grow_impl(store, result, delta, &mut resource_limiter)
    }

    /// Executes a generic `memory.grow` instruction.
    #[inline(never)]
    fn execute_memory_grow_impl<'store>(
        &mut self,
        store: &'store mut StoreInner,
        result: Reg,
        delta: u64,
        resource_limiter: &mut ResourceLimiterRef<'store>,
    ) -> Result<(), Error> {
        let memory = self.fetch_memory_index(1);
        if delta == 0 {
            // Case: growing by 0 pages means there is nothing to do
            self.execute_memory_size_impl(store, result, memory);
            return self.try_next_instr_at(2);
        }
        let memory = self.get_memory(memory);
        let (memory, fuel) = store.resolve_memory_and_fuel_mut(&memory);
        let return_value = memory.grow(delta, Some(fuel), resource_limiter);
        let return_value = match return_value {
            Ok(return_value) => {
                // The `memory.grow` operation might have invalidated the cached
                // linear memory so we need to reset it in order for the cache to
                // reload in case it is used again.
                //
                // Safety: the instance has not changed thus calling this is valid.
                unsafe { self.cache.update_memory(store) };
                return_value
            }
            Err(MemoryError::OutOfBoundsGrowth | MemoryError::OutOfSystemMemory) => {
                match memory.ty().is_64() {
                    true => u64::MAX,
                    false => u64::from(u32::MAX),
                }
            }
            Err(MemoryError::OutOfFuel { required_fuel }) => {
                return Err(Error::from(ResumableOutOfFuelError::new(required_fuel)))
            }
            Err(MemoryError::ResourceLimiterDeniedAllocation) => {
                return Err(Error::from(TrapCode::GrowthOperationLimited))
            }
            Err(error) => panic!("encountered an unexpected error: {error}"),
        };
        self.set_register(result, return_value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::MemoryCopy`].
    pub fn execute_memory_copy(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let src: u64 = self.get_register_as(src);
        let len: u64 = self.get_register_as(len);
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryCopyImm`].
    pub fn execute_memory_copy_imm(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Const16<u64>,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let src: u64 = self.get_register_as(src);
        let len: u64 = len.into();
        self.execute_memory_copy_impl(store, dst, src, len)
    }

    /// Executes a generic `memory.copy` instruction.
    #[inline(never)]
    fn execute_memory_copy_impl(
        &mut self,
        store: &mut StoreInner,
        dst_index: u64,
        src_index: u64,
        len: u64,
    ) -> Result<(), Error> {
        let Ok(dst_index) = usize::try_from(dst_index) else {
            return Err(Error::from(TrapCode::MemoryOutOfBounds));
        };
        let Ok(src_index) = usize::try_from(src_index) else {
            return Err(Error::from(TrapCode::MemoryOutOfBounds));
        };
        let Ok(len) = usize::try_from(len) else {
            return Err(Error::from(TrapCode::MemoryOutOfBounds));
        };
        let dst_memory = self.fetch_memory_index(1);
        let src_memory = self.fetch_memory_index(2);
        if src_memory == dst_memory {
            return self
                .execute_memory_copy_within_impl(store, src_memory, dst_index, src_index, len);
        }
        let (src_memory, dst_memory, fuel) = store.resolve_memory_pair_and_fuel(
            &self.get_memory(src_memory),
            &self.get_memory(dst_memory),
        );
        // These accesses just perform the bounds checks required by the Wasm spec.
        let src_bytes = src_memory
            .data()
            .get(src_index..)
            .and_then(|memory| memory.get(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        let dst_bytes = dst_memory
            .data_mut()
            .get_mut(dst_index..)
            .and_then(|memory| memory.get_mut(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        fuel.consume_fuel_if(|costs| costs.fuel_for_copying_bytes(len as u64))?;
        dst_bytes.copy_from_slice(src_bytes);
        self.try_next_instr_at(3)
    }

    /// Executes a generic `memory.copy` instruction.
    fn execute_memory_copy_within_impl(
        &mut self,
        store: &mut StoreInner,
        memory: Memory,
        dst_index: usize,
        src_index: usize,
        len: usize,
    ) -> Result<(), Error> {
        let memory = self.get_memory(memory);
        let (memory, fuel) = store.resolve_memory_and_fuel_mut(&memory);
        let bytes = memory.data_mut();
        // These accesses just perform the bounds checks required by the Wasm spec.
        bytes
            .get(src_index..)
            .and_then(|memory| memory.get(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        bytes
            .get(dst_index..)
            .and_then(|memory| memory.get(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        fuel.consume_fuel_if(|costs| costs.fuel_for_copying_bytes(len as u64))?;
        bytes.copy_within(src_index..src_index.wrapping_add(len), dst_index);
        self.try_next_instr_at(3)
    }

    /// Executes an [`Instruction::MemoryFill`].
    pub fn execute_memory_fill(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        value: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let value: u8 = self.get_register_as(value);
        let len: u64 = self.get_register_as(len);
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillImm`].
    pub fn execute_memory_fill_imm(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        value: u8,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let len: u64 = self.get_register_as(len);
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillExact`].
    pub fn execute_memory_fill_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        value: Reg,
        len: Const16<u64>,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let value: u8 = self.get_register_as(value);
        let len: u64 = len.into();
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes an [`Instruction::MemoryFillImmExact`].
    pub fn execute_memory_fill_imm_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        value: u8,
        len: Const16<u64>,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let len: u64 = len.into();
        self.execute_memory_fill_impl(store, dst, value, len)
    }

    /// Executes a generic `memory.fill` instruction.
    #[inline(never)]
    fn execute_memory_fill_impl(
        &mut self,
        store: &mut StoreInner,
        dst: u64,
        value: u8,
        len: u64,
    ) -> Result<(), Error> {
        let Ok(dst) = usize::try_from(dst) else {
            return Err(Error::from(TrapCode::MemoryOutOfBounds));
        };
        let Ok(len) = usize::try_from(len) else {
            return Err(Error::from(TrapCode::MemoryOutOfBounds));
        };
        let memory = self.fetch_memory_index(1);
        let memory = self.get_memory(memory);
        let (memory, fuel) = store.resolve_memory_and_fuel_mut(&memory);
        let slice = memory
            .data_mut()
            .get_mut(dst..)
            .and_then(|memory| memory.get_mut(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        fuel.consume_fuel_if(|costs| costs.fuel_for_copying_bytes(len as u64))?;
        slice.fill(value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::MemoryInit`].
    pub fn execute_memory_init(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::MemoryInitImm`].
    pub fn execute_memory_init_imm(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_memory_init_impl(store, dst, src, len)
    }

    /// Executes a generic `memory.init` instruction.
    #[inline(never)]
    fn execute_memory_init_impl(
        &mut self,
        store: &mut StoreInner,
        dst: u64,
        src: u32,
        len: u32,
    ) -> Result<(), Error> {
        let Ok(dst_index) = usize::try_from(dst) else {
            return Err(Error::from(TrapCode::MemoryOutOfBounds));
        };
        let Ok(src_index) = usize::try_from(src) else {
            return Err(Error::from(TrapCode::MemoryOutOfBounds));
        };
        let Ok(len) = usize::try_from(len) else {
            return Err(Error::from(TrapCode::MemoryOutOfBounds));
        };
        let memory_index: Memory = self.fetch_memory_index(1);
        let data_index: Data = self.fetch_data_segment_index(2);
        let (memory, data, fuel) = store.resolve_memory_init_params(
            &self.get_memory(memory_index),
            &self.get_data_segment(data_index),
        );
        let memory = memory
            .data_mut()
            .get_mut(dst_index..)
            .and_then(|memory| memory.get_mut(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        let data = data
            .bytes()
            .get(src_index..)
            .and_then(|data| data.get(..len))
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        fuel.consume_fuel_if(|costs| costs.fuel_for_copying_bytes(len as u64))?;
        memory.copy_from_slice(data);
        self.try_next_instr_at(3)
    }
}
