use super::Executor;
use crate::{
    core::TrapCode,
    engine::{
        bytecode::{ElementSegmentIdx, TableIdx},
        bytecode2::{Const16, Const32, Instruction, Register},
        code_map::InstructionPtr2 as InstructionPtr,
    },
    table::TableEntity,
};

impl<'engine, 'ctx> Executor<'engine, 'ctx> {
    /// Returns the [`Instruction::TableIdx`] parameter for an [`Instruction`].
    fn fetch_table_index(&self, offset: usize) -> TableIdx {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::TableIdx(table_index) => table_index,
            _ => unreachable!("expected an Instruction::TableIdx instruction word"),
        }
    }

    /// Returns the [`Instruction::ElementSegmentIdx`] parameter for an [`Instruction`].
    fn fetch_element_segment_index(&self, offset: usize) -> ElementSegmentIdx {
        let mut addr: InstructionPtr = self.ip;
        addr.add(offset);
        match *addr.get() {
            Instruction::ElementSegmentIdx(segment_index) => segment_index,
            _ => unreachable!("expected an Instruction::ElementSegmentIdx instruction word"),
        }
    }

    /// Executes an [`Instruction::TableGet`].
    #[inline(always)]
    pub fn execute_table_get(&mut self, result: Register, index: Register) -> Result<(), TrapCode> {
        let index: u32 = self.get_register_as(index);
        self.execute_table_get_impl(result, index)
    }

    /// Executes an [`Instruction::TableGetImm`].
    #[inline(always)]
    pub fn execute_table_get_imm(
        &mut self,
        result: Register,
        index: Const32<u32>,
    ) -> Result<(), TrapCode> {
        self.execute_table_get_impl(result, u32::from(index))
    }

    /// Executes a `table.get` instruction generically.
    fn execute_table_get_impl(&mut self, result: Register, index: u32) -> Result<(), TrapCode> {
        let table_index = self.fetch_table_index(1);
        let table = self.cache.get_table(self.ctx, table_index);
        let value = self
            .ctx
            .resolve_table(&table)
            .get_untyped(index)
            .ok_or(TrapCode::TableOutOfBounds)?;
        self.set_register(result, value);
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::TableSize`].
    #[inline(always)]
    pub fn execute_table_size(&mut self, result: Register, table_index: TableIdx) {
        let table = self.cache.get_table(self.ctx, table_index);
        let size = self.ctx.resolve_table(&table).size();
        self.set_register(result, size);
        self.next_instr();
    }

    /// Executes an [`Instruction::TableSet`].
    #[inline(always)]
    pub fn execute_table_set(&mut self, index: Register, value: Register) -> Result<(), TrapCode> {
        let index: u32 = self.get_register_as(index);
        self.execute_table_set_impl(index, value)
    }

    /// Executes an [`Instruction::TableSetAt`].
    #[inline(always)]
    pub fn execute_table_set_at(
        &mut self,
        index: Const32<u32>,
        value: Register,
    ) -> Result<(), TrapCode> {
        let index = u32::from(index);
        self.execute_table_set_impl(index, value)
    }

    /// Executes a generic `table.set` instruction.
    fn execute_table_set_impl(&mut self, index: u32, value: Register) -> Result<(), TrapCode> {
        let table_index = self.fetch_table_index(1);
        let table = self.cache.get_table(self.ctx, table_index);
        let value = self.get_register(value);
        self.ctx
            .resolve_table_mut(&table)
            .set_untyped(index, value)
            .map_err(|_| TrapCode::TableOutOfBounds)?;
        self.try_next_instr_at(2)
    }

    /// Executes an [`Instruction::TableCopy`].
    #[inline(always)]
    pub fn execute_table_copy(
        &mut self,
        dst: Register,
        src: Register,
        len: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_copy_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyTo`].
    #[inline(always)]
    pub fn execute_table_copy_to(
        &mut self,
        dst: Const16<u32>,
        src: Register,
        len: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_copy_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyFrom`].
    #[inline(always)]
    pub fn execute_table_copy_from(
        &mut self,
        dst: Register,
        src: Const16<u32>,
        len: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_copy_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyFromTo`].
    #[inline(always)]
    pub fn execute_table_copy_from_to(
        &mut self,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_copy_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyExact`].
    #[inline(always)]
    pub fn execute_table_copy_exact(
        &mut self,
        dst: Register,
        src: Register,
        len: Const16<u32>,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_copy_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyToExact`].
    #[inline(always)]
    pub fn execute_table_copy_to_exact(
        &mut self,
        dst: Const16<u32>,
        src: Register,
        len: Const16<u32>,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_copy_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyFromExact`].
    #[inline(always)]
    pub fn execute_table_copy_from_exact(
        &mut self,
        dst: Register,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_table_copy_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableCopyFromToExact`].
    #[inline(always)]
    pub fn execute_table_copy_from_to_exact(
        &mut self,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_table_copy_impl(dst, src, len)
    }

    /// Executes a generic `table.copy` instruction.
    fn execute_table_copy_impl(
        &mut self,
        dst_index: u32,
        src_index: u32,
        len: u32,
    ) -> Result<(), TrapCode> {
        if len == 0 {
            // Case: copying no elements means there is nothing to do
            return Ok(());
        }
        let dst_table_index = self.fetch_table_index(1);
        let src_table_index = self.fetch_table_index(2);
        self.consume_fuel_with(
            |costs| costs.fuel_for_elements(u64::from(len)),
            |this| {
                if dst_table_index == src_table_index {
                    // Case: copy within the same table
                    let table = this.cache.get_table(this.ctx, dst_table_index);
                    this.ctx
                        .resolve_table_mut(&table)
                        .copy_within(dst_index, src_index, len)?;
                } else {
                    // Case: copy between two different tables
                    let dst_table = this.cache.get_table(this.ctx, dst_table_index);
                    let src_table = this.cache.get_table(this.ctx, src_table_index);
                    // Copy from one table to another table:
                    let (dst_table, src_table) =
                        this.ctx.resolve_table_pair_mut(&dst_table, &src_table);
                    TableEntity::copy(dst_table, dst_index, src_table, src_index, len)?;
                }
                Ok(())
            },
        )?;
        self.try_next_instr_at(3)
    }

    /// Executes an [`Instruction::TableInit`].
    #[inline(always)]
    pub fn execute_table_init(
        &mut self,
        dst: Register,
        src: Register,
        len: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableInitTo`].
    #[inline(always)]
    pub fn execute_table_init_to(
        &mut self,
        dst: Const16<u32>,
        src: Register,
        len: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableInitFrom`].
    #[inline(always)]
    pub fn execute_table_init_from(
        &mut self,
        dst: Register,
        src: Const16<u32>,
        len: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableInitFromTo`].
    #[inline(always)]
    pub fn execute_table_init_from_to(
        &mut self,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_init_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableInitExact`].
    #[inline(always)]
    pub fn execute_table_init_exact(
        &mut self,
        dst: Register,
        src: Register,
        len: Const16<u32>,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_init_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableInitToExact`].
    #[inline(always)]
    pub fn execute_table_init_to_exact(
        &mut self,
        dst: Const16<u32>,
        src: Register,
        len: Const16<u32>,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let src: u32 = self.get_register_as(src);
        let len: u32 = len.into();
        self.execute_table_init_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableInitFromExact`].
    #[inline(always)]
    pub fn execute_table_init_from_exact(
        &mut self,
        dst: Register,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_table_init_impl(dst, src, len)
    }

    /// Executes an [`Instruction::TableInitFromToExact`].
    #[inline(always)]
    pub fn execute_table_init_from_to_exact(
        &mut self,
        dst: Const16<u32>,
        src: Const16<u32>,
        len: Const16<u32>,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let src: u32 = src.into();
        let len: u32 = len.into();
        self.execute_table_init_impl(dst, src, len)
    }

    /// Executes a generic `table.init` instruction.
    fn execute_table_init_impl(
        &mut self,
        dst_index: u32,
        src_index: u32,
        len: u32,
    ) -> Result<(), TrapCode> {
        if len == 0 {
            // Case: copying no elements means there is nothing to do
            return Ok(());
        }
        let table_index = self.fetch_table_index(1);
        let element_index = self.fetch_element_segment_index(2);
        self.consume_fuel_with(
            |costs| costs.fuel_for_elements(u64::from(len)),
            |this| {
                let (instance, table, element) =
                    this.cache
                        .get_table_and_element_segment(this.ctx, table_index, element_index);
                table.init(dst_index, element, src_index, len, |func_index| {
                    instance
                        .get_func(func_index)
                        .unwrap_or_else(|| panic!("missing function at index {func_index}"))
                })?;
                Ok(())
            },
        )?;
        self.try_next_instr_at(3)
    }

    /// Executes an [`Instruction::TableFill`].
    #[inline(always)]
    pub fn execute_table_fill(
        &mut self,
        dst: Register,
        len: Register,
        value: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let len: u32 = self.get_register_as(len);
        self.execute_table_fill_impl(dst, len, value)
    }

    /// Executes an [`Instruction::TableFillAt`].
    #[inline(always)]
    pub fn execute_table_fill_at(
        &mut self,
        dst: Const16<u32>,
        len: Register,
        value: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let len: u32 = self.get_register_as(len);
        self.execute_table_fill_impl(dst, len, value)
    }

    /// Executes an [`Instruction::TableFillExact`].
    #[inline(always)]
    pub fn execute_table_fill_exact(
        &mut self,
        dst: Register,
        len: Const16<u32>,
        value: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = self.get_register_as(dst);
        let len: u32 = len.into();
        self.execute_table_fill_impl(dst, len, value)
    }

    /// Executes an [`Instruction::TableFillAtExact`].
    #[inline(always)]
    pub fn execute_table_fill_at_exact(
        &mut self,
        dst: Const16<u32>,
        len: Const16<u32>,
        value: Register,
    ) -> Result<(), TrapCode> {
        let dst: u32 = dst.into();
        let len: u32 = len.into();
        self.execute_table_fill_impl(dst, len, value)
    }

    /// Executes a generic `table.fill` instruction.
    fn execute_table_fill_impl(
        &mut self,
        dst: u32,
        len: u32,
        value: Register,
    ) -> Result<(), TrapCode> {
        if len == 0 {
            // Case: copying no elements means there is nothing to do
            return Ok(());
        }
        let table_index = self.fetch_table_index(1);
        self.consume_fuel_with(
            |costs| costs.fuel_for_elements(u64::from(len)),
            |this| {
                let value = this.get_register(value);
                let table = this.cache.get_table(this.ctx, table_index);
                this.ctx
                    .resolve_table_mut(&table)
                    .fill_untyped(dst, value, len)?;
                Ok(())
            },
        )?;
        self.try_next_instr_at(2)
    }
}
