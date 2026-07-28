use super::state::{Freg32, Freg64, Inst, Ip, Ireg, Mem0Len, Mem0Ptr, Sp, VmState};
#[cfg(feature = "simd")]
use crate::core::simd::ImmLaneIdx;
use crate::{
    Error,
    Func,
    Instance,
    Memory,
    Nullable,
    RefType,
    Table,
    TrapCode,
    V128,
    core::{
        CoreElementSegment as ElementSegmentEntity,
        CoreGlobal as GlobalEntity,
        CoreMemory as MemoryEntity,
        CoreTable as TableEntity,
        CoreTable,
        RawVal,
        ShiftAmount,
    },
    engine::{
        CodeView,
        FuncEntry,
        InOutParams,
        executor::{
            LoadFromCellsByValue,
            StoreToCells,
            handler::{Break, Control, DoneReason, args::Args},
        },
        utils::unreachable_unchecked,
    },
    func::{FuncEntity, HostFuncEntity, Trampoline},
    instance::{DataAddr, ElemAddr, FuncAddr, GlobalAddr, InstanceEntity, MemoryAddr, TableAddr},
    ir::{self, Address, BoundedSlotSpan, Local, Offset, Offset16, Slot, SlotAndReg, SlotSpan},
    memory::DataSegmentEntity,
    store::{CallHooks, PrunedStore, StoreError, StoreInner},
};
use core::{num::NonZero, ptr::NonNull};

macro_rules! out_of_fuel {
    ($state:expr, $args:expr, $required_fuel:expr) => {{
        $state.stack.sync_ip($args.ip);
        $state
            .stack
            .sync_regs($args.ireg, $args.freg32, $args.freg64);
        done!(
            $state,
            $crate::engine::executor::handler::DoneReason::out_of_fuel($required_fuel),
        )
    }};
}

macro_rules! consume_fuel {
    ($state:expr, $ip:expr, $args:expr, $fuel:expr, $eval:expr $(,)? ) => {{
        if let ::core::result::Result::Err($crate::errors::FuelError::OutOfFuel { required_fuel }) =
            $fuel.consume_fuel_if($eval)
        {
            $args.set_ip($ip);
            out_of_fuel!($state, $args, required_fuel)
        }
    }};
}

pub fn compile_or_get_func_entry(
    state: &mut VmState,
    func: &FuncEntry,
) -> Result<(Ip, u16, u16), Error> {
    let fuel_mut = state.store.inner_mut().fuel_mut();
    let features = state.code.features();
    let compiled_func = func.get_or_compile(Some(fuel_mut), features)?;
    let ip = Ip::from(compiled_func.ops());
    let len_local_slots = compiled_func.len_local_slots();
    let len_stack_slots = compiled_func.len_stack_slots();
    Ok((ip, len_local_slots, len_stack_slots))
}

macro_rules! compile_or_get_func_entry {
    ($state:expr, $func:expr) => {{
        match $crate::engine::executor::handler::utils::compile_or_get_func_entry($state, $func) {
            Ok((ip, len_local_slots, len_stack_slots)) => (ip, len_local_slots, len_stack_slots),
            Err(error) => done!($state, DoneReason::error(error)),
        }
    }};
}

macro_rules! trap {
    ($trap_code:expr) => {{
        return $crate::engine::executor::handler::Control::Break(
            $crate::engine::executor::handler::Break::from($trap_code),
        );
    }};
}

macro_rules! done {
    ($state:expr, $reason:expr $(,)? ) => {{
        $state.done_with(move || {
            <_ as ::core::convert::Into<$crate::engine::executor::handler::DoneReason>>::into(
                $reason,
            )
        });
        return $crate::engine::executor::handler::dispatch::control_break();
    }};
}

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
    V128,
    RawVal,
}

pub trait GetValue<T> {
    fn get_value(src: Self, sp: Sp, ireg: Ireg, freg32: Freg32, freg64: Freg64) -> T;
}

