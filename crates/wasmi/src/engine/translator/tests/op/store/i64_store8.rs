use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I64, "store8");

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::i64_store8);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    let values = [
        i64::from(i16::MIN) - 1,
        i64::from(i16::MAX) + 1,
        i64::MIN,
        i64::MIN + 1,
        i64::MAX,
        i64::MAX - 1,
    ];
    for value in values {
        test_store_imm::<i64>(WASM_OP, value, Instruction::i64_store8);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm16() {
    let values = [
        0,
        1,
        -1,
        42,
        (i16::MIN + 1) as i8,
        (i16::MIN) as i8,
        (i16::MAX - 1) as i8,
        (i16::MAX) as i8,
    ];
    for value in values {
        test_store_imm16::<i8>(WASM_OP, Instruction::i64_store8_imm, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::i64_store8_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
fn at() {
    test_store_at(WASM_OP, Instruction::i64_store8_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm() {
    test_store_at_imm::<i64>(WASM_OP, i64::from(i16::MAX) + 1, Instruction::i64_store8_at);
    test_store_at_imm::<i64>(WASM_OP, i64::MAX - 1, Instruction::i64_store8_at);
    test_store_at_imm::<i64>(WASM_OP, i64::MAX, Instruction::i64_store8_at);
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
