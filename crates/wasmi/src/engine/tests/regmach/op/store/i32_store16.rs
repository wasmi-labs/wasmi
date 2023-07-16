use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I32, "store16");

#[test]
fn reg() {
    test_store(WASM_OP, Instruction::i32_store16);
}

#[test]
fn imm() {
    test_store_imm::<i32>(WASM_OP, 0, Instruction::i32_store16);
    test_store_imm::<i32>(WASM_OP, 1, Instruction::i32_store16);
    test_store_imm::<i32>(WASM_OP, -1, Instruction::i32_store16);
    test_store_imm::<i32>(WASM_OP, 42, Instruction::i32_store16);
    test_store_imm::<i32>(WASM_OP, i32::MIN, Instruction::i32_store16);
    test_store_imm::<i32>(WASM_OP, i32::MAX, Instruction::i32_store16);
}

#[test]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::i32_store16_offset16);
}

#[test]
fn offset16_imm() {
    test_store_offset16_imm::<i32>(
        WASM_OP,
        i32::from(i16::MIN) - 1,
        Instruction::i32_store16_offset16,
    );
    test_store_offset16_imm::<i32>(
        WASM_OP,
        i32::from(i16::MAX) + 1,
        Instruction::i32_store16_offset16,
    );
    test_store_offset16_imm::<i32>(WASM_OP, i32::MIN + 1, Instruction::i32_store16_offset16);
    test_store_offset16_imm::<i32>(WASM_OP, i32::MAX - 1, Instruction::i32_store16_offset16);
    test_store_offset16_imm::<i32>(WASM_OP, i32::MIN, Instruction::i32_store16_offset16);
    test_store_offset16_imm::<i32>(WASM_OP, i32::MAX, Instruction::i32_store16_offset16);
}

#[test]
fn offset16_imm16() {
    test_store_offset16_imm16::<i16>(WASM_OP, 0, Instruction::i32_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, 1, Instruction::i32_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, -1, Instruction::i32_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, i16::MIN + 1, Instruction::i32_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, i16::MAX - 1, Instruction::i32_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, i16::MIN, Instruction::i32_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, i16::MAX, Instruction::i32_store16_offset16_imm);
}

#[test]
fn at() {
    test_store_at(WASM_OP, Instruction::i32_store16_at);
}

#[test]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
fn at_imm() {
    test_store_at_imm::<i32>(
        WASM_OP,
        i32::from(i16::MAX) + 1,
        Instruction::i32_store16_at,
    );
    test_store_at_imm::<i32>(WASM_OP, i32::MAX - 1, Instruction::i32_store16_at);
    test_store_at_imm::<i32>(WASM_OP, i32::MAX, Instruction::i32_store16_at);
}

#[test]
fn imm_at_overflow() {
    test_store_at_imm_overflow(WASM_OP, 0);
    test_store_at_imm_overflow(WASM_OP, 1);
    test_store_at_imm_overflow(WASM_OP, -1);
    test_store_at_imm_overflow(WASM_OP, 42);
    test_store_at_imm_overflow(WASM_OP, i32::MIN);
    test_store_at_imm_overflow(WASM_OP, i32::MAX);
}
