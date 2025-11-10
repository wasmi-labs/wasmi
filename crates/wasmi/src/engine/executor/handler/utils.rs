use super::state::{mem0_bytes, Inst, Ip, Mem0Len, Mem0Ptr, Sp, VmState};
use crate::{
    core::{CoreElementSegment, CoreGlobal, CoreMemory, CoreTable, ReadAs, UntypedVal, WriteAs},
    engine::{
        executor::handler::{Break, Control, Done, DoneReason},
        utils::unreachable_unchecked,
        DedupFuncType,
        EngineFunc,
    },
    func::{FuncEntity, HostFuncEntity},
    instance::InstanceEntity,
    ir::{index, Address, BoundedSlotSpan, BranchOffset, Offset16, Sign, Slot, SlotSpan},
    memory::{DataSegment, DataSegmentEntity},
    store::{CallHooks, PrunedStore, StoreError, StoreInner},
    table::ElementSegment,
    Error,
    Func,
    Global,
    Instance,
    Memory,
    Ref,
    Table,
    TrapCode,
};
use core::num::NonZero;

pub trait IntoControl {
    type Value;

    fn into_control(self) -> Control<Self::Value, Break>;
}

impl<T> IntoControl for Result<T, TrapCode> {
    type Value = T;

    fn into_control(self) -> Control<Self::Value, Break> {
        match self {
            Ok(value) => Control::Continue(value),
            Err(trap_code) => Control::Break(Break::from(trap_code)),
        }
    }
}

macro_rules! impl_into_control {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl IntoControl for $ty {
                type Value = Self;

                fn into_control(self) -> Control<Self::Value, Break> {
                    Control::Continue(self)
                }
            }
        )*
    };
}
impl_into_control! {
    bool,
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
    f32, f64,
}

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
    T: Copy,
    UntypedVal: ReadAs<T>,
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
    UntypedVal: WriteAs<T>,
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

pub fn exec_return(
    state: &mut VmState,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
) -> Done {
    let Some((ip, sp, mem0, mem0_len, instance)) =
        state.stack.pop_frame(state.store, mem0, mem0_len, instance)
    else {
        // No more frames on the call stack -> break out of execution!
        done!(state, DoneReason::Return(sp))
    };
    dispatch!(state, ip, sp, mem0, mem0_len, instance)
}

pub fn exec_copy_span(sp: Sp, dst: SlotSpan, src: SlotSpan, len: u16) {
    let op = match dst.head() <= src.head() {
        true => exec_copy_span_asc,
        false => exec_copy_span_des,
    };
    op(sp, dst, src, len)
}

pub fn exec_copy_span_asc(sp: Sp, dst: SlotSpan, src: SlotSpan, len: u16) {
    debug_assert!(dst.head() <= src.head());
    let dst = dst.iter(len);
    let src = src.iter(len);
    for (dst, src) in dst.into_iter().zip(src.into_iter()) {
        let value: u64 = get_value(src, sp);
        set_value(sp, dst, value);
    }
}

pub fn exec_copy_span_des(sp: Sp, dst: SlotSpan, src: SlotSpan, len: u16) {
    debug_assert!(dst.head() >= src.head());
    let dst = dst.iter(len);
    let src = src.iter(len);
    for (dst, src) in dst.into_iter().zip(src.into_iter()).rev() {
        let value: u64 = get_value(src, sp);
        set_value(sp, dst, value);
    }
}

pub fn extract_mem0(store: &mut PrunedStore, instance: Inst) -> (Mem0Ptr, Mem0Len) {
    let instance = unsafe { instance.as_ref() };
    let Some(memory) = instance.get_memory(0) else {
        return (Mem0Ptr::from([].as_mut_ptr()), Mem0Len::from(0));
    };
    let mem0 = resolve_memory_mut(store, &memory).data_mut();
    let mem0_ptr = mem0.as_mut_ptr();
    let mem0_len = mem0.len();
    (Mem0Ptr::from(mem0_ptr), Mem0Len::from(mem0_len))
}

