use super::{Executor, InstructionPtr};
use crate::{
    core::CoreTable,
    engine::{utils::unreachable_unchecked, ResumableOutOfFuelError},
    errors::TableError,
    ir::{
        index::{Elem, Table},
        Op,
        Slot,
    },
    store::{PrunedStore, StoreError, StoreInner},
    Error,
    TrapCode,
};

impl Executor<'_> {
    /// Returns the [`Op::TableIndex`] parameter for an [`Op`].
    fn fetch_table_index(&self, offset: usize) -> Table {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Op::TableIndex { index } => index,
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Op::TableIndex`] exists.
                unsafe {
                    unreachable_unchecked!("expected `Op::TableIndex` but found: {unexpected:?}")
                }
            }
        }
    }

    /// Returns the [`Op::ElemIndex`] parameter for an [`Op`].
    fn fetch_element_segment_index(&self, offset: usize) -> Elem {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Op::ElemIndex { index } => index,
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Op::ElemIndex`] exists.
                unsafe {
                    unreachable_unchecked!("expected `Op::ElemIndex` but found: {unexpected:?}")
                }
            }
        }
    }

    /// Executes an [`Op::TableGet`].
    pub fn execute_table_get(
        &mut self,
        store: &StoreInner,
        result: Slot,
        index: Slot,
    ) -> Result<(), Error> {
        let index: u64 = self.get_stack_slot_as(index);
        self.execute_table_get_impl(store, result, index)
    }

    /// Executes an [`Op::TableGetImm`].
    pub fn execute_table_get_imm(
        &mut self,
        store: &StoreInner,
        result: Slot,
        index: Const32<u64>,
    ) -> Result<(), Error> {
        let index: u64 = index.into();
        self.execute_table_get_impl(store, result, index)
    }

    /// Executes a `table.get` instruction generically.
    fn execute_table_get_impl(
        &mut self,
        store: &StoreInner,
        result: Slot,
        index: u64,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        let table = self.get_table(table_index);
        let value = store
            .resolve_table(&table)
            .get_untyped(index)
            .ok_or(TrapCode::TableOutOfBounds)?;
        self.set_stack_slot(result, value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Op::TableSize`].
    pub fn execute_table_size(&mut self, store: &StoreInner, result: Slot, table_index: Table) {
        self.execute_table_size_impl(store, result, table_index);
        self.next_instr();
    }

    /// Executes a generic `table.size` instruction.
    fn execute_table_size_impl(&mut self, store: &StoreInner, result: Slot, table_index: Table) {
        let table = self.get_table(table_index);
        let size = store.resolve_table(&table).size();
        self.set_stack_slot(result, size);
    }

    /// Executes an [`Op::TableSet`].
    pub fn execute_table_set(
        &mut self,
        store: &mut StoreInner,
        index: Slot,
        value: Slot,
    ) -> Result<(), Error> {
        let index: u64 = self.get_stack_slot_as(index);
        self.execute_table_set_impl(store, index, value)
    }

    /// Executes an [`Op::TableSetAt`].
    pub fn execute_table_set_at(
        &mut self,
        store: &mut StoreInner,
        index: Const32<u64>,
        value: Slot,
    ) -> Result<(), Error> {
        let index: u64 = index.into();
        self.execute_table_set_impl(store, index, value)
    }

    /// Executes a generic `table.set` instruction.
    fn execute_table_set_impl(
        &mut self,
        store: &mut StoreInner,
        index: u64,
        value: Slot,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        let table = self.get_table(table_index);
        let value = self.get_stack_slot(value);
        store
            .resolve_table_mut(&table)
            .set_untyped(index, value)
            .map_err(|_| TrapCode::TableOutOfBounds)?;
        self.try_next_instr_at(2)
    }

    /// Executes an [`Op::TableCopy`].
    pub fn execute_table_copy(
        &mut self,
        store: &mut StoreInner,
        dst: Slot,
        src: Slot,
        len: Slot,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_stack_slot_as(dst);
        let src: u64 = self.get_stack_slot_as(src);
        let len: u64 = self.get_stack_slot_as(len);
        let dst_table_index = self.fetch_table_index(1);
        let src_table_index = self.fetch_table_index(2);
        if dst_table_index == src_table_index {
            // Case: copy within the same table
            let table = self.get_table(dst_table_index);
            let (table, fuel) = store.resolve_table_and_fuel_mut(&table);
            table.copy_within(dst, src, len, Some(fuel))?;
        } else {
            // Case: copy between two different tables
            let dst_table = self.get_table(dst_table_index);
            let src_table = self.get_table(src_table_index);
            // Copy from one table to another table:
            let (dst_table, src_table, fuel) =
                store.resolve_table_pair_and_fuel(&dst_table, &src_table);
            CoreTable::copy(dst_table, dst, src_table, src, len, Some(fuel))?;
        }
        self.try_next_instr_at(3)
    }

    /// Executes an [`Op::TableInit`].
    pub fn execute_table_init(
        &mut self,
        store: &mut StoreInner,
        dst: Slot,
        src: Slot,
        len: Slot,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_stack_slot_as(dst);
        let src: u32 = self.get_stack_slot_as(src);
        let len: u32 = self.get_stack_slot_as(len);
        let table_index = self.fetch_table_index(1);
        let element_index = self.fetch_element_segment_index(2);
        let (table, element, fuel) = store.resolve_table_init_params(
            &self.get_table(table_index),
            &self.get_element_segment(element_index),
        );
        table.init(element.as_ref(), dst, src, len, Some(fuel))?;
        self.try_next_instr_at(3)
    }

    /// Executes an [`Op::TableFill`].
    pub fn execute_table_fill(
        &mut self,
        store: &mut StoreInner,
        dst: Slot,
        len: Slot,
        value: Slot,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_stack_slot_as(dst);
        let len: u64 = self.get_stack_slot_as(len);
        let table_index = self.fetch_table_index(1);
        let value = self.get_stack_slot(value);
        let table = self.get_table(table_index);
        let (table, fuel) = store.resolve_table_and_fuel_mut(&table);
        table.fill_untyped(dst, value, len, Some(fuel))?;
        self.try_next_instr_at(2)
    }

    /// Executes an [`Op::TableGrow`].
    pub fn execute_table_grow(
        &mut self,
        store: &mut PrunedStore,
        result: Slot,
        delta: Slot,
        value: Slot,
    ) -> Result<(), Error> {
        let delta: u64 = self.get_stack_slot_as(delta);
        let table_index = self.fetch_table_index(1);
        if delta == 0 {
            // Case: growing by 0 elements means there is nothing to do
            self.execute_table_size_impl(store.inner(), result, table_index);
            return self.try_next_instr_at(2);
        }
        let table = self.get_table(table_index);
        let value = self.get_stack_slot(value);
        let return_value = match store.grow_table(&table, delta, value) {
            Ok(return_value) => return_value,
            Err(StoreError::External(
                TableError::GrowOutOfBounds | TableError::OutOfSystemMemory,
            )) => {
                let table_ty = store.inner().resolve_table(&table).ty();
                match table_ty.is_64() {
                    true => u64::MAX,
                    false => u64::from(u32::MAX),
                }
            }
            Err(StoreError::External(TableError::OutOfFuel { required_fuel })) => {
                return Err(Error::from(ResumableOutOfFuelError::new(required_fuel)))
            }
            Err(StoreError::External(TableError::ResourceLimiterDeniedAllocation)) => {
                return Err(Error::from(TrapCode::GrowthOperationLimited))
            }
            Err(error) => {
                panic!("`memory.grow`: internal interpreter error: {error}")
            }
        };
        self.set_stack_slot(result, return_value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Op::ElemDrop`].
    pub fn execute_element_drop(&mut self, store: &mut StoreInner, segment_index: Elem) {
        let segment = self.get_element_segment(segment_index);
        store.resolve_element_mut(&segment).drop_items();
        self.next_instr();
    }
}
