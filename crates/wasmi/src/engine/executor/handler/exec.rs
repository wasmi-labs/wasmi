#[macro_use]
mod macros;

#[cfg(feature = "simd")]
mod simd;

#[cfg(feature = "simd")]
pub use self::simd::*;

use super::{
    dispatch::Done,
    state::{Freg32, Freg64, Inst, Ip, Ireg, Mem0Len, Mem0Ptr, Sp, VmState},
    utils::{fetch_func, get_value, memory_bytes, offset_ip},
};
#[cfg(feature = "simd")]
use crate::V128;
use crate::{
    TrapCode,
    core::{CoreTable, RawRef, ReadAs, wasm},
    engine::{
        FuncEntry,
        eval,
        executor::handler::{
            Control,
            dispatch::Break,
            state::DoneReason,
            utils::{
                GetValue,
                IntoControl as _,
                call_func_entry,
                call_wasm_or_host,
                exec_copy_span,
                extract_mem0,
                fetch_data,
                fetch_elem,
                fetch_global,
                fetch_memory,
                fetch_table,
                memory_slice,
                memory_slice_mut,
                resolve_data_mut,
                resolve_elem_mut,
                resolve_global,
                resolve_indirect_func,
                resolve_memory,
                resolve_table,
                resolve_table_mut,
                return_call_func_entry,
                return_call_wasm_or_host,
                set_global,
            },
        },
        utils::unreachable_unchecked,
    },
    errors::{FuelError, MemoryError, TableError},
    ir::{self, BoundedSlotSpan, index},
    store::StoreError,
};
use core::{cmp, ptr};

#[inline(always)]
unsafe fn decode_op<Op: ir::Decode>(ip: Ip) -> (Ip, Op) {
    let (new_ip, op) = unsafe { decode_op_no_align(ip) };
    (new_ip.align_relative_to(ip), op)
}

#[inline(always)]
unsafe fn decode_op_no_align<Op: ir::Decode>(ip: Ip) -> (Ip, Op) {
    let ip = match cfg!(feature = "indirect-dispatch") {
        true => unsafe { ip.skip::<ir::OpCode>() },
        false => unsafe { ip.skip::<::core::primitive::usize>() },
    };
    unsafe { ip.decode() }
}

fn identity<T>(value: T) -> T {
    value
}

execution_handler! {
    fn trap(
        _state: &mut VmState,
        ip: Ip,
        _sp: Sp,
        _mem0: Mem0Ptr,
        _mem0_len: Mem0Len,
        _instance: Inst,
        _ireg: Ireg,
        _freg32: Freg32,
        _freg64: Freg64,
    ) -> Done = {
        let (_ip, crate::ir::decode::Trap { trap_code }) = unsafe { decode_op(ip) };
        trap!(trap_code)
    }
}

