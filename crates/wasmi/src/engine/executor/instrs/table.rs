use super::{Executor, InstructionPtr};
use crate::{
    core::TrapCode,
    engine::bytecode::{Const16, ElementSegmentIdx, Instruction, Reg, TableIdx},
    error::EntityGrowError,
    store::{ResourceLimiterRef, StoreInner},
    table::TableEntity,
    Error,
    Store,
};

impl<'engine> Executor<'engine> {
    /// Returns the [`Instruction::TableIdx`] parameter for an [`Instruction`].
    fn fetch_table_index(&self, offset: usize) -> TableIdx {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::TableIndex { index } => index,
            unexpected => {
                unreachable!("expected `Instruction::TableIndex` but found: {unexpected:?}")
            }
        }
    }

    /// Returns the [`Instruction::ElementSegmentIdx`] parameter for an [`Instruction`].
    fn fetch_element_segment_index(&self, offset: usize) -> ElementSegmentIdx {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::ElemIndex { index } => index,
            unexpected => {
                unreachable!("expected `Instruction::ElemIndex` but found: {unexpected:?}")
            }
        }
    }

    /// Executes an [`Instruction::TableGet`].
    #[inline(always)]
    pub fn execute_table_get(
        &mut self,
        store: &StoreInner,
        result: Reg,
        index: Reg,
    ) -> Result<(), Error> {
        let index: u32 = self.get_register_as(index);
        self.execute_table_get_impl(store, result, index)
    }

    /// Executes an [`Instruction::TableGetImm`].
    #[inline(always)]
    pub fn execute_table_get_imm(
        &mut self,
        store: &StoreInner,
        result: Reg,
        index: u32,
    ) -> Result<(), Error> {
        self.execute_table_get_impl(store, result, index)
    }

    /// Executes a `table.get` instruction generically.
    fn execute_table_get_impl(
        &mut self,
        store: &StoreInner,
        result: Reg,
        index: u32,
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
    #[inline(always)]
    pub fn execute_table_size(&mut self, store: &StoreInner, result: Reg, table_index: TableIdx) {
        self.execute_table_size_impl(store, result, table_index);
        self.next_instr();
    }

    /// Executes a generic `table.size` instruction.
    fn execute_table_size_impl(&mut self, store: &StoreInner, result: Reg, table_index: TableIdx) {
        let table = self.get_table(table_index);
        let size = store.resolve_table(&table).size();
        self.set_register(result, size);
    }

    /// Executes an [`Instruction::TableSet`].
    #[inline(always)]
    pub fn execute_table_set(
        &mut self,
        store: &mut StoreInner,
        index: Reg,
        value: Reg,
    ) -> Result<(), Error> {
        let index: u32 = self.get_register_as(index);
        self.execute_table_set_impl(store, index, value)
    }

    /// Executes an [`Instruction::TableSetAt`].
    #[inline(always)]
    pub fn execute_table_set_at(
        &mut self,
        store: &mut StoreInner,
        index: u32,
        value: Reg,
    ) -> Result<(), Error> {
        self.execute_table_set_impl(store, index, value)
    }

    /// Executes a generic `table.set` instruction.
    fn execute_table_set_impl(
        &mut self,
        store: &mut StoreInner,
        index: u32,
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
    #[inline(always)]
    pub fn execute_table_copy(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyTo`].
    #[inline(always)]
    pub fn execute_table_copy_to(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyFrom`].
    #[inline(always)]
    pub fn execute_table_copy_from(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Const16<u32>,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyFromTo`].
    #[inline(always)]
    pub fn execute_table_copy_from_to(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyExact`].
    #[inline(always)]
    pub fn execute_table_copy_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyToExact`].
    #[inline(always)]
    pub fn execute_table_copy_to_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Reg,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyFromExact`].
    #[inline(always)]
    pub fn execute_table_copy_from_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyFromToExact`].
    #[inline(always)]
    pub fn execute_table_copy_from_to_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_table_copy_impl(store, dst, src, len)
    }

    /// Executes a generic `table.copy` instruction.
    fn execute_table_copy_impl(
        &mut self,
        store: &mut StoreInner,
        dst_index: u32,
        src_index: u32,
        len: u32,
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
            TableEntity::copy(dst_table, dst_index, src_table, src_index, len, Some(fuel))?;
        }
        self.try_next_instr_at(3)
    }

    /// Executes an [`Instruction::TableInit`].
    #[inline(always)]
    pub fn execute_table_init(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableInitTo`].
    #[inline(always)]
    pub fn execute_table_init_to(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Reg,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableInitFrom`].
    #[inline(always)]
    pub fn execute_table_init_from(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Const16<u32>,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableInitFromTo`].
    #[inline(always)]
    pub fn execute_table_init_from_to(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableInitExact`].
    #[inline(always)]
    pub fn execute_table_init_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Reg,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableInitToExact`].
    #[inline(always)]
    pub fn execute_table_init_to_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Reg,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableInitFromExact`].
    #[inline(always)]
    pub fn execute_table_init_from_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes an [`Instruction::TableInitFromToExact`].
    #[inline(always)]
    pub fn execute_table_init_from_to_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_table_init_impl(store, dst, src, len)
    }

    /// Executes a generic `table.init` instruction.
    fn execute_table_init_impl(
        &mut self,
        store: &mut StoreInner,
        dst_index: u32,
        src_index: u32,
        len: u32,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        let element_index = self.fetch_element_segment_index(2);
        let (instance, table, element, fuel) = store.resolve_table_init_params(
            self.stack.calls.instance_expect(),
            &self.get_table(table_index),
            &self.get_element_segment(element_index),
        );
        table.init(
            dst_index,
            element,
            src_index,
            len,
            Some(fuel),
            |func_index| {
                instance
                    .get_func(func_index)
                    .unwrap_or_else(|| panic!("missing function at index {func_index}"))
            },
        )?;
        self.try_next_instr_at(3)
    }

    /// Executes an [`Instruction::TableFill`].
    #[inline(always)]
    pub fn execute_table_fill(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        len: Reg,
        value: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let len: u32 = self.get_register_as(len);
        self.execute_table_fill_impl(store, dst, len, value)
    }

    /// Executes an [`Instruction::TableFillAt`].
    #[inline(always)]
    pub fn execute_table_fill_at(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        len: Reg,
        value: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_fill_impl(store, dst, len, value)
    }

    /// Executes an [`Instruction::TableFillExact`].
    #[inline(always)]
    pub fn execute_table_fill_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Reg,
        len: Const16<u32>,
        value: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = self.get_register_as(dst);
        let len: u32 = len.into();
        self.execute_table_fill_impl(store, dst, len, value)
    }

    /// Executes an [`Instruction::TableFillAtExact`].
    #[inline(always)]
    pub fn execute_table_fill_at_exact(
        &mut self,
        store: &mut StoreInner,
        dst: Const16<u32>,
        len: Const16<u32>,
        value: Reg,
    ) -> Result<(), Error> {
        let dst: u32 = dst.into();
        let len: u32 = len.into();
        self.execute_table_fill_impl(store, dst, len, value)
    }

    /// Executes a generic `table.fill` instruction.
    fn execute_table_fill_impl(
        &mut self,
        store: &mut StoreInner,
        dst: u32,
        len: u32,
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
    #[inline(always)]
    pub fn execute_table_grow<T>(
        &mut self,
        store: &mut Store<T>,
        result: Reg,
        delta: Reg,
        value: Reg,
    ) -> Result<(), Error> {
        let delta: u32 = self.get_register_as(delta);
        let (store, mut resource_limiter) = store.store_inner_and_resource_limiter_ref();
        self.execute_table_grow_impl(store, result, delta, value, &mut resource_limiter)
    }

    /// Executes an [`Instruction::TableGrowImm`].
    #[inline(always)]
    pub fn execute_table_grow_imm<T>(
        &mut self,
        store: &mut Store<T>,
        result: Reg,
        delta: Const16<u32>,
        value: Reg,
    ) -> Result<(), Error> {
        let delta: u32 = delta.into();
        let (store, mut resource_limiter) = store.store_inner_and_resource_limiter_ref();
        self.execute_table_grow_impl(store, result, delta, value, &mut resource_limiter)
    }

    /// Executes a generic `table.grow` instruction.
    fn execute_table_grow_impl<'store>(
        &mut self,
        store: &'store mut StoreInner,
        result: Reg,
        delta: u32,
        value: Reg,
        resource_limiter: &mut ResourceLimiterRef<'store>,
    ) -> Result<(), Error> {
        let table_index = self.fetch_table_index(1);
        if delta == 0 {
            // Case: growing by 0 elements means there is nothing to do
            self.execute_table_size_impl(store, result, table_index);
            return self.try_next_instr_at(2);
        }
        let table = self.get_table(table_index);
        let value = self.get_register(value);
        let (table, fuel) = store.resolve_table_and_fuel_mut(&table);
        let return_value = table.grow_untyped(delta, value, Some(fuel), resource_limiter);
        let return_value = match return_value {
            Ok(return_value) => return_value,
            Err(EntityGrowError::InvalidGrow) => EntityGrowError::ERROR_CODE,
            Err(EntityGrowError::TrapCode(trap_code)) => return Err(Error::from(trap_code)),
        };
        self.set_register(result, return_value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::ElemDrop`].
    #[inline(always)]
    pub fn execute_element_drop(
        &mut self,
        store: &mut StoreInner,
        segment_index: ElementSegmentIdx,
    ) {
        let segment = self.get_element_segment(segment_index);
        store.resolve_element_segment_mut(&segment).drop_items();
        self.next_instr();
    }
}