pub fn memory_bytes<'a>(
    memory: index::Memory,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
    state: &'a mut VmState,
) -> &'a mut [u8] {
    match memory.is_default() {
        true => mem0_bytes::<'a>(mem0, mem0_len),
        false => {
            let instance = unsafe { instance.as_ref() };
            let Some(memory) = instance.get_memory(u32::from(u16::from(memory))) else {
                return &mut [];
            };
            resolve_memory_mut(state.store, &memory).data_mut()
        }
    }
}

pub fn memory_slice(memory: &CoreMemory, pos: usize, len: usize) -> Result<&[u8], TrapCode> {
    memory
        .data()
        .get(pos..)
        .and_then(|memory| memory.get(..len))
        .ok_or(TrapCode::MemoryOutOfBounds)
}

pub fn memory_slice_mut(
    memory: &mut CoreMemory,
    pos: usize,
    len: usize,
) -> Result<&mut [u8], TrapCode> {
    memory
        .data_mut()
        .get_mut(pos..)
        .and_then(|memory| memory.get_mut(..len))
        .ok_or(TrapCode::MemoryOutOfBounds)
}

pub fn offset_ip(ip: Ip, offset: BranchOffset) -> Ip {
    unsafe { ip.offset(i32::from(offset) as isize) }
}

macro_rules! impl_fetch_from_instance {
    (
        $( fn $fn:ident($param:ident: $ty:ty) -> $ret:ty = $getter:expr );* $(;)?
    ) => {
        $(
            pub fn $fn(instance: Inst, $param: $ty) -> $ret {
                let instance = unsafe { instance.as_ref() };
                let index = ::core::primitive::u32::from($param);
                let Some($param) = $getter(instance, index) else {
                    unsafe {
                        $crate::engine::utils::unreachable_unchecked!(
                            ::core::concat!("missing ", ::core::stringify!($param), " at: {:?}"),
                            index,
                        )
                    }
                };
                $param
            }
        )*
    };
}
impl_fetch_from_instance! {
    fn fetch_data(func: index::Data) -> DataSegment = InstanceEntity::get_data_segment;
    fn fetch_elem(func: index::Elem) -> ElementSegment = InstanceEntity::get_element_segment;
    fn fetch_func(func: index::Func) -> Func = InstanceEntity::get_func;
    fn fetch_global(global: index::Global) -> Global = InstanceEntity::get_global;
    fn fetch_memory(memory: index::Memory) -> Memory = InstanceEntity::get_memory;
    fn fetch_table(table: index::Table) -> Table = InstanceEntity::get_table;
    fn fetch_func_type(func_type: index::FuncType) -> DedupFuncType = {
        |instance: &InstanceEntity, index: u32| instance.get_signature(index).copied()
    };
}

macro_rules! impl_resolve_from_store {
    (
        $( fn $fn:ident($param:ident: $ty:ty) -> $ret:ty = $getter:expr );* $(;)?
    ) => {
        $(
            pub fn $fn<'a>(store: &'a mut PrunedStore, $param: $ty) -> $ret {
                match $getter(store.inner_mut(), $param) {
                    ::core::result::Result::Ok($param) => $param,
                    ::core::result::Result::Err(error) => unsafe {
                        $crate::engine::utils::unreachable_unchecked!(
                            ::core::concat!("could not resolve stored ", ::core::stringify!($param), ": {:?}"),
                            error,
                        )
                    },
                }
            }
        )*
    };
}
impl_resolve_from_store! {
    // fn resolve_elem(elem: &ElementSegment) -> &'a CoreElementSegment = StoreInner::try_resolve_element;
    fn resolve_func(func: &Func) -> &'a FuncEntity = StoreInner::try_resolve_func;
    fn resolve_global(global: &Global) -> &'a CoreGlobal = StoreInner::try_resolve_global;
    fn resolve_memory(memory: &Memory) -> &'a CoreMemory = StoreInner::try_resolve_memory;
    fn resolve_table(table: &Table) -> &'a CoreTable = StoreInner::try_resolve_table;
    fn resolve_instance(func: &Instance) -> &'a InstanceEntity = StoreInner::try_resolve_instance;
    // fn resolve_func_type(func_type: DedupFuncType) -> DedupFuncType = StoreInner::resolve_func_type;

    fn resolve_elem_mut(elem: &ElementSegment) -> &'a mut CoreElementSegment = StoreInner::try_resolve_element_mut;
    fn resolve_data_mut(data: &DataSegment) -> &'a mut DataSegmentEntity = StoreInner::try_resolve_data_mut;
    fn resolve_global_mut(global: &Global) -> &'a mut CoreGlobal = StoreInner::try_resolve_global_mut;
    fn resolve_memory_mut(memory: &Memory) -> &'a mut CoreMemory = StoreInner::try_resolve_memory_mut;
    fn resolve_table_mut(table: &Table) -> &'a mut CoreTable = StoreInner::try_resolve_table_mut;
}

