use crate::{
    DataSegmentEntity,
    Func,
    FuncEntity,
    core::{
        CoreElementSegment as ElementSegmentEntity,
        CoreGlobal as GlobalEntity,
        CoreMemory as MemoryEntity,
        CoreTable as TableEntity,
    },
    engine::{
        code_map::FuncEntry,
        executor::handler::{
            dispatch::{Break, Control},
            state::{self, DoneReason, Freg32, Freg64, Inst, Ip, Ireg, Mem0Len, Mem0Ptr, Sp},
            utils::{self, GetValue, IntoControl as _, LoadEntity, SetValue, get_value, set_value},
        },
    },
    ir::{self, BoundedSlotSpan, BranchOffset},
    store::PrunedStore,
};
use core::ptr::NonNull;

/// Utility type to store the arguments of an execution handler and provide a clean API.
#[derive(Debug, Copy, Clone)]
pub struct Args {
    /// The instruction pointer.
    pub ip: Ip,
    /// The stack pointer of the top frame.
    pub sp: Sp,
    /// The pointer to the data of the default memory at index 0.
    pub mem0_ptr: Mem0Ptr,
    /// The number of bytes of the default memory at index 0.
    pub mem0_len: Mem0Len,
    /// A reference to instance related entities.
    pub instance: Inst,
    /// The general purpose (or integer) accumulator register.
    pub ireg: Ireg,
    /// The 32-bit float accumulator register.
    pub freg32: Freg32,
    /// The 64-bit float accumulator register.
    pub freg64: Freg64,
}

