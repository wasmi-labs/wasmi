use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I64, "store");

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::store64);
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
        test_store_imm::<i64>(WASM_OP, Instruction::store64, value);
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
        i32::from(i16::MIN) + 1,
        i32::from(i16::MIN),
        i32::from(i16::MAX) - 1,
        i32::from(i16::MAX),
    ];
    for value in values {
        test_store_imm16::<i32>(WASM_OP, Instruction::i64_store_imm16, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::store64_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm() {
    test_store_offset16_imm::<i64>(
        WASM_OP,
        i64::from(i16::MIN) - 1,
        Instruction::store64_offset16,
    );
    test_store_offset16_imm::<i64>(
        WASM_OP,
        i64::from(i16::MAX) + 1,
        Instruction::store64_offset16,
    );
    test_store_offset16_imm::<i64>(WASM_OP, i64::MAX - 1, Instruction::store64_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MIN + 1, Instruction::store64_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MIN, Instruction::store64_offset16);
    test_store_offset16_imm::<i64>(WASM_OP, i64::MAX, Instruction::store64_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm16() {
    test_store_offset16_imm16::<i16>(WASM_OP, Instruction::i64_store_offset16_imm16, 0);
    test_store_offset16_imm16::<i16>(WASM_OP, Instruction::i64_store_offset16_imm16, 1);
    test_store_offset16_imm16::<i16>(WASM_OP, Instruction::i64_store_offset16_imm16, -1);
    test_store_offset16_imm16::<i16>(WASM_OP, Instruction::i64_store_offset16_imm16, i16::MIN + 1);
    test_store_offset16_imm16::<i16>(WASM_OP, Instruction::i64_store_offset16_imm16, i16::MAX - 1);
    test_store_offset16_imm16::<i16>(WASM_OP, Instruction::i64_store_offset16_imm16, i16::MIN);
    test_store_offset16_imm16::<i16>(WASM_OP, Instruction::i64_store_offset16_imm16, i16::MAX);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at() {
    test_store_at(WASM_OP, Instruction::store64_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_fallback() {
    test_store_at_fallback(WASM_OP, Instruction::store64);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm() {
    test_store_at_imm::<i64>(WASM_OP, i64::from(i16::MAX) + 1, Instruction::store64_at);
    test_store_at_imm::<i64>(WASM_OP, i64::MAX - 1, Instruction::store64_at);
    test_store_at_imm::<i64>(WASM_OP, i64::MAX, Instruction::store64_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_at_overflow() {
    [0, 1, -1, 42, i64::MIN, i64::MAX]
        .into_iter()
        .for_each(|value| {
            test_store_at_imm_overflow(WASM_OP, value);
        })
}
