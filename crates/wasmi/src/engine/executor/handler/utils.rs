use super::state::{Ip, Sp, VmState};
use crate::{
    core::UntypedVal,
    engine::DedupFuncType,
    instance::InstanceEntity,
    ir::{index, Address, BranchOffset, Offset16, Sign, Slot, SlotSpan},
    store::PrunedStore,
    Func,
    Global,
    Ref,
    Table,
    TrapCode,
};
use core::{num::NonZero, ptr::NonNull, slice};

pub trait GetValue<T> {
    fn get_value(src: Self, sp: Sp) -> T;
}

macro_rules! impl_get_value {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl GetValue<$ty> for $ty {
                #[inline(always)]
                fn get_value(src: Self, _sp: Sp) -> $ty {
                    src
                }
            }
        )*
    };
}
impl_get_value!(
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    f32,
    f64,
    NonZero<i32>,
    NonZero<i64>,
    NonZero<u32>,
    NonZero<u64>,
    Sign<f32>,
    Sign<f64>,
    Address,
    Offset16,
);

impl<T> GetValue<T> for Slot
where
    T: Copy + From<UntypedVal>,
{
    fn get_value(src: Self, sp: Sp) -> T {
        sp.get::<T>(src)
    }
}

pub fn get_value<T, L>(src: T, sp: Sp) -> L
where
    T: GetValue<L>,
{
    <T as GetValue<L>>::get_value(src, sp)
}

pub trait SetValue<T> {
    fn set_value(src: Self, value: T, sp: Sp);
}

impl<T> SetValue<T> for Slot
where
    T: Copy + Into<UntypedVal>,
{
    fn set_value(src: Self, value: T, sp: Sp) {
        sp.set::<T>(src, value)
    }
}

pub fn set_value<T, V>(sp: Sp, src: T, value: V)
where
    T: SetValue<V>,
{
    <T as SetValue<V>>::set_value(src, value, sp)
}

pub trait ExtendToResult {
    type Value;

    fn expand_to_result(self) -> Result<Self::Value, TrapCode>;
}

impl<T> ExtendToResult for Result<T, TrapCode> {
    type Value = T;

    #[inline(always)]
    fn expand_to_result(self) -> Result<Self::Value, TrapCode> {
        self
    }
}

macro_rules! impl_expand_to_result {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ExtendToResult for $ty {
                type Value = Self;

                #[inline(always)]
                fn expand_to_result(self) -> Result<Self::Value, TrapCode> {
                    Ok(self)
                }
            }
        )*
    };
}
impl_expand_to_result!(bool, i32, i64, u32, u64, f32, f64);

macro_rules! break_if_trap {
    ($value:expr, $state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr $(,)? ) => {{
        match <_ as $crate::engine::executor::handler::utils::ExtendToResult>::expand_to_result(
            $value,
        ) {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(trap) => {
                break_with_trap!(trap, $state, $ip, $sp, $mem0, $mem0_len, $instance)
            }
        }
    }};
}

macro_rules! break_with_trap {
    ($trap:expr, $state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr $(,)? ) => {{
        $state.done_reason = DoneReason::Trap($trap);
        return exec_break!($ip, $sp, $mem0, $mem0_len, $instance);
    }};
}

macro_rules! exec_return {
    ($state:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        let Some((ip, sp, mem0, mem0_len, instance)) =
            $state
                .stack
                .pop_frame(&mut $state.store, $mem0, $mem0_len, $instance)
        else {
            // No more frames on the call stack -> break out of execution!
            $state.done_reason = DoneReason::Return;
            return Done::control_break();
        };
        dispatch!($state, ip, sp, mem0, mem0_len, instance)
    }};
}

pub fn exec_copy_span(sp: Sp, dst: SlotSpan, src: SlotSpan, len: u16) {
    let dst = dst.iter(len);
    let src = src.iter(len);
    for (dst, src) in dst.into_iter().zip(src.into_iter()) {
        let value: u64 = get_value(src, sp);
        set_value(sp, dst, value);
    }
}

pub fn extract_mem0(
    store: &mut PrunedStore,
    instance: NonNull<InstanceEntity>,
) -> (*mut u8, usize) {
    let instance = unsafe { instance.as_ref() };
    let Some(memory) = instance.get_memory(0) else {
        return ([].as_mut_ptr(), 0);
    };
    let mem0 = store.inner_mut().resolve_memory_mut(&memory).data_mut();
    let mem0_ptr = mem0.as_mut_ptr();
    let mem0_len = mem0.len();
    (mem0_ptr, mem0_len)
}