impl Args {
    /// Creates a new [`Args`] from its parts.
    #[inline]
    #[expect(clippy::too_many_arguments)]
    pub fn from_parts(
        ip: Ip,
        sp: Sp,
        mem0_ptr: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Self {
        Self {
            ip,
            sp,
            mem0_ptr,
            mem0_len,
            instance,
            ireg,
            freg32,
            freg64,
        }
    }

    /// Consume `self` to return its parts.
    #[inline]
    pub fn into_parts(self) -> (Ip, Sp, Mem0Ptr, Mem0Len, Inst, Ireg, Freg32, Freg64) {
        (
            self.ip,
            self.sp,
            self.mem0_ptr,
            self.mem0_len,
            self.instance,
            self.ireg,
            self.freg32,
            self.freg64,
        )
    }

    /// Decodes and returns an [`Op`] of type `T` using `self`.
    ///
    /// Aligns `self.ip` to [`Op`] bounds if `indirect-dispatch` is disabled.
    ///
    /// [`Op`]: crate::ir::Op
    #[inline]
    pub unsafe fn decode_op<T: ir::Decode>(&mut self) -> T {
        let old_ip = self.ip;
        let op = unsafe { self.decode::<T>() };
        self.ip = self.ip.align_relative_to(old_ip);
        op
    }

    /// Decodes and returns a value of type `T` using `self`.
    #[inline]
    pub unsafe fn decode<T: ir::Decode>(&mut self) -> T {
        let ip = match cfg!(feature = "indirect-dispatch") {
            true => unsafe { self.ip.skip::<ir::OpCode>() },
            false => unsafe { self.ip.skip::<::core::primitive::usize>() },
        };
        let (new_ip, op) = unsafe { ip.decode() };
        self.ip = new_ip;
        op
    }

    /// Returns a value of type `Dst` from `src`.
    #[inline]
    pub fn get<Dst, Src>(&self, src: Src) -> Dst
    where
        Src: GetValue<Dst>,
    {
        get_value(src, self.sp, self.ireg, self.freg32, self.freg64)
    }

    /// Stores `src` of type `Src` in `dst`.
    #[inline]
    pub fn set<Dst, Src>(&mut self, dst: Dst, src: Src)
    where
        Dst: SetValue<Src>,
    {
        (self.ireg, self.freg32, self.freg64) =
            set_value(dst, src, self.sp, self.ireg, self.freg32, self.freg64);
    }

    /// Updates the [`Ip`] of `self` with `new_ip`.
    #[inline]
    pub fn set_ip(&mut self, new_ip: Ip) {
        self.ip = new_ip;
    }

    /// Offsets the [`Ip`] of `self` by `offset`.
    #[inline]
    pub fn offset_ip(&mut self, offset: BranchOffset) {
        self.ip = unsafe { self.ip.offset(i32::from(offset) as isize) };
    }

    /// Returns the bytes of the default memory at index 0.
    #[inline]
    pub fn fetch_default_memory_bytes<'a>(&mut self, _store: &'a mut PrunedStore) -> &'a mut [u8] {
        state::mem0_bytes::<'a>(self.mem0_ptr, self.mem0_len)
    }

    /// Returns an exclusive reference to the memory at `index`.
    #[inline]
    pub fn fetch_memory<'a, Addr>(
        &mut self,
        store: &'a mut PrunedStore,
        addr: Addr,
    ) -> &'a mut MemoryEntity
    where
        Inst: LoadEntity<Addr, Entity = MemoryEntity>,
    {
        // SAFETY: `addr` stems from a Wasmi IR operator and thus addresses a memory entry of
        //         `self.instance` whose cache was warmed at instantiation. The `state` borrow
        //         scopes the returned reference.
        unsafe { self.instance.load_entity_mut(store, addr) }
    }

    /// Returns an exclusive reference to the global at `index`.
    #[inline]
    pub fn fetch_global<'a, Addr>(
        &mut self,
        store: &'a mut PrunedStore,
        addr: Addr,
    ) -> &'a mut GlobalEntity
    where
        Inst: LoadEntity<Addr, Entity = GlobalEntity>,
    {
        // SAFETY: `addr` stems from a Wasmi IR operator and thus addresses a global entry of
        //         `self.instance` whose cache was warmed at instantiation. The `state` borrow
        //         scopes the returned reference.
        unsafe { self.instance.load_entity_mut(store, addr) }
    }

    /// Returns an exclusive reference to the table at `index`.
    #[inline]
    pub fn fetch_table<'a, Addr>(
        &mut self,
        store: &'a mut PrunedStore,
        addr: Addr,
    ) -> &'a mut TableEntity
    where
        Inst: LoadEntity<Addr, Entity = TableEntity>,
    {
        // SAFETY: `addr` stems from a Wasmi IR operator and thus addresses a table entry of
        //         `self.instance` whose cache was warmed at instantiation. The `state` borrow
        //         scopes the returned reference.
        unsafe { self.instance.load_entity_mut(store, addr) }
    }

    /// Returns an exclusive reference to the element segment at `index`.
    #[inline]
    pub fn fetch_elem<'a, Addr>(
        &mut self,
        store: &'a mut PrunedStore,
        addr: Addr,
    ) -> &'a mut ElementSegmentEntity
    where
        Inst: LoadEntity<Addr, Entity = ElementSegmentEntity>,
    {
        // SAFETY: `addr` stems from a Wasmi IR operator and thus addresses an element segment
        //         entry of `self.instance` whose cache was warmed at instantiation. The `state`
        //         borrow scopes the returned reference.
        unsafe { self.instance.load_entity_mut(store, addr) }
    }

    /// Returns an exclusive reference to the data segment at `index`.
    #[inline]
    pub fn fetch_data<'a, Addr>(
        &mut self,
        store: &'a mut PrunedStore,
        addr: Addr,
    ) -> &'a mut DataSegmentEntity
    where
        Inst: LoadEntity<Addr, Entity = DataSegmentEntity>,
    {
        // SAFETY: `addr` stems from a Wasmi IR operator and thus addresses a data segment entry
        //         of `self.instance` whose cache was warmed at instantiation. The `state` borrow
        //         scopes the returned reference.
        unsafe { self.instance.load_entity_mut(store, addr) }
    }

    /// Reloads the data pointer and length of the default memory at index 0 from `state`.
    #[inline]
    pub fn reload_mem0(&mut self) {
        (self.mem0_ptr, self.mem0_len) = utils::extract_mem0(self.instance);
    }

    /// Calls `func` with `params` on `instance` with `state` using `self`.
    #[inline(always)]
    pub fn call_func_entry(
        &mut self,
        store: &mut PrunedStore,
        func: &FuncEntry,
        params: BoundedSlotSpan,
        instance: Option<Inst>,
    ) -> Control<(), Break> {
        (self.ip, self.sp) =
            utils::call_func_entry(store, self.ip, self.sp, params, func, instance)?;
        Control::Continue(())
    }

    /// Tail-calls `func` with `params` on `instance` with `state` using `self`.
    #[inline(always)]
    pub fn return_call_func_entry(
        &mut self,
        store: &mut PrunedStore,
        func: &FuncEntry,
        params: BoundedSlotSpan,
        instance: Option<Inst>,
    ) -> Control<(), Break> {
        (self.ip, self.sp) = utils::return_call_func_entry(store, self.sp, params, func, instance)?;
        Control::Continue(())
    }

    /// Resolves the [`Func`] at `table[index]` of type `func_type` using `state`.
    #[inline]
    pub fn resolve_indirect_func<Idx, Table>(
        &mut self,
        store: &mut PrunedStore,
        index: Idx,
        table: Table,
        func_type: ir::FuncType,
    ) -> Control<(Func, NonNull<FuncEntity>), Break>
    where
        Idx: GetValue<u64>,
        Inst: LoadEntity<Table, Entity = TableEntity>,
    {
        utils::resolve_indirect_func(index, table, func_type, store, self).into_control()
    }

    /// Calls `func` with `params` with `state` using `self`.
    #[inline]
    pub fn call_wasm_or_host_func(
        &mut self,
        store: &mut PrunedStore,
        func: Func,
        func_entity: NonNull<FuncEntity>,
        params: BoundedSlotSpan,
    ) -> Control<(), Break> {
        (
            self.ip,
            self.sp,
            self.mem0_ptr,
            self.mem0_len,
            self.instance,
        ) = utils::call_wasm_or_host(
            store,
            self.ip,
            self.sp,
            func,
            func_entity,
            params,
            self.mem0_ptr,
            self.mem0_len,
            self.instance,
        )?;
        Control::Continue(())
    }

    /// Tail-calls `func` with `params` with `state` using `self`.
    #[inline]
    pub fn return_call_wasm_or_host_func(
        &mut self,
        store: &mut PrunedStore,
        func: Func,
        func_entity: NonNull<FuncEntity>,
        params: BoundedSlotSpan,
    ) -> Control<(), Break> {
        (
            self.ip,
            self.sp,
            self.mem0_ptr,
            self.mem0_len,
            self.instance,
        ) = utils::return_call_wasm_or_host(
            store,
            self.sp,
            func,
            func_entity,
            params,
            self.mem0_ptr,
            self.mem0_len,
            self.instance,
        )?;
        Control::Continue(())
    }

    /// Pops the top-most frame from the call stack.
    #[inline]
    pub fn pop_frame(&mut self, store: &mut PrunedStore) -> Control<(), Break> {
        let Some((ip, sp, mem0_ptr, mem0_len, instance)) =
            store
                .stack_mut()
                .pop_frame(self.mem0_ptr, self.mem0_len, self.instance)
        else {
            // No more frames on the call stack -> break out of execution!
            done!(store, DoneReason::Return(self.sp))
        };
        self.ip = ip;
        self.sp = sp;
        self.mem0_ptr = mem0_ptr;
        self.mem0_len = mem0_len;
        self.instance = instance;
        Control::Continue(())
    }
}
