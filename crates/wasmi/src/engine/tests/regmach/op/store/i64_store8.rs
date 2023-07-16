use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I64, "store8");

#[test]
fn reg() {
    test_store(WASM_OP, Instruction::i64_store8);
}

#[test]
fn imm() {
    test_store_imm::<i64>(WASM_OP, 0, Instruction::i64_store8);
    test_store_imm::<i64>(WASM_OP, 1, Instruction::i64_store8);
    test_store_imm::<i64>(WASM_OP, -1, Instruction::i64_store8);
    test_store_imm::<i64>(WASM_OP, 42, Instruction::i64_store8);
    test_store_imm::<i64>(WASM_OP, i64::MIN, Instruction::i64_store8);
    test_store_imm::<i64>(WASM_OP, i64::MAX, Instruction::i64_store8);
}

#[test]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::i64_store8_offset16);
}

#[test]
fn offset16_imm() {
    test_store_offset16_imm::<i64>(
        WASM_OP,
        i64::from(i16::MIN) - 1,
        Instruction::i64_store8_offset16,
    );
    test_store_offset16_imm::<i64>(
        WASM_OP,
        i64::from(i16::MAX) + 1,
        Instruction::i64_store8_offset16,
    );
    test_store_offset16_imm::<i64>(WASM_OP, i64::MIN + 1, Instruction::i64_store8_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MAX - 1, Instruction::i64_store8_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MIN, Instruction::i64_store8_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MAX, Instruction::i64_store8_offset16);
}

#[test]
fn offset16_imm16() {
    test_store_offset16_imm16::<i8>(WASM_OP, 0, Instruction::i64_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, 1, Instruction::i64_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, -1, Instruction::i64_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, i8::MIN + 1, Instruction::i64_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, i8::MAX - 1, Instruction::i64_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, i8::MIN, Instruction::i64_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, i8::MAX, Instruction::i64_store8_offset16_imm);
}

#[test]
fn at() {
    test_store_at(WASM_OP, Instruction::i64_store8_at);
}

#[test]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
fn at_imm() {
    test_store_at_imm::<i64>(WASM_OP, i64::from(i16::MAX) + 1, Instruction::i64_store8_at);
    test_store_at_imm::<i64>(WASM_OP, i64::MAX - 1, Instruction::i64_store8_at);
    test_store_at_imm::<i64>(WASM_OP, i64::MAX, Instruction::i64_store8_at);
}

#[test]
fn imm_at_overflow() {
    test_store_at_imm_overflow(WASM_OP, 0);
    test_store_at_imm_overflow(WASM_OP, 1);
    test_store_at_imm_overflow(WASM_OP, -1);
    test_store_at_imm_overflow(WASM_OP, 42);
    test_store_at_imm_overflow(WASM_OP, i64::MIN);
    test_store_at_imm_overflow(WASM_OP, i64::MAX);
}
