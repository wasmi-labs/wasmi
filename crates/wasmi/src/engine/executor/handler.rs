use crate::{
    core::{wasm, UntypedVal},
    engine::executor::{
        stack::{CallStack, ValueStack},
        CodeMap,
    },
    errors::HostError,
    instance::InstanceEntity,
    ir,
    ir::OpCode,
    ir::{decode, Slot},
    store::PrunedStore,
    TrapCode,
};
use alloc::boxed::Box;
use core::ptr::{self, NonNull};

#[derive(Debug, Default, Copy, Clone)]
pub struct Done {
    _priv: (),
}

pub struct VmState<'vm> {
    store: &'vm mut PrunedStore,
    frames: &'vm mut CallStack,
    stack: &'vm mut ValueStack,
    code: &'vm CodeMap,
    done_reason: DoneReason,
}

#[derive(Debug)]
pub enum DoneReason {
    Trap(TrapCode),
    OutOfFuel {
        required: u64,
    },
    Host(Box<dyn HostError>),
    Return,
    Continue {
        ip: Ip,
        sp: Sp,
        mem0: *mut u8,
        mem0_len: usize,
        instance: NonNull<InstanceEntity>,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct Ip {
    value: *const u8,
}

struct IpDecoder(Ip);
impl ir::Decoder for IpDecoder {
    fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), ir::DecodeError> {
        unsafe { ptr::copy_nonoverlapping(self.0.value, buffer.as_mut_ptr(), buffer.len()) };
        Ok(())
    }
}

impl Ip {
    pub unsafe fn decode<T: ir::Decode>(self) -> (Ip, T) {
        let mut ip = IpDecoder(self);
        let decoded = <T as ir::Decode>::decode(&mut ip).unwrap();
        (ip.0, decoded)
    }

    pub unsafe fn offset(self, delta: isize) -> Self {
        let value = unsafe { self.value.byte_offset(delta) };
        Self { value }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Sp {
    value: *mut UntypedVal,
}

impl Sp {
    fn get<T>(self, slot: Slot) -> T
    where
        T: From<UntypedVal>,
    {
        let index = usize::from(u16::from(slot));
        let value = unsafe { *self.value.add(index) };
        T::from(value)
    }

    fn set<T>(self, slot: Slot, value: T)
    where
        T: Into<UntypedVal>,
    {
        let index = usize::from(u16::from(slot));
        let cell = unsafe { &mut *self.value.add(index) };
        *cell = value.into();
    }

    pub fn get_i32(self, slot: Slot) -> i32 {
        self.get::<i32>(slot)
    }

    pub fn set_i32(self, slot: Slot, value: i32) {
        self.set::<i32>(slot, value)
    }
}

type Handler = fn(
    &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: *mut u8,
    mem0_len: usize,
    instance: NonNull<InstanceEntity>,
) -> Done;

#[cfg(feature = "trampolines")]
macro_rules! dispatch {
    ($state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        $state.done_reason = DoneReason::Continue {
            ip: $ip,
            sp: $sp,
            mem0: $mem0,
            mem0_len: $mem0_len,
            instance: $instance,
        };
        Done::default()
    }};
}

#[cfg(not(feature = "trampolines"))]
macro_rules! dispatch {
    ($state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        _ = ($state, $ip, $sp, $mem0, $mem0_len, $instance);
        todo!()
    }};
}

trait GetValue<T> {
    fn get_value(src: Self, sp: Sp) -> T;
}

impl<T> GetValue<T> for Slot
where
    T: From<UntypedVal>,
{
    fn get_value(src: Self, sp: Sp) -> T {
        sp.get::<T>(src)
    }
}

impl GetValue<i32> for i32 {
    fn get_value(src: Self, _sp: Sp) -> i32 {
        src
    }
}

fn get_value<T, L>(src: T, sp: Sp) -> L
where
    T: GetValue<L>,
{
    <T as GetValue<L>>::get_value(src, sp)
}

trait SetValue<T> {
    fn set_value(src: Self, value: T, sp: Sp);
}

impl<T> SetValue<T> for Slot
where
    T: Into<UntypedVal>,
{
    fn set_value(src: Self, value: T, sp: Sp) {
        sp.set::<T>(src, value)
    }
}

fn set_value<T, V>(sp: Sp, src: T, value: V)
where
    T: SetValue<V>,
{
    <T as SetValue<V>>::set_value(src, value, sp)
}

trait UnwrapResult {
    type Item;

    fn unwrap_result(self, state: &mut VmState) -> Option<Self::Item>;
}

impl<T> UnwrapResult for Result<T, TrapCode> {
    type Item = T;

