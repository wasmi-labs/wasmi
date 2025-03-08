use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I64, "store32");

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::i64_store32);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    let values = [
        i64::from(i16::MIN) - 1,
        i64::from(i16::MAX) + 1,
        i64::from(i32::MIN) + i64::from(i16::MAX),
        i64::from(i32::MIN) + i64::from(i16::MAX) - 1,
    ];
    for value in values {
        test_store_imm::<i64>(WASM_OP, Instruction::i64_store32, value);
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
        i64::from(i16::MIN) + 1,
        i64::from(i16::MIN),
        i64::from(i16::MAX) - 1,
        i64::from(i16::MAX),
        i64::MIN,
        i64::MIN + 1,
        i64::MAX,
        i64::MAX - 1,
    ];
    for value in values {
        test_store_wrap_imm::<i64, i32, i16>(WASM_OP, Instruction::i64_store32_imm16, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::i64_store32_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm() {
    let values = [
        i64::from(i16::MIN) - 1,
        i64::from(i16::MAX) + 1,
        i64::from(i32::MIN) + i64::from(i16::MAX),
        i64::from(i32::MIN) + i64::from(i16::MAX) - 1,
    ];
    for value in values {
        test_store_offset16_imm::<i64>(WASM_OP, value, Instruction::i64_store32_offset16);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm16() {
    let values = [
        0,
        1,
        -1,
        1000,
        -1000,
        i64::from(i16::MIN) + 1,
        i64::from(i16::MAX) - 1,
        i64::from(i16::MIN),
        i64::from(i16::MAX),
        i64::MAX - 1,
        i64::MIN + 1,
        i64::MIN,
        i64::MAX,
    ];
    for value in values {
        test_store_wrap_offset16_imm::<i64, i32, i16>(
            WASM_OP,
            value,
            Instruction::i64_store32_offset16_imm16,
        );
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn at() {
    test_store_at(WASM_OP, Instruction::i64_store32_at);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_overflow() {
    test_store_at_overflow(WASM_OP);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm() {
    let values = [
        i64::from(i16::MIN) - 1,
        i64::from(i16::MAX) + 1,
        i64::from(i32::MIN) + i64::from(i16::MAX),
        i64::from(i32::MIN) + i64::from(i16::MAX) - 1,
    ];
    for value in values {
        test_store_at_imm::<i64>(WASM_OP, value, Instruction::i64_store32_at);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_fallback() {
    test_store_at_fallback(WASM_OP, Instruction::i64_store32);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm16() {
    let values = [
        0,
        1,
        -1000,
        1000,
        i64::from(i16::MIN),
        i64::from(i16::MIN) + 1,
        i64::from(i16::MAX) - 1,
        i64::from(i16::MAX),
        i64::MIN,
        i64::MIN + 1,
        i64::MAX - 1,
        i64::MAX,
    ];
    for value in values {
        test_store_wrap_at_imm::<i64, i32, i16>(WASM_OP, Instruction::i64_store32_at_imm16, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_at_overflow() {
    let values = [0, 1, -1, 1000, -1000, i64::MIN, i64::MAX];
    for value in values {
        test_store_at_imm_overflow(WASM_OP, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_fallback() {
    [
        0,
        1,
        -1,
        1000,
        -1000,
        i64::from(i16::MIN),
        i64::from(i16::MIN) + 1,
        i64::from(i16::MAX) - 1,
        i64::from(i16::MAX),
        i64::MIN,
        i64::MIN + 1,
        i64::MAX - 1,
        i64::MAX,
    ]
    .into_iter()
    .for_each(|value| {
        test_store_wrap_at_imm_fallback::<i64, i32>(WASM_OP, Instruction::i64_store32_imm16, value);
    });
}
