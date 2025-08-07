use super::{Executor, InstructionPtr};
use crate::{
    core::CoreTable,
    engine::{utils::unreachable_unchecked, ResumableOutOfFuelError},
    errors::TableError,
    ir::{
        index::{Elem, Table},
        Const16,
        Const32,
        Instruction,
        Reg,
    },
    store::{PrunedStore, StoreInner},
    Error,
    TrapCode,
};

impl Executor<'_> {
    /// Returns the [`Instruction::TableIndex`] parameter for an [`Instruction`].
    fn fetch_table_index(&self, offset: usize) -> Table {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::TableIndex { index } => index,
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::TableIndex`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::TableIndex` but found: {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Returns the [`Instruction::ElemIndex`] parameter for an [`Instruction`].
    fn fetch_element_segment_index(&self, offset: usize) -> Elem {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::ElemIndex { index } => index,
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::ElemIndex`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::ElemIndex` but found: {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Executes an [`Instruction::TableGet`].
    pub fn execute_table_get(
        &mut self,
        store: &StoreInner,
        result: Reg,
        index: Reg,
    ) -> Result<(), Error> {
        let index: u64 = self.get_register_as(index);
        self.execute_table_get_impl(store, result, index)
    }

    /// Executes an [`Instruction::TableGetImm`].
    pub fn execute_table_get_imm(
        &mut self,
        store: &StoreInner,
        result: Reg,
        index: Const32<u64>,
    ) -> Result<(), Error> {
        let index: u64 = index.into();
        self.execute_table_get_impl(store, result, index)
    }

    /// Executes a `table.get` instruction generically.
    fn execute_table_get_impl(
        &mut self,
        store: &StoreInner,
        result: Reg,
        index: u64,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        let table = self.get_table(table_index);
        let value = store
            .resolve_table(&table)
            .get_untyped(index)
            .ok_or(TrapCode::TableOutOfBounds)?;
        self.set_register(result, value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::TableSize`].
    pub fn execute_table_size(&mut self, store: &StoreInner, result: Reg, table_index: Table) {
        self.execute_table_size_impl(store, result, table_index);
        self.next_instr();
    }

    /// Executes a generic `table.size` instruction.
    fn execute_table_size_impl(&mut self, store: &StoreInner, result: Reg, table_index: Table) {
        let table = self.get_table(table_index);
        let size = store.resolve_table(&table).size();
        self.set_register(result, size);
    }

    /// Executes an [`Instruction::TableSet`].
    pub fn execute_table_set(
        &mut self,
        store: &mut StoreInner,
        index: Reg,
        value: Reg,
    ) -> Result<(), Error> {
        let index: u64 = self.get_register_as(index);
        self.execute_table_set_impl(store, index, value)
    }

    /// Executes an [`Instruction::TableSetAt`].
    pub fn execute_table_set_at(
        &mut self,
        store: &mut StoreInner,
        index: Const32<u64>,
        value: Reg,
    ) -> Result<(), Error> {
        let index: u64 = index.into();
        self.execute_table_set_impl(store, index, value)
    }

    /// Executes a generic `table.set` instruction.
    fn execute_table_set_impl(
        &mut self,
        store: &mut StoreInner,
        index: u64,
        value: Reg,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        let table = self.get_table(table_index);
        let value = self.get_register(value);
        store
            .resolve_table_mut(&table)
            .set_untyped(index, value)
            .map_err(|_| TrapCode::TableOutOfBounds)?;
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::TableCopy`].
    pub fn execute_table_copy(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let src: u64 = self.get_register_as(src);
        let len: u64 = self.get_register_as(len);
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyImm`].
    pub fn execute_table_copy_imm(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Const16<u64>,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let src: u64 = self.get_register_as(src);
        let len: u64 = len.into();
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes a generic `table.copy` instruction.
    #[inline(never)]
    fn execute_table_copy_impl(
        &mut self,
        store: &mut StoreInner,
        dst_index: u64,
        src_index: u64,
        len: u64,
    ) -> Result<(), Error> {
        let dst_table_index = self.fetch_table_index(1);
        let src_table_index = self.fetch_table_index(2);
        if dst_table_index == src_table_index {
            // Case: copy within the same table
            let table = self.get_table(dst_table_index);
            let (table, fuel) = store.resolve_table_and_fuel_mut(&table);
            table.copy_within(dst_index, src_index, len, Some(fuel))?;
        } else {
            // Case: copy between two different tables
            let dst_table = self.get_table(dst_table_index);
            let src_table = self.get_table(src_table_index);
            // Copy from one table to another table:
            let (dst_table, src_table, fuel) =
                store.resolve_table_pair_and_fuel(&dst_table, &src_table);
            CoreTable::copy(dst_table, dst_index, src_table, src_index, len, Some(fuel))?;
        }
        self.try_next_instr_at(3)
    }

    /// Executes an [`Instruction::TableInit`].
    pub fn execute_table_init(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableInitImm`].
    pub fn execute_table_init_imm(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes a generic `table.init` instruction.
    #[inline(never)]
    fn execute_table_init_impl(
        &mut self,
        store: &mut StoreInner,
        dst_index: u64,
        src_index: u32,
        len: u32,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        let element_index = self.fetch_element_segment_index(2);
        let (table, element, fuel) = store.resolve_table_init_params(
            &self.get_table(table_index),
            &self.get_element_segment(element_index),
        );
        table.init(element.as_ref(), dst_index, src_index, len, Some(fuel))?;
        self.try_next_instr_at(3)
    }

    /// Executes an [`Instruction::TableFill`].
    pub fn execute_table_fill(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        len: Reg,
        value: Reg,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let len: u64 = self.get_register_as(len);
        self.execute_table_fill_impl(store, dst, len, value)
    }

    /// Executes an [`Instruction::TableFillImm`].
    pub fn execute_table_fill_imm(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        len: Const16<u64>,
        value: Reg,
    ) -> Result<(), Error> {
        let dst: u64 = self.get_register_as(dst);
        let len: u64 = len.into();
        self.execute_table_fill_impl(store, dst, len, value)
    }

    /// Executes a generic `table.fill` instruction.
    #[inline(never)]
    fn execute_table_fill_impl(
        &mut self,
        store: &mut StoreInner,
        dst: u64,
        len: u64,
        value: Reg,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        let value = self.get_register(value);
        let table = self.get_table(table_index);
        let (table, fuel) = store.resolve_table_and_fuel_mut(&table);
        table.fill_untyped(dst, value, len, Some(fuel))?;
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::TableGrow`].
    pub fn execute_table_grow(
        &mut self,
        store: &mut PrunedStore,
        result: Reg,
        delta: Reg,
        value: Reg,
    ) -> Result<(), Error> {
        let delta: u64 = self.get_register_as(delta);
        self.execute_table_grow_impl(store, result, delta, value)
    }

    /// Executes an [`Instruction::TableGrowImm`].
    pub fn execute_table_grow_imm(
        &mut self,
        store: &mut PrunedStore,
        result: Reg,
        delta: Const16<u64>,
        value: Reg,
    ) -> Result<(), Error> {
        let delta: u64 = delta.into();
        self.execute_table_grow_impl(store, result, delta, value)
    }

    /// Executes a generic `table.grow` instruction.
    #[inline(never)]
    fn execute_table_grow_impl(
        &mut self,
        store: &mut PrunedStore,
        result: Reg,
        delta: u64,
        value: Reg,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        if delta == 0 {
            // Case: growing by 0 elements means there is nothing to do
            self.execute_table_size_impl(store.inner(), result, table_index);
            return self.try_next_instr_at(2);
        }
        let table = self.get_table(table_index);
        let value = self.get_register(value);
        let return_value = match store.grow_table(&table, delta, value) {
            Ok(return_value) => return_value,
            Err(TableError::GrowOutOfBounds | TableError::OutOfSystemMemory) => {
                let table_ty = store.inner().resolve_table(&table).ty();
                match table_ty.is_64() {
                    true => u64::MAX,
                    false => u64::from(u32::MAX),
                }
            }
            Err(TableError::OutOfFuel { required_fuel }) => {
                return Err(Error::from(ResumableOutOfFuelError::new(required_fuel)))
            }
            Err(TableError::ResourceLimiterDeniedAllocation) => {
                return Err(Error::from(TrapCode::GrowthOperationLimited))
            }
            Err(error) => panic!("encountered unexpected error: {error}"),
        };
        self.set_register(result, return_value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::ElemDrop`].
    pub fn execute_element_drop(&mut self, store: &mut StoreInner, segment_index: Elem) {
        let segment = self.get_element_segment(segment_index);
        store.resolve_element_segment_mut(&segment).drop_items();
        self.next_instr();
    }
}
