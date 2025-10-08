use crate::{
    core::{wasm, UntypedVal},
    engine::executor::{
        stack::{CallStack, ValueStack},
        CodeMap,
    },
    errors::HostError,
    instance::InstanceEntity,
    ir,
    ir::{decode, OpCode, Slot},
    store::PrunedStore,
    TrapCode,
};
use alloc::boxed::Box;
use core::{
    num::NonZero,
    ops::{Div, Rem},
    ptr::{self, NonNull},
};

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
);

impl<T> GetValue<T> for Slot
where
    T: From<UntypedVal>,
{
    fn get_value(src: Self, sp: Sp) -> T {
        sp.get::<T>(src)
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
impl_unwrap_result!(bool, i32, i64, u32, u64, f32, f64);

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
        // unary
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
        // binary
        // i32
        OpCode::I32Eq_Sss => i32_eq_sss,
        OpCode::I32Eq_Ssi => i32_eq_ssi,
        OpCode::I32NotEq_Sss => i32_not_eq_sss,
        OpCode::I32NotEq_Ssi => i32_not_eq_ssi,
        OpCode::I32Add_Sss => i32_add_sss,
        OpCode::I32Add_Ssi => i32_add_ssi,
        OpCode::I32Mul_Sss => i32_mul_sss,
        OpCode::I32Mul_Ssi => i32_mul_ssi,
        OpCode::I32BitAnd_Sss => i32_bitand_sss,
        OpCode::I32BitAnd_Ssi => i32_bitand_ssi,
        OpCode::I32BitOr_Sss => i32_bitor_sss,
        OpCode::I32BitOr_Ssi => i32_bitor_ssi,
        OpCode::I32BitXor_Sss => i32_bitxor_sss,
        OpCode::I32BitXor_Ssi => i32_bitxor_ssi,
        OpCode::I32Sub_Sss => i32_sub_sss,
        OpCode::I32Sub_Ssi => i32_sub_ssi,
        OpCode::I32Sub_Sis => i32_sub_sis,
        OpCode::I32Div_Sss => i32_div_sss,
        OpCode::I32Div_Ssi => i32_div_ssi,
        OpCode::I32Div_Sis => i32_div_sis,
        OpCode::U32Div_Sss => u32_div_sss,
        OpCode::U32Div_Ssi => u32_div_ssi,
        OpCode::U32Div_Sis => u32_div_sis,
        OpCode::I32Rem_Sss => i32_rem_sss,
        OpCode::I32Rem_Ssi => i32_rem_ssi,
        OpCode::I32Rem_Sis => i32_rem_sis,
        OpCode::U32Rem_Sss => u32_rem_sss,
        OpCode::U32Rem_Ssi => u32_rem_ssi,
        OpCode::U32Rem_Sis => u32_rem_sis,
        OpCode::I32Le_Sss => i32_le_sss,
        OpCode::I32Le_Ssi => i32_le_ssi,
        OpCode::I32Le_Sis => i32_le_sis,
        OpCode::I32Lt_Sss => i32_lt_sss,
        OpCode::I32Lt_Ssi => i32_lt_ssi,
        OpCode::I32Lt_Sis => i32_lt_sis,
        OpCode::U32Le_Sss => u32_le_sss,
        OpCode::U32Le_Ssi => u32_le_ssi,
        OpCode::U32Le_Sis => u32_le_sis,
        OpCode::U32Lt_Sss => u32_lt_sss,
        OpCode::U32Lt_Ssi => u32_lt_ssi,
        OpCode::U32Lt_Sis => u32_lt_sis,
        OpCode::I32Shl_Sss => i32_shl_sss,
        OpCode::I32Shl_Ssi => i32_shl_ssi,
        OpCode::I32Shl_Sis => i32_shl_sis,
        OpCode::I32Shr_Sss => i32_shr_sss,
        OpCode::I32Shr_Ssi => i32_shr_ssi,
        OpCode::I32Shr_Sis => i32_shr_sis,
        OpCode::U32Shr_Sss => u32_shr_sss,
        OpCode::U32Shr_Ssi => u32_shr_ssi,
        OpCode::U32Shr_Sis => u32_shr_sis,
        OpCode::I32Rotl_Sss => i32_rotl_sss,
        OpCode::I32Rotl_Ssi => i32_rotl_ssi,
        OpCode::I32Rotl_Sis => i32_rotl_sis,
        OpCode::I32Rotr_Sss => i32_rotr_sss,
        OpCode::I32Rotr_Ssi => i32_rotr_ssi,
        OpCode::I32Rotr_Sis => i32_rotr_sis,
        // binary
        // i64
        OpCode::I64Eq_Sss => i64_eq_sss,
        OpCode::I64Eq_Ssi => i64_eq_ssi,
        OpCode::I64NotEq_Sss => i64_not_eq_sss,
        OpCode::I64NotEq_Ssi => i64_not_eq_ssi,
        OpCode::I64Add_Sss => i64_add_sss,
        OpCode::I64Add_Ssi => i64_add_ssi,
        OpCode::I64Mul_Sss => i64_mul_sss,
        OpCode::I64Mul_Ssi => i64_mul_ssi,
        OpCode::I64BitAnd_Sss => i64_bitand_sss,
        OpCode::I64BitAnd_Ssi => i64_bitand_ssi,
        OpCode::I64BitOr_Sss => i64_bitor_sss,
        OpCode::I64BitOr_Ssi => i64_bitor_ssi,
        OpCode::I64BitXor_Sss => i64_bitxor_sss,
        OpCode::I64BitXor_Ssi => i64_bitxor_ssi,
        OpCode::I64Sub_Sss => i64_sub_sss,
        OpCode::I64Sub_Ssi => i64_sub_ssi,
        OpCode::I64Sub_Sis => i64_sub_sis,
        OpCode::I64Div_Sss => i64_div_sss,
        OpCode::I64Div_Ssi => i64_div_ssi,
        OpCode::I64Div_Sis => i64_div_sis,
        OpCode::U64Div_Sss => u64_div_sss,
        OpCode::U64Div_Ssi => u64_div_ssi,
        OpCode::U64Div_Sis => u64_div_sis,
        OpCode::I64Rem_Sss => i64_rem_sss,
        OpCode::I64Rem_Ssi => i64_rem_ssi,
        OpCode::I64Rem_Sis => i64_rem_sis,
        OpCode::U64Rem_Sss => u64_rem_sss,
        OpCode::U64Rem_Ssi => u64_rem_ssi,
        OpCode::U64Rem_Sis => u64_rem_sis,
        OpCode::I64Le_Sss => i64_le_sss,
        OpCode::I64Le_Ssi => i64_le_ssi,
        OpCode::I64Le_Sis => i64_le_sis,
        OpCode::I64Lt_Sss => i64_lt_sss,
        OpCode::I64Lt_Ssi => i64_lt_ssi,
        OpCode::I64Lt_Sis => i64_lt_sis,
        OpCode::U64Le_Sss => u64_le_sss,
        OpCode::U64Le_Ssi => u64_le_ssi,
        OpCode::U64Le_Sis => u64_le_sis,
        OpCode::U64Lt_Sss => u64_lt_sss,
        OpCode::U64Lt_Ssi => u64_lt_ssi,
        OpCode::U64Lt_Sis => u64_lt_sis,
        OpCode::I64Shl_Sss => i64_shl_sss,
        OpCode::I64Shl_Ssi => i64_shl_ssi,
        OpCode::I64Shl_Sis => i64_shl_sis,
        OpCode::I64Shr_Sss => i64_shr_sss,
        OpCode::I64Shr_Ssi => i64_shr_ssi,
        OpCode::I64Shr_Sis => i64_shr_sis,
        OpCode::U64Shr_Sss => u64_shr_sss,
        OpCode::U64Shr_Ssi => u64_shr_ssi,
        OpCode::U64Shr_Sis => u64_shr_sis,
        OpCode::I64Rotl_Sss => i64_rotl_sss,
        OpCode::I64Rotl_Ssi => i64_rotl_ssi,
        OpCode::I64Rotl_Sis => i64_rotl_sis,
        OpCode::I64Rotr_Sss => i64_rotr_sss,
        OpCode::I64Rotr_Ssi => i64_rotr_ssi,
        OpCode::I64Rotr_Sis => i64_rotr_sis,
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

macro_rules! handler_binary {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: *mut u8,
                mem0_len: usize,
                instance: NonNull<InstanceEntity>,
            ) -> Done {
                let (ip, $crate::ir::decode::$decode { result, lhs, rhs }) = unsafe { ip.decode() };
                let lhs = get_value(lhs, sp);
                let rhs = get_value(rhs, sp);
                set_value(sp, result, unwrap_result!($eval(lhs, rhs), state));
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_binary! {
    // i32
    // i32: commutative
    fn i32_eq_sss(I32Eq_Sss) = wasm::i32_eq;
    fn i32_eq_ssi(I32Eq_Ssi) = wasm::i32_eq;
    fn i32_not_eq_sss(I32NotEq_Sss) = wasm::i32_ne;
    fn i32_not_eq_ssi(I32NotEq_Ssi) = wasm::i32_ne;
    fn i32_add_sss(I32Add_Sss) = wasm::i32_add;
    fn i32_add_ssi(I32Add_Ssi) = wasm::i32_add;
    fn i32_mul_sss(I32Mul_Sss) = wasm::i32_mul;
    fn i32_mul_ssi(I32Mul_Ssi) = wasm::i32_mul;
    fn i32_bitand_sss(I32BitAnd_Sss) = wasm::i32_bitand;
    fn i32_bitand_ssi(I32BitAnd_Ssi) = wasm::i32_bitand;
    fn i32_bitor_sss(I32BitOr_Sss) = wasm::i32_bitor;
    fn i32_bitor_ssi(I32BitOr_Ssi) = wasm::i32_bitor;
    fn i32_bitxor_sss(I32BitXor_Sss) = wasm::i32_bitxor;
    fn i32_bitxor_ssi(I32BitXor_Ssi) = wasm::i32_bitxor;
    // i32: non-commutative
    fn i32_sub_sss(I32Sub_Sss) = wasm::i32_sub;
    fn i32_sub_ssi(I32Sub_Ssi) = wasm::i32_sub;
    fn i32_sub_sis(I32Sub_Sis) = wasm::i32_sub;
    fn i32_div_sss(I32Div_Sss) = wasm::i32_div_s;
    fn i32_div_ssi(I32Div_Ssi) = wasmi_i32_div_ssi;
    fn i32_div_sis(I32Div_Sis) = wasm::i32_div_s;
    fn u32_div_sss(U32Div_Sss) = wasm::i32_div_u;
    fn u32_div_ssi(U32Div_Ssi) = wasmi_u32_div_ssi;
    fn u32_div_sis(U32Div_Sis) = wasm::i32_div_u;
    fn i32_rem_sss(I32Rem_Sss) = wasm::i32_rem_s;
    fn i32_rem_ssi(I32Rem_Ssi) = wasmi_i32_rem_ssi;
    fn i32_rem_sis(I32Rem_Sis) = wasm::i32_rem_s;
    fn u32_rem_sss(U32Rem_Sss) = wasm::i32_rem_u;
    fn u32_rem_ssi(U32Rem_Ssi) = wasmi_u32_rem_ssi;
    fn u32_rem_sis(U32Rem_Sis) = wasm::i32_rem_u;
    // i32: comparisons
    fn i32_le_sss(I32Le_Sss) = wasm::i32_le_s;
    fn i32_le_ssi(I32Le_Ssi) = wasm::i32_le_s;
    fn i32_le_sis(I32Le_Sis) = wasm::i32_le_s;
    fn i32_lt_sss(I32Lt_Sss) = wasm::i32_lt_s;
    fn i32_lt_ssi(I32Lt_Ssi) = wasm::i32_lt_s;
    fn i32_lt_sis(I32Lt_Sis) = wasm::i32_lt_s;
    fn u32_le_sss(U32Le_Sss) = wasm::i32_le_u;
    fn u32_le_ssi(U32Le_Ssi) = wasm::i32_le_u;
    fn u32_le_sis(U32Le_Sis) = wasm::i32_le_u;
    fn u32_lt_sss(U32Lt_Sss) = wasm::i32_lt_u;
    fn u32_lt_ssi(U32Lt_Ssi) = wasm::i32_lt_u;
    fn u32_lt_sis(U32Lt_Sis) = wasm::i32_lt_u;
    // i32: shift + rotate
    fn i32_shl_sss(I32Shl_Sss) = wasm::i32_shl;
    fn i32_shl_ssi(I32Shl_Ssi) = wasmi_i32_shl_ssi;
    fn i32_shl_sis(I32Shl_Sis) = wasm::i32_shl;
    fn i32_shr_sss(I32Shr_Sss) = wasm::i32_shr_s;
    fn i32_shr_ssi(I32Shr_Ssi) = wasmi_i32_shr_ssi;
    fn i32_shr_sis(I32Shr_Sis) = wasm::i32_shr_s;
    fn u32_shr_sss(U32Shr_Sss) = wasm::i32_shr_u;
    fn u32_shr_ssi(U32Shr_Ssi) = wasmi_u32_shr_ssi;
    fn u32_shr_sis(U32Shr_Sis) = wasm::i32_shr_u;
    fn i32_rotl_sss(I32Rotl_Sss) = wasm::i32_rotl;
    fn i32_rotl_ssi(I32Rotl_Ssi) = wasmi_i32_rotl_ssi;
    fn i32_rotl_sis(I32Rotl_Sis) = wasm::i32_rotl;
    fn i32_rotr_sss(I32Rotr_Sss) = wasm::i32_rotr;
    fn i32_rotr_ssi(I32Rotr_Ssi) = wasmi_i32_rotr_ssi;
    fn i32_rotr_sis(I32Rotr_Sis) = wasm::i32_rotr;
    // i64
    // i64: commutative
    fn i64_eq_sss(I64Eq_Sss) = wasm::i64_eq;
    fn i64_eq_ssi(I64Eq_Ssi) = wasm::i64_eq;
    fn i64_not_eq_sss(I64NotEq_Sss) = wasm::i64_ne;
    fn i64_not_eq_ssi(I64NotEq_Ssi) = wasm::i64_ne;
    fn i64_add_sss(I64Add_Sss) = wasm::i64_add;
    fn i64_add_ssi(I64Add_Ssi) = wasm::i64_add;
    fn i64_mul_sss(I64Mul_Sss) = wasm::i64_mul;
    fn i64_mul_ssi(I64Mul_Ssi) = wasm::i64_mul;
    fn i64_bitand_sss(I64BitAnd_Sss) = wasm::i64_bitand;
    fn i64_bitand_ssi(I64BitAnd_Ssi) = wasm::i64_bitand;
    fn i64_bitor_sss(I64BitOr_Sss) = wasm::i64_bitor;
    fn i64_bitor_ssi(I64BitOr_Ssi) = wasm::i64_bitor;
    fn i64_bitxor_sss(I64BitXor_Sss) = wasm::i64_bitxor;
    fn i64_bitxor_ssi(I64BitXor_Ssi) = wasm::i64_bitxor;
    // i64: non-commutative
    fn i64_sub_sss(I64Sub_Sss) = wasm::i64_sub;
    fn i64_sub_ssi(I64Sub_Ssi) = wasm::i64_sub;
    fn i64_sub_sis(I64Sub_Sis) = wasm::i64_sub;
    fn i64_div_sss(I64Div_Sss) = wasm::i64_div_s;
    fn i64_div_ssi(I64Div_Ssi) = wasmi_i64_div_ssi;
    fn i64_div_sis(I64Div_Sis) = wasm::i64_div_s;
    fn u64_div_sss(U64Div_Sss) = wasm::i64_div_u;
    fn u64_div_ssi(U64Div_Ssi) = wasmi_u64_div_ssi;
    fn u64_div_sis(U64Div_Sis) = wasm::i64_div_u;
    fn i64_rem_sss(I64Rem_Sss) = wasm::i64_rem_s;
    fn i64_rem_ssi(I64Rem_Ssi) = wasmi_i64_rem_ssi;
    fn i64_rem_sis(I64Rem_Sis) = wasm::i64_rem_s;
    fn u64_rem_sss(U64Rem_Sss) = wasm::i64_rem_u;
    fn u64_rem_ssi(U64Rem_Ssi) = wasmi_u64_rem_ssi;
    fn u64_rem_sis(U64Rem_Sis) = wasm::i64_rem_u;
    // i64: comparisons
    fn i64_le_sss(I64Le_Sss) = wasm::i64_le_s;
    fn i64_le_ssi(I64Le_Ssi) = wasm::i64_le_s;
    fn i64_le_sis(I64Le_Sis) = wasm::i64_le_s;
    fn i64_lt_sss(I64Lt_Sss) = wasm::i64_lt_s;
    fn i64_lt_ssi(I64Lt_Ssi) = wasm::i64_lt_s;
    fn i64_lt_sis(I64Lt_Sis) = wasm::i64_lt_s;
    fn u64_le_sss(U64Le_Sss) = wasm::i64_le_u;
    fn u64_le_ssi(U64Le_Ssi) = wasm::i64_le_u;
    fn u64_le_sis(U64Le_Sis) = wasm::i64_le_u;
    fn u64_lt_sss(U64Lt_Sss) = wasm::i64_lt_u;
    fn u64_lt_ssi(U64Lt_Ssi) = wasm::i64_lt_u;
    fn u64_lt_sis(U64Lt_Sis) = wasm::i64_lt_u;
    // i64: shift + rotate
    fn i64_shl_sss(I64Shl_Sss) = wasm::i64_shl;
    fn i64_shl_ssi(I64Shl_Ssi) = wasmi_i64_shl_ssi;
    fn i64_shl_sis(I64Shl_Sis) = wasm::i64_shl;
    fn i64_shr_sss(I64Shr_Sss) = wasm::i64_shr_s;
    fn i64_shr_ssi(I64Shr_Ssi) = wasmi_i64_shr_ssi;
    fn i64_shr_sis(I64Shr_Sis) = wasm::i64_shr_s;
    fn u64_shr_sss(U64Shr_Sss) = wasm::i64_shr_u;
    fn u64_shr_ssi(U64Shr_Ssi) = wasmi_u64_shr_ssi;
    fn u64_shr_sis(U64Shr_Sis) = wasm::i64_shr_u;
    fn i64_rotl_sss(I64Rotl_Sss) = wasm::i64_rotl;
    fn i64_rotl_ssi(I64Rotl_Ssi) = wasmi_i64_rotl_ssi;
    fn i64_rotl_sis(I64Rotl_Sis) = wasm::i64_rotl;
    fn i64_rotr_sss(I64Rotr_Sss) = wasm::i64_rotr;
    fn i64_rotr_ssi(I64Rotr_Ssi) = wasmi_i64_rotr_ssi;
    fn i64_rotr_sis(I64Rotr_Sis) = wasm::i64_rotr;
}

fn wasmi_i32_div_ssi(lhs: i32, rhs: NonZero<i32>) -> Result<i32, TrapCode> {
    wasm::i32_div_s(lhs, rhs.get())
}

fn wasmi_i64_div_ssi(lhs: i64, rhs: NonZero<i64>) -> Result<i64, TrapCode> {
    wasm::i64_div_s(lhs, rhs.get())
}

fn wasmi_u32_div_ssi(lhs: u32, rhs: NonZero<u32>) -> u32 {
    <u32 as Div<NonZero<u32>>>::div(lhs, rhs)
}

fn wasmi_u64_div_ssi(lhs: u64, rhs: NonZero<u64>) -> u64 {
    <u64 as Div<NonZero<u64>>>::div(lhs, rhs)
}

fn wasmi_i32_rem_ssi(lhs: i32, rhs: NonZero<i32>) -> Result<i32, TrapCode> {
    wasm::i32_rem_s(lhs, rhs.get())
}

fn wasmi_i64_rem_ssi(lhs: i64, rhs: NonZero<i64>) -> Result<i64, TrapCode> {
    wasm::i64_rem_s(lhs, rhs.get())
}

fn wasmi_u32_rem_ssi(lhs: u32, rhs: NonZero<u32>) -> u32 {
    <u32 as Rem<NonZero<u32>>>::rem(lhs, rhs)
}

fn wasmi_u64_rem_ssi(lhs: u64, rhs: NonZero<u64>) -> u64 {
    <u64 as Rem<NonZero<u64>>>::rem(lhs, rhs)
}

fn wasmi_i32_shl_ssi(lhs: i32, rhs: u8) -> i32 {
    wasm::i32_shl(lhs, i32::from(rhs))
}

fn wasmi_i32_shr_ssi(lhs: i32, rhs: u8) -> i32 {
    wasm::i32_shr_s(lhs, i32::from(rhs))
}

fn wasmi_u32_shr_ssi(lhs: u32, rhs: u8) -> u32 {
    wasm::i32_shr_u(lhs, u32::from(rhs))
}

fn wasmi_i32_rotl_ssi(lhs: i32, rhs: u8) -> i32 {
    wasm::i32_rotl(lhs, i32::from(rhs))
}

fn wasmi_i32_rotr_ssi(lhs: i32, rhs: u8) -> i32 {
    wasm::i32_rotr(lhs, i32::from(rhs))
}

fn wasmi_i64_shl_ssi(lhs: i64, rhs: u8) -> i64 {
    wasm::i64_shl(lhs, i64::from(rhs))
}

fn wasmi_i64_shr_ssi(lhs: i64, rhs: u8) -> i64 {
    wasm::i64_shr_s(lhs, i64::from(rhs))
}

fn wasmi_u64_shr_ssi(lhs: u64, rhs: u8) -> u64 {
    wasm::i64_shr_u(lhs, u64::from(rhs))
}

fn wasmi_i64_rotl_ssi(lhs: i64, rhs: u8) -> i64 {
    wasm::i64_rotl(lhs, i64::from(rhs))
}

fn wasmi_i64_rotr_ssi(lhs: i64, rhs: u8) -> i64 {
    wasm::i64_rotr(lhs, i64::from(rhs))
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