pub fn default_memory_bytes<'a>(mem0: *mut u8, mem0_len: usize) -> &'a mut [u8] {
    unsafe { slice::from_raw_parts_mut(mem0, mem0_len) }
}

pub fn memory_bytes<'a>(
    memory: index::Memory,
    mem0: *mut u8,
    mem0_len: usize,
    instance: NonNull<InstanceEntity>,
    state: &'a mut VmState,
) -> &'a mut [u8] {
    match memory.is_default() {
        true => default_memory_bytes::<'a>(mem0, mem0_len),
        false => {
            let instance = unsafe { instance.as_ref() };
            let Some(memory) = instance.get_memory(u32::from(u16::from(memory))) else {
                // unreachable!("missing memory at: {}", u16::from(memory))
                return &mut [];
            };
            state
                .store
                .inner_mut()
                .resolve_memory_mut(&memory)
                .data_mut()
        }
    }
}

pub fn offset_ip(ip: Ip, offset: BranchOffset) -> Ip {
    unsafe { ip.offset(i32::from(offset) as isize) }
}

pub fn resolve_func(instance: NonNull<InstanceEntity>, func: index::Func) -> Func {
    let inst = unsafe { instance.as_ref() };
    let Some(func) = inst.get_func(u32::from(func)) else {
        unreachable!("missing func at: {}", u32::from(func))
    };
    func
}

pub fn resolve_global(instance: NonNull<InstanceEntity>, global: index::Global) -> Global {
    let inst = unsafe { instance.as_ref() };
    let Some(global) = inst.get_global(u32::from(global)) else {
        unreachable!("missing global at: {}", u32::from(global))
    };
    global
}

pub fn resolve_table(instance: NonNull<InstanceEntity>, table: index::Table) -> Table {
    let inst = unsafe { instance.as_ref() };
    let Some(table) = inst.get_table(u32::from(table)) else {
        unreachable!("missing table at: {}", u32::from(table))
    };
    table
}

pub fn resolve_func_type_dedup(
    instance: NonNull<InstanceEntity>,
    func_type: index::FuncType,
) -> DedupFuncType {
    let inst = unsafe { instance.as_ref() };
    let Some(func_type) = inst.get_signature(u32::from(func_type)) else {
        unreachable!("missing func type at: {}", u32::from(func_type))
    };
    *func_type
}

pub fn resolve_indirect_func(
    index: Slot,
    table: index::Table,
    func_type: index::FuncType,
    state: &mut VmState<'_>,
    sp: Sp,
    instance: NonNull<InstanceEntity>,
) -> Result<Func, TrapCode> {
    let table = resolve_table(instance, table);
    let index = get_value(index, sp);
    let funcref = state
        .store
        .inner()
        .resolve_table(&table)
        .get_untyped(index)
        .map(<Ref<Func>>::from)
        .ok_or(TrapCode::TableOutOfBounds)?;
    let func = funcref.val().ok_or(TrapCode::IndirectCallToNull)?;
    let actual_signature = state.store.inner().resolve_func(func).ty_dedup();
    let expected_signature = resolve_func_type_dedup(instance, func_type);
    if expected_signature.ne(actual_signature) {
        return Err(TrapCode::BadSignature);
    }
    Ok(*func)
}

pub fn set_global(
    global: index::Global,
    value: UntypedVal,
    state: &mut VmState,
    instance: NonNull<InstanceEntity>,
) {
    let global = resolve_global(instance, global);
    let mut global_ptr = state
        .store
        .inner_mut()
        .resolve_global_mut(&global)
        .get_untyped_ptr();
    let global_ref = unsafe { global_ptr.as_mut() };
    *global_ref = value;
}

pub fn update_instance(
    store: &mut PrunedStore,
    instance: NonNull<InstanceEntity>,
    new_instance: NonNull<InstanceEntity>,
    mem0: *mut u8,
    mem0_len: usize,
) -> (NonNull<InstanceEntity>, *mut u8, usize) {
    if new_instance == instance {
        return (instance, mem0, mem0_len);
    }
    let (mem0, mem0_len) = extract_mem0(store, new_instance);
    (new_instance, mem0, mem0_len)
}
