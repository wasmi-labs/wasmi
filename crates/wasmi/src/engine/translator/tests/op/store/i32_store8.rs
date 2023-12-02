use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I32, "store8");

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::i32_store8);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    test_store_imm::<i32>(WASM_OP, 0, Instruction::i32_store8);
    test_store_imm::<i32>(WASM_OP, 1, Instruction::i32_store8);
    test_store_imm::<i32>(WASM_OP, -1, Instruction::i32_store8);
    test_store_imm::<i32>(WASM_OP, 42, Instruction::i32_store8);
    test_store_imm::<i32>(WASM_OP, i32::MIN, Instruction::i32_store8);
    test_store_imm::<i32>(WASM_OP, i32::MAX, Instruction::i32_store8);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::i32_store8_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm() {
    test_store_offset16_imm::<i32>(
        WASM_OP,
        i32::from(i16::MIN) - 1,
        Instruction::i32_store8_offset16,
    );
    test_store_offset16_imm::<i32>(
        WASM_OP,
        i32::from(i16::MAX) + 1,
        Instruction::i32_store8_offset16,
    );
    test_store_offset16_imm::<i32>(WASM_OP, i32::MIN + 1, Instruction::i32_store8_offset16);
    test_store_offset16_imm::<i32>(WASM_OP, i32::MAX - 1, Instruction::i32_store8_offset16);
    test_store_offset16_imm::<i32>(WASM_OP, i32::MIN, Instruction::i32_store8_offset16);
    test_store_offset16_imm::<i32>(WASM_OP, i32::MAX, Instruction::i32_store8_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm16() {
    test_store_offset16_imm16::<i8>(WASM_OP, 0, Instruction::i32_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, 1, Instruction::i32_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, -1, Instruction::i32_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, i8::MIN + 1, Instruction::i32_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, i8::MAX - 1, Instruction::i32_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, i8::MIN, Instruction::i32_store8_offset16_imm);
    test_store_offset16_imm16::<i8>(WASM_OP, i8::MAX, Instruction::i32_store8_offset16_imm);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at() {
    test_store_at(WASM_OP, Instruction::i32_store8_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm() {
    test_store_at_imm::<i32>(WASM_OP, i32::from(i16::MAX) + 1, Instruction::i32_store8_at);
    test_store_at_imm::<i32>(WASM_OP, i32::MAX - 1, Instruction::i32_store8_at);
    test_store_at_imm::<i32>(WASM_OP, i32::MAX, Instruction::i32_store8_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_at_overflow() {
    test_store_at_imm_overflow(WASM_OP, 0);
    test_store_at_imm_overflow(WASM_OP, 1);
    test_store_at_imm_overflow(WASM_OP, -1);
    test_store_at_imm_overflow(WASM_OP, 42);
    test_store_at_imm_overflow(WASM_OP, i32::MIN);
    test_store_at_imm_overflow(WASM_OP, i32::MAX);
}
