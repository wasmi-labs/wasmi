use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I64, "store32");

fn make_instr_param(value: i64) -> Instruction {
    Instruction::const32(value as i32)
}

#[test]
fn reg() {
    test_store(WASM_OP, 0, Instruction::i64_store32);
    test_store(WASM_OP, 42, Instruction::i64_store32);
    test_store(WASM_OP, u32::MAX, Instruction::i64_store32);
}

#[test]
fn imm() {
    test_store_imm::<i64>(
        WASM_OP,
        0,
        1,
        Instruction::i64_store32_imm,
        make_instr_param,
    );
    test_store_imm::<i64>(
        WASM_OP,
        42,
        1,
        Instruction::i64_store32_imm,
        make_instr_param,
    );
    test_store_imm::<i64>(
        WASM_OP,
        u32::MAX,
        1,
        Instruction::i64_store32_imm,
        make_instr_param,
    );
}

#[test]
fn at() {
    test_store_at(WASM_OP, 0, 0, Instruction::i64_store32_at);
    test_store_at(WASM_OP, 5, 42, Instruction::i64_store32_at);
    test_store_at(WASM_OP, 0, u32::MAX, Instruction::i64_store32_at);
    test_store_at(WASM_OP, u32::MAX, 0, Instruction::i64_store32_at);
}

#[test]
fn at_overflow() {
    test_store_at_overflow(WASM_OP, u32::MAX, 1);
    test_store_at_overflow(WASM_OP, 1, u32::MAX);
    test_store_at_overflow(WASM_OP, u32::MAX, u32::MAX);
}

#[test]
fn imm_at() {
    test_store_imm_at::<i64>(
        WASM_OP,
        0,
        0,
        1,
        Instruction::i64_store32_imm_at,
        make_instr_param,
    );
    test_store_imm_at::<i64>(
        WASM_OP,
        5,
        42,
        1,
        Instruction::i64_store32_imm_at,
        make_instr_param,
    );
    test_store_imm_at::<i64>(
        WASM_OP,
        42,
        5,
        1,
        Instruction::i64_store32_imm_at,
        make_instr_param,
    );
    test_store_imm_at::<i64>(
        WASM_OP,
        0,
        u32::MAX,
        1,
        Instruction::i64_store32_imm_at,
        make_instr_param,
    );
    test_store_imm_at::<i64>(
        WASM_OP,
        u32::MAX,
        0,
        1,
        Instruction::i64_store32_imm_at,
        make_instr_param,
    );
}

#[test]
fn imm_at_overflow() {
    test_store_imm_at_overflow(WASM_OP, 1, u32::MAX, 1);
    test_store_imm_at_overflow(WASM_OP, u32::MAX, 1, 1);
    test_store_imm_at_overflow(WASM_OP, u32::MAX, u32::MAX, 1);
}
