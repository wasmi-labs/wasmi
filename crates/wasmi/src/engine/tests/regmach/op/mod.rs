mod binary;
mod block;
mod br;
mod br_if;
mod cmp;
mod global_get;
mod global_set;
mod load;
mod loop_;
mod return_;
mod select;
mod store;
mod unary;

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
    wasm_type,
    wat2wasm,
    Const16,
    Const32,
    ConstRef,
    DisplayWasm,
    Instruction,
    Register,
    TranslationTest,
    WasmOp,
    WasmType,
};