    fn unwrap_result(self, state: &mut VmState) -> Option<Self::Item> {
        match self {
            Ok(item) => Some(item),
            Err(trap_code) => {
                state.done_reason = DoneReason::Trap(trap_code);
                None
            }
        }
    }
}

macro_rules! impl_unwrap_result {
    ($($ty:ty),* $(,)?) => {
        $(
            impl UnwrapResult for $ty {
                type Item = Self;

                #[inline(always)]
                fn unwrap_result(self, _state: &mut VmState) -> Option<Self::Item> {
                    Some(self)
                }
            }
        )*
    };
}
impl_unwrap_result!(i32, i64, u32, u64, f32, f64);

macro_rules! unwrap_result {
    ($value:expr, $state:expr) => {{
        match <_ as UnwrapResult>::unwrap_result($value, $state) {
            Some(value) => value,
            None => return Done::default(),
        }
    }};
}

fn op_code_to_handler(code: OpCode) -> Handler {
    match code {
        OpCode::I32Popcnt_Ss => i32_popcnt_ss,
        OpCode::I32Ctz_Ss => i32_ctz_ss,
        OpCode::I32Clz_Ss => i32_clz_ss,
        OpCode::I32Sext8_Ss => i32_sext8_ss,
        OpCode::I32Sext16_Ss => i32_sext16_ss,
        OpCode::I32WrapI64_Ss => i32_wrap_i64,
        OpCode::I64Popcnt_Ss => i64_popcnt_ss,
        OpCode::I64Ctz_Ss => i64_ctz_ss,
        OpCode::I64Clz_Ss => i64_clz_ss,
        OpCode::I64Sext8_Ss => i64_sext8_ss,
        OpCode::I64Sext16_Ss => i64_sext16_ss,
        OpCode::I64Sext32_Ss => i64_sext32_ss,
        OpCode::F32Abs_Ss => f32_abs_ss,
        OpCode::F32Neg_Ss => f32_neg_ss,
        OpCode::F32Ceil_Ss => f32_ceil_ss,
        OpCode::F32Floor_Ss => f32_floor_ss,
        OpCode::F32Trunc_Ss => f32_trunc_ss,
        OpCode::F32Nearest_Ss => f32_nearest_ss,
        OpCode::F32Sqrt_Ss => f32_sqrt_ss,
        OpCode::F32ConvertI32_Ss => f32_convert_i32_ss,
        OpCode::F32ConvertU32_Ss => f32_convert_u32_ss,
        OpCode::F32ConvertI64_Ss => f32_convert_i64_ss,
        OpCode::F32ConvertU64_Ss => f32_convert_u64_ss,
        OpCode::F32DemoteF64_Ss => f32_demote_f64_ss,
        OpCode::F64Abs_Ss => f64_abs_ss,
        OpCode::F64Neg_Ss => f64_neg_ss,
        OpCode::F64Ceil_Ss => f64_ceil_ss,
        OpCode::F64Floor_Ss => f64_floor_ss,
        OpCode::F64Trunc_Ss => f64_trunc_ss,
        OpCode::F64Nearest_Ss => f64_nearest_ss,
        OpCode::F64Sqrt_Ss => f64_sqrt_ss,
        OpCode::F64ConvertI32_Ss => f64_convert_i32_ss,
        OpCode::F64ConvertU32_Ss => f64_convert_u32_ss,
        OpCode::F64ConvertI64_Ss => f64_convert_i64_ss,
        OpCode::F64ConvertU64_Ss => f64_convert_u64_ss,
        OpCode::F64PromoteF32_Ss => f64_demote_f64_ss,
        OpCode::I32TruncF32_Ss => i32_trunc_f32,
        OpCode::U32TruncF32_Ss => u32_trunc_f32,
        OpCode::I32TruncF64_Ss => i32_trunc_f64,
        OpCode::U32TruncF64_Ss => u32_trunc_f64,
        OpCode::I64TruncF32_Ss => i64_trunc_f32,
        OpCode::U64TruncF32_Ss => u64_trunc_f32,
        OpCode::I64TruncF64_Ss => i64_trunc_f64,
        OpCode::U64TruncF64_Ss => u64_trunc_f64,
        OpCode::I32TruncSatF32_Ss => i32_trunc_sat_f32,
        OpCode::U32TruncSatF32_Ss => u32_trunc_sat_f32,
        OpCode::I32TruncSatF64_Ss => i32_trunc_sat_f64,
        OpCode::U32TruncSatF64_Ss => u32_trunc_sat_f64,
        OpCode::I64TruncSatF32_Ss => i64_trunc_sat_f32,
        OpCode::U64TruncSatF32_Ss => u64_trunc_sat_f32,
        OpCode::I64TruncSatF64_Ss => i64_trunc_sat_f64,
        OpCode::U64TruncSatF64_Ss => u64_trunc_sat_f64,
        _ => todo!(),
    }
}

macro_rules! handler_unary {
    ( $( fn $handler:ident($op:ident) = $eval:expr );* $(;)? ) => {
        $(
            fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: *mut u8,
                mem0_len: usize,
                instance: NonNull<InstanceEntity>,
            ) -> Done {
                let (ip, $crate::ir::decode::$op { result, value }) = unsafe { ip.decode() };
                let value = get_value(value, sp);
                set_value(sp, result, unwrap_result!($eval(value), state));
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_unary! {
    // i32
    fn i32_popcnt_ss(I32Popcnt_Ss) = wasm::i32_popcnt;
    fn i32_ctz_ss(I32Ctz_Ss) = wasm::i32_ctz;
    fn i32_clz_ss(I32Clz_Ss) = wasm::i32_clz;
    fn i32_sext8_ss(I32Sext8_Ss) = wasm::i32_extend8_s;
    fn i32_sext16_ss(I32Sext16_Ss) = wasm::i32_extend16_s;
    fn i32_wrap_i64(I32WrapI64_Ss) = wasm::i32_wrap_i64;
    // i64
    fn i64_popcnt_ss(I64Popcnt_Ss) = wasm::i64_popcnt;
    fn i64_ctz_ss(I64Ctz_Ss) = wasm::i64_ctz;
    fn i64_clz_ss(I64Clz_Ss) = wasm::i64_clz;
    fn i64_sext8_ss(I64Sext8_Ss) = wasm::i64_extend8_s;
    fn i64_sext16_ss(I64Sext16_Ss) = wasm::i64_extend16_s;
    fn i64_sext32_ss(I64Sext32_Ss) = wasm::i64_extend32_s;
    // f32
    fn f32_abs_ss(F32Abs_Ss) = wasm::f32_abs;
    fn f32_neg_ss(F32Neg_Ss) = wasm::f32_neg;
    fn f32_ceil_ss(F32Ceil_Ss) = wasm::f32_ceil;
    fn f32_floor_ss(F32Floor_Ss) = wasm::f32_floor;
    fn f32_trunc_ss(F32Trunc_Ss) = wasm::f32_trunc;
    fn f32_nearest_ss(F32Nearest_Ss) = wasm::f32_nearest;
    fn f32_sqrt_ss(F32Sqrt_Ss) = wasm::f32_sqrt;
    fn f32_convert_i32_ss(F32ConvertI32_Ss) = wasm::f32_convert_i32_s;
    fn f32_convert_u32_ss(F32ConvertU32_Ss) = wasm::f32_convert_i32_u;
    fn f32_convert_i64_ss(F32ConvertI64_Ss) = wasm::f32_convert_i64_s;
    fn f32_convert_u64_ss(F32ConvertU64_Ss) = wasm::f32_convert_i64_u;
    fn f32_demote_f64_ss(F32DemoteF64_Ss) = wasm::f32_demote_f64;
    // f64
    fn f64_abs_ss(F64Abs_Ss) = wasm::f64_abs;
    fn f64_neg_ss(F64Neg_Ss) = wasm::f64_neg;
    fn f64_ceil_ss(F64Ceil_Ss) = wasm::f64_ceil;
    fn f64_floor_ss(F64Floor_Ss) = wasm::f64_floor;
    fn f64_trunc_ss(F64Trunc_Ss) = wasm::f64_trunc;
    fn f64_nearest_ss(F64Nearest_Ss) = wasm::f64_nearest;
    fn f64_sqrt_ss(F64Sqrt_Ss) = wasm::f64_sqrt;
    fn f64_convert_i32_ss(F64ConvertI32_Ss) = wasm::f64_convert_i32_s;
    fn f64_convert_u32_ss(F64ConvertU32_Ss) = wasm::f64_convert_i32_u;
    fn f64_convert_i64_ss(F64ConvertI64_Ss) = wasm::f64_convert_i64_s;
    fn f64_convert_u64_ss(F64ConvertU64_Ss) = wasm::f64_convert_i64_u;
    fn f64_demote_f64_ss(F64PromoteF32_Ss) = wasm::f64_promote_f32;
    // f2i conversions
    fn i32_trunc_f32(I32TruncF32_Ss) = wasm::i32_trunc_f32_s;
    fn u32_trunc_f32(U32TruncF32_Ss) = wasm::i32_trunc_f32_u;
    fn i32_trunc_f64(I32TruncF64_Ss) = wasm::i32_trunc_f64_s;
    fn u32_trunc_f64(U32TruncF64_Ss) = wasm::i32_trunc_f64_u;
    fn i64_trunc_f32(I64TruncF32_Ss) = wasm::i64_trunc_f32_s;
    fn u64_trunc_f32(U64TruncF32_Ss) = wasm::i64_trunc_f32_u;
    fn i64_trunc_f64(I64TruncF64_Ss) = wasm::i64_trunc_f64_s;
    fn u64_trunc_f64(U64TruncF64_Ss) = wasm::i64_trunc_f64_u;
    fn i32_trunc_sat_f32(I32TruncSatF32_Ss) = wasm::i32_trunc_sat_f32_s;
    fn u32_trunc_sat_f32(U32TruncSatF32_Ss) = wasm::i32_trunc_sat_f32_u;
    fn i32_trunc_sat_f64(I32TruncSatF64_Ss) = wasm::i32_trunc_sat_f64_s;
    fn u32_trunc_sat_f64(U32TruncSatF64_Ss) = wasm::i32_trunc_sat_f64_u;
    fn i64_trunc_sat_f32(I64TruncSatF32_Ss) = wasm::i64_trunc_sat_f32_s;
    fn u64_trunc_sat_f32(U64TruncSatF32_Ss) = wasm::i64_trunc_sat_f32_u;
    fn i64_trunc_sat_f64(I64TruncSatF64_Ss) = wasm::i64_trunc_sat_f64_s;
    fn u64_trunc_sat_f64(U64TruncSatF64_Ss) = wasm::i64_trunc_sat_f64_u;
}

fn i32_add_sss(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: *mut u8,
    mem0_len: usize,
    instance: NonNull<InstanceEntity>,
) -> Done {
    let (ip, decode::I32Add_Sss { result, lhs, rhs }) = unsafe { ip.decode() };
    let lhs = get_value(lhs, sp);
    let rhs = get_value(rhs, sp);
    set_value(sp, result, wasm::i32_add(lhs, rhs));
    dispatch!(state, ip, sp, mem0, mem0_len, instance)
}

fn i32_add_ssi(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: *mut u8,
    mem0_len: usize,
    instance: NonNull<InstanceEntity>,
) -> Done {
    let (ip, decode::I32Add_Ssi { result, lhs, rhs }) = unsafe { ip.decode() };
    let lhs = get_value(lhs, sp);
    let rhs = get_value(rhs, sp);
    set_value(sp, result, wasm::i32_add(lhs, rhs));
    dispatch!(state, ip, sp, mem0, mem0_len, instance)
}

fn branch_i32_eq_ss(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: *mut u8,
    mem0_len: usize,
    instance: NonNull<InstanceEntity>,
) -> Done {
    let (next_ip, decode::BranchI32Eq_Ss { offset, lhs, rhs }) = unsafe { ip.decode() };
    let lhs = get_value(lhs, sp);
    let rhs = get_value(rhs, sp);
    let ip = match wasm::i32_eq(lhs, rhs) {
        true => unsafe { ip.offset(i32::from(offset) as isize) },
        false => next_ip,
    };
    dispatch!(state, ip, sp, mem0, mem0_len, instance)
}

fn branch_i32_eq_si(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: *mut u8,
    mem0_len: usize,
    instance: NonNull<InstanceEntity>,
) -> Done {
    let (next_ip, decode::BranchI32Eq_Si { offset, lhs, rhs }) = unsafe { ip.decode() };
    let lhs = get_value(lhs, sp);
    let rhs = get_value(rhs, sp);
    let ip = match wasm::i32_eq(lhs, rhs) {
        true => unsafe { ip.offset(i32::from(offset) as isize) },
        false => next_ip,
    };
    dispatch!(state, ip, sp, mem0, mem0_len, instance)
}

fn select_i32_eq_sss(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: *mut u8,
    mem0_len: usize,
    instance: NonNull<InstanceEntity>,
) -> Done {
    let (
        ip,
        decode::SelectI32Eq_Sss {
            result,
            val_true,
            val_false,
            lhs,
            rhs,
        },
    ) = unsafe { ip.decode() };
    let lhs = get_value(lhs, sp);
    let rhs = get_value(rhs, sp);
    let src = match wasm::i32_eq(lhs, rhs) {
        true => val_true,
        false => val_false,
    };
    let src: UntypedVal = get_value(src, sp);
    set_value(sp, result, src);
    dispatch!(state, ip, sp, mem0, mem0_len, instance)
}

fn select_i32_eq_ssi(
    state: &mut VmState,
    ip: Ip,
    sp: Sp,
    mem0: *mut u8,
    mem0_len: usize,
    instance: NonNull<InstanceEntity>,
) -> Done {
    let (
        ip,
        decode::SelectI32Eq_Ssi {
            result,
            val_true,
            val_false,
            lhs,
            rhs,
        },
    ) = unsafe { ip.decode() };
    let lhs = get_value(lhs, sp);
    let rhs = get_value(rhs, sp);
    let src = match wasm::i32_eq(lhs, rhs) {
        true => val_true,
        false => val_false,
    };
    let src: UntypedVal = get_value(src, sp);
    set_value(sp, result, src);
    dispatch!(state, ip, sp, mem0, mem0_len, instance)
}
