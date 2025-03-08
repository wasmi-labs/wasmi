use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::F32, "store");

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::store32);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    test_store_imm::<f32>(WASM_OP, Instruction::store32, 0.0);
    test_store_imm::<f32>(WASM_OP, Instruction::store32, 1.0);
    test_store_imm::<f32>(WASM_OP, Instruction::store32, -1.0);
    test_store_imm::<f32>(WASM_OP, Instruction::store32, 42.25);
    test_store_imm::<f32>(WASM_OP, Instruction::store32, f32::INFINITY);
    test_store_imm::<f32>(WASM_OP, Instruction::store32, f32::NEG_INFINITY);
    test_store_imm::<f32>(WASM_OP, Instruction::store32, f32::NAN);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::store32_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm() {
    test_store_offset16_imm::<f32>(WASM_OP, 0.0, Instruction::store32_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, 1.0, Instruction::store32_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, -1.0, Instruction::store32_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, f32::INFINITY, Instruction::store32_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, f32::NEG_INFINITY, Instruction::store32_offset16);
    test_store_offset16_imm::<f32>(WASM_OP, f32::NAN, Instruction::store32_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at() {
    test_store_at(WASM_OP, Instruction::store32_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_fallback() {
    test_store_at_fallback(WASM_OP, Instruction::store32);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm() {
    test_store_at_imm::<f32>(WASM_OP, 0.0, Instruction::store32_at);
    test_store_at_imm::<f32>(WASM_OP, 1.0, Instruction::store32_at);
    test_store_at_imm::<f32>(WASM_OP, -1.0, Instruction::store32_at);
    test_store_at_imm::<f32>(WASM_OP, f32::NEG_INFINITY, Instruction::store32_at);
    test_store_at_imm::<f32>(WASM_OP, f32::INFINITY, Instruction::store32_at);
    test_store_at_imm::<f32>(WASM_OP, f32::NAN, Instruction::store32_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_overflow() {
    test_store_at_imm_overflow(WASM_OP, 0.0);
    test_store_at_imm_overflow(WASM_OP, 1.0);
    test_store_at_imm_overflow(WASM_OP, -1.0);
    test_store_at_imm_overflow(WASM_OP, 42.25);
    test_store_at_imm_overflow(WASM_OP, f32::NEG_INFINITY);
    test_store_at_imm_overflow(WASM_OP, f32::INFINITY);
    test_store_at_imm_overflow(WASM_OP, f32::NAN);
}
