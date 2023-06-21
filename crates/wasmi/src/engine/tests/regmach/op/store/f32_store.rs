use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::F32, "store");

#[test]
fn reg() {
    test_store(WASM_OP, 0, Instruction::f32_store);
    test_store(WASM_OP, 42, Instruction::f32_store);
    test_store(WASM_OP, u32::MAX, Instruction::f32_store);
}

#[test]
fn imm() {
    test_store_imm::<f32>(
        WASM_OP,
        0,
        1.0,
        Instruction::f32_store_imm,
        Instruction::const32,
    );
    test_store_imm::<f32>(
        WASM_OP,
        42,
        1.0,
        Instruction::f32_store_imm,
        Instruction::const32,
    );
    test_store_imm::<f32>(
        WASM_OP,
        u32::MAX,
        1.0,
        Instruction::f32_store_imm,
        Instruction::const32,
    );
}

#[test]
fn at() {
    test_store_at(WASM_OP, 0, 0, Instruction::f32_store_at);
    test_store_at(WASM_OP, 5, 42, Instruction::f32_store_at);
    test_store_at(WASM_OP, 0, u32::MAX, Instruction::f32_store_at);
    test_store_at(WASM_OP, u32::MAX, 0, Instruction::f32_store_at);
}

#[test]
fn at_overflow() {
    test_store_at_overflow(WASM_OP, u32::MAX, 1);
    test_store_at_overflow(WASM_OP, 1, u32::MAX);
    test_store_at_overflow(WASM_OP, u32::MAX, u32::MAX);
}

#[test]
fn imm_at() {
    test_store_imm_at::<f32>(
        WASM_OP,
        0,
        0,
        1.0,
        Instruction::f32_store_imm_at,
        Instruction::const32,
    );
    test_store_imm_at::<f32>(
        WASM_OP,
        5,
        42,
        1.0,
        Instruction::f32_store_imm_at,
        Instruction::const32,
    );
    test_store_imm_at::<f32>(
        WASM_OP,
        42,
        5,
        1.0,
        Instruction::f32_store_imm_at,
        Instruction::const32,
    );
    test_store_imm_at::<f32>(
        WASM_OP,
        0,
        u32::MAX,
        1.0,
        Instruction::f32_store_imm_at,
        Instruction::const32,
    );
    test_store_imm_at::<f32>(
        WASM_OP,
        u32::MAX,
        0,
        1.0,
        Instruction::f32_store_imm_at,
        Instruction::const32,
    );
}

#[test]
fn imm_at_overflow() {
    test_store_imm_at_overflow(WASM_OP, 1, u32::MAX, 1);
    test_store_imm_at_overflow(WASM_OP, u32::MAX, 1, 1);
    test_store_imm_at_overflow(WASM_OP, u32::MAX, u32::MAX, 1);
}
