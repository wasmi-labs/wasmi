mod i32_add;
mod i32_and;
mod i32_mul;
mod i32_or;
mod i32_rotl;
mod i32_rotr;
mod i32_shl;
mod i32_shr_s;
mod i32_shr_u;
mod i32_sub;
mod i32_xor;
mod i64_add;
mod i64_and;
mod i64_mul;
mod i64_or;
mod i64_rotl;
mod i64_shl;
mod i64_shr_s;
mod i64_shr_u;
mod i64_sub;
mod i64_xor;

use super::{
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
    Const16,
    Const32,
    Instruction,
    Register,
    WasmOp,
};
