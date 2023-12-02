use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::F64, "store");

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::f64_store);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    test_store_imm::<f64>(WASM_OP, 0.0, Instruction::f64_store);
    test_store_imm::<f64>(WASM_OP, 1.0, Instruction::f64_store);
    test_store_imm::<f64>(WASM_OP, -1.0, Instruction::f64_store);
    test_store_imm::<f64>(WASM_OP, 42.25, Instruction::f64_store);
    test_store_imm::<f64>(WASM_OP, f64::INFINITY, Instruction::f64_store);
    test_store_imm::<f64>(WASM_OP, f64::NEG_INFINITY, Instruction::f64_store);
    test_store_imm::<f64>(WASM_OP, f64::NAN, Instruction::f64_store);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::f64_store_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm() {
    test_store_offset16_imm::<f64>(WASM_OP, 0.0, Instruction::f64_store_offset16);
    test_store_offset16_imm::<f64>(WASM_OP, 1.0, Instruction::f64_store_offset16);
    test_store_offset16_imm::<f64>(WASM_OP, -1.0, Instruction::f64_store_offset16);
    test_store_offset16_imm::<f64>(WASM_OP, f64::INFINITY, Instruction::f64_store_offset16);
    test_store_offset16_imm::<f64>(WASM_OP, f64::NEG_INFINITY, Instruction::f64_store_offset16);
    test_store_offset16_imm::<f64>(WASM_OP, f64::NAN, Instruction::f64_store_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at() {
    test_store_at(WASM_OP, Instruction::f64_store_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm() {
    test_store_at_imm::<f64>(WASM_OP, 0.0, Instruction::f64_store_at);
    test_store_at_imm::<f64>(WASM_OP, 1.0, Instruction::f64_store_at);
    test_store_at_imm::<f64>(WASM_OP, -1.0, Instruction::f64_store_at);
    test_store_at_imm::<f64>(WASM_OP, f64::NEG_INFINITY, Instruction::f64_store_at);
    test_store_at_imm::<f64>(WASM_OP, f64::INFINITY, Instruction::f64_store_at);
    test_store_at_imm::<f64>(WASM_OP, f64::NAN, Instruction::f64_store_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_overflow() {
    test_store_at_imm_overflow(WASM_OP, 0.0);
    test_store_at_imm_overflow(WASM_OP, 1.0);
    test_store_at_imm_overflow(WASM_OP, -1.0);
    test_store_at_imm_overflow(WASM_OP, 42.25);
    test_store_at_imm_overflow(WASM_OP, f64::NEG_INFINITY);
    test_store_at_imm_overflow(WASM_OP, f64::INFINITY);
    test_store_at_imm_overflow(WASM_OP, f64::NAN);
}