macro_rules! impl_get_value {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl GetValue<$ty> for $ty {
                #[inline(always)]
                fn get_value(src: Self, _sp: Sp, _ireg: Ireg, _freg32: Freg32, _freg64: Freg64) -> $ty {
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
    Address,
    Offset16,
    ShiftAmount,
    V128,
);
#[cfg(feature = "simd")]
impl_get_value!([ImmLaneIdx<32>; 16]);

impl GetValue<u64> for Offset {
    #[inline(always)]
    fn get_value(src: Self, _sp: Sp, _ireg: Ireg, _freg32: Freg32, _freg64: Freg64) -> u64 {
        u64::from(src)
    }
}

macro_rules! impl_get_value_for_ireg {
    ( $($prim:ty),* $(,)? ) => {
        $(
            impl GetValue<$prim> for ir::Reg<i64> {
                #[inline]
                fn get_value(_src: Self, _sp: Sp, ireg: Ireg, _freg32: Freg32, _freg64: Freg64) -> $prim {
                    <$prim as From<Ireg>>::from(ireg)
                }
            }
        )*
    };
}
impl_get_value_for_ireg!(bool, i8, i16, i32, i64, u8, u16, u32, u64, ShiftAmount);

impl From<Ireg> for ShiftAmount {
    #[inline]
    fn from(value: Ireg) -> Self {
        Self::from(u8::from(value))
    }
}

impl GetValue<f32> for ir::Reg<f32> {
    #[inline]
    fn get_value(_src: Self, _sp: Sp, _ireg: Ireg, freg32: Freg32, _freg64: Freg64) -> f32 {
        f32::from(freg32)
    }
}

impl GetValue<f64> for ir::Reg<f64> {
    #[inline]
    fn get_value(_src: Self, _sp: Sp, _ireg: Ireg, _freg32: Freg32, freg64: Freg64) -> f64 {
        f64::from(freg64)
    }
}

impl<const N: u16, T> GetValue<T> for Local<N>
where
    T: LoadFromCellsByValue,
{
    #[inline]
    fn get_value(_src: Self, sp: Sp, ireg: Ireg, freg32: Freg32, freg64: Freg64) -> T {
        <Slot as GetValue<T>>::get_value(Slot::from(N), sp, ireg, freg32, freg64)
    }
}

impl<T> GetValue<T> for Slot
where
    T: LoadFromCellsByValue,
{
    #[inline]
    fn get_value(src: Self, sp: Sp, _ireg: Ireg, _freg32: Freg32, _freg64: Freg64) -> T {
        // # Safety
        //
        // This implementation's correctness relies on the caller's inputs.
        //
        // - The caller needs to make sure that the offset `sp` via `src` yields
        //   a memory region that is alive and belonging to the same memory region
        //   as `sp` itself.
        // - This method (or trait) was not marked `unsafe` for ergonomic reasons
        //   since practically any direct callers of this methods cannot enforce or
        //   assert those invariants themselves.
        // - In essence the use of this method relies on the correct `decode` implementation
        //   and the correct use of `decode` implementation in all execution handlers and
        //   their callers, e.g. the interpreter loop as well as the setup routine.
        // - Marking all execution handlers as `unsafe` was another option that has been
        //   ruled out because it would yield `unsafe` blocks way too large to be effective.
        // - Therefore: this method not being marked `unsafe` was a design trade-off.
        unsafe { sp.get::<T>(src) }
    }
}

/// Returns the value at `sp[src]`.
#[inline]
pub fn get_slot_value<L>(src: Slot, sp: Sp) -> L
where
    Slot: GetValue<L>,
{
    <Slot as GetValue<L>>::get_value(
        src,
        sp,
        Ireg::default(),
        Freg32::default(),
        Freg64::default(),
    )
}

/// Returns the `src` value from its destination:
///
/// - `sp[src]` if `src` is `Slot`
/// - `src` if `src` is a primitive type, such as `i32`
/// - `ireg` if `src` is `Ireg`
/// - `freg32` if `src` is `Freg32`
/// - `freg64` if `src` is `Freg64`
#[inline]
pub fn get_value<T, L>(src: T, sp: Sp, ireg: Ireg, freg32: Freg32, freg64: Freg64) -> L
where
    T: GetValue<L>,
{
    <T as GetValue<L>>::get_value(src, sp, ireg, freg32, freg64)
}

pub trait SetValue<T> {
    #[must_use]
    fn set_value(
        dst: Self,
        src: T,
        sp: Sp,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64);
}

impl<T> SetValue<T> for ir::Reg<i64>
where
    T: Into<Ireg>,
{
    #[inline]
    fn set_value(
        _dst: Self,
        src: T,
        _sp: Sp,
        _ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64) {
        (src.into(), freg32, freg64)
    }
}

impl<T> SetValue<T> for ir::Reg<f32>
where
    T: Into<Freg32>,
{
    #[inline]
    fn set_value(
        _dst: Self,
        src: T,
        _sp: Sp,
        ireg: Ireg,
        _freg32: Freg32,
        freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64) {
        (ireg, src.into(), freg64)
    }
}

impl<T> SetValue<T> for ir::Reg<f64>
where
    T: Into<Freg64>,
{
    #[inline]
    fn set_value(
        _dst: Self,
        src: T,
        _sp: Sp,
        ireg: Ireg,
        freg32: Freg32,
        _freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64) {
        (ireg, freg32, src.into())
    }
}

impl<const N: u16, T> SetValue<T> for Local<N>
where
    T: StoreToCells,
{
    #[inline]
    fn set_value(
        _dst: Self,
        src: T,
        sp: Sp,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64) {
        <Slot as SetValue<T>>::set_value(Slot::from(N), src, sp, ireg, freg32, freg64)
    }
}

impl<T> SetValue<T> for SlotAndReg<i64>
where
    T: StoreToCells + Into<Ireg> + Copy,
{
    #[inline]
    fn set_value(
        dst: Self,
        src: T,
        sp: Sp,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64) {
        let (ireg, freg32, freg64) = set_value(dst.slot, src, sp, ireg, freg32, freg64);
        set_value(dst.reg, src, sp, ireg, freg32, freg64)
    }
}

impl<T> SetValue<T> for SlotAndReg<f32>
where
    T: StoreToCells + Into<Freg32> + Copy,
{
    #[inline]
    fn set_value(
        dst: Self,
        src: T,
        sp: Sp,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64) {
        let (ireg, freg32, freg64) = set_value(dst.slot, src, sp, ireg, freg32, freg64);
        set_value(dst.reg, src, sp, ireg, freg32, freg64)
    }
}

impl<T> SetValue<T> for SlotAndReg<f64>
where
    T: StoreToCells + Into<Freg64> + Copy,
{
    #[inline]
    fn set_value(
        dst: Self,
        src: T,
        sp: Sp,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64) {
        let (ireg, freg32, freg64) = set_value(dst.slot, src, sp, ireg, freg32, freg64);
        set_value(dst.reg, src, sp, ireg, freg32, freg64)
    }
}

impl<T> SetValue<T> for Slot
where
    T: StoreToCells,
{
    #[inline]
    fn set_value(
        dst: Self,
        src: T,
        sp: Sp,
        ireg: Ireg,
        freg32: Freg32,
        freg64: Freg64,
    ) -> (Ireg, Freg32, Freg64) {
        // # Safety
        //
        // This implementation's correctness relies on the caller's inputs.
        //
        // - The caller needs to make sure that the offset `sp` via `src` yields
        //   a memory region that is alive and belonging to the same memory region
        //   as `sp` itself.
        // - This method (or trait) was not marked `unsafe` for ergonomic reasons
        //   since practically any direct callers of this methods cannot enforce or
        //   assert those invariants themselves.
        // - In essence the use of this method relies on the correct `decode` implementation
        //   and the correct use of `decode` implementation in all execution handlers and
        //   their callers, e.g. the interpreter loop as well as the setup routine.
        // - Marking all execution handlers as `unsafe` was another option that has been
        //   ruled out because it would yield `unsafe` blocks way too large to be effective.
        // - Therefore: this method not being marked `unsafe` was a design trade-off.
        unsafe { sp.set::<T>(dst, src) };
        (ireg, freg32, freg64)
    }
}

/// Sets the value at `sp` at offset `dst` to `value`: `sp[dst] = src`
#[inline]
pub fn set_slot_value<T, V>(dst: T, src: V, sp: Sp)
where
    T: SetValue<V>,
{
    // Note: safe to drop the return value here immediate in this case.
    _ = <T as SetValue<V>>::set_value(
        dst,
        src,
        sp,
        Ireg::default(),
        Freg32::default(),
        Freg64::default(),
    );
}

/// Sets the value at `sp` at offset `dst` to `value`: `sp[dst] = src`
#[inline]
#[must_use]
pub fn set_value<T, V>(
    dst: T,
    src: V,
    sp: Sp,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
) -> (Ireg, Freg32, Freg64)
where
    T: SetValue<V>,
{
    <T as SetValue<V>>::set_value(dst, src, sp, ireg, freg32, freg64)
}

pub fn exec_copy_span(sp: Sp, dst: SlotSpan, src: SlotSpan, len: u16) {
    debug_assert_ne!(dst, src);
    debug_assert!(len > 0);
    let op = match dst.head() <= src.head() {
        true => exec_copy_span_asc,
        false => exec_copy_span_des,
    };
    op(sp, dst, src, len)
}

pub fn exec_copy_span_asc(sp: Sp, dst: SlotSpan, src: SlotSpan, len: u16) {
    debug_assert!(dst.head() < src.head());
    debug_assert!(len > 0);
    let mut dst = dst.head();
    let mut src = src.head();
    let dst_end = dst.next_n(len);
    while dst != dst_end {
        let value: u64 = get_slot_value(src, sp);
        set_slot_value(dst, value, sp);
        dst = dst.next();
        src = src.next();
    }
}

pub fn exec_copy_span_des(sp: Sp, dst: SlotSpan, src: SlotSpan, len: u16) {
    debug_assert!(dst.head() > src.head());
    debug_assert!(len > 0);
    let dst_end = dst.head();
    let mut dst = dst.head().next_n(len);
    let mut src = src.head().next_n(len);
    while dst != dst_end {
        dst = dst.prev();
        src = src.prev();
        let value: u64 = get_slot_value(src, sp);
        set_slot_value(dst, value, sp);
    }
}

/// Returns `true` if `memory` addresses the default linear memory (Wasm index 0).
#[inline]
pub fn is_default_memory(_instance: Inst, memory: ir::MemoryAddr) -> bool {
    // Note: this returns `true` even if the instance does not contain a memory.
    //       It is guaranteed that linear memories are placed first in an instance's
    //       handle buffer, therefore `MemoryAddr(0)` always refers to the default
    //       memory if one exists.
    u32::from(memory) == 0
}

pub fn extract_mem0(_store: &mut PrunedStore, inst: Inst) -> (Mem0Ptr, Mem0Len) {
    // SAFETY: `inst` refers to a live instance for the duration of the call.
    let Some(addr) = (unsafe { inst.as_ptr().layout() }).memory_addr(0) else {
        return (Mem0Ptr::from([].as_mut_ptr()), Mem0Len::from(0));
    };
    // SAFETY: `addr` stems from the instance layout and thus addresses a memory entry
    //         whose cache was warmed at instantiation.
    let Some(mem0) = (unsafe { inst.as_ptr().get_memory(addr) }) else {
        unsafe { unreachable_unchecked!("missing memory at: {addr:?}") }
    };
    // SAFETY: warmed at instantiation; the `_store` borrow scopes exclusive memory access.
    let mem0 = mem0.entity();
    let mem0 = unsafe { &mut *mem0.as_ptr() }.data_mut();
    let mem0_ptr = mem0.as_mut_ptr();
    let mem0_len = mem0.len();
    (Mem0Ptr::from(mem0_ptr), Mem0Len::from(mem0_len))
}

pub fn memory_slice(memory: &MemoryEntity, pos: usize, len: usize) -> Result<&[u8], TrapCode> {
    memory
        .data()
        .get(pos..)
        .and_then(|memory| memory.get(..len))
        .ok_or(TrapCode::MemoryOutOfBounds)
}

pub fn memory_slice_mut(
    memory: &mut MemoryEntity,
    pos: usize,
    len: usize,
) -> Result<&mut [u8], TrapCode> {
    memory
        .data_mut()
        .get_mut(pos..)
        .and_then(|memory| memory.get_mut(..len))
        .ok_or(TrapCode::MemoryOutOfBounds)
}

macro_rules! impl_resolve_from_instance {
    (
        $( fn $fn:ident(inst: Inst, store: &mut StoreInner, $param:ident: ir::$param_ty:ident) -> &mut $ret:ty = $entry:ident );* $(;)?
    ) => {
        $(
            /// Resolves the entity from the warmed up instance cache.
            ///
            /// The `store` borrow is not read; it ties the returned reference so that it
            /// cannot alias a concurrent store mutation (this is the safe wrapper).
            #[inline]
            pub fn $fn(inst: Inst, _store: &mut StoreInner, $param: ir::$param_ty) -> &mut $ret {
                let addr = <$param_ty>::from(::core::primitive::u32::from($param));
                // SAFETY: `addr` addresses an entry of this kind by translation invariant.
                let Some(entry) = (unsafe { inst.as_ptr().$entry(addr) }) else {
                    unsafe {
                        $crate::engine::utils::unreachable_unchecked!(
                            ::core::concat!("missing ", ::core::stringify!($param), " at: {:?}"),
                            addr,
                        )
                    }
                };
                // SAFETY: warmed at instantiation; the `_store` borrow scopes the reference.
                unsafe { &mut *entry.entity().as_ptr() }
            }
        )*
    };
}
impl_resolve_from_instance! {
    fn load_data(inst: Inst, store: &mut StoreInner, data: ir::DataAddr) -> &mut DataSegmentEntity = get_data;
    fn load_elem(inst: Inst, store: &mut StoreInner, elem: ir::ElemAddr) -> &mut ElementSegmentEntity = get_elem;
    fn load_global(inst: Inst, store: &mut StoreInner, global: ir::GlobalAddr) -> &mut GlobalEntity = get_global;
    fn load_memory(inst: Inst, store: &mut StoreInner, memory: ir::MemoryAddr) -> &mut MemoryEntity = get_memory;
    fn load_table(inst: Inst, store: &mut StoreInner, table: ir::TableAddr) -> &mut TableEntity = get_table;
}

macro_rules! impl_resolve_ptr_from_instance {
    (
        $( fn $fn:ident(inst: Inst, $param:ident: ir::$param_ty:ident) -> $ret:ty = $entry:ident );* $(;)?
    ) => {
        $(
            /// Resolves a pointer to the entity from the warmed up instance cache.
            ///
            /// Unlike [`load_memory`] and friends this returns a raw [`NonNull`] instead of a
            /// reference tied to the store. This lets the caller hold several entity pointers
            /// (or an entity pointer alongside a `&mut` borrow of `fuel`) at once and materialize
            /// a reference only in the narrow scope where the entity is actually accessed.
            ///
            /// # Safety
            ///
            /// The caller must ensure that `inst` refers to a live, warmed up [`InstanceEntity`]
            /// and that `$param` is a valid address within it. The returned pointer is only sound
            /// to dereference while the instance cache remains warmed up, and the caller must
            /// ensure the resulting reference does not alias any other live reference.
            #[inline]
            pub unsafe fn $fn(inst: Inst, $param: ir::$param_ty) -> NonNull<$ret> {
                let addr = <$param_ty>::from(::core::primitive::u32::from($param));
                // Safety: guaranteed by the caller.
                let Some(entry) = (unsafe { inst.as_ptr().$entry(addr) }) else {
                    unsafe {
                        $crate::engine::utils::unreachable_unchecked!(
                            ::core::concat!("missing ", ::core::stringify!($param), " at: {:?}"),
                            addr,
                        )
                    }
                };
                entry.entity()
            }
        )*
    };
}
impl_resolve_ptr_from_instance! {
    fn load_memory_ptr(inst: Inst, memory: ir::MemoryAddr) -> MemoryEntity = get_memory;
    fn load_table_ptr(inst: Inst, table: ir::TableAddr) -> TableEntity = get_table;
    fn load_data_ptr(inst: Inst, data: ir::DataAddr) -> DataSegmentEntity = get_data;
    fn load_elem_ptr(inst: Inst, elem: ir::ElemAddr) -> ElementSegmentEntity = get_elem;
}

/// Resolves the [`Func`] handle and its warmed up cached [`FuncEntity`] pointer from `inst`.
///
/// The returned pointer is only sound to dereference while the instance cache remains warmed up
/// and must not be turned into a reference that aliases a concurrent store mutation.
#[inline]
pub fn load_func_entry(inst: Inst, func: ir::FuncAddr) -> (Func, NonNull<FuncEntity>) {
    let addr = FuncAddr::from(u32::from(func));
    // SAFETY: `addr` addresses a func entry by translation invariant and its cache was
    //         warmed at instantiation.
    let Some(entry) = (unsafe { inst.as_ptr().get_func(addr) }) else {
        unsafe { unreachable_unchecked!("missing func at: {addr:?}") }
    };
    (entry.handle(), entry.entity())
}

macro_rules! impl_fetch_from_instance {
    (
        $( fn $fn:ident($param:ident: ir::$param_ty:ident) -> $ret:ty = $entry:ident );* $(;)?
    ) => {
        $(
            pub fn $fn(instance: Inst, $param: ir::$param_ty) -> $ret {
                let addr = <$param_ty>::from(::core::primitive::u32::from($param));
                // SAFETY: `addr` addresses an entry of this kind by translation invariant.
                let Some(entry) = (unsafe { instance.as_ptr().$entry(addr) }) else {
                    unsafe {
                        $crate::engine::utils::unreachable_unchecked!(
                            ::core::concat!("missing ", ::core::stringify!($param), " at: {:?}"),
                            addr,
                        )
                    }
                };
                entry.handle()
            }
        )*
    };
}
impl_fetch_from_instance! {
    fn fetch_func(func: ir::FuncAddr) -> Func = get_func;
    fn fetch_memory(memory: ir::MemoryAddr) -> Memory = get_memory;
    fn fetch_table(table: ir::TableAddr) -> Table = get_table;
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
    // fn resolve_global(global: &Global) -> &'a CoreGlobal = StoreInner::try_resolve_global;
    fn resolve_memory(memory: &Memory) -> &'a MemoryEntity = StoreInner::try_resolve_memory;
    fn resolve_table(table: &Table) -> &'a CoreTable = StoreInner::try_resolve_table;
    fn resolve_instance(instance: &Instance) -> &'a InstanceEntity = StoreInner::try_resolve_instance;
    // fn resolve_func_type(func_type: DedupFuncType) -> DedupFuncType = StoreInner::resolve_func_type;
}

#[inline]
pub fn resolve_indirect_func<I>(
    index: I,
    table: ir::TableAddr,
    func_type: ir::FuncType,
    state: &mut VmState<'_>,
    args: &mut Args,
) -> Result<(Func, NonNull<FuncEntity>), TrapCode>
where
    I: GetValue<u64>,
{
    let index = get_value(index, args.sp, args.ireg, args.freg32, args.freg64);
    let table = args.fetch_table(state, table);
    let rawref = table.get(index).ok_or(TrapCode::TableOutOfBounds)?;
    debug_assert!(matches!(rawref.ty(), RefType::Func));
    let funcref = <Nullable<Func>>::from_raw_parts(rawref.raw(), &*state.store);
    let func = funcref.val().ok_or(TrapCode::IndirectCallToNull)?;
    let func_entity = state.store.inner_mut().resolve_func_ptr(func);
    // SAFETY: freshly resolved from the store; only read here for the signature check.
    let actual_fnty = unsafe { func_entity.as_ref() }.ty_dedup().repr_entity();
    // Note: `func_type` already stores the resolved deduplicated function type, thus the
    //       engine association dropped by `repr_entity` is implied by both operands
    //       belonging to the very engine that is executing them.
    if actual_fnty != u32::from(func_type) {
        return Err(TrapCode::BadSignature);
    }
    Ok((*func, func_entity))
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

#[inline(always)]
pub fn call_func_entry(
    state: &mut VmState,
    caller_ip: Ip,
    params: BoundedSlotSpan,
    func: &FuncEntry,
    instance: Option<Inst>,
) -> Control<(Ip, Sp), Break> {
    let (callee_ip, len_local_slots, len_stack_slots) = compile_or_get_func_entry!(state, func);
    let callee_sp = state
        .stack
        .push_frame(
            Some(caller_ip),
            callee_ip,
            params,
            len_local_slots,
            len_stack_slots,
            instance,
        )
        .into_control()?;
    Control::Continue((callee_ip, callee_sp))
}

#[inline(always)]
pub fn return_call_func_entry(
    state: &mut VmState,
    params: BoundedSlotSpan,
    func: &FuncEntry,
    instance: Option<Inst>,
) -> Control<(Ip, Sp), Break> {
    let (callee_ip, len_local_slots, len_stack_slots) = compile_or_get_func_entry!(state, func);
    let callee_sp = state
        .stack
        .replace_frame(
            callee_ip,
            params,
            len_local_slots,
            len_stack_slots,
            instance,
        )
        .into_control()?;
    Control::Continue((callee_ip, callee_sp))
}

/// Invokes the host function behind `trampoline`.
///
/// Returns the host provided error if the host function trapped.
///
/// # Note
///
/// Takes `store` and `code` instead of the whole [`VmState`] since `inout` already
/// borrows its [`Stack`].
///
/// [`Stack`]: crate::engine::executor::Stack
#[inline]
fn invoke_host(
    store: &mut PrunedStore,
    code: &mut CodeView,
    trampoline: Trampoline,
    instance: Option<Inst>,
    inout: InOutParams<'_>,
    call_hooks: CallHooks,
) -> Result<(), Error> {
    match store.call_host_func(trampoline, instance, inout, call_hooks) {
        Ok(()) => {}
        Err(StoreError::External(error)) => return Err(error),
        Err(StoreError::Internal(error)) => unsafe {
            unreachable_unchecked!(
                "internal interpreter error while executing host function: {error}"
            )
        },
    }
    code.refresh();
    Ok(())
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
    debug_assert_eq!(params.len(), host_func.len_param_cells());
    let trampoline = *host_func.trampoline();
    let (sp, inout) = state
        .stack
        .prepare_host_frame(caller_ip, params, host_func.len_result_cells())
        .into_control()?;
    if let Err(error) = invoke_host(
        state.store,
        &mut state.code,
        trampoline,
        instance,
        inout,
        call_hooks,
    ) {
        done!(state, DoneReason::host_error(error, func, params.span()))
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
    debug_assert_eq!(params.len(), host_func.len_param_cells());
    let trampoline = *host_func.trampoline();
    let (control, inout) = state
        .stack
        .return_prepare_host_frame(params, host_func.len_result_cells(), instance)
        .into_control()?;
    if let Err(error) = invoke_host(
        state.store,
        &mut state.code,
        trampoline,
        Some(instance),
        inout,
        CallHooks::Call,
    ) {
        // Note: we won't allow resumption in case the execution would
        //       have returned with this the host function tail call.
        let reason = match control {
            Control::Continue(_) => DoneReason::host_error(error, func, params.span()),
            Control::Break(_) => DoneReason::error(error),
        };
        done!(state, reason)
    }
    match control {
        Control::Continue((ip, sp, instance)) => Control::Continue((ip, sp, instance)),
        Control::Break(sp) => done!(state, DoneReason::Return(sp)),
    }
}

#[inline]
#[expect(clippy::too_many_arguments)]
pub fn call_wasm_or_host(
    state: &mut VmState,
    caller_ip: Ip,
    func: Func,
    func_entity: NonNull<FuncEntity>,
    params: BoundedSlotSpan,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
) -> Control<(Ip, Sp, Mem0Ptr, Mem0Len, Inst), Break> {
    // SAFETY: the pointer is warmed up at instantiation (imported calls) or freshly resolved
    //         (indirect calls); the reference is only used to copy out the callee data below,
    //         before any store mutation, so it never aliases a `&mut` into the funcs arena.
    let func_entity = unsafe { func_entity.as_ref() };
    let wasm_func = match func_entity {
        FuncEntity::Wasm(wasm_func) => wasm_func,
        FuncEntity::Host(host_func) => {
            let sp = call_host(
                state,
                func,
                Some(caller_ip),
                *host_func,
                params,
                Some(instance),
                CallHooks::Call,
            )?;
            // Note: host functions may grow memories, invalidating the cached `(memory 0)`.
            //       Therefore, it is required to re-extract `(memory 0)` to avoid a stale cache.
            let (mem0, mem0_len) = extract_mem0(state.store, instance);
            return Control::Continue((caller_ip, sp, mem0, mem0_len, instance));
        }
    };
    // Hot path: calling a Wasm function. Uses the cached `FuncEntry` and the same inlined
    // machinery as `call_internal`, differing only in the possible instance switch.
    let callee_instance: Inst = resolve_instance(state.store, wasm_func.instance()).into();
    let (callee_ip, callee_sp) = call_func_entry(
        state,
        caller_ip,
        params,
        wasm_func.func_entry(),
        Some(callee_instance),
    )?;
    let (instance, mem0, mem0_len) =
        update_instance(state.store, instance, callee_instance, mem0, mem0_len);
    Control::Continue((callee_ip, callee_sp, mem0, mem0_len, instance))
}

/// Tail-call (`return_call`) twin of [`call_wasm_or_host`].
#[inline]
pub fn return_call_wasm_or_host(
    state: &mut VmState,
    func: Func,
    func_entity: NonNull<FuncEntity>,
    params: BoundedSlotSpan,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
) -> Control<(Ip, Sp, Mem0Ptr, Mem0Len, Inst), Break> {
    // SAFETY: see `call_wasm_or_host`.
    let func_entity = unsafe { func_entity.as_ref() };
    let wasm_func = match func_entity {
        FuncEntity::Wasm(wasm_func) => wasm_func,
        FuncEntity::Host(host_func) => {
            let (callee_ip, sp, new_instance) =
                return_call_host(state, func, *host_func, params, instance)?;
            // Note: host functions may grow memories, invalidating the cached `(memory 0)`.
            //       Therefore, it is required to re-extract `(memory 0)` to avoid a stale
            //       cache - even if the tail call returned into the very same instance.
            let (mem0, mem0_len) = extract_mem0(state.store, new_instance);
            return Control::Continue((callee_ip, sp, mem0, mem0_len, new_instance));
        }
    };
    // Hot path: tail-calling a Wasm function. See `call_wasm_or_host` for the shape.
    let callee_instance: Inst = resolve_instance(state.store, wasm_func.instance()).into();
    let (callee_ip, callee_sp) =
        return_call_func_entry(state, params, wasm_func.func_entry(), Some(callee_instance))?;
    let (instance, mem0, mem0_len) =
        update_instance(state.store, instance, callee_instance, mem0, mem0_len);
    Control::Continue((callee_ip, callee_sp, mem0, mem0_len, instance))
}
