use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::F32, "store");

#[test]
fn reg() {
    test_store(WASM_OP, Instruction::f32_store);
}

#[test]
fn imm() {
    test_store_imm::<f32>(WASM_OP, 0.0, Instruction::f32_store);
    test_store_imm::<f32>(WASM_OP, 1.0, Instruction::f32_store);
    test_store_imm::<f32>(WASM_OP, -1.0, Instruction::f32_store);
    test_store_imm::<f32>(WASM_OP, 42.25, Instruction::f32_store);
    test_store_imm::<f32>(WASM_OP, f32::INFINITY, Instruction::f32_store);
    test_store_imm::<f32>(WASM_OP, f32::NEG_INFINITY, Instruction::f32_store);
    test_store_imm::<f32>(WASM_OP, f32::NAN, Instruction::f32_store);
}

#[test]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::f32_store_offset16);
}

#[test]
fn offset16_imm() {
    test_store_offset16_imm::<f32>(WASM_OP, 0.0, Instruction::f32_store_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, 1.0, Instruction::f32_store_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, -1.0, Instruction::f32_store_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, f32::INFINITY, Instruction::f32_store_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, f32::NEG_INFINITY, Instruction::f32_store_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, f32::NAN, Instruction::f32_store_offset16);
}

#[test]
fn at() {
    test_store_at(WASM_OP, Instruction::f32_store_at);
}

#[test]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
fn at_imm() {
    test_store_at_imm::<f32>(WASM_OP, 0.0, Instruction::f32_store_at);
    test_store_at_imm::<f32>(WASM_OP, 1.0, Instruction::f32_store_at);
    test_store_at_imm::<f32>(WASM_OP, -1.0, Instruction::f32_store_at);
    test_store_at_imm::<f32>(WASM_OP, f32::NEG_INFINITY, Instruction::f32_store_at);
    test_store_at_imm::<f32>(WASM_OP, f32::INFINITY, Instruction::f32_store_at);
    test_store_at_imm::<f32>(WASM_OP, f32::NAN, Instruction::f32_store_at);
}

#[test]
fn at_imm_overflow() {
    test_store_at_imm_overflow(WASM_OP, 0.0);
    test_store_at_imm_overflow(WASM_OP, 1.0);
    test_store_at_imm_overflow(WASM_OP, -1.0);
    test_store_at_imm_overflow(WASM_OP, 42.25);
    test_store_at_imm_overflow(WASM_OP, f32::NEG_INFINITY);
    test_store_at_imm_overflow(WASM_OP, f32::INFINITY);
    test_store_at_imm_overflow(WASM_OP, f32::NAN);
}
