use crate::{
    engine::executor::handler::{
        exec,
        state::{self, Freg32, Freg64, Inst, Ip, Ireg, Mem0Len, Mem0Ptr, Sp, VmState},
        utils::{self, GetValue, SetValue, get_value, set_value},
    },
    ir,
    ir::index,
};

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

    /// Decodes and returns a value of type `T` using `self`.
    #[inline]
    pub unsafe fn decode_op<T: ir::Decode>(&mut self) -> T {
        let (new_ip, op) = unsafe { exec::decode_op::<T>(self.ip) };
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

    /// Returns the bytes of the `memory`.
    #[inline]
    pub fn fetch_memory<'a>(&self, state: &'a mut VmState, memory: index::Memory) -> &'a mut [u8] {
        if memory.is_default() {
            return state::mem0_bytes::<'a>(self.mem0_ptr, self.mem0_len);
        }
        let memory = utils::fetch_memory(self.instance, memory);
        utils::resolve_memory_mut(state.store, &memory).data_mut()
    }

    /// Returns the bytes of the default memory at index 0.
    #[inline]
    pub fn fetch_default_memory<'a>(&self) -> &'a mut [u8] {
        state::mem0_bytes::<'a>(self.mem0_ptr, self.mem0_len)
    }
}
