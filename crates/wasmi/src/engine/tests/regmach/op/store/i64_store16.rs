use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I64, "store16");

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::i64_store16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    test_store_imm::<i64>(WASM_OP, 0, Instruction::i64_store16);
    test_store_imm::<i64>(WASM_OP, 1, Instruction::i64_store16);
    test_store_imm::<i64>(WASM_OP, -1, Instruction::i64_store16);
    test_store_imm::<i64>(WASM_OP, 42, Instruction::i64_store16);
    test_store_imm::<i64>(WASM_OP, i64::MIN, Instruction::i64_store16);
    test_store_imm::<i64>(WASM_OP, i64::MAX, Instruction::i64_store16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::i64_store16_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm() {
    test_store_offset16_imm::<i64>(
        WASM_OP,
        i64::from(i16::MIN) - 1,
        Instruction::i64_store16_offset16,
    );
    test_store_offset16_imm::<i64>(
        WASM_OP,
        i64::from(i16::MAX) + 1,
        Instruction::i64_store16_offset16,
    );
    test_store_offset16_imm::<i64>(WASM_OP, i64::MIN + 1, Instruction::i64_store16_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MAX - 1, Instruction::i64_store16_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MIN, Instruction::i64_store16_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MAX, Instruction::i64_store16_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm16() {
    test_store_offset16_imm16::<i16>(WASM_OP, 0, Instruction::i64_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, 1, Instruction::i64_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, -1, Instruction::i64_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, i16::MIN + 1, Instruction::i64_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, i16::MAX - 1, Instruction::i64_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, i16::MIN, Instruction::i64_store16_offset16_imm);
    test_store_offset16_imm16::<i16>(WASM_OP, i16::MAX, Instruction::i64_store16_offset16_imm);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at() {
    test_store_at(WASM_OP, Instruction::i64_store16_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm() {
    test_store_at_imm::<i64>(
        WASM_OP,
        i64::from(i16::MAX) + 1,
        Instruction::i64_store16_at,
    );
    test_store_at_imm::<i64>(WASM_OP, i64::MAX - 1, Instruction::i64_store16_at);
    test_store_at_imm::<i64>(WASM_OP, i64::MAX, Instruction::i64_store16_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_at_overflow() {
    test_store_at_imm_overflow(WASM_OP, 0);
    test_store_at_imm_overflow(WASM_OP, 1);
    test_store_at_imm_overflow(WASM_OP, -1);
    test_store_at_imm_overflow(WASM_OP, 42);
    test_store_at_imm_overflow(WASM_OP, i64::MIN);
    test_store_at_imm_overflow(WASM_OP, i64::MAX);
}
