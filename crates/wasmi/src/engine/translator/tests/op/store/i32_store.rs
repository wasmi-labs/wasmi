use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::I32, "store");

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::store32);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    let values = [
        i32::from(i16::MIN) - 1,
        i32::from(i16::MAX) + 1,
        i32::MIN,
        i32::MIN + 1,
        i32::MAX,
        i32::MAX - 1,
    ];
    for value in values {
        test_store_imm::<i32>(WASM_OP, Instruction::store32, value);
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
        test_store_imm16::<i32>(WASM_OP, Instruction::i32_store_imm16, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16() {
    test_store_offset16(WASM_OP, Instruction::store32_offset16);
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm() {
    [
        i32::from(i16::MIN) - 1,
        i32::from(i16::MAX) + 1,
        i32::MIN + 1,
        i32::MAX - 1,
        i32::MAX,
    ]
    .into_iter()
    .for_each(|value| {
        test_store_offset16_imm::<i32>(WASM_OP, value, Instruction::store32_offset16);
    })
}

#[test]
#[cfg_attr(miri, ignore)]
fn offset16_imm16() {
    [
        0,
        -1,
        1,
        -42,
        42,
        i16::MIN + 1,
        i16::MIN,
        i16::MAX - 1,
        i16::MAX,
    ]
    .into_iter()
    .for_each(|value| {
        test_store_offset16_imm16::<i16>(WASM_OP, Instruction::i32_store_offset16_imm16, value);
    })
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
    [i32::from(i16::MAX) + 1, i32::MAX - 1, i32::MAX]
        .into_iter()
        .for_each(|value| {
            test_store_at_imm::<i32>(WASM_OP, value, Instruction::store32_at);
        })
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_at_overflow() {
    [
        0,
        1,
        -1,
        42,
        -42,
        i32::MIN,
        i32::MIN + 1,
        i32::MAX - 1,
        i32::MAX,
    ]
    .into_iter()
    .for_each(|value| {
        test_store_at_imm_overflow(WASM_OP, value);
    })
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm16_fallback() {
    [
        0,
        -1,
        1,
        -42,
        42,
        i32::from(i16::MIN),
        i32::from(i16::MIN) + 1,
        i32::from(i16::MAX) - 1,
        i32::from(i16::MAX),
    ]
    .into_iter()
    .for_each(|value| {
        test_store_at_imm16_fallback::<i32>(WASM_OP, Instruction::i32_store_imm16, value);
    })
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_fallback() {
    for value in [
        i32::from(i16::MIN) - 1,
        i32::from(i16::MAX) + 1,
        i32::MIN,
        i32::MIN + 1,
        i32::MAX - 1,
        i32::MAX,
    ] {
        test_store_at_imm_fallback::<i32>(WASM_OP, Instruction::store32, value);
    }
}
