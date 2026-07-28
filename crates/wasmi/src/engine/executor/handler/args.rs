use crate::{
    DataSegmentEntity,
    Func,
    FuncEntity,
    core::{CoreElementSegment, CoreGlobal, CoreMemory, CoreTable},
    engine::{
        code_map::FuncEntry,
        executor::handler::{
            dispatch::{Break, Control},
            state::{
                self,
                DoneReason,
                Freg32,
                Freg64,
                Inst,
                Ip,
                Ireg,
                Mem0Len,
                Mem0Ptr,
                Sp,
                VmState,
            },
            utils::{self, GetValue, IntoControl as _, SetValue, get_value, set_value},
        },
    },
    ir::{self, BoundedSlotSpan, BranchOffset},
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
    pub fn fetch_default_memory_bytes<'a>(&mut self) -> &'a mut [u8] {
        state::mem0_bytes::<'a>(self.mem0_ptr, self.mem0_len)
    }

    /// Returns an exclusive reference to the memory at `index`.
    #[inline]
    pub fn fetch_memory<'a>(
        &mut self,
        state: &'a mut VmState,
        addr: ir::MemoryAddr,
    ) -> &'a mut CoreMemory {
        utils::load_memory(self.instance, state.store.inner_mut(), addr)
    }

    /// Returns an exclusive reference to the global at `index`.
    #[inline]
    pub fn fetch_global<'a>(
        &mut self,
        state: &'a mut VmState,
        addr: ir::GlobalAddr,
    ) -> &'a mut CoreGlobal {
        utils::load_global(self.instance, state.store.inner_mut(), addr)
    }

    /// Returns an exclusive reference to the table at `index`.
    #[inline]
    pub fn fetch_table<'a>(
        &mut self,
        state: &'a mut VmState,
        addr: ir::TableAddr,
    ) -> &'a mut CoreTable {
        utils::load_table(self.instance, state.store.inner_mut(), addr)
    }

    /// Returns an exclusive reference to the element segment at `index`.
    #[inline]
    pub fn fetch_elem<'a>(
        &mut self,
        state: &'a mut VmState,
        addr: ir::ElemAddr,
    ) -> &'a mut CoreElementSegment {
        utils::load_elem(self.instance, state.store.inner_mut(), addr)
    }

    /// Returns an exclusive reference to the data segment at `index`.
    #[inline]
    pub fn fetch_data<'a>(
        &mut self,
        state: &'a mut VmState,
        addr: ir::DataAddr,
    ) -> &'a mut DataSegmentEntity {
        utils::load_data(self.instance, state.store.inner_mut(), addr)
    }

    /// Reloads the data pointer and length of the default memory at index 0 from `state`.
    #[inline]
    pub fn reload_mem0(&mut self, state: &mut VmState) {
        (self.mem0_ptr, self.mem0_len) = utils::extract_mem0(state.store, self.instance);
    }

    /// Calls `func` with `params` on `instance` with `state` using `self`.
    #[inline(always)]
    pub fn call_func_entry(
        &mut self,
        state: &mut VmState,
        func: &FuncEntry,
        params: BoundedSlotSpan,
        instance: Option<Inst>,
    ) -> Control<(), Break> {
        (self.ip, self.sp) = utils::call_func_entry(state, self.ip, params, func, instance)?;
        Control::Continue(())
    }

    /// Tail-calls `func` with `params` on `instance` with `state` using `self`.
    #[inline(always)]
    pub fn return_call_func_entry(
        &mut self,
        state: &mut VmState,
        func: &FuncEntry,
        params: BoundedSlotSpan,
        instance: Option<Inst>,
    ) -> Control<(), Break> {
        (self.ip, self.sp) = utils::return_call_func_entry(state, params, func, instance)?;
        Control::Continue(())
    }

    /// Resolves the [`Func`] at `table[index]` of type `func_type` using `state`.
    #[inline]
    pub fn resolve_indirect_func<Idx>(
        &mut self,
        state: &mut VmState<'_>,
        index: Idx,
        table: ir::TableAddr,
        func_type: ir::FuncType,
    ) -> Control<(Func, NonNull<FuncEntity>), Break>
    where
        Idx: GetValue<u64>,
    {
        utils::resolve_indirect_func(index, table, func_type, state, self).into_control()
    }

    /// Calls `func` with `params` with `state` using `self`.
    #[inline]
    pub fn call_wasm_or_host_func(
        &mut self,
        state: &mut VmState,
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
            state,
            self.ip,
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
        state: &mut VmState,
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
            state,
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
    pub fn pop_frame(&mut self, state: &mut VmState) -> Control<(), Break> {
        let Some((ip, sp, mem0_ptr, mem0_len, instance)) =
            state
                .stack
                .pop_frame(state.store, self.mem0_ptr, self.mem0_len, self.instance)
        else {
            // No more frames on the call stack -> break out of execution!
            done!(state, DoneReason::Return(self.sp))
        };
        self.ip = ip;
        self.sp = sp;
        self.mem0_ptr = mem0_ptr;
        self.mem0_len = mem0_len;
        self.instance = instance;
        Control::Continue(())
    }
}
