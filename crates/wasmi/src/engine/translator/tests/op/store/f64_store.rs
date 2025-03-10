use super::*;

const WASM_OP: WasmOp = WasmOp::store(WasmType::F64, "store");

const DEFAULT_TEST_VALUES: [f64; 10] = [
    0.0,
    0.5,
    -0.5,
    1.0,
    -1.0,
    42.25,
    -42.25,
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::NAN,
];

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    test_store(WASM_OP, Instruction::store64);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    for value in DEFAULT_TEST_VALUES {
        test_store_imm::<f64>(WASM_OP, Instruction::store64, value);
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
    for value in DEFAULT_TEST_VALUES {
        test_store_offset16_imm::<f64>(WASM_OP, value, Instruction::store64_offset16);
    }
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
    for value in DEFAULT_TEST_VALUES {
        test_store_at_imm::<f64>(WASM_OP, value, Instruction::store64_at);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_overflow() {
    for value in DEFAULT_TEST_VALUES {
        test_store_at_imm_overflow(WASM_OP, value);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn at_imm_fallback() {
    for value in DEFAULT_TEST_VALUES {
        test_store_at_imm_fallback(WASM_OP, Instruction::store64, value);
    }
}
