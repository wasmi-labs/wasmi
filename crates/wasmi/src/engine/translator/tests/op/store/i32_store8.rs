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
    let values = [
        0,
        1,
        -1,
        42,
        i32::from(i16::MIN) - 1,
        i32::from(i16::MIN),
        i32::from(i16::MIN + 1),
        i32::from(i16::MAX - 1),
        i32::from(i16::MAX),
        i32::from(i16::MAX) + 1,
        i32::MIN,
        i32::MIN + 1,
        i32::MAX,
        i32::MAX - 1,
    ];
    for value in values {
        test_store_wrap_imm::<i32, i8, i8>(WASM_OP, Instruction::i32_store8_imm, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::i32_store8_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm() {
    let values = [
        0,
        1,
        -1,
        -1000,
        1000,
        i32::from(i8::MIN) - 1,
        i32::from(i8::MIN),
        i32::from(i8::MIN) + 1,
        i32::from(i8::MAX) - 1,
        i32::from(i8::MAX),
        i32::from(i8::MAX) + 1,
        i32::from(i16::MIN) - 1,
        i32::from(i16::MIN),
        i32::from(i16::MIN) + 1,
        i32::from(i16::MAX) - 1,
        i32::from(i16::MAX),
        i32::from(i16::MAX) + 1,
        i32::MIN,
        i32::MIN + 1,
        i32::MAX - 1,
        i32::MAX,
    ];
    for value in values {
        test_store_wrap_offset16_imm::<i32, i8, i8>(
            WASM_OP,
            value,
            Instruction::i32_store8_offset16_imm,
        );
    }
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
fn at_fallback() {
    test_store_at_fallback(WASM_OP, Instruction::i32_store8);
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm() {
    let values = [
        0,
        1,
        -1000,
        1000,
        i32::from(i16::MIN),
        i32::from(i16::MIN) + 1,
        i32::from(i16::MAX) - 1,
        i32::from(i16::MAX),
        i32::MIN,
        i32::MIN + 1,
        i32::MAX - 1,
        i32::MAX,
    ];
    for value in values {
        test_store_wrap_at_imm::<i32, i8, i8>(WASM_OP, Instruction::i32_store8_at_imm, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_at_overflow() {
    let values = [0, 1, -1, i32::MIN, i32::MAX];
    for value in values {
        test_store_at_imm_overflow(WASM_OP, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_fallback() {
    [
        0,
        -1,
        1,
        -42,
        42,
        i32::from(i8::MIN),
        i32::from(i8::MIN) + 1,
        i32::from(i8::MAX) - 1,
        i32::from(i8::MAX),
    ]
    .into_iter()
    .for_each(|value| {
        test_store_wrap_at_imm16_fallback::<i32, i8>(WASM_OP, Instruction::i32_store8_imm, value);
    })
}