pub fn resolve_indirect_func(
    index: Slot,
    table: index::Table,
    func_type: index::FuncType,
    state: &mut VmState<'_>,
    sp: Sp,
    instance: Inst,
) -> Result<Func, TrapCode> {
    let index = get_value(index, sp);
    let table = fetch_table(instance, table);
    let table = resolve_table(state.store, &table);
    let funcref = table
        .get_untyped(index)
        .map(<Ref<Func>>::from)
        .ok_or(TrapCode::TableOutOfBounds)?;
    let func = funcref.val().ok_or(TrapCode::IndirectCallToNull)?;
    let actual_fnty = resolve_func(state.store, func).ty_dedup();
    let expected_fnty = fetch_func_type(instance, func_type);
    if expected_fnty.ne(actual_fnty) {
        return Err(TrapCode::BadSignature);
    }
    Ok(*func)
}

pub fn set_global(global: index::Global, value: UntypedVal, state: &mut VmState, instance: Inst) {
    let global = fetch_global(instance, global);
    let global = resolve_global_mut(state.store, &global);
    let mut value_ptr = global.get_untyped_ptr();
    let global_ref = unsafe { value_ptr.as_mut() };
    *global_ref = value;
}

pub fn update_instance(
    store: &mut PrunedStore,
    instance: Inst,
    new_instance: Inst,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
) -> (Inst, Mem0Ptr, Mem0Len) {
    if new_instance == instance {
        return (instance, mem0, mem0_len);
    }
    let (mem0, mem0_len) = extract_mem0(store, new_instance);
    (new_instance, mem0, mem0_len)
}

pub fn compile_or_get_func(state: &mut VmState, func: EngineFunc) -> Result<(Ip, usize), Error> {
    let fuel_mut = state.store.inner_mut().fuel_mut();
    let compiled_func = state.code.get(Some(fuel_mut), func)?;
    let ip = Ip::from(compiled_func.ops());
    let size = usize::from(compiled_func.len_stack_slots());
    Ok((ip, size))
}

macro_rules! consume_fuel {
    ($state:expr, $ip:expr, $fuel:expr, $eval:expr) => {{
        if let ::core::result::Result::Err($crate::errors::FuelError::OutOfFuel { required_fuel }) =
            $fuel.consume_fuel_if($eval)
        {
            out_of_fuel!($state, $ip, required_fuel)
        }
    }};
}

macro_rules! out_of_fuel {
    ($state:expr, $ip:expr, $required_fuel:expr) => {{
        $state.stack.sync_ip($ip);
        done!(
            $state,
            $crate::engine::executor::handler::DoneReason::out_of_fuel($required_fuel),
        )
    }};
}

pub fn call_wasm(
    state: &mut VmState,
    caller_ip: Ip,
    params: BoundedSlotSpan,
    func: EngineFunc,
    instance: Option<Inst>,
) -> Control<(Ip, Sp), Break> {
    let (callee_ip, size) = compile_or_get_func!(state, func);
    let callee_sp = state
        .stack
        .push_frame(Some(caller_ip), callee_ip, params, size, instance)
        .into_control()?;
    Control::Continue((callee_ip, callee_sp))
}