execution_handler! {
    fn consume_fuel(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (next_ip, crate::ir::decode::ConsumeFuel { fuel }) = unsafe { decode_op(ip) };
        let consumption_result = state
            .store
            .inner_mut()
            .fuel_mut()
            .consume_fuel_unchecked(u64::from(fuel));
        if let Err(FuelError::OutOfFuel { required_fuel }) = consumption_result {
            out_of_fuel!(state, ip, ireg, freg32, freg64, required_fuel)
        }
        dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn branch(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (_new_ip, crate::ir::decode::Branch { offset }) = unsafe { decode_op(ip) };
        let ip = offset_ip(ip, offset);
        dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

macro_rules! global_get_execution_handler {
    (
        $(
            $( #[$attr:meta] )*
            fn $snake_name:ident($camel_name:ident, $ty:ty)
        );* $(;)?
    ) => {
        $(
            $( #[$attr] )*
            execution_handler! {
                fn $snake_name(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (ip, crate::ir::decode::$camel_name { global, result }) = unsafe { decode_op(ip) };
                    let global = fetch_global(instance, global);
                    let global = resolve_global(state.store, &global);
                    let value: $ty = global.get_raw().read_as();
                    set_value!(result, value, sp, ireg, freg32, freg64);
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
global_get_execution_handler! {
    fn global_get_f32_r(GlobalGetF32_R, f32);
    fn global_get_f64_r(GlobalGetF64_R, f64);
    fn global_get_u64_r(GlobalGetU64_R, u64);

    #[cfg(feature = "simd")]
    fn global_get_v128_s(GlobalGetV128_S, V128);
}

macro_rules! global_set_execution_handler {
    (
        $(
            $( #[$attr:meta] )*
            fn $snake_name:ident($camel_name:ident, $ty:ty)
        );* $(;)?
    ) => {
        $(
            $( #[$attr] )*
            execution_handler! {
                fn $snake_name(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (ip, crate::ir::decode::$camel_name { global, value }) = unsafe { decode_op(ip) };
                    let value: $ty = get_value(value, sp, ireg, freg32, freg64);
                    set_global(global, value, state, instance);
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
global_set_execution_handler! {
    fn global_set_u32_i(GlobalSetU32_I, u32);
    fn global_set_u64_r(GlobalSetU64_R, u64);
    fn global_set_u64_s(GlobalSetU64_S, u64);
    fn global_set_u64_i(GlobalSetU64_I, u64);
    fn global_set_f32_r(GlobalSetF32_R, f32);
    fn global_set_f64_r(GlobalSetF64_R, f64);

    #[cfg(feature = "simd")]
    fn global_set_v128_s(GlobalSetV128_S, V128);
}

execution_handler! {
    fn call_internal(
        state: &mut VmState,
        ip: Ip,
        _sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (caller_ip, crate::ir::decode::CallInternal { params, func }) = unsafe { decode_op(ip) };
        // Safety:
        //
        // `func` is the exposed address of a `FuncEntity` in the engine's append-only `CodeMap`.
        //
        //  - the used `with_exposed_provenance` recovers the original provenance.
        //  - its allocation is never moved or freed while this bytecode runs.
        //  - the `FuncEntry` type mutates only guarded by lock-free atomics.
        let func = unsafe { &*ptr::with_exposed_provenance::<FuncEntry>(usize::from(func)) };
        let (callee_ip, callee_sp) = call_func_entry(state, caller_ip, params, func, None)?;
        dispatch!(state, callee_ip, callee_sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn call_imported(
        state: &mut VmState,
        ip: Ip,
        _sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (caller_ip, crate::ir::decode::CallImported { params, func }) = unsafe { decode_op(ip) };
        let func = fetch_func(instance, func);
        let (ip, sp, mem0, mem0_len, instance) =
            call_wasm_or_host(state, caller_ip, func, params, mem0, mem0_len, instance)?;
        dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

macro_rules! call_indirect_execution_handler {
    ( $( fn $snake_name:ident($camel_name:ident) );* $(;)? ) => {
        $(
            execution_handler! {
                fn $snake_name(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (
                        caller_ip,
                        crate::ir::decode::$camel_name {
                            table,
                            func_type,
                            params,
                            index,
                        },
                    ) = unsafe { decode_op(ip) };
                    let func =
                        resolve_indirect_func(
                            index,
                            table,
                            func_type,
                            state,
                            sp,
                            instance,
                            ireg,
                            freg32,
                            freg64,
                        ).into_control()?;
                    let (callee_ip, sp, mem0, mem0_len, instance) =
                        call_wasm_or_host(state, caller_ip, func, params, mem0, mem0_len, instance)?;
                    dispatch!(state, callee_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
call_indirect_execution_handler! {
    fn call_indirect_r(CallIndirect_R);
    fn call_indirect_s(CallIndirect_S);
}

execution_handler! {
    fn return_call_internal(
        state: &mut VmState,
        ip: Ip,
        _sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (_, crate::ir::decode::ReturnCallInternal { params, func }) = unsafe { decode_op(ip) };
        // Safety:
        //
        // `func` is the exposed address of a `FuncEntity` in the engine's append-only `CodeMap`.
        //
        //  - the used `with_exposed_provenance` recovers the original provenance.
        //  - its allocation is never moved or freed while this bytecode runs.
        //  - the `FuncEntry` type mutates only guarded by lock-free atomics.
        let func = unsafe { &*ptr::with_exposed_provenance::<FuncEntry>(usize::from(func)) };
        let (callee_ip, callee_sp) = return_call_func_entry(state, params, func, None)?;
        dispatch!(state, callee_ip, callee_sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn return_call_imported(
        state: &mut VmState,
        ip: Ip,
        _sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (_, crate::ir::decode::ReturnCallImported { params, func }) = unsafe { decode_op(ip) };
        let func = fetch_func(instance, func);
        let (callee_ip, sp, mem0, mem0_len, instance) =
            return_call_wasm_or_host(state, func, params, mem0, mem0_len, instance)?;
        dispatch!(state, callee_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

macro_rules! return_call_indirect_execution_handler {
    ( $( fn $snake_name:ident($camel_name:ident) );* $(;)? ) => {
        $(
            execution_handler! {
                fn $snake_name(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (
                        _,
                        crate::ir::decode::$camel_name {
                            params,
                            index,
                            func_type,
                            table,
                        },
                    ) = unsafe { decode_op(ip) };
                    let func =
                        resolve_indirect_func(
                            index,
                            table,
                            func_type,
                            state,
                            sp,
                            instance,
                            ireg,
                            freg32,
                            freg64,
                        ).into_control()?;
                    let (callee_ip, sp, mem0, mem0_len, instance) =
                        return_call_wasm_or_host(state, func, params, mem0, mem0_len, instance)?;
                    dispatch!(state, callee_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
return_call_indirect_execution_handler! {
    fn return_call_indirect_r(ReturnCallIndirect_R);
    fn return_call_indirect_s(ReturnCallIndirect_S);
}

execution_handler! {
    fn r#return(
        state: &mut VmState,
        _ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let Some((ip, sp, mem0, mem0_len, instance)) =
            state.stack.pop_frame(state.store, mem0, mem0_len, instance)
        else {
            // No more frames on the call stack -> break out of execution!
            done!(state, DoneReason::Return(sp))
        };
        dispatch!(
            state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64
        )
    }
}

execution_handler! {
    fn memory_size(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (ip, crate::ir::decode::MemorySize { memory, result }) = unsafe { decode_op(ip) };
        let memory = fetch_memory(instance, memory);
        let size = resolve_memory(state.store, &memory).size();
        set_value!(result, size, sp, ireg, freg32, freg64);
        dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn memory_grow(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (
            next_ip,
            crate::ir::decode::MemoryGrow {
                memory,
                result,
                delta,
            },
        ) = unsafe { decode_op(ip) };
        let delta: u64 = get_value(delta, sp, ireg, freg32, freg64);
        let memref = fetch_memory(instance, memory);
        let mut mem0 = mem0;
        let mut mem0_len = mem0_len;
        let return_value = match state.store.grow_memory(&memref, delta) {
            Ok(return_value) => {
                // The `memory.grow` operation might have invalidated the cached
                // linear memory so we need to reset it in order for the cache to
                // reload in case it is used again.
                if memory.is_default() {
                    (mem0, mem0_len) = extract_mem0(state.store, instance);
                }
                return_value
            }
            Err(StoreError::External(
                MemoryError::OutOfBoundsGrowth | MemoryError::OutOfSystemMemory,
            )) => {
                let memory_ty = resolve_memory(state.store, &memref).ty();
                match memory_ty.is_64() {
                    true => u64::MAX,
                    false => u64::from(u32::MAX),
                }
            }
            Err(StoreError::External(MemoryError::OutOfFuel { required_fuel })) => {
                out_of_fuel!(state, ip, ireg, freg32, freg64, required_fuel)
            }
            Err(StoreError::External(MemoryError::ResourceLimiterDeniedAllocation)) => {
                trap!(TrapCode::GrowthOperationLimited);
            }
            Err(StoreError::Internal(error)) => unsafe {
                unreachable_unchecked!("internal interpreter error: {error}")
            },
            Err(error) => {
                panic!("`memory.grow`: internal interpreter error: {error}")
            }
        };
        set_value!(result, return_value, sp, ireg, freg32, freg64);
        dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn memory_copy(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (
            next_ip,
            crate::ir::decode::MemoryCopy {
                dst_memory,
                src_memory,
                dst,
                src,
                len,
            },
        ) = unsafe { decode_op(ip) };
        let dst: u64 = get_value(dst, sp, ireg, freg32, freg64);
        let src: u64 = get_value(src, sp, ireg, freg32, freg64);
        let len: u64 = get_value(len, sp, ireg, freg32, freg64);
        let Ok(dst_index) = usize::try_from(dst) else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        let Ok(src_index) = usize::try_from(src) else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        let Ok(len) = usize::try_from(len) else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        if dst_memory == src_memory {
            memory_copy_within(state, ip, instance, ireg, freg32, freg64, dst_memory, dst_index, src_index, len)?;
            dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
        }
        let dst_memory = fetch_memory(instance, dst_memory);
        let src_memory = fetch_memory(instance, src_memory);
        let (src_memory, dst_memory, fuel) = state
            .store
            .inner_mut()
            .resolve_memory_pair_and_fuel(&src_memory, &dst_memory);
        // These accesses just perform the bounds checks required by the Wasm spec.
        let src_bytes = memory_slice(src_memory, src_index, len).into_control()?;
        let dst_bytes = memory_slice_mut(dst_memory, dst_index, len).into_control()?;
        consume_fuel!(
            state,
            ip,
            ireg,
            freg32,
            freg64,
            fuel,
            |costs| costs.fuel_for_copying_values::<u8>(len as u64),
        );
        dst_bytes.copy_from_slice(src_bytes);
        dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

#[expect(clippy::too_many_arguments)]
fn memory_copy_within(
    state: &mut VmState<'_>,
    ip: Ip,
    instance: Inst,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
    dst_memory: index::Memory,
    dst_index: usize,
    src_index: usize,
    len: usize,
) -> Control<(), Break> {
    let memory = fetch_memory(instance, dst_memory);
    let (memory, fuel) = state.store.inner_mut().resolve_memory_and_fuel_mut(&memory);
    // These accesses just perform the bounds checks required by the Wasm spec.
    memory_slice(memory, src_index, len).into_control()?;
    memory_slice(memory, dst_index, len).into_control()?;
    consume_fuel!(state, ip, ireg, freg32, freg64, fuel, |costs| costs
        .fuel_for_copying_values::<u8>(len as u64));
    memory
        .data_mut()
        .copy_within(src_index..src_index.wrapping_add(len), dst_index);
    Control::Continue(())
}

execution_handler! {
    fn memory_fill(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (
            next_ip,
            crate::ir::decode::MemoryFill {
                memory,
                dst,
                len,
                value,
            },
        ) = unsafe { decode_op(ip) };
        let dst: u64 = get_value(dst, sp, ireg, freg32, freg64);
        let len: u64 = get_value(len, sp, ireg, freg32, freg64);
        let value: u8 = get_value(value, sp, ireg, freg32, freg64);
        let Ok(dst) = usize::try_from(dst) else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        let Ok(len) = usize::try_from(len) else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        let memory = fetch_memory(instance, memory);
        let (memory, fuel) = state.store.inner_mut().resolve_memory_and_fuel_mut(&memory);
        let slice = memory_slice_mut(memory, dst, len).into_control()?;
        consume_fuel!(state, ip, ireg, freg32, freg64, fuel, |costs| costs
            .fuel_for_copying_values::<u8>(len as u64));
        slice.fill(value);
        dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn memory_init(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (
            next_ip,
            crate::ir::decode::MemoryInit {
                memory,
                data,
                dst,
                src,
                len,
            },
        ) = unsafe { decode_op(ip) };
        let dst: u64 = get_value(dst, sp, ireg, freg32, freg64);
        let src: u32 = get_value(src, sp, ireg, freg32, freg64);
        let len: u32 = get_value(len, sp, ireg, freg32, freg64);
        let Ok(dst_index) = usize::try_from(dst) else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        let Ok(src_index) = usize::try_from(src) else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        let Ok(len) = usize::try_from(len) else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        let (memory, data, fuel) = state
            .store
            .inner_mut()
            .resolve_memory_init_params(&fetch_memory(instance, memory), &fetch_data(instance, data));
        let memory = memory_slice_mut(memory, dst_index, len).into_control()?;
        let Some(data) = data
            .bytes()
            .get(src_index..)
            .and_then(|data| data.get(..len))
        else {
            trap!(TrapCode::MemoryOutOfBounds)
        };
        consume_fuel!(state, ip, ireg, freg32, freg64, fuel, |costs| costs
            .fuel_for_copying_values::<u8>(len as u64));
        memory.copy_from_slice(data);
        dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn data_drop(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (ip, crate::ir::decode::DataDrop { data }) = unsafe { decode_op(ip) };
        let data = fetch_data(instance, data);
        resolve_data_mut(state.store, &data).drop_bytes();
        dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn table_size(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (ip, crate::ir::decode::TableSize { table, result }) = unsafe { decode_op(ip) };
        let table = fetch_table(instance, table);
        let size = resolve_table(state.store, &table).size();
        set_value!(result, size, sp, ireg, freg32, freg64);
        dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn table_grow(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (
            ip,
            crate::ir::decode::TableGrow {
                table,
                result,
                delta,
                value,
            },
        ) = unsafe { decode_op(ip) };
        let table = fetch_table(instance, table);
        let delta = get_value(delta, sp, ireg, freg32, freg64);
        let value = get_value(value, sp, ireg, freg32, freg64);
        let return_value = match state.store.grow_table(&table, delta, value) {
            Ok(return_value) => return_value,
            Err(StoreError::External(TableError::GrowOutOfBounds | TableError::OutOfSystemMemory)) => {
                let table = resolve_table(state.store, &table);
                match table.ty().is_64() {
                    true => u64::MAX,
                    false => u64::from(u32::MAX),
                }
            }
            Err(StoreError::External(TableError::OutOfFuel { required_fuel })) => {
                done!(state, DoneReason::out_of_fuel(required_fuel));
            }
            Err(StoreError::External(TableError::ResourceLimiterDeniedAllocation)) => {
                trap!(TrapCode::GrowthOperationLimited);
            }
            Err(StoreError::Internal(error)) => unsafe {
                unreachable_unchecked!("internal interpreter error: {error}")
            },
            Err(error) => {
                panic!("`table.grow`: internal interpreter error: {error}")
            }
        };
        set_value!(result, return_value, sp, ireg, freg32, freg64);
        dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn table_copy(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (
            next_ip,
            crate::ir::decode::TableCopy {
                dst_table,
                src_table,
                dst,
                src,
                len,
            },
        ) = unsafe { decode_op(ip) };
        let dst: u64 = get_value(dst, sp, ireg, freg32, freg64);
        let src: u64 = get_value(src, sp, ireg, freg32, freg64);
        let len: u64 = get_value(len, sp, ireg, freg32, freg64);
        if dst_table == src_table {
            // Case: copy within the same table
            let table = fetch_table(instance, dst_table);
            let (table, fuel) = state.store.inner_mut().resolve_table_and_fuel_mut(&table);
            if let Err(error) = table.copy_within(dst, src, len, Some(fuel)) {
                let trap_code = match error {
                    TableError::CopyOutOfBounds => TrapCode::TableOutOfBounds,
                    TableError::OutOfSystemMemory => TrapCode::OutOfSystemMemory,
                    TableError::OutOfFuel { required_fuel } => {
                        out_of_fuel!(state, ip, ireg, freg32, freg64, required_fuel)
                    }
                    _ => panic!("table.copy: unexpected error: {error:?}"),
                };
                trap!(trap_code)
            }
            dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
        }
        // Case: copy between two different tables
        let dst_table = fetch_table(instance, dst_table);
        let src_table = fetch_table(instance, src_table);
        let (dst_table, src_table, fuel) = state
            .store
            .inner_mut()
            .resolve_table_pair_and_fuel(&dst_table, &src_table);
        if let Err(error) = CoreTable::copy(dst_table, dst, src_table, src, len, Some(fuel)) {
            let trap_code = match error {
                TableError::CopyOutOfBounds => TrapCode::TableOutOfBounds,
                TableError::OutOfSystemMemory => TrapCode::OutOfSystemMemory,
                TableError::OutOfFuel { required_fuel } => {
                    out_of_fuel!(state, ip, ireg, freg32, freg64, required_fuel)
                }
                _ => panic!("table.copy: unexpected error: {error:?}"),
            };
            trap!(trap_code)
        }
        dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn table_fill(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (
            next_ip,
            crate::ir::decode::TableFill {
                table,
                dst,
                len,
                value,
            },
        ) = unsafe { decode_op(ip) };
        let dst: u64 = get_value(dst, sp, ireg, freg32, freg64);
        let len: u64 = get_value(len, sp, ireg, freg32, freg64);
        let value: RawRef = get_value(value, sp, ireg, freg32, freg64);
        let table = fetch_table(instance, table);
        let (table, fuel) = state.store.inner_mut().resolve_table_and_fuel_mut(&table);
        if let Err(error) = table.fill_raw(dst, value, len, Some(fuel)) {
            let trap_code = match error {
                TableError::OutOfSystemMemory => TrapCode::OutOfSystemMemory,
                TableError::FillOutOfBounds => TrapCode::TableOutOfBounds,
                TableError::OutOfFuel { required_fuel } => {
                    out_of_fuel!(state, ip, ireg, freg32, freg64, required_fuel)
                }
                _ => panic!("table.fill: unexpected error: {error:?}"),
            };
            trap!(trap_code)
        }
        dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn table_init(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (
            next_ip,
            crate::ir::decode::TableInit {
                table,
                elem,
                dst,
                src,
                len,
            },
        ) = unsafe { decode_op(ip) };
        let dst: u64 = get_value(dst, sp, ireg, freg32, freg64);
        let src: u32 = get_value(src, sp, ireg, freg32, freg64);
        let len: u32 = get_value(len, sp, ireg, freg32, freg64);
        let table = fetch_table(instance, table);
        let elem = fetch_elem(instance, elem);
        let (table, element, fuel) = state
            .store
            .inner_mut()
            .resolve_table_init_params(&table, &elem);
        if let Err(error) = table.init(element.as_ref(), dst, src, len, Some(fuel)) {
            let trap_code = match error {
                TableError::OutOfSystemMemory => TrapCode::OutOfSystemMemory,
                TableError::InitOutOfBounds => TrapCode::TableOutOfBounds,
                TableError::OutOfFuel { required_fuel } => {
                    out_of_fuel!(state, ip, ireg, freg32, freg64, required_fuel)
                }
                _ => panic!("table.init: unexpected error: {error:?}"),
            };
            trap!(trap_code)
        }
        dispatch!(state, next_ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

execution_handler! {
    fn elem_drop(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (ip, crate::ir::decode::ElemDrop { elem }) = unsafe { decode_op(ip) };
        let elem = fetch_elem(instance, elem);
        resolve_elem_mut(state.store, &elem).drop_items();
        dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

macro_rules! impl_table_get {
    ( $( fn $handler:ident($op:ident) = $ext:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (ip, crate::ir::decode::$op { table, result, index }) = unsafe { decode_op(ip) };
                    let table = fetch_table(instance, table);
                    let table = resolve_table(state.store, &table);
                    let index = $ext(get_value(index, sp, ireg, freg32, freg64));
                    let value = match table.get(index) {
                        Some(value) => value.raw(),
                        None => trap!(TrapCode::TableOutOfBounds)
                    };
                    set_value!(result, value, sp, ireg, freg32, freg64);
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
impl_table_get! {
    fn table_get_rr(TableGet_Rr) = identity;
    fn table_get_rs(TableGet_Rs) = identity;
    fn table_get_ri(TableGet_Ri) = u64::from;
}

macro_rules! impl_table_set {
    ( $( fn $handler:ident($op:ident) = $ext:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (ip, crate::ir::decode::$op { table, index, value }) = unsafe { decode_op(ip) };
                    let table = fetch_table(instance, table);
                    let table = resolve_table_mut(state.store, &table);
                    let index = $ext(get_value(index, sp, ireg, freg32, freg64));
                    let value: u32 = get_value(value, sp, ireg, freg32, freg64);
                    if let Err(TableError::SetOutOfBounds) = table.set_raw(index, RawRef::from(value)) {
                        trap!(TrapCode::TableOutOfBounds)
                    };
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
impl_table_set! {
    fn table_set_rs(TableSet_Rs) = identity;
    fn table_set_ri(TableSet_Ri) = identity;
    fn table_set_sr(TableSet_Sr) = identity;
    fn table_set_ss(TableSet_Ss) = identity;
    fn table_set_si(TableSet_Si) = identity;
    fn table_set_ir(TableSet_Ir) = u64::from;
    fn table_set_is(TableSet_Is) = u64::from;
    fn table_set_ii(TableSet_Ii) = u64::from;
}

execution_handler! {
    fn ref_func(
        state: &mut VmState,
        ip: Ip,
        sp: Sp,
        mem0: Mem0Ptr,
        mem0_len: Mem0Len,
        instance: Inst,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> Done = {
        let (ip, crate::ir::decode::RefFunc { func, result }) = unsafe { decode_op(ip) };
        let func = fetch_func(instance, func);
        let Some(rawref) = func.unwrap_raw(&*state.store) else {
            unsafe { unreachable_unchecked!("store mismatch with: {func:?}") }
        };
        set_value!(result, rawref, sp, ireg, freg32, freg64);
        dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
    }
}

macro_rules! impl_i64_binop128 {
    (
        $( fn $handler:ident($op:ident) = $eval:expr );* $(;)?
    ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (ip, crate::ir::decode::$op { results, lhs_lo, lhs_hi, rhs_lo, rhs_hi }) = unsafe { decode_op(ip) };
                    let lhs_lo: i64 = get_value(lhs_lo, sp, ireg, freg32, freg64);
                    let lhs_hi: i64 = get_value(lhs_hi, sp, ireg, freg32, freg64);
                    let rhs_lo: i64 = get_value(rhs_lo, sp, ireg, freg32, freg64);
                    let rhs_hi: i64 = get_value(rhs_hi, sp, ireg, freg32, freg64);
                    let results = results.to_array();
                    let (result_lo, result_hi) = $eval(lhs_lo, lhs_hi, rhs_lo, rhs_hi);
                    set_value!(results[0], result_lo, sp, ireg, freg32, freg64);
                    set_value!(results[1], result_hi, sp, ireg, freg32, freg64);
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
impl_i64_binop128! {
    fn i64_add128(I64Add128) = wasm::i64_add128;
    fn i64_sub128(I64Sub128) = wasm::i64_sub128;
}

macro_rules! impl_i64_mul_wide {
    (
        $( fn $handler:ident($op:ident) = $eval:expr );* $(;)?
    ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (ip, crate::ir::decode::$op { results, lhs, rhs }) = unsafe { decode_op(ip) };
                    let lhs: i64 = get_value(lhs, sp, ireg, freg32, freg64);
                    let rhs: i64 = get_value(rhs, sp, ireg, freg32, freg64);
                    let (result_lo, result_hi) = $eval(lhs, rhs);
                    let results = results.to_array();
                    set_value!(results[0], result_lo, sp, ireg, freg32, freg64);
                    set_value!(results[1], result_hi, sp, ireg, freg32, freg64);
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
impl_i64_mul_wide! {
    fn i64_mul_wide(I64MulWide) = wasm::i64_mul_wide_s;
    fn u64_mul_wide(U64MulWide) = wasm::i64_mul_wide_u;
}

/// Fetches the branch table index value and normalizes it to clamp between `0..len_targets`.
#[inline]
fn fetch_branch_table_target<Index>(
    sp: Sp,
    index: Index,
    len_targets: u32,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
) -> usize
where
    Index: GetValue<u32>,
{
    let index: u32 = get_value(index, sp, ireg, freg32, freg64);
    let max_index = len_targets - 1;
    cmp::min(index, max_index) as usize
}

/// Executes a generic branch table operator that does not any copy values.
#[inline]
#[expect(clippy::too_many_arguments)]
fn exec_branch_table<Index>(
    index: Index,
    len_targets: u32,
    _values: (),
    ip: Ip,
    sp: Sp,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
) -> Ip
where
    Index: GetValue<u32>,
{
    let chosen_target = fetch_branch_table_target(sp, index, len_targets, ireg, freg32, freg64);
    let target_offset = 4 * chosen_target;
    let ip = unsafe { ip.add(target_offset) };
    let (_, offset) = unsafe { ip.decode::<ir::BranchOffset>() };
    offset_ip(ip, offset)
}

/// Executes a generic branch table operator that does not any copy values.
#[inline]
#[expect(clippy::too_many_arguments)]
fn exec_branch_table_with_copies<Index>(
    index: Index,
    len_targets: u32,
    values: BoundedSlotSpan,
    ip: Ip,
    sp: Sp,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
) -> Ip
where
    Index: GetValue<u32>,
{
    let chosen_target = fetch_branch_table_target(sp, index, len_targets, ireg, freg32, freg64);
    let len_encoded_target = match cfg!(feature = "indirect-dispatch") {
        // TODO: add and use `Encode` trait assoc constants
        true => 6,
        false => 8,
    };
    let target_offset = len_encoded_target * chosen_target;
    let ip = unsafe { ip.add(target_offset) };
    let (_, ir::BranchTableTarget { results, offset }) =
        unsafe { ip.decode::<ir::BranchTableTarget>() };
    let values_len = values.len();
    let values = values.span();
    if results != values {
        exec_copy_span(sp, results, values, values_len);
    }
    offset_ip(ip, offset)
}

macro_rules! impl_branch_table_exec_handler {
    (
        $( fn $snake_case:ident($camel_case:ident) = $exec:expr; )*
    ) => {
        $(
            execution_handler! {
                fn $snake_case(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (ip, crate::ir::decode::$camel_case { len_targets, index, values }) = unsafe { decode_op_no_align(ip) };
                    let ip = $exec(index, len_targets, values, ip, sp, ireg, freg32, freg64);
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
impl_branch_table_exec_handler! {
    fn branch_table_s(BranchTable_S) = exec_branch_table;
    fn branch_table_r(BranchTable_R) = exec_branch_table;
    fn branch_table_span_s(BranchTableSpan_S) = exec_branch_table_with_copies;
    fn branch_table_span_r(BranchTableSpan_R) = exec_branch_table_with_copies;
}

handler_unary! {
    // specialized copy_sNsM operators
    fn u64_copy_s0s1(U64Copy_S0s1) = identity::<u64>;
    fn u64_copy_s0s2(U64Copy_S0s2) = identity::<u64>;
    fn u64_copy_s0s3(U64Copy_S0s3) = identity::<u64>;
    fn u64_copy_s0s4(U64Copy_S0s4) = identity::<u64>;
    fn u64_copy_s0s5(U64Copy_S0s5) = identity::<u64>;
    fn u64_copy_s1s0(U64Copy_S1s0) = identity::<u64>;
    fn u64_copy_s1s2(U64Copy_S1s2) = identity::<u64>;
    fn u64_copy_s1s3(U64Copy_S1s3) = identity::<u64>;
    fn u64_copy_s1s4(U64Copy_S1s4) = identity::<u64>;
    fn u64_copy_s1s5(U64Copy_S1s5) = identity::<u64>;
    fn u64_copy_s2s0(U64Copy_S2s0) = identity::<u64>;
    fn u64_copy_s2s1(U64Copy_S2s1) = identity::<u64>;
    fn u64_copy_s2s3(U64Copy_S2s3) = identity::<u64>;
    fn u64_copy_s2s4(U64Copy_S2s4) = identity::<u64>;
    fn u64_copy_s2s5(U64Copy_S2s5) = identity::<u64>;
    fn u64_copy_s3s0(U64Copy_S3s0) = identity::<u64>;
    fn u64_copy_s3s1(U64Copy_S3s1) = identity::<u64>;
    fn u64_copy_s3s2(U64Copy_S3s2) = identity::<u64>;
    fn u64_copy_s3s4(U64Copy_S3s4) = identity::<u64>;
    fn u64_copy_s3s5(U64Copy_S3s5) = identity::<u64>;
    fn u64_copy_s4s0(U64Copy_S4s0) = identity::<u64>;
    fn u64_copy_s4s1(U64Copy_S4s1) = identity::<u64>;
    fn u64_copy_s4s2(U64Copy_S4s2) = identity::<u64>;
    fn u64_copy_s4s3(U64Copy_S4s3) = identity::<u64>;
    fn u64_copy_s4s5(U64Copy_S4s5) = identity::<u64>;
    fn u64_copy_s5s0(U64Copy_S5s0) = identity::<u64>;
    fn u64_copy_s5s1(U64Copy_S5s1) = identity::<u64>;
    fn u64_copy_s5s2(U64Copy_S5s2) = identity::<u64>;
    fn u64_copy_s5s3(U64Copy_S5s3) = identity::<u64>;
    fn u64_copy_s5s4(U64Copy_S5s4) = identity::<u64>;
    // specialized copy_sNr operators
    fn u64_copy_s0r(U64Copy_S0r) = identity::<u64>;
    fn u64_copy_s1r(U64Copy_S1r) = identity::<u64>;
    fn u64_copy_s2r(U64Copy_S2r) = identity::<u64>;
    fn u64_copy_s3r(U64Copy_S3r) = identity::<u64>;
    fn u64_copy_s4r(U64Copy_S4r) = identity::<u64>;
    fn u64_copy_s5r(U64Copy_S5r) = identity::<u64>;
    fn u64_copy_s6r(U64Copy_S6r) = identity::<u64>;
    fn u64_copy_s7r(U64Copy_S7r) = identity::<u64>;
    fn u64_copy_s8r(U64Copy_S8r) = identity::<u64>;
    fn u64_copy_s9r(U64Copy_S9r) = identity::<u64>;
    fn f32_copy_s0r(F32Copy_S0r) = identity::<f32>;
    fn f32_copy_s1r(F32Copy_S1r) = identity::<f32>;
    fn f32_copy_s2r(F32Copy_S2r) = identity::<f32>;
    fn f32_copy_s3r(F32Copy_S3r) = identity::<f32>;
    fn f32_copy_s4r(F32Copy_S4r) = identity::<f32>;
    fn f32_copy_s5r(F32Copy_S5r) = identity::<f32>;
    fn f32_copy_s6r(F32Copy_S6r) = identity::<f32>;
    fn f32_copy_s7r(F32Copy_S7r) = identity::<f32>;
    fn f32_copy_s8r(F32Copy_S8r) = identity::<f32>;
    fn f32_copy_s9r(F32Copy_S9r) = identity::<f32>;
    fn f64_copy_s0r(F64Copy_S0r) = identity::<f64>;
    fn f64_copy_s1r(F64Copy_S1r) = identity::<f64>;
    fn f64_copy_s2r(F64Copy_S2r) = identity::<f64>;
    fn f64_copy_s3r(F64Copy_S3r) = identity::<f64>;
    fn f64_copy_s4r(F64Copy_S4r) = identity::<f64>;
    fn f64_copy_s5r(F64Copy_S5r) = identity::<f64>;
    fn f64_copy_s6r(F64Copy_S6r) = identity::<f64>;
    fn f64_copy_s7r(F64Copy_S7r) = identity::<f64>;
    fn f64_copy_s8r(F64Copy_S8r) = identity::<f64>;
    fn f64_copy_s9r(F64Copy_S9r) = identity::<f64>;
    // copy
    fn u32_copy_ri(U32Copy_Ri) = identity::<u32>;
    fn u32_copy_si(U32Copy_Si) = identity::<u32>;
    fn u64_copy_rs(U64Copy_Rs) = identity::<u64>;
    fn u64_copy_ri(U64Copy_Ri) = identity::<u64>;
    fn u64_copy_sr(U64Copy_Sr) = identity::<u64>;
    fn u64_copy_ss(U64Copy_Ss) = identity::<u64>;
    fn u64_copy_si(U64Copy_Si) = identity::<u64>;
    fn f32_copy_ri(F32Copy_Ri) = identity::<f32>;
    fn f32_copy_rs(F32Copy_Rs) = identity::<f32>;
    fn f32_copy_sr(F32Copy_Sr) = identity::<f32>;
    fn f64_copy_rs(F64Copy_Rs) = identity::<f64>;
    fn f64_copy_ri(F64Copy_Ri) = identity::<f64>;
    fn f64_copy_sr(F64Copy_Sr) = identity::<f64>;
    // reinterpret
    fn f32_reinterpret_i32_rr(F32ReinterpretI32_Rr) = wasm::f32_reinterpret_i32;
    fn i32_reinterpret_f32_rr(I32ReinterpretF32_Rr) = wasm::i32_reinterpret_f32;
    fn f64_reinterpret_i64_rr(F64ReinterpretI64_Rr) = wasm::f64_reinterpret_i64;
    fn i64_reinterpret_f64_rr(I64ReinterpretF64_Rr) = wasm::i64_reinterpret_f64;
    // i32
    fn i32_popcnt_rs(I32Popcnt_Rs) = wasm::i32_popcnt;
    fn i32_popcnt_rr(I32Popcnt_Rr) = wasm::i32_popcnt;
    fn i32_ctz_rs(I32Ctz_Rs) = wasm::i32_ctz;
    fn i32_ctz_rr(I32Ctz_Rr) = wasm::i32_ctz;
    fn i32_clz_rs(I32Clz_Rs) = wasm::i32_clz;
    fn i32_clz_rr(I32Clz_Rr) = wasm::i32_clz;
    fn i32_sext8_rs(I32Sext8_Rs) = wasm::i32_extend8_s;
    fn i32_sext8_rr(I32Sext8_Rr) = wasm::i32_extend8_s;
    fn i32_sext16_rs(I32Sext16_Rs) = wasm::i32_extend16_s;
    fn i32_sext16_rr(I32Sext16_Rr) = wasm::i32_extend16_s;
    fn i32_wrap_i64_rs(I32WrapI64_Rs) = wasm::i32_wrap_i64;
    fn i32_wrap_i64_rr(I32WrapI64_Rr) = wasm::i32_wrap_i64;
    // i64
    fn i64_popcnt_rs(I64Popcnt_Rs) = wasm::i64_popcnt;
    fn i64_popcnt_rr(I64Popcnt_Rr) = wasm::i64_popcnt;
    fn i64_ctz_rs(I64Ctz_Rs) = wasm::i64_ctz;
    fn i64_ctz_rr(I64Ctz_Rr) = wasm::i64_ctz;
    fn i64_clz_rs(I64Clz_Rs) = wasm::i64_clz;
    fn i64_clz_rr(I64Clz_Rr) = wasm::i64_clz;
    fn i64_sext8_rs(I64Sext8_Rs) = wasm::i64_extend8_s;
    fn i64_sext8_rr(I64Sext8_Rr) = wasm::i64_extend8_s;
    fn i64_sext16_rs(I64Sext16_Rs) = wasm::i64_extend16_s;
    fn i64_sext16_rr(I64Sext16_Rr) = wasm::i64_extend16_s;
    fn i64_sext32_rs(I64Sext32_Rs) = wasm::i64_extend32_s;
    fn i64_sext32_rr(I64Sext32_Rr) = wasm::i64_extend32_s;
    // f32
    fn f32_abs_rs(F32Abs_Rs) = wasm::f32_abs;
    fn f32_abs_rr(F32Abs_Rr) = wasm::f32_abs;
    fn f32_neg_rs(F32Neg_Rs) = wasm::f32_neg;
    fn f32_neg_rr(F32Neg_Rr) = wasm::f32_neg;
    fn f32_nabs_rs(F32Nabs_Rs) = eval::wasmi_f32_nabs;
    fn f32_nabs_rr(F32Nabs_Rr) = eval::wasmi_f32_nabs;
    fn f32_ceil_rs(F32Ceil_Rs) = wasm::f32_ceil;
    fn f32_ceil_rr(F32Ceil_Rr) = wasm::f32_ceil;
    fn f32_floor_rs(F32Floor_Rs) = wasm::f32_floor;
    fn f32_floor_rr(F32Floor_Rr) = wasm::f32_floor;
    fn f32_trunc_rs(F32Trunc_Rs) = wasm::f32_trunc;
    fn f32_trunc_rr(F32Trunc_Rr) = wasm::f32_trunc;
    fn f32_nearest_rs(F32Nearest_Rs) = wasm::f32_nearest;
    fn f32_nearest_rr(F32Nearest_Rr) = wasm::f32_nearest;
    fn f32_sqrt_rs(F32Sqrt_Rs) = wasm::f32_sqrt;
    fn f32_sqrt_rr(F32Sqrt_Rr) = wasm::f32_sqrt;
    fn f32_convert_i32_rs(F32ConvertI32_Rs) = wasm::f32_convert_i32_s;
    fn f32_convert_i32_rr(F32ConvertI32_Rr) = wasm::f32_convert_i32_s;
    fn f32_convert_u32_rs(F32ConvertU32_Rs) = wasm::f32_convert_i32_u;
    fn f32_convert_u32_rr(F32ConvertU32_Rr) = wasm::f32_convert_i32_u;
    fn f32_convert_i64_rs(F32ConvertI64_Rs) = wasm::f32_convert_i64_s;
    fn f32_convert_i64_rr(F32ConvertI64_Rr) = wasm::f32_convert_i64_s;
    fn f32_convert_u64_rs(F32ConvertU64_Rs) = wasm::f32_convert_i64_u;
    fn f32_convert_u64_rr(F32ConvertU64_Rr) = wasm::f32_convert_i64_u;
    fn f32_demote_f64_rs(F32DemoteF64_Rs) = wasm::f32_demote_f64;
    fn f32_demote_f64_rr(F32DemoteF64_Rr) = wasm::f32_demote_f64;
    // f64
    fn f64_abs_rs(F64Abs_Rs) = wasm::f64_abs;
    fn f64_abs_rr(F64Abs_Rr) = wasm::f64_abs;
    fn f64_neg_rs(F64Neg_Rs) = wasm::f64_neg;
    fn f64_neg_rr(F64Neg_Rr) = wasm::f64_neg;
    fn f64_nabs_rs(F64Nabs_Rs) = eval::wasmi_f64_nabs;
    fn f64_nabs_rr(F64Nabs_Rr) = eval::wasmi_f64_nabs;
    fn f64_ceil_rs(F64Ceil_Rs) = wasm::f64_ceil;
    fn f64_ceil_rr(F64Ceil_Rr) = wasm::f64_ceil;
    fn f64_floor_rs(F64Floor_Rs) = wasm::f64_floor;
    fn f64_floor_rr(F64Floor_Rr) = wasm::f64_floor;
    fn f64_trunc_rs(F64Trunc_Rs) = wasm::f64_trunc;
    fn f64_trunc_rr(F64Trunc_Rr) = wasm::f64_trunc;
    fn f64_nearest_rs(F64Nearest_Rs) = wasm::f64_nearest;
    fn f64_nearest_rr(F64Nearest_Rr) = wasm::f64_nearest;
    fn f64_sqrt_rs(F64Sqrt_Rs) = wasm::f64_sqrt;
    fn f64_sqrt_rr(F64Sqrt_Rr) = wasm::f64_sqrt;
    fn f64_convert_i32_rs(F64ConvertI32_Rs) = wasm::f64_convert_i32_s;
    fn f64_convert_i32_rr(F64ConvertI32_Rr) = wasm::f64_convert_i32_s;
    fn f64_convert_u32_rs(F64ConvertU32_Rs) = wasm::f64_convert_i32_u;
    fn f64_convert_u32_rr(F64ConvertU32_Rr) = wasm::f64_convert_i32_u;
    fn f64_convert_i64_rs(F64ConvertI64_Rs) = wasm::f64_convert_i64_s;
    fn f64_convert_i64_rr(F64ConvertI64_Rr) = wasm::f64_convert_i64_s;
    fn f64_convert_u64_rs(F64ConvertU64_Rs) = wasm::f64_convert_i64_u;
    fn f64_convert_u64_rr(F64ConvertU64_Rr) = wasm::f64_convert_i64_u;
    fn f64_promote_f32_rs(F64PromoteF32_Rs) = wasm::f64_promote_f32;
    fn f64_promote_f32_rr(F64PromoteF32_Rr) = wasm::f64_promote_f32;
    // f2i conversions
    fn i32_trunc_f32_rs(I32TruncF32_Rs) = wasm::i32_trunc_f32_s;
    fn i32_trunc_f32_rr(I32TruncF32_Rr) = wasm::i32_trunc_f32_s;
    fn u32_trunc_f32_rs(U32TruncF32_Rs) = wasm::i32_trunc_f32_u;
    fn u32_trunc_f32_rr(U32TruncF32_Rr) = wasm::i32_trunc_f32_u;
    fn i32_trunc_f64_rs(I32TruncF64_Rs) = wasm::i32_trunc_f64_s;
    fn i32_trunc_f64_rr(I32TruncF64_Rr) = wasm::i32_trunc_f64_s;
    fn u32_trunc_f64_rs(U32TruncF64_Rs) = wasm::i32_trunc_f64_u;
    fn u32_trunc_f64_rr(U32TruncF64_Rr) = wasm::i32_trunc_f64_u;
    fn i64_trunc_f32_rs(I64TruncF32_Rs) = wasm::i64_trunc_f32_s;
    fn i64_trunc_f32_rr(I64TruncF32_Rr) = wasm::i64_trunc_f32_s;
    fn u64_trunc_f32_rs(U64TruncF32_Rs) = wasm::i64_trunc_f32_u;
    fn u64_trunc_f32_rr(U64TruncF32_Rr) = wasm::i64_trunc_f32_u;
    fn i64_trunc_f64_rs(I64TruncF64_Rs) = wasm::i64_trunc_f64_s;
    fn i64_trunc_f64_rr(I64TruncF64_Rr) = wasm::i64_trunc_f64_s;
    fn u64_trunc_f64_rs(U64TruncF64_Rs) = wasm::i64_trunc_f64_u;
    fn u64_trunc_f64_rr(U64TruncF64_Rr) = wasm::i64_trunc_f64_u;
    fn i32_trunc_sat_f32_rs(I32TruncSatF32_Rs) = wasm::i32_trunc_sat_f32_s;
    fn i32_trunc_sat_f32_rr(I32TruncSatF32_Rr) = wasm::i32_trunc_sat_f32_s;
    fn u32_trunc_sat_f32_rs(U32TruncSatF32_Rs) = wasm::i32_trunc_sat_f32_u;
    fn u32_trunc_sat_f32_rr(U32TruncSatF32_Rr) = wasm::i32_trunc_sat_f32_u;
    fn i32_trunc_sat_f64_rs(I32TruncSatF64_Rs) = wasm::i32_trunc_sat_f64_s;
    fn i32_trunc_sat_f64_rr(I32TruncSatF64_Rr) = wasm::i32_trunc_sat_f64_s;
    fn u32_trunc_sat_f64_rs(U32TruncSatF64_Rs) = wasm::i32_trunc_sat_f64_u;
    fn u32_trunc_sat_f64_rr(U32TruncSatF64_Rr) = wasm::i32_trunc_sat_f64_u;
    fn i64_trunc_sat_f32_rs(I64TruncSatF32_Rs) = wasm::i64_trunc_sat_f32_s;
    fn i64_trunc_sat_f32_rr(I64TruncSatF32_Rr) = wasm::i64_trunc_sat_f32_s;
    fn u64_trunc_sat_f32_rs(U64TruncSatF32_Rs) = wasm::i64_trunc_sat_f32_u;
    fn u64_trunc_sat_f32_rr(U64TruncSatF32_Rr) = wasm::i64_trunc_sat_f32_u;
    fn i64_trunc_sat_f64_rs(I64TruncSatF64_Rs) = wasm::i64_trunc_sat_f64_s;
    fn i64_trunc_sat_f64_rr(I64TruncSatF64_Rr) = wasm::i64_trunc_sat_f64_s;
    fn u64_trunc_sat_f64_rs(U64TruncSatF64_Rs) = wasm::i64_trunc_sat_f64_u;
    fn u64_trunc_sat_f64_rr(U64TruncSatF64_Rr) = wasm::i64_trunc_sat_f64_u;
}

handler_binary! {
    // i32
    // i32: commutative
    fn i32_eq_rrs(I32Eq_Rrs) = wasm::i32_eq;
    fn i32_eq_rri(I32Eq_Rri) = wasm::i32_eq;
    fn i32_eq_rss(I32Eq_Rss) = wasm::i32_eq;
    fn i32_eq_rsi(I32Eq_Rsi) = wasm::i32_eq;
    fn i32_and_rrs(I32And_Rrs) = eval::wasmi_i32_and;
    fn i32_and_rri(I32And_Rri) = eval::wasmi_i32_and;
    fn i32_and_rss(I32And_Rss) = eval::wasmi_i32_and;
    fn i32_and_rsi(I32And_Rsi) = eval::wasmi_i32_and;
    fn i32_or_rrs(I32Or_Rrs) = eval::wasmi_i32_or;
    fn i32_or_rri(I32Or_Rri) = eval::wasmi_i32_or;
    fn i32_or_rss(I32Or_Rss) = eval::wasmi_i32_or;
    fn i32_or_rsi(I32Or_Rsi) = eval::wasmi_i32_or;
    fn i32_not_eq_rrs(I32NotEq_Rrs) = wasm::i32_ne;
    fn i32_not_eq_rri(I32NotEq_Rri) = wasm::i32_ne;
    fn i32_not_eq_rss(I32NotEq_Rss) = wasm::i32_ne;
    fn i32_not_eq_rsi(I32NotEq_Rsi) = wasm::i32_ne;
    fn i32_not_and_rrs(I32NotAnd_Rrs) = eval::wasmi_i32_not_and;
    fn i32_not_and_rri(I32NotAnd_Rri) = eval::wasmi_i32_not_and;
    fn i32_not_and_rss(I32NotAnd_Rss) = eval::wasmi_i32_not_and;
    fn i32_not_and_rsi(I32NotAnd_Rsi) = eval::wasmi_i32_not_and;
    fn i32_not_or_rrs(I32NotOr_Rrs) = eval::wasmi_i32_not_or;
    fn i32_not_or_rri(I32NotOr_Rri) = eval::wasmi_i32_not_or;
    fn i32_not_or_rss(I32NotOr_Rss) = eval::wasmi_i32_not_or;
    fn i32_not_or_rsi(I32NotOr_Rsi) = eval::wasmi_i32_not_or;
    fn i32_add_rrs(I32Add_Rrs) = wasm::i32_add;
    fn i32_add_rri(I32Add_Rri) = wasm::i32_add;
    fn i32_add_rss(I32Add_Rss) = wasm::i32_add;
    fn i32_add_rsi(I32Add_Rsi) = wasm::i32_add;
    fn i32_add_rs_rs(I32Add_Rs_rs) = wasm::i32_add;
    fn i32_add_rs_ri(I32Add_Rs_ri) = wasm::i32_add;
    fn i32_add_rs_ss(I32Add_Rs_ss) = wasm::i32_add;
    fn i32_add_rs_si(I32Add_Rs_si) = wasm::i32_add;
    fn i32_mul_rrr(I32Mul_Rrr) = wasm::i32_mul;
    fn i32_mul_rrs(I32Mul_Rrs) = wasm::i32_mul;
    fn i32_mul_rri(I32Mul_Rri) = wasm::i32_mul;
    fn i32_mul_rss(I32Mul_Rss) = wasm::i32_mul;
    fn i32_mul_rsi(I32Mul_Rsi) = wasm::i32_mul;
    fn i32_bitand_rrs(I32BitAnd_Rrs) = wasm::i32_bitand;
    fn i32_bitand_rri(I32BitAnd_Rri) = wasm::i32_bitand;
    fn i32_bitand_rss(I32BitAnd_Rss) = wasm::i32_bitand;
    fn i32_bitand_rsi(I32BitAnd_Rsi) = wasm::i32_bitand;
    fn i32_bitor_rrs(I32BitOr_Rrs) = wasm::i32_bitor;
    fn i32_bitor_rri(I32BitOr_Rri) = wasm::i32_bitor;
    fn i32_bitor_rss(I32BitOr_Rss) = wasm::i32_bitor;
    fn i32_bitor_rsi(I32BitOr_Rsi) = wasm::i32_bitor;
    fn i32_bitxor_rrs(I32BitXor_Rrs) = wasm::i32_bitxor;
    fn i32_bitxor_rri(I32BitXor_Rri) = wasm::i32_bitxor;
    fn i32_bitxor_rss(I32BitXor_Rss) = wasm::i32_bitxor;
    fn i32_bitxor_rsi(I32BitXor_Rsi) = wasm::i32_bitxor;
    // i32: non-commutative
    fn i32_sub_rrs(I32Sub_Rrs) = wasm::i32_sub;
    fn i32_sub_rsr(I32Sub_Rsr) = wasm::i32_sub;
    fn i32_sub_rss(I32Sub_Rss) = wasm::i32_sub;
    fn i32_sub_rir(I32Sub_Rir) = wasm::i32_sub;
    fn i32_sub_ris(I32Sub_Ris) = wasm::i32_sub;
    fn i32_div_rrs(I32Div_Rrs) = wasm::i32_div_s;
    fn i32_div_rri(I32Div_Rri) = eval::wasmi_i32_div_ssi;
    fn i32_div_rsr(I32Div_Rsr) = wasm::i32_div_s;
    fn i32_div_rss(I32Div_Rss) = wasm::i32_div_s;
    fn i32_div_rsi(I32Div_Rsi) = eval::wasmi_i32_div_ssi;
    fn i32_div_rir(I32Div_Rir) = wasm::i32_div_s;
    fn i32_div_ris(I32Div_Ris) = wasm::i32_div_s;
    fn u32_div_rrs(U32Div_Rrs) = wasm::i32_div_u;
    fn u32_div_rri(U32Div_Rri) = eval::wasmi_u32_div_ssi;
    fn u32_div_rsr(U32Div_Rsr) = wasm::i32_div_u;
    fn u32_div_rss(U32Div_Rss) = wasm::i32_div_u;
    fn u32_div_rsi(U32Div_Rsi) = eval::wasmi_u32_div_ssi;
    fn u32_div_rir(U32Div_Rir) = wasm::i32_div_u;
    fn u32_div_ris(U32Div_Ris) = wasm::i32_div_u;
    fn i32_rem_rrs(I32Rem_Rrs) = wasm::i32_rem_s;
    fn i32_rem_rri(I32Rem_Rri) = eval::wasmi_i32_rem_ssi;
    fn i32_rem_rsr(I32Rem_Rsr) = wasm::i32_rem_s;
    fn i32_rem_rss(I32Rem_Rss) = wasm::i32_rem_s;
    fn i32_rem_rsi(I32Rem_Rsi) = eval::wasmi_i32_rem_ssi;
    fn i32_rem_rir(I32Rem_Rir) = wasm::i32_rem_s;
    fn i32_rem_ris(I32Rem_Ris) = wasm::i32_rem_s;
    fn u32_rem_rrs(U32Rem_Rrs) = wasm::i32_rem_u;
    fn u32_rem_rri(U32Rem_Rri) = eval::wasmi_u32_rem_ssi;
    fn u32_rem_rsr(U32Rem_Rsr) = wasm::i32_rem_u;
    fn u32_rem_rss(U32Rem_Rss) = wasm::i32_rem_u;
    fn u32_rem_rsi(U32Rem_Rsi) = eval::wasmi_u32_rem_ssi;
    fn u32_rem_rir(U32Rem_Rir) = wasm::i32_rem_u;
    fn u32_rem_ris(U32Rem_Ris) = wasm::i32_rem_u;
    // i32: comparisons
    fn i32_le_rrs(I32Le_Rrs) = wasm::i32_le_s;
    fn i32_le_rri(I32Le_Rri) = wasm::i32_le_s;
    fn i32_le_rsr(I32Le_Rsr) = wasm::i32_le_s;
    fn i32_le_rss(I32Le_Rss) = wasm::i32_le_s;
    fn i32_le_rsi(I32Le_Rsi) = wasm::i32_le_s;
    fn i32_le_rir(I32Le_Rir) = wasm::i32_le_s;
    fn i32_le_ris(I32Le_Ris) = wasm::i32_le_s;
    fn i32_lt_rrs(I32Lt_Rrs) = wasm::i32_lt_s;
    fn i32_lt_rri(I32Lt_Rri) = wasm::i32_lt_s;
    fn i32_lt_rsr(I32Lt_Rsr) = wasm::i32_lt_s;
    fn i32_lt_rss(I32Lt_Rss) = wasm::i32_lt_s;
    fn i32_lt_rsi(I32Lt_Rsi) = wasm::i32_lt_s;
    fn i32_lt_rir(I32Lt_Rir) = wasm::i32_lt_s;
    fn i32_lt_ris(I32Lt_Ris) = wasm::i32_lt_s;
    fn u32_le_rrs(U32Le_Rrs) = wasm::i32_le_u;
    fn u32_le_rri(U32Le_Rri) = wasm::i32_le_u;
    fn u32_le_rsr(U32Le_Rsr) = wasm::i32_le_u;
    fn u32_le_rss(U32Le_Rss) = wasm::i32_le_u;
    fn u32_le_rsi(U32Le_Rsi) = wasm::i32_le_u;
    fn u32_le_rir(U32Le_Rir) = wasm::i32_le_u;
    fn u32_le_ris(U32Le_Ris) = wasm::i32_le_u;
    fn u32_lt_rrs(U32Lt_Rrs) = wasm::i32_lt_u;
    fn u32_lt_rri(U32Lt_Rri) = wasm::i32_lt_u;
    fn u32_lt_rsr(U32Lt_Rsr) = wasm::i32_lt_u;
    fn u32_lt_rss(U32Lt_Rss) = wasm::i32_lt_u;
    fn u32_lt_rsi(U32Lt_Rsi) = wasm::i32_lt_u;
    fn u32_lt_rir(U32Lt_Rir) = wasm::i32_lt_u;
    fn u32_lt_ris(U32Lt_Ris) = wasm::i32_lt_u;
    // i32: shift + rotate
    fn i32_shl_rrs(I32Shl_Rrs) = wasm::i32_shl;
    fn i32_shl_rri(I32Shl_Rri) = eval::wasmi_i32_shl_ssi;
    fn i32_shl_rsr(I32Shl_Rsr) = wasm::i32_shl;
    fn i32_shl_rss(I32Shl_Rss) = wasm::i32_shl;
    fn i32_shl_rsi(I32Shl_Rsi) = eval::wasmi_i32_shl_ssi;
    fn i32_shl_rir(I32Shl_Rir) = wasm::i32_shl;
    fn i32_shl_ris(I32Shl_Ris) = wasm::i32_shl;
    fn i32_shr_rrs(I32Shr_Rrs) = wasm::i32_shr_s;
    fn i32_shr_rri(I32Shr_Rri) = eval::wasmi_i32_shr_ssi;
    fn i32_shr_rsr(I32Shr_Rsr) = wasm::i32_shr_s;
    fn i32_shr_rss(I32Shr_Rss) = wasm::i32_shr_s;
    fn i32_shr_rsi(I32Shr_Rsi) = eval::wasmi_i32_shr_ssi;
    fn i32_shr_rir(I32Shr_Rir) = wasm::i32_shr_s;
    fn i32_shr_ris(I32Shr_Ris) = wasm::i32_shr_s;
    fn u32_shr_rrs(U32Shr_Rrs) = wasm::i32_shr_u;
    fn u32_shr_rri(U32Shr_Rri) = eval::wasmi_u32_shr_ssi;
    fn u32_shr_rsr(U32Shr_Rsr) = wasm::i32_shr_u;
    fn u32_shr_rss(U32Shr_Rss) = wasm::i32_shr_u;
    fn u32_shr_rsi(U32Shr_Rsi) = eval::wasmi_u32_shr_ssi;
    fn u32_shr_rir(U32Shr_Rir) = wasm::i32_shr_u;
    fn u32_shr_ris(U32Shr_Ris) = wasm::i32_shr_u;
    fn i32_rotl_rrs(I32Rotl_Rrs) = wasm::i32_rotl;
    fn i32_rotl_rri(I32Rotl_Rri) = eval::wasmi_i32_rotl_ssi;
    fn i32_rotl_rsr(I32Rotl_Rsr) = wasm::i32_rotl;
    fn i32_rotl_rss(I32Rotl_Rss) = wasm::i32_rotl;
    fn i32_rotl_rsi(I32Rotl_Rsi) = eval::wasmi_i32_rotl_ssi;
    fn i32_rotl_rir(I32Rotl_Rir) = wasm::i32_rotl;
    fn i32_rotl_ris(I32Rotl_Ris) = wasm::i32_rotl;
    fn i32_rotr_rrs(I32Rotr_Rrs) = wasm::i32_rotr;
    fn i32_rotr_rri(I32Rotr_Rri) = eval::wasmi_i32_rotr_ssi;
    fn i32_rotr_rsr(I32Rotr_Rsr) = wasm::i32_rotr;
    fn i32_rotr_rss(I32Rotr_Rss) = wasm::i32_rotr;
    fn i32_rotr_rsi(I32Rotr_Rsi) = eval::wasmi_i32_rotr_ssi;
    fn i32_rotr_rir(I32Rotr_Rir) = wasm::i32_rotr;
    fn i32_rotr_ris(I32Rotr_Ris) = wasm::i32_rotr;
    // i64
    // i64: commutative
    fn i64_eq_rrs(I64Eq_Rrs) = wasm::i64_eq;
    fn i64_eq_rri(I64Eq_Rri) = wasm::i64_eq;
    fn i64_eq_rss(I64Eq_Rss) = wasm::i64_eq;
    fn i64_eq_rsi(I64Eq_Rsi) = wasm::i64_eq;
    fn i64_and_rrs(I64And_Rrs) = eval::wasmi_i64_and;
    fn i64_and_rri(I64And_Rri) = eval::wasmi_i64_and;
    fn i64_and_rss(I64And_Rss) = eval::wasmi_i64_and;
    fn i64_and_rsi(I64And_Rsi) = eval::wasmi_i64_and;
    fn i64_or_rrs(I64Or_Rrs) = eval::wasmi_i64_or;
    fn i64_or_rri(I64Or_Rri) = eval::wasmi_i64_or;
    fn i64_or_rss(I64Or_Rss) = eval::wasmi_i64_or;
    fn i64_or_rsi(I64Or_Rsi) = eval::wasmi_i64_or;
    fn i64_not_eq_rrs(I64NotEq_Rrs) = wasm::i64_ne;
    fn i64_not_eq_rri(I64NotEq_Rri) = wasm::i64_ne;
    fn i64_not_eq_rss(I64NotEq_Rss) = wasm::i64_ne;
    fn i64_not_eq_rsi(I64NotEq_Rsi) = wasm::i64_ne;
    fn i64_not_and_rrs(I64NotAnd_Rrs) = eval::wasmi_i64_not_and;
    fn i64_not_and_rri(I64NotAnd_Rri) = eval::wasmi_i64_not_and;
    fn i64_not_and_rss(I64NotAnd_Rss) = eval::wasmi_i64_not_and;
    fn i64_not_and_rsi(I64NotAnd_Rsi) = eval::wasmi_i64_not_and;
    fn i64_not_or_rrs(I64NotOr_Rrs) = eval::wasmi_i64_not_or;
    fn i64_not_or_rri(I64NotOr_Rri) = eval::wasmi_i64_not_or;
    fn i64_not_or_rss(I64NotOr_Rss) = eval::wasmi_i64_not_or;
    fn i64_not_or_rsi(I64NotOr_Rsi) = eval::wasmi_i64_not_or;
    fn i64_add_rrs(I64Add_Rrs) = wasm::i64_add;
    fn i64_add_rri(I64Add_Rri) = wasm::i64_add;
    fn i64_add_rss(I64Add_Rss) = wasm::i64_add;
    fn i64_add_rsi(I64Add_Rsi) = wasm::i64_add;
    fn i64_add_rs_rs(I64Add_Rs_rs) = wasm::i64_add;
    fn i64_add_rs_ri(I64Add_Rs_ri) = wasm::i64_add;
    fn i64_add_rs_ss(I64Add_Rs_ss) = wasm::i64_add;
    fn i64_add_rs_si(I64Add_Rs_si) = wasm::i64_add;
    fn i64_mul_rrr(I64Mul_Rrr) = wasm::i64_mul;
    fn i64_mul_rrs(I64Mul_Rrs) = wasm::i64_mul;
    fn i64_mul_rri(I64Mul_Rri) = wasm::i64_mul;
    fn i64_mul_rss(I64Mul_Rss) = wasm::i64_mul;
    fn i64_mul_rsi(I64Mul_Rsi) = wasm::i64_mul;
    fn i64_bitand_rrs(I64BitAnd_Rrs) = wasm::i64_bitand;
    fn i64_bitand_rri(I64BitAnd_Rri) = wasm::i64_bitand;
    fn i64_bitand_rss(I64BitAnd_Rss) = wasm::i64_bitand;
    fn i64_bitand_rsi(I64BitAnd_Rsi) = wasm::i64_bitand;
    fn i64_bitor_rrs(I64BitOr_Rrs) = wasm::i64_bitor;
    fn i64_bitor_rri(I64BitOr_Rri) = wasm::i64_bitor;
    fn i64_bitor_rss(I64BitOr_Rss) = wasm::i64_bitor;
    fn i64_bitor_rsi(I64BitOr_Rsi) = wasm::i64_bitor;
    fn i64_bitxor_rrs(I64BitXor_Rrs) = wasm::i64_bitxor;
    fn i64_bitxor_rri(I64BitXor_Rri) = wasm::i64_bitxor;
    fn i64_bitxor_rss(I64BitXor_Rss) = wasm::i64_bitxor;
    fn i64_bitxor_rsi(I64BitXor_Rsi) = wasm::i64_bitxor;
    // i64: non-commutative
    fn i64_sub_rrs(I64Sub_Rrs) = wasm::i64_sub;
    fn i64_sub_rsr(I64Sub_Rsr) = wasm::i64_sub;
    fn i64_sub_rss(I64Sub_Rss) = wasm::i64_sub;
    fn i64_sub_rir(I64Sub_Rir) = wasm::i64_sub;
    fn i64_sub_ris(I64Sub_Ris) = wasm::i64_sub;
    fn i64_div_rrs(I64Div_Rrs) = wasm::i64_div_s;
    fn i64_div_rri(I64Div_Rri) = eval::wasmi_i64_div_ssi;
    fn i64_div_rsr(I64Div_Rsr) = wasm::i64_div_s;
    fn i64_div_rss(I64Div_Rss) = wasm::i64_div_s;
    fn i64_div_rsi(I64Div_Rsi) = eval::wasmi_i64_div_ssi;
    fn i64_div_rir(I64Div_Rir) = wasm::i64_div_s;
    fn i64_div_ris(I64Div_Ris) = wasm::i64_div_s;
    fn u64_div_rrs(U64Div_Rrs) = wasm::i64_div_u;
    fn u64_div_rri(U64Div_Rri) = eval::wasmi_u64_div_ssi;
    fn u64_div_rsr(U64Div_Rsr) = wasm::i64_div_u;
    fn u64_div_rss(U64Div_Rss) = wasm::i64_div_u;
    fn u64_div_rsi(U64Div_Rsi) = eval::wasmi_u64_div_ssi;
    fn u64_div_rir(U64Div_Rir) = wasm::i64_div_u;
    fn u64_div_ris(U64Div_Ris) = wasm::i64_div_u;
    fn i64_rem_rrs(I64Rem_Rrs) = wasm::i64_rem_s;
    fn i64_rem_rri(I64Rem_Rri) = eval::wasmi_i64_rem_ssi;
    fn i64_rem_rsr(I64Rem_Rsr) = wasm::i64_rem_s;
    fn i64_rem_rss(I64Rem_Rss) = wasm::i64_rem_s;
    fn i64_rem_rsi(I64Rem_Rsi) = eval::wasmi_i64_rem_ssi;
    fn i64_rem_rir(I64Rem_Rir) = wasm::i64_rem_s;
    fn i64_rem_ris(I64Rem_Ris) = wasm::i64_rem_s;
    fn u64_rem_rrs(U64Rem_Rrs) = wasm::i64_rem_u;
    fn u64_rem_rri(U64Rem_Rri) = eval::wasmi_u64_rem_ssi;
    fn u64_rem_rsr(U64Rem_Rsr) = wasm::i64_rem_u;
    fn u64_rem_rss(U64Rem_Rss) = wasm::i64_rem_u;
    fn u64_rem_rsi(U64Rem_Rsi) = eval::wasmi_u64_rem_ssi;
    fn u64_rem_rir(U64Rem_Rir) = wasm::i64_rem_u;
    fn u64_rem_ris(U64Rem_Ris) = wasm::i64_rem_u;
    // i64: comparisons
    fn i64_le_rrs(I64Le_Rrs) = wasm::i64_le_s;
    fn i64_le_rri(I64Le_Rri) = wasm::i64_le_s;
    fn i64_le_rsr(I64Le_Rsr) = wasm::i64_le_s;
    fn i64_le_rss(I64Le_Rss) = wasm::i64_le_s;
    fn i64_le_rsi(I64Le_Rsi) = wasm::i64_le_s;
    fn i64_le_rir(I64Le_Rir) = wasm::i64_le_s;
    fn i64_le_ris(I64Le_Ris) = wasm::i64_le_s;
    fn i64_lt_rrs(I64Lt_Rrs) = wasm::i64_lt_s;
    fn i64_lt_rri(I64Lt_Rri) = wasm::i64_lt_s;
    fn i64_lt_rsr(I64Lt_Rsr) = wasm::i64_lt_s;
    fn i64_lt_rss(I64Lt_Rss) = wasm::i64_lt_s;
    fn i64_lt_rsi(I64Lt_Rsi) = wasm::i64_lt_s;
    fn i64_lt_rir(I64Lt_Rir) = wasm::i64_lt_s;
    fn i64_lt_ris(I64Lt_Ris) = wasm::i64_lt_s;
    fn u64_le_rrs(U64Le_Rrs) = wasm::i64_le_u;
    fn u64_le_rri(U64Le_Rri) = wasm::i64_le_u;
    fn u64_le_rsr(U64Le_Rsr) = wasm::i64_le_u;
    fn u64_le_rss(U64Le_Rss) = wasm::i64_le_u;
    fn u64_le_rsi(U64Le_Rsi) = wasm::i64_le_u;
    fn u64_le_rir(U64Le_Rir) = wasm::i64_le_u;
    fn u64_le_ris(U64Le_Ris) = wasm::i64_le_u;
    fn u64_lt_rrs(U64Lt_Rrs) = wasm::i64_lt_u;
    fn u64_lt_rri(U64Lt_Rri) = wasm::i64_lt_u;
    fn u64_lt_rsr(U64Lt_Rsr) = wasm::i64_lt_u;
    fn u64_lt_rss(U64Lt_Rss) = wasm::i64_lt_u;
    fn u64_lt_rsi(U64Lt_Rsi) = wasm::i64_lt_u;
    fn u64_lt_rir(U64Lt_Rir) = wasm::i64_lt_u;
    fn u64_lt_ris(U64Lt_Ris) = wasm::i64_lt_u;
    // i64: shift + rotate
    fn i64_shl_rrs(I64Shl_Rrs) = wasm::i64_shl;
    fn i64_shl_rri(I64Shl_Rri) = eval::wasmi_i64_shl_ssi;
    fn i64_shl_rsr(I64Shl_Rsr) = wasm::i64_shl;
    fn i64_shl_rss(I64Shl_Rss) = wasm::i64_shl;
    fn i64_shl_rsi(I64Shl_Rsi) = eval::wasmi_i64_shl_ssi;
    fn i64_shl_rir(I64Shl_Rir) = wasm::i64_shl;
    fn i64_shl_ris(I64Shl_Ris) = wasm::i64_shl;
    fn i64_shr_rrs(I64Shr_Rrs) = wasm::i64_shr_s;
    fn i64_shr_rri(I64Shr_Rri) = eval::wasmi_i64_shr_ssi;
    fn i64_shr_rsr(I64Shr_Rsr) = wasm::i64_shr_s;
    fn i64_shr_rss(I64Shr_Rss) = wasm::i64_shr_s;
    fn i64_shr_rsi(I64Shr_Rsi) = eval::wasmi_i64_shr_ssi;
    fn i64_shr_rir(I64Shr_Rir) = wasm::i64_shr_s;
    fn i64_shr_ris(I64Shr_Ris) = wasm::i64_shr_s;
    fn u64_shr_rrs(U64Shr_Rrs) = wasm::i64_shr_u;
    fn u64_shr_rri(U64Shr_Rri) = eval::wasmi_u64_shr_ssi;
    fn u64_shr_rsr(U64Shr_Rsr) = wasm::i64_shr_u;
    fn u64_shr_rss(U64Shr_Rss) = wasm::i64_shr_u;
    fn u64_shr_rsi(U64Shr_Rsi) = eval::wasmi_u64_shr_ssi;
    fn u64_shr_rir(U64Shr_Rir) = wasm::i64_shr_u;
    fn u64_shr_ris(U64Shr_Ris) = wasm::i64_shr_u;
    fn i64_rotl_rrs(I64Rotl_Rrs) = wasm::i64_rotl;
    fn i64_rotl_rri(I64Rotl_Rri) = eval::wasmi_i64_rotl_ssi;
    fn i64_rotl_rsr(I64Rotl_Rsr) = wasm::i64_rotl;
    fn i64_rotl_rss(I64Rotl_Rss) = wasm::i64_rotl;
    fn i64_rotl_rsi(I64Rotl_Rsi) = eval::wasmi_i64_rotl_ssi;
    fn i64_rotl_rir(I64Rotl_Rir) = wasm::i64_rotl;
    fn i64_rotl_ris(I64Rotl_Ris) = wasm::i64_rotl;
    fn i64_rotr_rrs(I64Rotr_Rrs) = wasm::i64_rotr;
    fn i64_rotr_rri(I64Rotr_Rri) = eval::wasmi_i64_rotr_ssi;
    fn i64_rotr_rsr(I64Rotr_Rsr) = wasm::i64_rotr;
    fn i64_rotr_rss(I64Rotr_Rss) = wasm::i64_rotr;
    fn i64_rotr_rsi(I64Rotr_Rsi) = eval::wasmi_i64_rotr_ssi;
    fn i64_rotr_rir(I64Rotr_Rir) = wasm::i64_rotr;
    fn i64_rotr_ris(I64Rotr_Ris) = wasm::i64_rotr;
    // f32
    // f32: binary
    fn f32_add_rrs(F32Add_Rrs) = wasm::f32_add;
    fn f32_add_rri(F32Add_Rri) = wasm::f32_add;
    fn f32_add_rsr(F32Add_Rsr) = wasm::f32_add;
    fn f32_add_rss(F32Add_Rss) = wasm::f32_add;
    fn f32_add_rsi(F32Add_Rsi) = wasm::f32_add;
    fn f32_add_rir(F32Add_Rir) = wasm::f32_add;
    fn f32_add_ris(F32Add_Ris) = wasm::f32_add;
    fn f32_sub_rrs(F32Sub_Rrs) = wasm::f32_sub;
    fn f32_sub_rri(F32Sub_Rri) = wasm::f32_sub;
    fn f32_sub_rsr(F32Sub_Rsr) = wasm::f32_sub;
    fn f32_sub_rss(F32Sub_Rss) = wasm::f32_sub;
    fn f32_sub_rsi(F32Sub_Rsi) = wasm::f32_sub;
    fn f32_sub_rir(F32Sub_Rir) = wasm::f32_sub;
    fn f32_sub_ris(F32Sub_Ris) = wasm::f32_sub;
    fn f32_mul_rrr(F32Mul_Rrr) = wasm::f32_mul;
    fn f32_mul_rrs(F32Mul_Rrs) = wasm::f32_mul;
    fn f32_mul_rri(F32Mul_Rri) = wasm::f32_mul;
    fn f32_mul_rsr(F32Mul_Rsr) = wasm::f32_mul;
    fn f32_mul_rss(F32Mul_Rss) = wasm::f32_mul;
    fn f32_mul_rsi(F32Mul_Rsi) = wasm::f32_mul;
    fn f32_mul_rir(F32Mul_Rir) = wasm::f32_mul;
    fn f32_mul_ris(F32Mul_Ris) = wasm::f32_mul;
    fn f32_div_rrs(F32Div_Rrs) = wasm::f32_div;
    fn f32_div_rri(F32Div_Rri) = wasm::f32_div;
    fn f32_div_rsr(F32Div_Rsr) = wasm::f32_div;
    fn f32_div_rss(F32Div_Rss) = wasm::f32_div;
    fn f32_div_rsi(F32Div_Rsi) = wasm::f32_div;
    fn f32_div_rir(F32Div_Rir) = wasm::f32_div;
    fn f32_div_ris(F32Div_Ris) = wasm::f32_div;
    fn f32_min_rrs(F32Min_Rrs) = wasm::f32_min;
    fn f32_min_rri(F32Min_Rri) = wasm::f32_min;
    fn f32_min_rsr(F32Min_Rsr) = wasm::f32_min;
    fn f32_min_rss(F32Min_Rss) = wasm::f32_min;
    fn f32_min_rsi(F32Min_Rsi) = wasm::f32_min;
    fn f32_min_rir(F32Min_Rir) = wasm::f32_min;
    fn f32_min_ris(F32Min_Ris) = wasm::f32_min;
    fn f32_max_rrs(F32Max_Rrs) = wasm::f32_max;
    fn f32_max_rri(F32Max_Rri) = wasm::f32_max;
    fn f32_max_rsr(F32Max_Rsr) = wasm::f32_max;
    fn f32_max_rss(F32Max_Rss) = wasm::f32_max;
    fn f32_max_rsi(F32Max_Rsi) = wasm::f32_max;
    fn f32_max_rir(F32Max_Rir) = wasm::f32_max;
    fn f32_max_ris(F32Max_Ris) = wasm::f32_max;
    fn f32_copysign_rrs(F32Copysign_Rrs) = wasm::f32_copysign;
    fn f32_copysign_rsr(F32Copysign_Rsr) = wasm::f32_copysign;
    fn f32_copysign_rss(F32Copysign_Rss) = wasm::f32_copysign;
    fn f32_copysign_rir(F32Copysign_Rir) = wasm::f32_copysign;
    fn f32_copysign_ris(F32Copysign_Ris) = wasm::f32_copysign;
    // f32: comparisons
    fn f32_eq_rrs(F32Eq_Rrs) = wasm::f32_eq;
    fn f32_eq_rri(F32Eq_Rri) = wasm::f32_eq;
    fn f32_eq_rss(F32Eq_Rss) = wasm::f32_eq;
    fn f32_eq_rsi(F32Eq_Rsi) = wasm::f32_eq;
    fn f32_lt_rrs(F32Lt_Rrs) = wasm::f32_lt;
    fn f32_lt_rri(F32Lt_Rri) = wasm::f32_lt;
    fn f32_lt_rsr(F32Lt_Rsr) = wasm::f32_lt;
    fn f32_lt_rss(F32Lt_Rss) = wasm::f32_lt;
    fn f32_lt_rsi(F32Lt_Rsi) = wasm::f32_lt;
    fn f32_lt_rir(F32Lt_Rir) = wasm::f32_lt;
    fn f32_lt_ris(F32Lt_Ris) = wasm::f32_lt;
    fn f32_le_rrs(F32Le_Rrs) = wasm::f32_le;
    fn f32_le_rri(F32Le_Rri) = wasm::f32_le;
    fn f32_le_rsr(F32Le_Rsr) = wasm::f32_le;
    fn f32_le_rss(F32Le_Rss) = wasm::f32_le;
    fn f32_le_rsi(F32Le_Rsi) = wasm::f32_le;
    fn f32_le_rir(F32Le_Rir) = wasm::f32_le;
    fn f32_le_ris(F32Le_Ris) = wasm::f32_le;
    fn f32_not_eq_rrs(F32NotEq_Rrs) = wasm::f32_ne;
    fn f32_not_eq_rri(F32NotEq_Rri) = wasm::f32_ne;
    fn f32_not_eq_rss(F32NotEq_Rss) = wasm::f32_ne;
    fn f32_not_eq_rsi(F32NotEq_Rsi) = wasm::f32_ne;
    fn f32_not_lt_rrs(F32NotLt_Rrs) = eval::wasmi_f32_not_lt;
    fn f32_not_lt_rri(F32NotLt_Rri) = eval::wasmi_f32_not_lt;
    fn f32_not_lt_rsr(F32NotLt_Rsr) = eval::wasmi_f32_not_lt;
    fn f32_not_lt_rss(F32NotLt_Rss) = eval::wasmi_f32_not_lt;
    fn f32_not_lt_rsi(F32NotLt_Rsi) = eval::wasmi_f32_not_lt;
    fn f32_not_lt_rir(F32NotLt_Rir) = eval::wasmi_f32_not_lt;
    fn f32_not_lt_ris(F32NotLt_Ris) = eval::wasmi_f32_not_lt;
    fn f32_not_le_rrs(F32NotLe_Rrs) = eval::wasmi_f32_not_le;
    fn f32_not_le_rri(F32NotLe_Rri) = eval::wasmi_f32_not_le;
    fn f32_not_le_rsr(F32NotLe_Rsr) = eval::wasmi_f32_not_le;
    fn f32_not_le_rss(F32NotLe_Rss) = eval::wasmi_f32_not_le;
    fn f32_not_le_rsi(F32NotLe_Rsi) = eval::wasmi_f32_not_le;
    fn f32_not_le_rir(F32NotLe_Rir) = eval::wasmi_f32_not_le;
    fn f32_not_le_ris(F32NotLe_Ris) = eval::wasmi_f32_not_le;
    // f64
    // f64: binary
    fn f64_add_rrs(F64Add_Rrs) = wasm::f64_add;
    fn f64_add_rri(F64Add_Rri) = wasm::f64_add;
    fn f64_add_rsr(F64Add_Rsr) = wasm::f64_add;
    fn f64_add_rss(F64Add_Rss) = wasm::f64_add;
    fn f64_add_rsi(F64Add_Rsi) = wasm::f64_add;
    fn f64_add_rir(F64Add_Rir) = wasm::f64_add;
    fn f64_add_ris(F64Add_Ris) = wasm::f64_add;
    fn f64_sub_rrs(F64Sub_Rrs) = wasm::f64_sub;
    fn f64_sub_rri(F64Sub_Rri) = wasm::f64_sub;
    fn f64_sub_rsr(F64Sub_Rsr) = wasm::f64_sub;
    fn f64_sub_rss(F64Sub_Rss) = wasm::f64_sub;
    fn f64_sub_rsi(F64Sub_Rsi) = wasm::f64_sub;
    fn f64_sub_rir(F64Sub_Rir) = wasm::f64_sub;
    fn f64_sub_ris(F64Sub_Ris) = wasm::f64_sub;
    fn f64_mul_rrr(F64Mul_Rrr) = wasm::f64_mul;
    fn f64_mul_rrs(F64Mul_Rrs) = wasm::f64_mul;
    fn f64_mul_rri(F64Mul_Rri) = wasm::f64_mul;
    fn f64_mul_rsr(F64Mul_Rsr) = wasm::f64_mul;
    fn f64_mul_rss(F64Mul_Rss) = wasm::f64_mul;
    fn f64_mul_rsi(F64Mul_Rsi) = wasm::f64_mul;
    fn f64_mul_rir(F64Mul_Rir) = wasm::f64_mul;
    fn f64_mul_ris(F64Mul_Ris) = wasm::f64_mul;
    fn f64_div_rrs(F64Div_Rrs) = wasm::f64_div;
    fn f64_div_rri(F64Div_Rri) = wasm::f64_div;
    fn f64_div_rsr(F64Div_Rsr) = wasm::f64_div;
    fn f64_div_rss(F64Div_Rss) = wasm::f64_div;
    fn f64_div_rsi(F64Div_Rsi) = wasm::f64_div;
    fn f64_div_rir(F64Div_Rir) = wasm::f64_div;
    fn f64_div_ris(F64Div_Ris) = wasm::f64_div;
    fn f64_min_rrs(F64Min_Rrs) = wasm::f64_min;
    fn f64_min_rri(F64Min_Rri) = wasm::f64_min;
    fn f64_min_rsr(F64Min_Rsr) = wasm::f64_min;
    fn f64_min_rss(F64Min_Rss) = wasm::f64_min;
    fn f64_min_rsi(F64Min_Rsi) = wasm::f64_min;
    fn f64_min_rir(F64Min_Rir) = wasm::f64_min;
    fn f64_min_ris(F64Min_Ris) = wasm::f64_min;
    fn f64_max_rrs(F64Max_Rrs) = wasm::f64_max;
    fn f64_max_rri(F64Max_Rri) = wasm::f64_max;
    fn f64_max_rsr(F64Max_Rsr) = wasm::f64_max;
    fn f64_max_rss(F64Max_Rss) = wasm::f64_max;
    fn f64_max_rsi(F64Max_Rsi) = wasm::f64_max;
    fn f64_max_rir(F64Max_Rir) = wasm::f64_max;
    fn f64_max_ris(F64Max_Ris) = wasm::f64_max;
    fn f64_copysign_rrs(F64Copysign_Rrs) = wasm::f64_copysign;
    fn f64_copysign_rsr(F64Copysign_Rsr) = wasm::f64_copysign;
    fn f64_copysign_rss(F64Copysign_Rss) = wasm::f64_copysign;
    fn f64_copysign_rir(F64Copysign_Rir) = wasm::f64_copysign;
    fn f64_copysign_ris(F64Copysign_Ris) = wasm::f64_copysign;
    // f64: comparisons
    fn f64_eq_rrs(F64Eq_Rrs) = wasm::f64_eq;
    fn f64_eq_rri(F64Eq_Rri) = wasm::f64_eq;
    fn f64_eq_rss(F64Eq_Rss) = wasm::f64_eq;
    fn f64_eq_rsi(F64Eq_Rsi) = wasm::f64_eq;
    fn f64_lt_rrs(F64Lt_Rrs) = wasm::f64_lt;
    fn f64_lt_rri(F64Lt_Rri) = wasm::f64_lt;
    fn f64_lt_rsr(F64Lt_Rsr) = wasm::f64_lt;
    fn f64_lt_rss(F64Lt_Rss) = wasm::f64_lt;
    fn f64_lt_rsi(F64Lt_Rsi) = wasm::f64_lt;
    fn f64_lt_rir(F64Lt_Rir) = wasm::f64_lt;
    fn f64_lt_ris(F64Lt_Ris) = wasm::f64_lt;
    fn f64_le_rrs(F64Le_Rrs) = wasm::f64_le;
    fn f64_le_rri(F64Le_Rri) = wasm::f64_le;
    fn f64_le_rsr(F64Le_Rsr) = wasm::f64_le;
    fn f64_le_rss(F64Le_Rss) = wasm::f64_le;
    fn f64_le_rsi(F64Le_Rsi) = wasm::f64_le;
    fn f64_le_rir(F64Le_Rir) = wasm::f64_le;
    fn f64_le_ris(F64Le_Ris) = wasm::f64_le;
    fn f64_not_eq_rrs(F64NotEq_Rrs) = wasm::f64_ne;
    fn f64_not_eq_rri(F64NotEq_Rri) = wasm::f64_ne;
    fn f64_not_eq_rss(F64NotEq_Rss) = wasm::f64_ne;
    fn f64_not_eq_rsi(F64NotEq_Rsi) = wasm::f64_ne;
    fn f64_not_lt_rrs(F64NotLt_Rrs) = eval::wasmi_f64_not_lt;
    fn f64_not_lt_rri(F64NotLt_Rri) = eval::wasmi_f64_not_lt;
    fn f64_not_lt_rsr(F64NotLt_Rsr) = eval::wasmi_f64_not_lt;
    fn f64_not_lt_rss(F64NotLt_Rss) = eval::wasmi_f64_not_lt;
    fn f64_not_lt_rsi(F64NotLt_Rsi) = eval::wasmi_f64_not_lt;
    fn f64_not_lt_rir(F64NotLt_Rir) = eval::wasmi_f64_not_lt;
    fn f64_not_lt_ris(F64NotLt_Ris) = eval::wasmi_f64_not_lt;
    fn f64_not_le_rrs(F64NotLe_Rrs) = eval::wasmi_f64_not_le;
    fn f64_not_le_rri(F64NotLe_Rri) = eval::wasmi_f64_not_le;
    fn f64_not_le_rsr(F64NotLe_Rsr) = eval::wasmi_f64_not_le;
    fn f64_not_le_rss(F64NotLe_Rss) = eval::wasmi_f64_not_le;
    fn f64_not_le_rsi(F64NotLe_Rsi) = eval::wasmi_f64_not_le;
    fn f64_not_le_rir(F64NotLe_Rir) = eval::wasmi_f64_not_le;
    fn f64_not_le_ris(F64NotLe_Ris) = eval::wasmi_f64_not_le;
}

macro_rules! handler_cmp_branch {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (next_ip, $crate::ir::decode::$decode { offset, lhs, rhs }) = unsafe { decode_op(ip) };
                    let lhs = get_value(lhs, sp, ireg, freg32, freg64);
                    let rhs = get_value(rhs, sp, ireg, freg32, freg64);
                    let ip = match $eval(lhs, rhs) {
                        true => offset_ip(ip, offset),
                        false => next_ip,
                    };
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
handler_cmp_branch! {
    // i32

    fn branch_i32_eq_rs(BranchI32Eq_Rs) = wasm::i32_eq;
    fn branch_i32_eq_ri(BranchI32Eq_Ri) = wasm::i32_eq;
    fn branch_i32_eq_ss(BranchI32Eq_Ss) = wasm::i32_eq;
    fn branch_i32_eq_si(BranchI32Eq_Si) = wasm::i32_eq;
    fn branch_i32_and_rs(BranchI32And_Rs) = eval::wasmi_i32_and;
    fn branch_i32_and_ri(BranchI32And_Ri) = eval::wasmi_i32_and;
    fn branch_i32_and_ss(BranchI32And_Ss) = eval::wasmi_i32_and;
    fn branch_i32_and_si(BranchI32And_Si) = eval::wasmi_i32_and;
    fn branch_i32_or_rs(BranchI32Or_Rs) = eval::wasmi_i32_or;
    fn branch_i32_or_ri(BranchI32Or_Ri) = eval::wasmi_i32_or;
    fn branch_i32_or_ss(BranchI32Or_Ss) = eval::wasmi_i32_or;
    fn branch_i32_or_si(BranchI32Or_Si) = eval::wasmi_i32_or;
    fn branch_i32_not_eq_rs(BranchI32NotEq_Rs) = wasm::i32_ne;
    fn branch_i32_not_eq_ri(BranchI32NotEq_Ri) = wasm::i32_ne;
    fn branch_i32_not_eq_ss(BranchI32NotEq_Ss) = wasm::i32_ne;
    fn branch_i32_not_eq_si(BranchI32NotEq_Si) = wasm::i32_ne;
    fn branch_i32_not_and_rs(BranchI32NotAnd_Rs) = eval::wasmi_i32_not_and;
    fn branch_i32_not_and_ri(BranchI32NotAnd_Ri) = eval::wasmi_i32_not_and;
    fn branch_i32_not_and_ss(BranchI32NotAnd_Ss) = eval::wasmi_i32_not_and;
    fn branch_i32_not_and_si(BranchI32NotAnd_Si) = eval::wasmi_i32_not_and;
    fn branch_i32_not_or_rs(BranchI32NotOr_Rs) = eval::wasmi_i32_not_or;
    fn branch_i32_not_or_ri(BranchI32NotOr_Ri) = eval::wasmi_i32_not_or;
    fn branch_i32_not_or_ss(BranchI32NotOr_Ss) = eval::wasmi_i32_not_or;
    fn branch_i32_not_or_si(BranchI32NotOr_Si) = eval::wasmi_i32_not_or;

    fn branch_i32_le_rs(BranchI32Le_Rs) = wasm::i32_le_s;
    fn branch_i32_le_ri(BranchI32Le_Ri) = wasm::i32_le_s;
    fn branch_i32_le_sr(BranchI32Le_Sr) = wasm::i32_le_s;
    fn branch_i32_le_ss(BranchI32Le_Ss) = wasm::i32_le_s;
    fn branch_i32_le_si(BranchI32Le_Si) = wasm::i32_le_s;
    fn branch_i32_le_ir(BranchI32Le_Ir) = wasm::i32_le_s;
    fn branch_i32_le_is(BranchI32Le_Is) = wasm::i32_le_s;

    fn branch_i32_lt_rs(BranchI32Lt_Rs) = wasm::i32_lt_s;
    fn branch_i32_lt_ri(BranchI32Lt_Ri) = wasm::i32_lt_s;
    fn branch_i32_lt_sr(BranchI32Lt_Sr) = wasm::i32_lt_s;
    fn branch_i32_lt_ss(BranchI32Lt_Ss) = wasm::i32_lt_s;
    fn branch_i32_lt_si(BranchI32Lt_Si) = wasm::i32_lt_s;
    fn branch_i32_lt_ir(BranchI32Lt_Ir) = wasm::i32_lt_s;
    fn branch_i32_lt_is(BranchI32Lt_Is) = wasm::i32_lt_s;

    fn branch_u32_le_rs(BranchU32Le_Rs) = wasm::i32_le_u;
    fn branch_u32_le_ri(BranchU32Le_Ri) = wasm::i32_le_u;
    fn branch_u32_le_sr(BranchU32Le_Sr) = wasm::i32_le_u;
    fn branch_u32_le_ss(BranchU32Le_Ss) = wasm::i32_le_u;
    fn branch_u32_le_si(BranchU32Le_Si) = wasm::i32_le_u;
    fn branch_u32_le_ir(BranchU32Le_Ir) = wasm::i32_le_u;
    fn branch_u32_le_is(BranchU32Le_Is) = wasm::i32_le_u;

    fn branch_u32_lt_rs(BranchU32Lt_Rs) = wasm::i32_lt_u;
    fn branch_u32_lt_ri(BranchU32Lt_Ri) = wasm::i32_lt_u;
    fn branch_u32_lt_sr(BranchU32Lt_Sr) = wasm::i32_lt_u;
    fn branch_u32_lt_ss(BranchU32Lt_Ss) = wasm::i32_lt_u;
    fn branch_u32_lt_si(BranchU32Lt_Si) = wasm::i32_lt_u;
    fn branch_u32_lt_ir(BranchU32Lt_Ir) = wasm::i32_lt_u;
    fn branch_u32_lt_is(BranchU32Lt_Is) = wasm::i32_lt_u;

    // i64

    fn branch_i64_eq_rs(BranchI64Eq_Rs) = wasm::i64_eq;
    fn branch_i64_eq_ri(BranchI64Eq_Ri) = wasm::i64_eq;
    fn branch_i64_eq_ss(BranchI64Eq_Ss) = wasm::i64_eq;
    fn branch_i64_eq_si(BranchI64Eq_Si) = wasm::i64_eq;
    fn branch_i64_and_rs(BranchI64And_Rs) = eval::wasmi_i64_and;
    fn branch_i64_and_ri(BranchI64And_Ri) = eval::wasmi_i64_and;
    fn branch_i64_and_ss(BranchI64And_Ss) = eval::wasmi_i64_and;
    fn branch_i64_and_si(BranchI64And_Si) = eval::wasmi_i64_and;
    fn branch_i64_or_rs(BranchI64Or_Rs) = eval::wasmi_i64_or;
    fn branch_i64_or_ri(BranchI64Or_Ri) = eval::wasmi_i64_or;
    fn branch_i64_or_ss(BranchI64Or_Ss) = eval::wasmi_i64_or;
    fn branch_i64_or_si(BranchI64Or_Si) = eval::wasmi_i64_or;
    fn branch_i64_not_eq_rs(BranchI64NotEq_Rs) = wasm::i64_ne;
    fn branch_i64_not_eq_ri(BranchI64NotEq_Ri) = wasm::i64_ne;
    fn branch_i64_not_eq_ss(BranchI64NotEq_Ss) = wasm::i64_ne;
    fn branch_i64_not_eq_si(BranchI64NotEq_Si) = wasm::i64_ne;
    fn branch_i64_not_and_rs(BranchI64NotAnd_Rs) = eval::wasmi_i64_not_and;
    fn branch_i64_not_and_ri(BranchI64NotAnd_Ri) = eval::wasmi_i64_not_and;
    fn branch_i64_not_and_ss(BranchI64NotAnd_Ss) = eval::wasmi_i64_not_and;
    fn branch_i64_not_and_si(BranchI64NotAnd_Si) = eval::wasmi_i64_not_and;
    fn branch_i64_not_or_rs(BranchI64NotOr_Rs) = eval::wasmi_i64_not_or;
    fn branch_i64_not_or_ri(BranchI64NotOr_Ri) = eval::wasmi_i64_not_or;
    fn branch_i64_not_or_ss(BranchI64NotOr_Ss) = eval::wasmi_i64_not_or;
    fn branch_i64_not_or_si(BranchI64NotOr_Si) = eval::wasmi_i64_not_or;

    fn branch_i64_le_rs(BranchI64Le_Rs) = wasm::i64_le_s;
    fn branch_i64_le_ri(BranchI64Le_Ri) = wasm::i64_le_s;
    fn branch_i64_le_sr(BranchI64Le_Sr) = wasm::i64_le_s;
    fn branch_i64_le_ss(BranchI64Le_Ss) = wasm::i64_le_s;
    fn branch_i64_le_si(BranchI64Le_Si) = wasm::i64_le_s;
    fn branch_i64_le_ir(BranchI64Le_Ir) = wasm::i64_le_s;
    fn branch_i64_le_is(BranchI64Le_Is) = wasm::i64_le_s;

    fn branch_i64_lt_rs(BranchI64Lt_Rs) = wasm::i64_lt_s;
    fn branch_i64_lt_ri(BranchI64Lt_Ri) = wasm::i64_lt_s;
    fn branch_i64_lt_sr(BranchI64Lt_Sr) = wasm::i64_lt_s;
    fn branch_i64_lt_ss(BranchI64Lt_Ss) = wasm::i64_lt_s;
    fn branch_i64_lt_si(BranchI64Lt_Si) = wasm::i64_lt_s;
    fn branch_i64_lt_ir(BranchI64Lt_Ir) = wasm::i64_lt_s;
    fn branch_i64_lt_is(BranchI64Lt_Is) = wasm::i64_lt_s;

    fn branch_u64_le_rs(BranchU64Le_Rs) = wasm::i64_le_u;
    fn branch_u64_le_ri(BranchU64Le_Ri) = wasm::i64_le_u;
    fn branch_u64_le_sr(BranchU64Le_Sr) = wasm::i64_le_u;
    fn branch_u64_le_ss(BranchU64Le_Ss) = wasm::i64_le_u;
    fn branch_u64_le_si(BranchU64Le_Si) = wasm::i64_le_u;
    fn branch_u64_le_ir(BranchU64Le_Ir) = wasm::i64_le_u;
    fn branch_u64_le_is(BranchU64Le_Is) = wasm::i64_le_u;

    fn branch_u64_lt_rs(BranchU64Lt_Rs) = wasm::i64_lt_u;
    fn branch_u64_lt_ri(BranchU64Lt_Ri) = wasm::i64_lt_u;
    fn branch_u64_lt_sr(BranchU64Lt_Sr) = wasm::i64_lt_u;
    fn branch_u64_lt_ss(BranchU64Lt_Ss) = wasm::i64_lt_u;
    fn branch_u64_lt_si(BranchU64Lt_Si) = wasm::i64_lt_u;
    fn branch_u64_lt_ir(BranchU64Lt_Ir) = wasm::i64_lt_u;
    fn branch_u64_lt_is(BranchU64Lt_Is) = wasm::i64_lt_u;

    // f32

    fn branch_f32_eq_rs(BranchF32Eq_Rs) = wasm::f32_eq;
    fn branch_f32_eq_ri(BranchF32Eq_Ri) = wasm::f32_eq;
    fn branch_f32_eq_ss(BranchF32Eq_Ss) = wasm::f32_eq;
    fn branch_f32_eq_si(BranchF32Eq_Si) = wasm::f32_eq;

    fn branch_f32_le_rs(BranchF32Le_Rs) = wasm::f32_le;
    fn branch_f32_le_ri(BranchF32Le_Ri) = wasm::f32_le;
    fn branch_f32_le_sr(BranchF32Le_Sr) = wasm::f32_le;
    fn branch_f32_le_ss(BranchF32Le_Ss) = wasm::f32_le;
    fn branch_f32_le_si(BranchF32Le_Si) = wasm::f32_le;
    fn branch_f32_le_ir(BranchF32Le_Ir) = wasm::f32_le;
    fn branch_f32_le_is(BranchF32Le_Is) = wasm::f32_le;

    fn branch_f32_lt_rs(BranchF32Lt_Rs) = wasm::f32_lt;
    fn branch_f32_lt_ri(BranchF32Lt_Ri) = wasm::f32_lt;
    fn branch_f32_lt_sr(BranchF32Lt_Sr) = wasm::f32_lt;
    fn branch_f32_lt_ss(BranchF32Lt_Ss) = wasm::f32_lt;
    fn branch_f32_lt_si(BranchF32Lt_Si) = wasm::f32_lt;
    fn branch_f32_lt_ir(BranchF32Lt_Ir) = wasm::f32_lt;
    fn branch_f32_lt_is(BranchF32Lt_Is) = wasm::f32_lt;

    fn branch_f32_not_eq_rs(BranchF32NotEq_Rs) = wasm::f32_ne;
    fn branch_f32_not_eq_ri(BranchF32NotEq_Ri) = wasm::f32_ne;
    fn branch_f32_not_eq_ss(BranchF32NotEq_Ss) = wasm::f32_ne;
    fn branch_f32_not_eq_si(BranchF32NotEq_Si) = wasm::f32_ne;

    fn branch_f32_not_le_rs(BranchF32NotLe_Rs) = eval::wasmi_f32_not_le;
    fn branch_f32_not_le_ri(BranchF32NotLe_Ri) = eval::wasmi_f32_not_le;
    fn branch_f32_not_le_sr(BranchF32NotLe_Sr) = eval::wasmi_f32_not_le;
    fn branch_f32_not_le_ss(BranchF32NotLe_Ss) = eval::wasmi_f32_not_le;
    fn branch_f32_not_le_si(BranchF32NotLe_Si) = eval::wasmi_f32_not_le;
    fn branch_f32_not_le_ir(BranchF32NotLe_Ir) = eval::wasmi_f32_not_le;
    fn branch_f32_not_le_is(BranchF32NotLe_Is) = eval::wasmi_f32_not_le;

    fn branch_f32_not_lt_rs(BranchF32NotLt_Rs) = eval::wasmi_f32_not_lt;
    fn branch_f32_not_lt_ri(BranchF32NotLt_Ri) = eval::wasmi_f32_not_lt;
    fn branch_f32_not_lt_sr(BranchF32NotLt_Sr) = eval::wasmi_f32_not_lt;
    fn branch_f32_not_lt_ss(BranchF32NotLt_Ss) = eval::wasmi_f32_not_lt;
    fn branch_f32_not_lt_si(BranchF32NotLt_Si) = eval::wasmi_f32_not_lt;
    fn branch_f32_not_lt_ir(BranchF32NotLt_Ir) = eval::wasmi_f32_not_lt;
    fn branch_f32_not_lt_is(BranchF32NotLt_Is) = eval::wasmi_f32_not_lt;

    // f64

    fn branch_f64_eq_rs(BranchF64Eq_Rs) = wasm::f64_eq;
    fn branch_f64_eq_ri(BranchF64Eq_Ri) = wasm::f64_eq;
    fn branch_f64_eq_ss(BranchF64Eq_Ss) = wasm::f64_eq;
    fn branch_f64_eq_si(BranchF64Eq_Si) = wasm::f64_eq;

    fn branch_f64_le_rs(BranchF64Le_Rs) = wasm::f64_le;
    fn branch_f64_le_ri(BranchF64Le_Ri) = wasm::f64_le;
    fn branch_f64_le_sr(BranchF64Le_Sr) = wasm::f64_le;
    fn branch_f64_le_ss(BranchF64Le_Ss) = wasm::f64_le;
    fn branch_f64_le_si(BranchF64Le_Si) = wasm::f64_le;
    fn branch_f64_le_ir(BranchF64Le_Ir) = wasm::f64_le;
    fn branch_f64_le_is(BranchF64Le_Is) = wasm::f64_le;

    fn branch_f64_lt_rs(BranchF64Lt_Rs) = wasm::f64_lt;
    fn branch_f64_lt_ri(BranchF64Lt_Ri) = wasm::f64_lt;
    fn branch_f64_lt_sr(BranchF64Lt_Sr) = wasm::f64_lt;
    fn branch_f64_lt_ss(BranchF64Lt_Ss) = wasm::f64_lt;
    fn branch_f64_lt_si(BranchF64Lt_Si) = wasm::f64_lt;
    fn branch_f64_lt_ir(BranchF64Lt_Ir) = wasm::f64_lt;
    fn branch_f64_lt_is(BranchF64Lt_Is) = wasm::f64_lt;

    fn branch_f64_not_eq_rs(BranchF64NotEq_Rs) = wasm::f64_ne;
    fn branch_f64_not_eq_ri(BranchF64NotEq_Ri) = wasm::f64_ne;
    fn branch_f64_not_eq_ss(BranchF64NotEq_Ss) = wasm::f64_ne;
    fn branch_f64_not_eq_si(BranchF64NotEq_Si) = wasm::f64_ne;

    fn branch_f64_not_le_rs(BranchF64NotLe_Rs) = eval::wasmi_f64_not_le;
    fn branch_f64_not_le_ri(BranchF64NotLe_Ri) = eval::wasmi_f64_not_le;
    fn branch_f64_not_le_sr(BranchF64NotLe_Sr) = eval::wasmi_f64_not_le;
    fn branch_f64_not_le_ss(BranchF64NotLe_Ss) = eval::wasmi_f64_not_le;
    fn branch_f64_not_le_si(BranchF64NotLe_Si) = eval::wasmi_f64_not_le;
    fn branch_f64_not_le_ir(BranchF64NotLe_Ir) = eval::wasmi_f64_not_le;
    fn branch_f64_not_le_is(BranchF64NotLe_Is) = eval::wasmi_f64_not_le;

    fn branch_f64_not_lt_rs(BranchF64NotLt_Rs) = eval::wasmi_f64_not_lt;
    fn branch_f64_not_lt_ri(BranchF64NotLt_Ri) = eval::wasmi_f64_not_lt;
    fn branch_f64_not_lt_sr(BranchF64NotLt_Sr) = eval::wasmi_f64_not_lt;
    fn branch_f64_not_lt_ss(BranchF64NotLt_Ss) = eval::wasmi_f64_not_lt;
    fn branch_f64_not_lt_si(BranchF64NotLt_Si) = eval::wasmi_f64_not_lt;
    fn branch_f64_not_lt_ir(BranchF64NotLt_Ir) = eval::wasmi_f64_not_lt;
    fn branch_f64_not_lt_is(BranchF64NotLt_Is) = eval::wasmi_f64_not_lt;
}

macro_rules! handler_select {
    ( $( fn $handler:ident($decode:ident) = $width:ty );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (
                        ip,
                        $crate::ir::decode::$decode {
                            result,
                            condition,
                            true_val,
                            false_val,
                        },
                    ) = unsafe { decode_op(ip) };
                    let condition: i32 = get_value(condition, sp, ireg, freg32, freg64);
                    let true_val: $width = get_value(true_val, sp, ireg, freg32, freg64);
                    let false_val: $width = get_value(false_val, sp, ireg, freg32, freg64);
                    let selected = match condition {
                        0 => get_value(false_val, sp, ireg, freg32, freg64),
                        _ => get_value(true_val, sp, ireg, freg32, freg64),
                    };
                    set_value!(result, selected, sp, ireg, freg32, freg64);
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
handler_select! {
    // u32
    // fn u32_select_rrrs(U32Select_Rrrs) = u32; - subsumed by u64_select_rrrs
    fn u32_select_rrri(U32Select_Rrri) = u32;
    // fn u32_select_rrsr(U32Select_Rrsr) = u32; - subsumed by u64_select_rrsr
    fn u32_select_rrir(U32Select_Rrir) = u32;
    // fn u32_select_rrss(U32Select_Rrss) = u32; - subsumed by u64_select_rrss
    fn u32_select_rrsi(U32Select_Rrsi) = u32;
    fn u32_select_rris(U32Select_Rris) = u32;
    fn u32_select_rrii(U32Select_Rrii) = u32;
    // fn u32_select_rsrs(U32Select_Rsrs) = u32; - subsumed by u64_select_rsrs
    fn u32_select_rsri(U32Select_Rsri) = u32;
    // fn u32_select_rssr(U32Select_Rssr) = u32; - subsumed by u64_select_rssr
    // fn u32_select_rsss(U32Select_Rsss) = u32; - subsumed by u64_select_rsss
    fn u32_select_rssi(U32Select_Rssi) = u32;
    fn u32_select_rsir(U32Select_Rsir) = u32;
    fn u32_select_rsis(U32Select_Rsis) = u32;
    fn u32_select_rsii(U32Select_Rsii) = u32;
    // u64
    fn u64_select_rrrs(U64Select_Rrrs) = u64;
    fn u64_select_rrri(U64Select_Rrri) = u64;
    fn u64_select_rrsr(U64Select_Rrsr) = u64;
    fn u64_select_rrir(U64Select_Rrir) = u64;
    fn u64_select_rrss(U64Select_Rrss) = u64;
    fn u64_select_rrsi(U64Select_Rrsi) = u64;
    fn u64_select_rris(U64Select_Rris) = u64;
    fn u64_select_rrii(U64Select_Rrii) = u64;
    fn u64_select_rsrs(U64Select_Rsrs) = u64;
    fn u64_select_rsri(U64Select_Rsri) = u64;
    fn u64_select_rssr(U64Select_Rssr) = u64;
    fn u64_select_rsss(U64Select_Rsss) = u64;
    fn u64_select_rssi(U64Select_Rssi) = u64;
    fn u64_select_rsir(U64Select_Rsir) = u64;
    fn u64_select_rsis(U64Select_Rsis) = u64;
    fn u64_select_rsii(U64Select_Rsii) = u64;
    // f32
    fn f32_select_rrrs(F32Select_Rrrs) = f32;
    fn f32_select_rrri(F32Select_Rrri) = f32;
    fn f32_select_rrsr(F32Select_Rrsr) = f32;
    fn f32_select_rrss(F32Select_Rrss) = f32;
    fn f32_select_rrsi(F32Select_Rrsi) = f32;
    fn f32_select_rrir(F32Select_Rrir) = f32;
    fn f32_select_rris(F32Select_Rris) = f32;
    fn f32_select_rrii(F32Select_Rrii) = f32;
    fn f32_select_rsrs(F32Select_Rsrs) = f32;
    fn f32_select_rsri(F32Select_Rsri) = f32;
    fn f32_select_rssr(F32Select_Rssr) = f32;
    fn f32_select_rsss(F32Select_Rsss) = f32;
    fn f32_select_rssi(F32Select_Rssi) = f32;
    fn f32_select_rsir(F32Select_Rsir) = f32;
    fn f32_select_rsis(F32Select_Rsis) = f32;
    fn f32_select_rsii(F32Select_Rsii) = f32;
    // f64
    fn f64_select_rrrs(F64Select_Rrrs) = f64;
    fn f64_select_rrri(F64Select_Rrri) = f64;
    fn f64_select_rrsr(F64Select_Rrsr) = f64;
    fn f64_select_rrss(F64Select_Rrss) = f64;
    fn f64_select_rrsi(F64Select_Rrsi) = f64;
    fn f64_select_rrir(F64Select_Rrir) = f64;
    fn f64_select_rris(F64Select_Rris) = f64;
    fn f64_select_rrii(F64Select_Rrii) = f64;
    fn f64_select_rsrs(F64Select_Rsrs) = f64;
    fn f64_select_rsri(F64Select_Rsri) = f64;
    fn f64_select_rssr(F64Select_Rssr) = f64;
    fn f64_select_rsss(F64Select_Rsss) = f64;
    fn f64_select_rssi(F64Select_Rssi) = f64;
    fn f64_select_rsir(F64Select_Rsir) = f64;
    fn f64_select_rsis(F64Select_Rsis) = f64;
    fn f64_select_rsii(F64Select_Rsii) = f64;
}

handler_load! {
    fn u32_load_rr(U32Load_Rr) = wasm::load_u32;
    fn u32_load_rs(U32Load_Rs) = wasm::load_u32;
    fn u64_load_rr(U64Load_Rr) = wasm::load_u64;
    fn u64_load_rs(U64Load_Rs) = wasm::load_u64;
    fn f32_load_rr(F32Load_Rr) = wasm::load_f32;
    fn f32_load_rs(F32Load_Rs) = wasm::load_f32;
    fn f64_load_rr(F64Load_Rr) = wasm::load_f64;
    fn f64_load_rs(F64Load_Rs) = wasm::load_f64;
    fn i32_load_extend8_rr(I32LoadExtend8_Rr) = wasm::i32_load8_s;
    fn i32_load_extend8_rs(I32LoadExtend8_Rs) = wasm::i32_load8_s;
    fn u32_load_extend8_rr(U32LoadExtend8_Rr) = wasm::i32_load8_u;
    fn u32_load_extend8_rs(U32LoadExtend8_Rs) = wasm::i32_load8_u;
    fn i32_load_extend16_rr(I32LoadExtend16_Rr) = wasm::i32_load16_s;
    fn i32_load_extend16_rs(I32LoadExtend16_Rs) = wasm::i32_load16_s;
    fn u32_load_extend16_rr(U32LoadExtend16_Rr) = wasm::i32_load16_u;
    fn u32_load_extend16_rs(U32LoadExtend16_Rs) = wasm::i32_load16_u;
    fn i64_load_extend8_rr(I64LoadExtend8_Rr) = wasm::i64_load8_s;
    fn i64_load_extend8_rs(I64LoadExtend8_Rs) = wasm::i64_load8_s;
    fn u64_load_extend8_rr(U64LoadExtend8_Rr) = wasm::i64_load8_u;
    fn u64_load_extend8_rs(U64LoadExtend8_Rs) = wasm::i64_load8_u;
    fn i64_load_extend16_rr(I64LoadExtend16_Rr) = wasm::i64_load16_s;
    fn i64_load_extend16_rs(I64LoadExtend16_Rs) = wasm::i64_load16_s;
    fn u64_load_extend16_rr(U64LoadExtend16_Rr) = wasm::i64_load16_u;
    fn u64_load_extend16_rs(U64LoadExtend16_Rs) = wasm::i64_load16_u;
    fn i64_load_extend32_rr(I64LoadExtend32_Rr) = wasm::i64_load32_s;
    fn i64_load_extend32_rs(I64LoadExtend32_Rs) = wasm::i64_load32_s;
    fn u64_load_extend32_rr(U64LoadExtend32_Rr) = wasm::i64_load32_u;
    fn u64_load_extend32_rs(U64LoadExtend32_Rs) = wasm::i64_load32_u;
}

macro_rules! handler_load_ri {
    ( $( fn $handler:ident($decode:ident) = $load:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (
                        ip,
                        crate::ir::decode::$decode {
                            result,
                            address,
                            memory,
                        },
                    ) = unsafe { decode_op(ip) };
                    let address = get_value(address, sp, ireg, freg32, freg64);
                    let mem_bytes = memory_bytes(memory, mem0, mem0_len, instance, state);
                    let loaded = $load(mem_bytes, usize::from(address)).into_control()?;
                    set_value!(result, loaded, sp, ireg, freg32, freg64);
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
handler_load_ri! {
    fn u32_load_ri(U32Load_Ri) = wasm::load_u32_at;
    fn u64_load_ri(U64Load_Ri) = wasm::load_u64_at;
    fn f32_load_ri(F32Load_Ri) = wasm::load_f32_at;
    fn f64_load_ri(F64Load_Ri) = wasm::load_f64_at;
    fn i32_load_extend8_ri(I32LoadExtend8_Ri) = wasm::i32_load8_s_at;
    fn u32_load_extend8_ri(U32LoadExtend8_Ri) = wasm::i32_load8_u_at;
    fn i32_load_extend16_ri(I32LoadExtend16_Ri) = wasm::i32_load16_s_at;
    fn u32_load_extend16_ri(U32LoadExtend16_Ri) = wasm::i32_load16_u_at;
    fn i64_load_extend8_ri(I64LoadExtend8_Ri) = wasm::i64_load8_s_at;
    fn u64_load_extend8_ri(U64LoadExtend8_Ri) = wasm::i64_load8_u_at;
    fn i64_load_extend16_ri(I64LoadExtend16_Ri) = wasm::i64_load16_s_at;
    fn u64_load_extend16_ri(U64LoadExtend16_Ri) = wasm::i64_load16_u_at;
    fn i64_load_extend32_ri(I64LoadExtend32_Ri) = wasm::i64_load32_s_at;
    fn u64_load_extend32_ri(U64LoadExtend32_Ri) = wasm::i64_load32_u_at;
}

handler_load_mem0_offset16_ss! {
    // reg results
    fn u32_load_mem0_offset16_rr(U32LoadMem0Offset16_Rr) = wasm::load_u32;
    fn u32_load_mem0_offset16_rs(U32LoadMem0Offset16_Rs) = wasm::load_u32;
    fn u64_load_mem0_offset16_rr(U64LoadMem0Offset16_Rr) = wasm::load_u64;
    fn u64_load_mem0_offset16_rs(U64LoadMem0Offset16_Rs) = wasm::load_u64;
    fn f32_load_mem0_offset16_rr(F32LoadMem0Offset16_Rr) = wasm::load_f32;
    fn f32_load_mem0_offset16_rs(F32LoadMem0Offset16_Rs) = wasm::load_f32;
    fn f64_load_mem0_offset16_rr(F64LoadMem0Offset16_Rr) = wasm::load_f64;
    fn f64_load_mem0_offset16_rs(F64LoadMem0Offset16_Rs) = wasm::load_f64;
    fn i32_load_extend8_mem0_offset16_rr(I32LoadExtend8Mem0Offset16_Rr) = wasm::i32_load8_s;
    fn i32_load_extend8_mem0_offset16_rs(I32LoadExtend8Mem0Offset16_Rs) = wasm::i32_load8_s;
    fn u32_load_extend8_mem0_offset16_rr(U32LoadExtend8Mem0Offset16_Rr) = wasm::i32_load8_u;
    fn u32_load_extend8_mem0_offset16_rs(U32LoadExtend8Mem0Offset16_Rs) = wasm::i32_load8_u;
    fn i32_load_extend16_mem0_offset16_rr(I32LoadExtend16Mem0Offset16_Rr) = wasm::i32_load16_s;
    fn i32_load_extend16_mem0_offset16_rs(I32LoadExtend16Mem0Offset16_Rs) = wasm::i32_load16_s;
    fn u32_load_extend16_mem0_offset16_rr(U32LoadExtend16Mem0Offset16_Rr) = wasm::i32_load16_u;
    fn u32_load_extend16_mem0_offset16_rs(U32LoadExtend16Mem0Offset16_Rs) = wasm::i32_load16_u;
    fn i64_load_extend8_mem0_offset16_rr(I64LoadExtend8Mem0Offset16_Rr) = wasm::i64_load8_s;
    fn i64_load_extend8_mem0_offset16_rs(I64LoadExtend8Mem0Offset16_Rs) = wasm::i64_load8_s;
    fn u64_load_extend8_mem0_offset16_rr(U64LoadExtend8Mem0Offset16_Rr) = wasm::i64_load8_u;
    fn u64_load_extend8_mem0_offset16_rs(U64LoadExtend8Mem0Offset16_Rs) = wasm::i64_load8_u;
    fn i64_load_extend16_mem0_offset16_rr(I64LoadExtend16Mem0Offset16_Rr) = wasm::i64_load16_s;
    fn i64_load_extend16_mem0_offset16_rs(I64LoadExtend16Mem0Offset16_Rs) = wasm::i64_load16_s;
    fn u64_load_extend16_mem0_offset16_rr(U64LoadExtend16Mem0Offset16_Rr) = wasm::i64_load16_u;
    fn u64_load_extend16_mem0_offset16_rs(U64LoadExtend16Mem0Offset16_Rs) = wasm::i64_load16_u;
    fn i64_load_extend32_mem0_offset16_rr(I64LoadExtend32Mem0Offset16_Rr) = wasm::i64_load32_s;
    fn i64_load_extend32_mem0_offset16_rs(I64LoadExtend32Mem0Offset16_Rs) = wasm::i64_load32_s;
    fn u64_load_extend32_mem0_offset16_rr(U64LoadExtend32Mem0Offset16_Rr) = wasm::i64_load32_u;
    fn u64_load_extend32_mem0_offset16_rs(U64LoadExtend32Mem0Offset16_Rs) = wasm::i64_load32_u;
    // reg + slot results
    fn u32_load_mem0_offset16_rs_r(U32LoadMem0Offset16_Rs_r) = wasm::load_u32;
    fn u32_load_mem0_offset16_rs_s(U32LoadMem0Offset16_Rs_s) = wasm::load_u32;
    fn u64_load_mem0_offset16_rs_r(U64LoadMem0Offset16_Rs_r) = wasm::load_u64;
    fn u64_load_mem0_offset16_rs_s(U64LoadMem0Offset16_Rs_s) = wasm::load_u64;
    fn f32_load_mem0_offset16_rs_r(F32LoadMem0Offset16_Rs_r) = wasm::load_f32;
    fn f32_load_mem0_offset16_rs_s(F32LoadMem0Offset16_Rs_s) = wasm::load_f32;
    fn f64_load_mem0_offset16_rs_r(F64LoadMem0Offset16_Rs_r) = wasm::load_f64;
    fn f64_load_mem0_offset16_rs_s(F64LoadMem0Offset16_Rs_s) = wasm::load_f64;
    fn i32_load_extend8_mem0_offset16_rs_r(I32LoadExtend8Mem0Offset16_Rs_r) = wasm::i32_load8_s;
    fn i32_load_extend8_mem0_offset16_rs_s(I32LoadExtend8Mem0Offset16_Rs_s) = wasm::i32_load8_s;
    fn i32_load_extend16_mem0_offset16_rs_r(I32LoadExtend16Mem0Offset16_Rs_r) = wasm::i32_load8_u;
    fn i32_load_extend16_mem0_offset16_rs_s(I32LoadExtend16Mem0Offset16_Rs_s) = wasm::i32_load8_u;
    fn u32_load_extend8_mem0_offset16_rs_r(U32LoadExtend8Mem0Offset16_Rs_r) = wasm::i32_load16_s;
    fn u32_load_extend8_mem0_offset16_rs_s(U32LoadExtend8Mem0Offset16_Rs_s) = wasm::i32_load16_s;
    fn u32_load_extend16_mem0_offset16_rs_r(U32LoadExtend16Mem0Offset16_Rs_r) = wasm::i32_load16_u;
    fn u32_load_extend16_mem0_offset16_rs_s(U32LoadExtend16Mem0Offset16_Rs_s) = wasm::i32_load16_u;
    fn i64_load_extend8_mem0_offset16_rs_r(I64LoadExtend8Mem0Offset16_Rs_r) = wasm::i64_load8_s;
    fn i64_load_extend8_mem0_offset16_rs_s(I64LoadExtend8Mem0Offset16_Rs_s) = wasm::i64_load8_s;
    fn i64_load_extend16_mem0_offset16_rs_r(I64LoadExtend16Mem0Offset16_Rs_r) = wasm::i64_load8_u;
    fn i64_load_extend16_mem0_offset16_rs_s(I64LoadExtend16Mem0Offset16_Rs_s) = wasm::i64_load8_u;
    fn i64_load_extend32_mem0_offset16_rs_r(I64LoadExtend32Mem0Offset16_Rs_r) = wasm::i64_load16_s;
    fn i64_load_extend32_mem0_offset16_rs_s(I64LoadExtend32Mem0Offset16_Rs_s) = wasm::i64_load16_s;
    fn u64_load_extend8_mem0_offset16_rs_r(U64LoadExtend8Mem0Offset16_Rs_r) = wasm::i64_load16_u;
    fn u64_load_extend8_mem0_offset16_rs_s(U64LoadExtend8Mem0Offset16_Rs_s) = wasm::i64_load16_u;
    fn u64_load_extend16_mem0_offset16_rs_r(U64LoadExtend16Mem0Offset16_Rs_r) = wasm::i64_load32_s;
    fn u64_load_extend16_mem0_offset16_rs_s(U64LoadExtend16Mem0Offset16_Rs_s) = wasm::i64_load32_s;
    fn u64_load_extend32_mem0_offset16_rs_r(U64LoadExtend32Mem0Offset16_Rs_r) = wasm::i64_load32_u;
    fn u64_load_extend32_mem0_offset16_rs_s(U64LoadExtend32Mem0Offset16_Rs_s) = wasm::i64_load32_u;
}

handler_store_sx! {
    fn u32_store_rs(U32Store_Rs, u32) = wasm::store32;
    fn u32_store_ri(U32Store_Ri, u32) = wasm::store32;
    fn u32_store_sr(U32Store_Sr, u32) = wasm::store32;
    fn u32_store_ss(U32Store_Ss, u32) = wasm::store32;
    fn u32_store_si(U32Store_Si, u32) = wasm::store32;

    fn u64_store_rs(U64Store_Rs, u64) = wasm::store64;
    fn u64_store_ri(U64Store_Ri, u64) = wasm::store64;
    fn u64_store_sr(U64Store_Sr, u64) = wasm::store64;
    fn u64_store_ss(U64Store_Ss, u64) = wasm::store64;
    fn u64_store_si(U64Store_Si, u64) = wasm::store64;

    fn f32_store_rr(F32Store_Rr, f32) = wasm::store_f32;
    fn f32_store_sr(F32Store_Sr, f32) = wasm::store_f32;
    fn f64_store_rr(F64Store_Rr, f64) = wasm::store_f64;
    fn f64_store_sr(F64Store_Sr, f64) = wasm::store_f64;

    fn i32_store_wrap8_rs(I32StoreWrap8_Rs, i8) = wasm::i32_store8;
    fn i32_store_wrap8_ri(I32StoreWrap8_Ri, i8) = wasm::i32_store8;
    fn i32_store_wrap8_sr(I32StoreWrap8_Sr, i8) = wasm::i32_store8;
    fn i32_store_wrap8_ss(I32StoreWrap8_Ss, i8) = wasm::i32_store8;
    fn i32_store_wrap8_si(I32StoreWrap8_Si, i8) = wasm::i32_store8;

    fn i32_store_wrap16_rs(I32StoreWrap16_Rs, i16) = wasm::i32_store16;
    fn i32_store_wrap16_ri(I32StoreWrap16_Ri, i16) = wasm::i32_store16;
    fn i32_store_wrap16_sr(I32StoreWrap16_Sr, i16) = wasm::i32_store16;
    fn i32_store_wrap16_ss(I32StoreWrap16_Ss, i16) = wasm::i32_store16;
    fn i32_store_wrap16_si(I32StoreWrap16_Si, i16) = wasm::i32_store16;

    fn i64_store_wrap8_rs(I64StoreWrap8_Rs, i8) = wasm::i64_store8;
    fn i64_store_wrap8_ri(I64StoreWrap8_Ri, i8) = wasm::i64_store8;
    fn i64_store_wrap8_sr(I64StoreWrap8_Sr, i8) = wasm::i64_store8;
    fn i64_store_wrap8_ss(I64StoreWrap8_Ss, i8) = wasm::i64_store8;
    fn i64_store_wrap8_si(I64StoreWrap8_Si, i8) = wasm::i64_store8;

    fn i64_store_wrap16_rs(I64StoreWrap16_Rs, i16) = wasm::i64_store16;
    fn i64_store_wrap16_ri(I64StoreWrap16_Ri, i16) = wasm::i64_store16;
    fn i64_store_wrap16_sr(I64StoreWrap16_Sr, i16) = wasm::i64_store16;
    fn i64_store_wrap16_ss(I64StoreWrap16_Ss, i16) = wasm::i64_store16;
    fn i64_store_wrap16_si(I64StoreWrap16_Si, i16) = wasm::i64_store16;

    fn i64_store_wrap32_rs(I64StoreWrap32_Rs, i32) = wasm::i64_store32;
    fn i64_store_wrap32_ri(I64StoreWrap32_Ri, i32) = wasm::i64_store32;
    fn i64_store_wrap32_sr(I64StoreWrap32_Sr, i32) = wasm::i64_store32;
    fn i64_store_wrap32_ss(I64StoreWrap32_Ss, i32) = wasm::i64_store32;
    fn i64_store_wrap32_si(I64StoreWrap32_Si, i32) = wasm::i64_store32;
}

macro_rules! handler_store_ix {
    ( $( fn $handler:ident($decode:ident, $hint:ty) = $store:expr );* $(;)? ) => {
        $(
            execution_handler! {
                fn $handler(
                    state: &mut VmState,
                    ip: Ip,
                    sp: Sp,
                    mem0: Mem0Ptr,
                    mem0_len: Mem0Len,
                    instance: Inst,
                    ireg: Ireg,
                    freg32: Freg32,
                    freg64: Freg64,
                ) -> Done = {
                    let (
                        ip,
                        crate::ir::decode::$decode {
                            address,
                            value,
                            memory,
                        },
                    ) = unsafe { decode_op(ip) };
                    let address = get_value(address, sp, ireg, freg32, freg64);
                    let value: $hint = get_value(value, sp, ireg, freg32, freg64);
                    let mem_bytes = memory_bytes(memory, mem0, mem0_len, instance, state);
                    $store(mem_bytes, usize::from(address), value.into()).into_control()?;
                    dispatch!(state, ip, sp, mem0, mem0_len, instance, ireg, freg32, freg64)
                }
            }
        )*
    };
}
handler_store_ix! {
    fn u32_store_ir(U32Store_Ir, u32) = wasm::store32_at;
    fn u32_store_is(U32Store_Is, u32) = wasm::store32_at;
    fn u32_store_ii(U32Store_Ii, u32) = wasm::store32_at;

    fn u64_store_ir(U64Store_Ir, u64) = wasm::store64_at;
    fn u64_store_is(U64Store_Is, u64) = wasm::store64_at;
    fn u64_store_ii(U64Store_Ii, u64) = wasm::store64_at;

    fn f32_store_ir(F32Store_Ir, f32) = wasm::store_f32_at;
    fn f64_store_ir(F64Store_Ir, f64) = wasm::store_f64_at;

    fn i32_store_wrap8_ir(I32StoreWrap8_Ir, i8) = wasm::i32_store8_at;
    fn i32_store_wrap8_is(I32StoreWrap8_Is, i8) = wasm::i32_store8_at;
    fn i32_store_wrap8_ii(I32StoreWrap8_Ii, i8) = wasm::i32_store8_at;

    fn i32_store_wrap16_ir(I32StoreWrap16_Ir, i16) = wasm::i32_store16_at;
    fn i32_store_wrap16_is(I32StoreWrap16_Is, i16) = wasm::i32_store16_at;
    fn i32_store_wrap16_ii(I32StoreWrap16_Ii, i16) = wasm::i32_store16_at;

    fn i64_store_wrap8_ir(I64StoreWrap8_Ir, i8) = wasm::i64_store8_at;
    fn i64_store_wrap8_is(I64StoreWrap8_Is, i8) = wasm::i64_store8_at;
    fn i64_store_wrap8_ii(I64StoreWrap8_Ii, i8) = wasm::i64_store8_at;

    fn i64_store_wrap16_ir(I64StoreWrap16_Ir, i16) = wasm::i64_store16_at;
    fn i64_store_wrap16_is(I64StoreWrap16_Is, i16) = wasm::i64_store16_at;
    fn i64_store_wrap16_ii(I64StoreWrap16_Ii, i16) = wasm::i64_store16_at;

    fn i64_store_wrap32_ir(I64StoreWrap32_Ir, i32) = wasm::i64_store32_at;
    fn i64_store_wrap32_is(I64StoreWrap32_Is, i32) = wasm::i64_store32_at;
    fn i64_store_wrap32_ii(I64StoreWrap32_Ii, i32) = wasm::i64_store32_at;
}

handler_store_mem0_offset16_sx! {
    fn u32_store_mem0_offset16_rs(U32StoreMem0Offset16_Rs, u32) = wasm::store32;
    fn u32_store_mem0_offset16_ri(U32StoreMem0Offset16_Ri, u32) = wasm::store32;
    fn u32_store_mem0_offset16_sr(U32StoreMem0Offset16_Sr, u32) = wasm::store32;
    fn u32_store_mem0_offset16_ss(U32StoreMem0Offset16_Ss, u32) = wasm::store32;
    fn u32_store_mem0_offset16_si(U32StoreMem0Offset16_Si, u32) = wasm::store32;

    fn u64_store_mem0_offset16_rs(U64StoreMem0Offset16_Rs, u64) = wasm::store64;
    fn u64_store_mem0_offset16_ri(U64StoreMem0Offset16_Ri, u64) = wasm::store64;
    fn u64_store_mem0_offset16_sr(U64StoreMem0Offset16_Sr, u64) = wasm::store64;
    fn u64_store_mem0_offset16_ss(U64StoreMem0Offset16_Ss, u64) = wasm::store64;
    fn u64_store_mem0_offset16_si(U64StoreMem0Offset16_Si, u64) = wasm::store64;

    fn f32_store_mem0_offset16_rr(F32StoreMem0Offset16_Rr, f32) = wasm::store_f32;
    fn f32_store_mem0_offset16_sr(F32StoreMem0Offset16_Sr, f32) = wasm::store_f32;
    fn f64_store_mem0_offset16_rr(F64StoreMem0Offset16_Rr, f64) = wasm::store_f64;
    fn f64_store_mem0_offset16_sr(F64StoreMem0Offset16_Sr, f64) = wasm::store_f64;

    fn i32_store_wrap8_mem0_offset16_rs(I32StoreWrap8Mem0Offset16_Rs, i8) = wasm::i32_store8;
    fn i32_store_wrap8_mem0_offset16_ri(I32StoreWrap8Mem0Offset16_Ri, i8) = wasm::i32_store8;
    fn i32_store_wrap8_mem0_offset16_sr(I32StoreWrap8Mem0Offset16_Sr, i8) = wasm::i32_store8;
    fn i32_store_wrap8_mem0_offset16_ss(I32StoreWrap8Mem0Offset16_Ss, i8) = wasm::i32_store8;
    fn i32_store_wrap8_mem0_offset16_si(I32StoreWrap8Mem0Offset16_Si, i8) = wasm::i32_store8;

    fn i32_store_wrap16_mem0_offset16_rs(I32StoreWrap16Mem0Offset16_Rs, i16) = wasm::i32_store16;
    fn i32_store_wrap16_mem0_offset16_ri(I32StoreWrap16Mem0Offset16_Ri, i16) = wasm::i32_store16;
    fn i32_store_wrap16_mem0_offset16_sr(I32StoreWrap16Mem0Offset16_Sr, i16) = wasm::i32_store16;
    fn i32_store_wrap16_mem0_offset16_ss(I32StoreWrap16Mem0Offset16_Ss, i16) = wasm::i32_store16;
    fn i32_store_wrap16_mem0_offset16_si(I32StoreWrap16Mem0Offset16_Si, i16) = wasm::i32_store16;

    fn i64_store_wrap8_mem0_offset16_rs(I64StoreWrap8Mem0Offset16_Rs, i8) = wasm::i64_store8;
    fn i64_store_wrap8_mem0_offset16_ri(I64StoreWrap8Mem0Offset16_Ri, i8) = wasm::i64_store8;
    fn i64_store_wrap8_mem0_offset16_sr(I64StoreWrap8Mem0Offset16_Sr, i8) = wasm::i64_store8;
    fn i64_store_wrap8_mem0_offset16_ss(I64StoreWrap8Mem0Offset16_Ss, i8) = wasm::i64_store8;
    fn i64_store_wrap8_mem0_offset16_si(I64StoreWrap8Mem0Offset16_Si, i8) = wasm::i64_store8;

    fn i64_store_wrap16_mem0_offset16_rs(I64StoreWrap16Mem0Offset16_Rs, i16) = wasm::i64_store16;
    fn i64_store_wrap16_mem0_offset16_ri(I64StoreWrap16Mem0Offset16_Ri, i16) = wasm::i64_store16;
    fn i64_store_wrap16_mem0_offset16_sr(I64StoreWrap16Mem0Offset16_Sr, i16) = wasm::i64_store16;
    fn i64_store_wrap16_mem0_offset16_ss(I64StoreWrap16Mem0Offset16_Ss, i16) = wasm::i64_store16;
    fn i64_store_wrap16_mem0_offset16_si(I64StoreWrap16Mem0Offset16_Si, i16) = wasm::i64_store16;

    fn i64_store_wrap32_mem0_offset16_rs(I64StoreWrap32Mem0Offset16_Rs, i32) = wasm::i64_store32;
    fn i64_store_wrap32_mem0_offset16_ri(I64StoreWrap32Mem0Offset16_Ri, i32) = wasm::i64_store32;
    fn i64_store_wrap32_mem0_offset16_sr(I64StoreWrap32Mem0Offset16_Sr, i32) = wasm::i64_store32;
    fn i64_store_wrap32_mem0_offset16_ss(I64StoreWrap32Mem0Offset16_Ss, i32) = wasm::i64_store32;
    fn i64_store_wrap32_mem0_offset16_si(I64StoreWrap32Mem0Offset16_Si, i32) = wasm::i64_store32;
}
