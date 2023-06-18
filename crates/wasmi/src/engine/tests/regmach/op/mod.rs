mod f32_add;
mod f32_copysign;
mod f32_div;
mod f32_eq;
mod f32_max;
mod f32_min;
mod f32_mul;
mod f32_ne;
mod f32_sub;
mod f64_add;
mod f64_copysign;
mod f64_div;
mod f64_eq;
mod f64_max;
mod f64_min;
mod f64_mul;
mod f64_ne;
mod f64_sub;
mod i32_add;
mod i32_and;
mod i32_div_s;
mod i32_div_u;
mod i32_eq;
mod i32_eqz;
mod i32_gt_s;
mod i32_gt_u;
mod i32_lt_s;
mod i32_lt_u;
mod i32_mul;
mod i32_ne;
mod i32_or;
mod i32_rem_s;
mod i32_rem_u;
mod i32_rotl;
mod i32_rotr;
mod i32_shl;
mod i32_shr_s;
mod i32_shr_u;
mod i32_sub;
mod i32_xor;
mod i64_add;
mod i64_and;
mod i64_div_s;
mod i64_div_u;
mod i64_eq;
mod i64_eqz;
mod i64_mul;
mod i64_ne;
mod i64_or;
mod i64_rem_s;
mod i64_rem_u;
mod i64_rotl;
mod i64_rotr;
mod i64_shl;
mod i64_shr_s;
mod i64_shr_u;
mod i64_sub;
mod i64_xor;
mod unary;

use super::{
    assert_func_bodies,
    swap_ops,
    test_binary_consteval,
    test_binary_reg_imm16,
    test_binary_reg_imm16_rev,
    test_binary_reg_imm32,
    test_binary_reg_imm32_rev,
    test_binary_reg_imm64,
    test_binary_reg_imm64_rev,
    test_binary_reg_imm_rev_with,
    test_binary_reg_imm_with,
    test_binary_reg_reg,
    test_binary_same_reg,
    wat2wasm,
    Const16,
    Const32,
    ConstRef,
    Instruction,
    Register,
    TranslationTest,
    WasmOp,
    WasmType,
};