pub fn return_call_wasm(
    state: &mut VmState,
    params: BoundedSlotSpan,
    func: EngineFunc,
    instance: Option<Inst>,
) -> Control<(Ip, Sp), Break> {
    let (callee_ip, size) = compile_or_get_func!(state, func);
    let callee_sp = state
        .stack
        .replace_frame(callee_ip, params, size, instance)
        .into_control()?;
    Control::Continue((callee_ip, callee_sp))
}

pub fn call_host(
    state: &mut VmState,
    func: Func,
    caller_ip: Option<Ip>,
    host_func: HostFuncEntity,
    params: BoundedSlotSpan,
    instance: Option<Inst>,
    call_hooks: CallHooks,
) -> Control<Sp, Break> {
    debug_assert_eq!(params.len(), host_func.len_params());
    let trampoline = *host_func.trampoline();
    let (sp, params_results) = state
        .stack
        .prepare_host_frame(caller_ip, params, host_func.len_results())
        .into_control()?;
    match state
        .store
        .call_host_func(trampoline, instance, params_results, call_hooks)
    {
        Ok(()) => {}
        Err(StoreError::External(error)) => {
            done!(state, DoneReason::host_error(error, func, params.span()))
        }
        Err(StoreError::Internal(error)) => unsafe {
            unreachable_unchecked!(
                "internal interpreter error while executing host function: {error}"
            )
        },
    }
    Control::Continue(sp)
}

pub fn return_call_host(
    state: &mut VmState,
    func: Func,
    host_func: HostFuncEntity,
    params: BoundedSlotSpan,
    instance: Inst,
) -> Control<(Ip, Sp, Inst), Break> {
    debug_assert_eq!(params.len(), host_func.len_params());
    let trampoline = *host_func.trampoline();
    let (control, params_results) = state
        .stack
        .return_prepare_host_frame(params, host_func.len_results(), instance)
        .into_control()?;
    match state
        .store
        .call_host_func(trampoline, Some(instance), params_results, CallHooks::Call)
    {
        Ok(()) => {}
        Err(StoreError::External(error)) => {
            done!(state, DoneReason::host_error(error, func, params.span()))
        }
        Err(StoreError::Internal(error)) => unsafe {
            unreachable_unchecked!(
                "internal interpreter error while executing host function: {error}"
            )
        },
    }
    match control {
        Control::Continue((ip, sp, instance)) => Control::Continue((ip, sp, instance)),
        Control::Break(sp) => done!(state, DoneReason::Return(sp)),
    }
}

pub fn call_wasm_or_host(
    state: &mut VmState,
    caller_ip: Ip,
    func: Func,
    params: BoundedSlotSpan,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
) -> Control<(Ip, Sp, Mem0Ptr, Mem0Len, Inst), Break> {
    let func_entity = resolve_func(state.store, &func);
    let next_state = match func_entity {
        FuncEntity::Wasm(wasm_func) => {
            let func = wasm_func.func_body();
            let callee_instance = *wasm_func.instance();
            let callee_instance: Inst = resolve_instance(state.store, &callee_instance).into();
            let (callee_ip, callee_sp) =
                call_wasm(state, caller_ip, params, func, Some(callee_instance))?;
            let (instance, mem0, mem0_len) =
                update_instance(state.store, instance, callee_instance, mem0, mem0_len);
            (callee_ip, callee_sp, mem0, mem0_len, instance)
        }
        FuncEntity::Host(host_func) => {
            let host_func = *host_func;
            let sp = call_host(
                state,
                func,
                Some(caller_ip),
                host_func,
                params,
                Some(instance),
                CallHooks::Call,
            )?;
            (caller_ip, sp, mem0, mem0_len, instance)
        }
    };
    Control::Continue(next_state)
}
