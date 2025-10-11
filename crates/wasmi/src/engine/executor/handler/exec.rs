use super::{
    dispatch::Done,
    eval,
    state::{Ip, Sp, VmState},
    utils::{default_memory_bytes, get_value, memory_bytes, set_value},
};
use crate::{
    core::{wasm, UntypedVal},
    instance::InstanceEntity,
};
use core::ptr::NonNull;

macro_rules! handler_unary {
    ( $( fn $handler:ident($op:ident) = $eval:expr );* $(;)? ) => {
        $(
            pub fn $handler(
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
            pub fn $handler(
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
    fn i32_and_sss(I32And_Sss) = eval::wasmi_i32_and;
    fn i32_and_ssi(I32And_Ssi) = eval::wasmi_i32_and;
    fn i32_or_sss(I32Or_Sss) = eval::wasmi_i32_or;
    fn i32_or_ssi(I32Or_Ssi) = eval::wasmi_i32_or;
    fn i32_not_eq_sss(I32NotEq_Sss) = wasm::i32_ne;
    fn i32_not_eq_ssi(I32NotEq_Ssi) = wasm::i32_ne;
    fn i32_not_and_sss(I32NotAnd_Sss) = eval::wasmi_i32_not_and;
    fn i32_not_and_ssi(I32NotAnd_Ssi) = eval::wasmi_i32_not_and;
    fn i32_not_or_sss(I32NotOr_Sss) = eval::wasmi_i32_not_or;
    fn i32_not_or_ssi(I32NotOr_Ssi) = eval::wasmi_i32_not_or;
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
    fn i32_div_ssi(I32Div_Ssi) = eval::wasmi_i32_div_ssi;
    fn i32_div_sis(I32Div_Sis) = wasm::i32_div_s;
    fn u32_div_sss(U32Div_Sss) = wasm::i32_div_u;
    fn u32_div_ssi(U32Div_Ssi) = eval::wasmi_u32_div_ssi;
    fn u32_div_sis(U32Div_Sis) = wasm::i32_div_u;
    fn i32_rem_sss(I32Rem_Sss) = wasm::i32_rem_s;
    fn i32_rem_ssi(I32Rem_Ssi) = eval::wasmi_i32_rem_ssi;
    fn i32_rem_sis(I32Rem_Sis) = wasm::i32_rem_s;
    fn u32_rem_sss(U32Rem_Sss) = wasm::i32_rem_u;
    fn u32_rem_ssi(U32Rem_Ssi) = eval::wasmi_u32_rem_ssi;
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
    fn i32_shl_ssi(I32Shl_Ssi) = eval::wasmi_i32_shl_ssi;
    fn i32_shl_sis(I32Shl_Sis) = wasm::i32_shl;
    fn i32_shr_sss(I32Shr_Sss) = wasm::i32_shr_s;
    fn i32_shr_ssi(I32Shr_Ssi) = eval::wasmi_i32_shr_ssi;
    fn i32_shr_sis(I32Shr_Sis) = wasm::i32_shr_s;
    fn u32_shr_sss(U32Shr_Sss) = wasm::i32_shr_u;
    fn u32_shr_ssi(U32Shr_Ssi) = eval::wasmi_u32_shr_ssi;
    fn u32_shr_sis(U32Shr_Sis) = wasm::i32_shr_u;
    fn i32_rotl_sss(I32Rotl_Sss) = wasm::i32_rotl;
    fn i32_rotl_ssi(I32Rotl_Ssi) = eval::wasmi_i32_rotl_ssi;
    fn i32_rotl_sis(I32Rotl_Sis) = wasm::i32_rotl;
    fn i32_rotr_sss(I32Rotr_Sss) = wasm::i32_rotr;
    fn i32_rotr_ssi(I32Rotr_Ssi) = eval::wasmi_i32_rotr_ssi;
    fn i32_rotr_sis(I32Rotr_Sis) = wasm::i32_rotr;
    // i64
    // i64: commutative
    fn i64_eq_sss(I64Eq_Sss) = wasm::i64_eq;
    fn i64_eq_ssi(I64Eq_Ssi) = wasm::i64_eq;
    fn i64_and_sss(I64And_Sss) = eval::wasmi_i64_and;
    fn i64_and_ssi(I64And_Ssi) = eval::wasmi_i64_and;
    fn i64_or_sss(I64Or_Sss) = eval::wasmi_i64_or;
    fn i64_or_ssi(I64Or_Ssi) = eval::wasmi_i64_or;
    fn i64_not_and_sss(I64NotAnd_Sss) = eval::wasmi_i64_not_and;
    fn i64_not_and_ssi(I64NotAnd_Ssi) = eval::wasmi_i64_not_and;
    fn i64_not_or_sss(I64NotOr_Sss) = eval::wasmi_i64_not_or;
    fn i64_not_or_ssi(I64NotOr_Ssi) = eval::wasmi_i64_not_or;
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
    fn i64_div_ssi(I64Div_Ssi) = eval::wasmi_i64_div_ssi;
    fn i64_div_sis(I64Div_Sis) = wasm::i64_div_s;
    fn u64_div_sss(U64Div_Sss) = wasm::i64_div_u;
    fn u64_div_ssi(U64Div_Ssi) = eval::wasmi_u64_div_ssi;
    fn u64_div_sis(U64Div_Sis) = wasm::i64_div_u;
    fn i64_rem_sss(I64Rem_Sss) = wasm::i64_rem_s;
    fn i64_rem_ssi(I64Rem_Ssi) = eval::wasmi_i64_rem_ssi;
    fn i64_rem_sis(I64Rem_Sis) = wasm::i64_rem_s;
    fn u64_rem_sss(U64Rem_Sss) = wasm::i64_rem_u;
    fn u64_rem_ssi(U64Rem_Ssi) = eval::wasmi_u64_rem_ssi;
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
    fn i64_shl_ssi(I64Shl_Ssi) = eval::wasmi_i64_shl_ssi;
    fn i64_shl_sis(I64Shl_Sis) = wasm::i64_shl;
    fn i64_shr_sss(I64Shr_Sss) = wasm::i64_shr_s;
    fn i64_shr_ssi(I64Shr_Ssi) = eval::wasmi_i64_shr_ssi;
    fn i64_shr_sis(I64Shr_Sis) = wasm::i64_shr_s;
    fn u64_shr_sss(U64Shr_Sss) = wasm::i64_shr_u;
    fn u64_shr_ssi(U64Shr_Ssi) = eval::wasmi_u64_shr_ssi;
    fn u64_shr_sis(U64Shr_Sis) = wasm::i64_shr_u;
    fn i64_rotl_sss(I64Rotl_Sss) = wasm::i64_rotl;
    fn i64_rotl_ssi(I64Rotl_Ssi) = eval::wasmi_i64_rotl_ssi;
    fn i64_rotl_sis(I64Rotl_Sis) = wasm::i64_rotl;
    fn i64_rotr_sss(I64Rotr_Sss) = wasm::i64_rotr;
    fn i64_rotr_ssi(I64Rotr_Ssi) = eval::wasmi_i64_rotr_ssi;
    fn i64_rotr_sis(I64Rotr_Sis) = wasm::i64_rotr;
    // f32
    // f32: binary
    fn f32_add_sss(F32Add_Sss) = wasm::f32_add;
    fn f32_add_ssi(F32Add_Ssi) = wasm::f32_add;
    fn f32_add_sis(F32Add_Sis) = wasm::f32_add;
    fn f32_sub_sss(F32Sub_Sss) = wasm::f32_sub;
    fn f32_sub_ssi(F32Sub_Ssi) = wasm::f32_sub;
    fn f32_sub_sis(F32Sub_Sis) = wasm::f32_sub;
    fn f32_mul_sss(F32Mul_Sss) = wasm::f32_mul;
    fn f32_mul_ssi(F32Mul_Ssi) = wasm::f32_mul;
    fn f32_mul_sis(F32Mul_Sis) = wasm::f32_mul;
    fn f32_div_sss(F32Div_Sss) = wasm::f32_div;
    fn f32_div_ssi(F32Div_Ssi) = wasm::f32_div;
    fn f32_div_sis(F32Div_Sis) = wasm::f32_div;
    fn f32_min_sss(F32Min_Sss) = wasm::f32_min;
    fn f32_min_ssi(F32Min_Ssi) = wasm::f32_min;
    fn f32_min_sis(F32Min_Sis) = wasm::f32_min;
    fn f32_max_sss(F32Max_Sss) = wasm::f32_max;
    fn f32_max_ssi(F32Max_Ssi) = wasm::f32_max;
    fn f32_max_sis(F32Max_Sis) = wasm::f32_max;
    fn f32_copysign_sss(F32Copysign_Sss) = wasm::f32_copysign;
    fn f32_copysign_ssi(F32Copysign_Ssi) = eval::wasmi_f32_copysign_ssi;
    fn f32_copysign_sis(F32Copysign_Sis) = wasm::f32_copysign;
    // f32: comparisons
    fn f32_eq_sss(F32Eq_Sss) = wasm::f32_eq;
    fn f32_eq_ssi(F32Eq_Ssi) = wasm::f32_eq;
    fn f32_lt_sss(F32Lt_Sss) = wasm::f32_lt;
    fn f32_lt_ssi(F32Lt_Ssi) = wasm::f32_lt;
    fn f32_lt_sis(F32Lt_Sis) = wasm::f32_lt;
    fn f32_le_sss(F32Le_Sss) = wasm::f32_le;
    fn f32_le_ssi(F32Le_Ssi) = wasm::f32_le;
    fn f32_le_sis(F32Le_Sis) = wasm::f32_le;
    fn f32_not_eq_sss(F32NotEq_Sss) = eval::wasmi_f32_not_eq;
    fn f32_not_eq_ssi(F32NotEq_Ssi) = eval::wasmi_f32_not_eq;
    fn f32_not_lt_sss(F32NotLt_Sss) = eval::wasmi_f32_not_lt;
    fn f32_not_lt_ssi(F32NotLt_Ssi) = eval::wasmi_f32_not_lt;
    fn f32_not_lt_sis(F32NotLt_Sis) = eval::wasmi_f32_not_lt;
    fn f32_not_le_sss(F32NotLe_Sss) = eval::wasmi_f32_not_le;
    fn f32_not_le_ssi(F32NotLe_Ssi) = eval::wasmi_f32_not_le;
    fn f32_not_le_sis(F32NotLe_Sis) = eval::wasmi_f32_not_le;
    // f64
    // f64: binary
    fn f64_add_sss(F64Add_Sss) = wasm::f64_add;
    fn f64_add_ssi(F64Add_Ssi) = wasm::f64_add;
    fn f64_add_sis(F64Add_Sis) = wasm::f64_add;
    fn f64_sub_sss(F64Sub_Sss) = wasm::f64_sub;
    fn f64_sub_ssi(F64Sub_Ssi) = wasm::f64_sub;
    fn f64_sub_sis(F64Sub_Sis) = wasm::f64_sub;
    fn f64_mul_sss(F64Mul_Sss) = wasm::f64_mul;
    fn f64_mul_ssi(F64Mul_Ssi) = wasm::f64_mul;
    fn f64_mul_sis(F64Mul_Sis) = wasm::f64_mul;
    fn f64_div_sss(F64Div_Sss) = wasm::f64_div;
    fn f64_div_ssi(F64Div_Ssi) = wasm::f64_div;
    fn f64_div_sis(F64Div_Sis) = wasm::f64_div;
    fn f64_min_sss(F64Min_Sss) = wasm::f64_min;
    fn f64_min_ssi(F64Min_Ssi) = wasm::f64_min;
    fn f64_min_sis(F64Min_Sis) = wasm::f64_min;
    fn f64_max_sss(F64Max_Sss) = wasm::f64_max;
    fn f64_max_ssi(F64Max_Ssi) = wasm::f64_max;
    fn f64_max_sis(F64Max_Sis) = wasm::f64_max;
    fn f64_copysign_sss(F64Copysign_Sss) = wasm::f64_copysign;
    fn f64_copysign_ssi(F64Copysign_Ssi) = eval::wasmi_f64_copysign_ssi;
    fn f64_copysign_sis(F64Copysign_Sis) = wasm::f64_copysign;
    // f64: comparisons
    fn f64_eq_sss(F64Eq_Sss) = wasm::f64_eq;
    fn f64_eq_ssi(F64Eq_Ssi) = wasm::f64_eq;
    fn f64_lt_sss(F64Lt_Sss) = wasm::f64_lt;
    fn f64_lt_ssi(F64Lt_Ssi) = wasm::f64_lt;
    fn f64_lt_sis(F64Lt_Sis) = wasm::f64_lt;
    fn f64_le_sss(F64Le_Sss) = wasm::f64_le;
    fn f64_le_ssi(F64Le_Ssi) = wasm::f64_le;
    fn f64_le_sis(F64Le_Sis) = wasm::f64_le;
    fn f64_not_eq_sss(F64NotEq_Sss) = eval::wasmi_f64_not_eq;
    fn f64_not_eq_ssi(F64NotEq_Ssi) = eval::wasmi_f64_not_eq;
    fn f64_not_lt_sss(F64NotLt_Sss) = eval::wasmi_f64_not_lt;
    fn f64_not_lt_ssi(F64NotLt_Ssi) = eval::wasmi_f64_not_lt;
    fn f64_not_lt_sis(F64NotLt_Sis) = eval::wasmi_f64_not_lt;
    fn f64_not_le_sss(F64NotLe_Sss) = eval::wasmi_f64_not_le;
    fn f64_not_le_ssi(F64NotLe_Ssi) = eval::wasmi_f64_not_le;
    fn f64_not_le_sis(F64NotLe_Sis) = eval::wasmi_f64_not_le;
}

macro_rules! handler_cmp_branch {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: *mut u8,
                mem0_len: usize,
                instance: NonNull<InstanceEntity>,
            ) -> Done {
                let (next_ip, $crate::ir::decode::$decode { offset, lhs, rhs }) = unsafe { ip.decode() };
                let lhs = get_value(lhs, sp);
                let rhs = get_value(rhs, sp);
                let ip = match $eval(lhs, rhs) {
                    true => unsafe { ip.offset(i32::from(offset) as isize) },
                    false => next_ip,
                };
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_cmp_branch! {
    // i32
    fn branch_i32_eq_ss(BranchI32Eq_Ss) = wasm::i32_eq;
    fn branch_i32_eq_si(BranchI32Eq_Si) = wasm::i32_eq;
    fn branch_i32_and_ss(BranchI32And_Ss) = eval::wasmi_i32_and;
    fn branch_i32_and_si(BranchI32And_Si) = eval::wasmi_i32_and;
    fn branch_i32_or_ss(BranchI32Or_Ss) = eval::wasmi_i32_or;
    fn branch_i32_or_si(BranchI32Or_Si) = eval::wasmi_i32_or;
    fn branch_i32_not_eq_ss(BranchI32NotEq_Ss) = wasm::i32_ne;
    fn branch_i32_not_eq_si(BranchI32NotEq_Si) = wasm::i32_ne;
    fn branch_i32_not_and_ss(BranchI32NotAnd_Ss) = eval::wasmi_i32_not_and;
    fn branch_i32_not_and_si(BranchI32NotAnd_Si) = eval::wasmi_i32_not_and;
    fn branch_i32_not_or_ss(BranchI32NotOr_Ss) = eval::wasmi_i32_not_or;
    fn branch_i32_not_or_si(BranchI32NotOr_Si) = eval::wasmi_i32_not_or;
    fn branch_i32_le_ss(BranchI32Le_Ss) = wasm::i32_le_s;
    fn branch_i32_le_si(BranchI32Le_Si) = wasm::i32_le_s;
    fn branch_i32_le_is(BranchI32Le_Is) = wasm::i32_le_s;
    fn branch_i32_lt_ss(BranchI32Lt_Ss) = wasm::i32_lt_s;
    fn branch_i32_lt_si(BranchI32Lt_Si) = wasm::i32_lt_s;
    fn branch_i32_lt_is(BranchI32Lt_Is) = wasm::i32_lt_s;
    fn branch_u32_le_ss(BranchU32Le_Ss) = wasm::i32_le_u;
    fn branch_u32_le_si(BranchU32Le_Si) = wasm::i32_le_u;
    fn branch_u32_le_is(BranchU32Le_Is) = wasm::i32_le_u;
    fn branch_u32_lt_ss(BranchU32Lt_Ss) = wasm::i32_lt_u;
    fn branch_u32_lt_si(BranchU32Lt_Si) = wasm::i32_lt_u;
    fn branch_u32_lt_is(BranchU32Lt_Is) = wasm::i32_lt_u;
    // i64
    fn branch_i64_eq_ss(BranchI64Eq_Ss) = wasm::i64_eq;
    fn branch_i64_eq_si(BranchI64Eq_Si) = wasm::i64_eq;
    fn branch_i64_and_ss(BranchI64And_Ss) = eval::wasmi_i64_and;
    fn branch_i64_and_si(BranchI64And_Si) = eval::wasmi_i64_and;
    fn branch_i64_or_ss(BranchI64Or_Ss) = eval::wasmi_i64_or;
    fn branch_i64_or_si(BranchI64Or_Si) = eval::wasmi_i64_or;
    fn branch_i64_not_eq_ss(BranchI64NotEq_Ss) = wasm::i64_ne;
    fn branch_i64_not_eq_si(BranchI64NotEq_Si) = wasm::i64_ne;
    fn branch_i64_not_and_ss(BranchI64NotAnd_Ss) = eval::wasmi_i64_not_and;
    fn branch_i64_not_and_si(BranchI64NotAnd_Si) = eval::wasmi_i64_not_and;
    fn branch_i64_not_or_ss(BranchI64NotOr_Ss) = eval::wasmi_i64_not_or;
    fn branch_i64_not_or_si(BranchI64NotOr_Si) = eval::wasmi_i64_not_or;
    fn branch_i64_le_ss(BranchI64Le_Ss) = wasm::i64_le_s;
    fn branch_i64_le_si(BranchI64Le_Si) = wasm::i64_le_s;
    fn branch_i64_le_is(BranchI64Le_Is) = wasm::i64_le_s;
    fn branch_i64_lt_ss(BranchI64Lt_Ss) = wasm::i64_lt_s;
    fn branch_i64_lt_si(BranchI64Lt_Si) = wasm::i64_lt_s;
    fn branch_i64_lt_is(BranchI64Lt_Is) = wasm::i64_lt_s;
    fn branch_u64_le_ss(BranchU64Le_Ss) = wasm::i64_le_u;
    fn branch_u64_le_si(BranchU64Le_Si) = wasm::i64_le_u;
    fn branch_u64_le_is(BranchU64Le_Is) = wasm::i64_le_u;
    fn branch_u64_lt_ss(BranchU64Lt_Ss) = wasm::i64_lt_u;
    fn branch_u64_lt_si(BranchU64Lt_Si) = wasm::i64_lt_u;
    fn branch_u64_lt_is(BranchU64Lt_Is) = wasm::i64_lt_u;
    // f32
    fn branch_f32_eq_ss(BranchF32Eq_Ss) = wasm::f32_eq;
    fn branch_f32_eq_si(BranchF32Eq_Si) = wasm::f32_eq;
    fn branch_f32_le_ss(BranchF32Le_Ss) = wasm::f32_le;
    fn branch_f32_le_si(BranchF32Le_Si) = wasm::f32_le;
    fn branch_f32_le_is(BranchF32Le_Is) = wasm::f32_le;
    fn branch_f32_lt_ss(BranchF32Lt_Ss) = wasm::f32_lt;
    fn branch_f32_lt_si(BranchF32Lt_Si) = wasm::f32_lt;
    fn branch_f32_lt_is(BranchF32Lt_Is) = wasm::f32_lt;
    fn branch_f32_not_eq_ss(BranchF32NotEq_Ss) = wasm::f32_ne;
    fn branch_f32_not_eq_si(BranchF32NotEq_Si) = wasm::f32_ne;
    fn branch_f32_not_le_ss(BranchF32NotLe_Ss) = eval::wasmi_f32_not_le;
    fn branch_f32_not_le_si(BranchF32NotLe_Si) = eval::wasmi_f32_not_le;
    fn branch_f32_not_le_is(BranchF32NotLe_Is) = eval::wasmi_f32_not_le;
    fn branch_f32_not_lt_ss(BranchF32NotLt_Ss) = eval::wasmi_f32_not_lt;
    fn branch_f32_not_lt_si(BranchF32NotLt_Si) = eval::wasmi_f32_not_lt;
    fn branch_f32_not_lt_is(BranchF32NotLt_Is) = eval::wasmi_f32_not_lt;
    // f64
    fn branch_f64_eq_ss(BranchF64Eq_Ss) = wasm::f64_eq;
    fn branch_f64_eq_si(BranchF64Eq_Si) = wasm::f64_eq;
    fn branch_f64_le_ss(BranchF64Le_Ss) = wasm::f64_le;
    fn branch_f64_le_si(BranchF64Le_Si) = wasm::f64_le;
    fn branch_f64_le_is(BranchF64Le_Is) = wasm::f64_le;
    fn branch_f64_lt_ss(BranchF64Lt_Ss) = wasm::f64_lt;
    fn branch_f64_lt_si(BranchF64Lt_Si) = wasm::f64_lt;
    fn branch_f64_lt_is(BranchF64Lt_Is) = wasm::f64_lt;
    fn branch_f64_not_eq_ss(BranchF64NotEq_Ss) = wasm::f64_ne;
    fn branch_f64_not_eq_si(BranchF64NotEq_Si) = wasm::f64_ne;
    fn branch_f64_not_le_ss(BranchF64NotLe_Ss) = eval::wasmi_f64_not_le;
    fn branch_f64_not_le_si(BranchF64NotLe_Si) = eval::wasmi_f64_not_le;
    fn branch_f64_not_le_is(BranchF64NotLe_Is) = eval::wasmi_f64_not_le;
    fn branch_f64_not_lt_ss(BranchF64NotLt_Ss) = eval::wasmi_f64_not_lt;
    fn branch_f64_not_lt_si(BranchF64NotLt_Si) = eval::wasmi_f64_not_lt;
    fn branch_f64_not_lt_is(BranchF64NotLt_Is) = eval::wasmi_f64_not_lt;
}

macro_rules! handler_select {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: *mut u8,
                mem0_len: usize,
                instance: NonNull<InstanceEntity>,
            ) -> Done {
                let (
                    ip,
                    $crate::ir::decode::$decode {
                        result,
                        val_true,
                        val_false,
                        lhs,
                        rhs,
                    },
                ) = unsafe { ip.decode() };
                let lhs = get_value(lhs, sp);
                let rhs = get_value(rhs, sp);
                let src = match $eval(lhs, rhs) {
                    true => val_true,
                    false => val_false,
                };
                let src: UntypedVal = get_value(src, sp);
                set_value(sp, result, src);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_select! {
    // i32
    fn select_i32_eq_sss(SelectI32Eq_Sss) = wasm::i32_eq;
    fn select_i32_eq_ssi(SelectI32Eq_Ssi) = wasm::i32_eq;
    fn select_i32_and_sss(SelectI32And_Sss) = eval::wasmi_i32_and;
    fn select_i32_and_ssi(SelectI32And_Ssi) = eval::wasmi_i32_and;
    fn select_i32_or_sss(SelectI32Or_Sss) = eval::wasmi_i32_or;
    fn select_i32_or_ssi(SelectI32Or_Ssi) = eval::wasmi_i32_or;
    fn select_i32_le_sss(SelectI32Le_Sss) = wasm::i32_le_s;
    fn select_i32_le_ssi(SelectI32Le_Ssi) = wasm::i32_le_s;
    fn select_i32_lt_sss(SelectI32Lt_Sss) = wasm::i32_lt_s;
    fn select_i32_lt_ssi(SelectI32Lt_Ssi) = wasm::i32_lt_s;
    fn select_u32_le_sss(SelectU32Le_Sss) = wasm::i32_le_u;
    fn select_u32_le_ssi(SelectU32Le_Ssi) = wasm::i32_le_u;
    fn select_u32_lt_sss(SelectU32Lt_Sss) = wasm::i32_lt_u;
    fn select_u32_lt_ssi(SelectU32Lt_Ssi) = wasm::i32_lt_u;
    // i64
    fn select_i64_eq_sss(SelectI64Eq_Sss) = wasm::i64_eq;
    fn select_i64_eq_ssi(SelectI64Eq_Ssi) = wasm::i64_eq;
    fn select_i64_and_sss(SelectI64And_Sss) = eval::wasmi_i64_and;
    fn select_i64_and_ssi(SelectI64And_Ssi) = eval::wasmi_i64_and;
    fn select_i64_or_sss(SelectI64Or_Sss) = eval::wasmi_i64_or;
    fn select_i64_or_ssi(SelectI64Or_Ssi) = eval::wasmi_i64_or;
    fn select_i64_le_sss(SelectI64Le_Sss) = wasm::i64_le_s;
    fn select_i64_le_ssi(SelectI64Le_Ssi) = wasm::i64_le_s;
    fn select_i64_lt_sss(SelectI64Lt_Sss) = wasm::i64_lt_s;
    fn select_i64_lt_ssi(SelectI64Lt_Ssi) = wasm::i64_lt_s;
    fn select_u64_le_sss(SelectU64Le_Sss) = wasm::i64_le_u;
    fn select_u64_le_ssi(SelectU64Le_Ssi) = wasm::i64_le_u;
    fn select_u64_lt_sss(SelectU64Lt_Sss) = wasm::i64_lt_u;
    fn select_u64_lt_ssi(SelectU64Lt_Ssi) = wasm::i64_lt_u;
    // f32
    fn select_f32_eq_sss(SelectF32Eq_Sss) = wasm::f32_eq;
    fn select_f32_eq_ssi(SelectF32Eq_Ssi) = wasm::f32_eq;
    fn select_f32_le_sss(SelectF32Le_Sss) = wasm::f32_le;
    fn select_f32_le_ssi(SelectF32Le_Ssi) = wasm::f32_le;
    fn select_f32_le_sis(SelectF32Le_Sis) = wasm::f32_le;
    fn select_f32_lt_sss(SelectF32Lt_Sss) = wasm::f32_lt;
    fn select_f32_lt_ssi(SelectF32Lt_Ssi) = wasm::f32_lt;
    fn select_f32_lt_sis(SelectF32Lt_Sis) = wasm::f32_lt;
    // f64
    fn select_f64_eq_sss(SelectF64Eq_Sss) = wasm::f64_eq;
    fn select_f64_eq_ssi(SelectF64Eq_Ssi) = wasm::f64_eq;
    fn select_f64_le_sss(SelectF64Le_Sss) = wasm::f64_le;
    fn select_f64_le_ssi(SelectF64Le_Ssi) = wasm::f64_le;
    fn select_f64_le_sis(SelectF64Le_Sis) = wasm::f64_le;
    fn select_f64_lt_sss(SelectF64Lt_Sss) = wasm::f64_lt;
    fn select_f64_lt_ssi(SelectF64Lt_Ssi) = wasm::f64_lt;
    fn select_f64_lt_sis(SelectF64Lt_Sis) = wasm::f64_lt;
}

macro_rules! handler_load_ss {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: *mut u8,
                mem0_len: usize,
                instance: NonNull<InstanceEntity>,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$decode {
                        result,
                        ptr,
                        offset,
                        memory,
                    },
                ) = unsafe { ip.decode() };
                let ptr: u64 = get_value(ptr, sp);
                let offset: u64 = get_value(offset, sp);
                let mem_bytes = memory_bytes(memory, mem0, mem0_len, instance, state);
                let loaded = unwrap_result!($eval(mem_bytes, ptr, offset), state);
                set_value(sp, result, loaded);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_load_ss! {
    fn load32_ss(Load32_Ss) = wasm::load32;
    fn load64_ss(Load64_Ss) = wasm::load64;
    fn i32_load8_ss(I32Load8_Ss) = wasm::i32_load8_s;
    fn u32_load8_ss(U32Load8_Ss) = wasm::i32_load8_u;
    fn i32_load16_ss(I32Load16_Ss) = wasm::i32_load16_s;
    fn u32_load16_ss(U32Load16_Ss) = wasm::i32_load16_u;
    fn i64_load8_ss(I64Load8_Ss) = wasm::i64_load8_s;
    fn u64_load8_ss(U64Load8_Ss) = wasm::i64_load8_u;
    fn i64_load16_ss(I64Load16_Ss) = wasm::i64_load16_s;
    fn u64_load16_ss(U64Load16_Ss) = wasm::i64_load16_u;
    fn i64_load32_ss(I64Load32_Ss) = wasm::i64_load32_s;
    fn u64_load32_ss(U64Load32_Ss) = wasm::i64_load32_u;
}

macro_rules! handler_load_si {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: *mut u8,
                mem0_len: usize,
                instance: NonNull<InstanceEntity>,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$decode {
                        result,
                        address,
                        memory,
                    },
                ) = unsafe { ip.decode() };
                let address = get_value(address, sp);
                let mem_bytes = memory_bytes(memory, mem0, mem0_len, instance, state);
                let loaded = unwrap_result!($eval(mem_bytes, usize::from(address)), state);
                set_value(sp, result, loaded);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_load_si! {
    fn load32_si(Load32_Si) = wasm::load32_at;
    fn load64_si(Load64_Si) = wasm::load64_at;
    fn i32_load8_si(I32Load8_Si) = wasm::i32_load8_s_at;
    fn u32_load8_si(U32Load8_Si) = wasm::i32_load8_u_at;
    fn i32_load16_si(I32Load16_Si) = wasm::i32_load16_s_at;
    fn u32_load16_si(U32Load16_Si) = wasm::i32_load16_u_at;
    fn i64_load8_si(I64Load8_Si) = wasm::i64_load8_s_at;
    fn u64_load8_si(U64Load8_Si) = wasm::i64_load8_u_at;
    fn i64_load16_si(I64Load16_Si) = wasm::i64_load16_s_at;
    fn u64_load16_si(U64Load16_Si) = wasm::i64_load16_u_at;
    fn i64_load32_si(I64Load32_Si) = wasm::i64_load32_s_at;
    fn u64_load32_si(U64Load32_Si) = wasm::i64_load32_u_at;
}

macro_rules! handler_load_mem0_offset16_ss {
    ( $( fn $handler:ident($decode:ident) = $eval:expr );* $(;)? ) => {
        $(
            pub fn $handler(
                state: &mut VmState,
                ip: Ip,
                sp: Sp,
                mem0: *mut u8,
                mem0_len: usize,
                instance: NonNull<InstanceEntity>,
            ) -> Done {
                let (
                    ip,
                    crate::ir::decode::$decode {
                        result,
                        ptr,
                        offset,
                    },
                ) = unsafe { ip.decode() };
                let ptr = get_value(ptr, sp);
                let offset = get_value(offset, sp);
                let mem_bytes = default_memory_bytes(mem0, mem0_len);
                let loaded = unwrap_result!($eval(mem_bytes, ptr, u64::from(u16::from(offset))), state);
                set_value(sp, result, loaded);
                dispatch!(state, ip, sp, mem0, mem0_len, instance)
            }
        )*
    };
}
handler_load_mem0_offset16_ss! {
    fn load32_mem0_offset16_ss(Load32Mem0Offset16_Ss) = wasm::load32;
    fn load64_mem0_offset16_ss(Load64Mem0Offset16_Ss) = wasm::load64;
    fn i32_load8_mem0_offset16_ss(I32Load8Mem0Offset16_Ss) = wasm::i32_load8_s;
    fn u32_load8_mem0_offset16_ss(U32Load8Mem0Offset16_Ss) = wasm::i32_load8_u;
    fn i32_load16_mem0_offset16_ss(I32Load16Mem0Offset16_Ss) = wasm::i32_load16_s;
    fn u32_load16_mem0_offset16_ss(U32Load16Mem0Offset16_Ss) = wasm::i32_load16_u;
    fn i64_load8_mem0_offset16_ss(I64Load8Mem0Offset16_Ss) = wasm::i64_load8_s;
    fn u64_load8_mem0_offset16_ss(U64Load8Mem0Offset16_Ss) = wasm::i64_load8_u;
    fn i64_load16_mem0_offset16_ss(I64Load16Mem0Offset16_Ss) = wasm::i64_load16_s;
    fn u64_load16_mem0_offset16_ss(U64Load16Mem0Offset16_Ss) = wasm::i64_load16_u;
    fn i64_load32_mem0_offset16_ss(I64Load32Mem0Offset16_Ss) = wasm::i64_load32_s;
    fn u64_load32_mem0_offset16_ss(U64Load32Mem0Offset16_Ss) = wasm::i64_load32_u;
}
