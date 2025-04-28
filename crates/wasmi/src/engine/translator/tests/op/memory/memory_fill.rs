use super::*;

#[test]
#[cfg_attr(miri, ignore)]
fn fill() {
    let wasm = r"
        (module
            (memory 1)
            (func (param $dst i32) (param $value i32) (param $len i32)
                (local.get $dst)
                (local.get $value)
                (local.get $len)
                (memory.fill)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_fill(Reg::from(0), Reg::from(1), Reg::from(2)),
            Instruction::memory_index(0),
            Instruction::Return,
        ])
        .run()
}

fn testcase_fill_exact(len: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (func (param $dst i32) (param $value i32)
                (local.get $dst)
                (local.get $value)
                (i32.const {len})
                (memory.fill)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_fill_exact16(len: u64) {
    testcase_fill_exact(len)
        .expect_func_instrs([
            Instruction::memory_fill_exact(Reg::from(0), Reg::from(1), u64imm16(len)),
            Instruction::memory_index(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_exact16() {
    test_fill_exact16(0);
    test_fill_exact16(1);
    test_fill_exact16(42);
    test_fill_exact16(u64::from(u16::MAX));
}

fn test_fill_exact(len: u64) {
    testcase_fill_exact(len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill(Reg::from(0), Reg::from(1), Reg::from(-1)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_exact() {
    test_fill_exact(u64::from(u16::MAX) + 1);
    test_fill_exact(u64::from(u32::MAX));
}

fn testcase_fill_imm(value: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (func (param $dst i32) (param $len i32)
                (local.get $dst)
                (i32.const {value})
                (local.get $len)
                (memory.fill)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_fill_imm(value: u32) {
    testcase_fill_imm(value)
        .expect_func_instrs([
            Instruction::memory_fill_imm(Reg::from(0), value as u8, Reg::from(1)),
            Instruction::memory_index(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_imm() {
    test_fill_imm(0);
    test_fill_imm(42);
    test_fill_imm(u32::from(u16::MAX));
    test_fill_imm(u32::from(u16::MAX) + 1);
    test_fill_imm(u32::MAX - 1);
    test_fill_imm(u32::MAX);
}

fn testcase_fill_at(dst: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (func (param $value i32) (param $len i32)
                (i32.const {dst})
                (local.get $value)
                (local.get $len)
                (memory.fill)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_fill_at16(dst: u64) {
    testcase_fill_at(dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at16() {
    test_fill_at16(0);
    test_fill_at16(u64::from(u16::MAX));
}

fn test_fill_at(dst: u64) {
    testcase_fill_at(dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at() {
    test_fill_at(u64::from(u16::MAX) + 1);
    test_fill_at(u64::from(u32::MAX));
}

fn testcase_fill_at_imm(dst: u64, value: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (func (param $len i32)
                (i32.const {dst})
                (i32.const {value})
                (local.get $len)
                (memory.fill)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_fill_at16_imm(dst: u64, value: u64) {
    testcase_fill_at_imm(dst, value)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill_imm(Reg::from(-1), value as u8, Reg::from(0)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at16_imm() {
    let dst_values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    let test_values = [
        0,
        1,
        42,
        u64::from(u16::MAX) - 1,
        u64::from(u16::MAX),
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
    for dst in dst_values {
        for value in test_values {
            test_fill_at16_imm(dst, value);
        }
    }
}

fn test_fill_at_imm(dst: u64, value: u64) {
    testcase_fill_at_imm(dst, value)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill_imm(Reg::from(-1), value as u8, Reg::from(0)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_imm() {
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
    for dst in values {
        for value in values {
            if dst == value {
                // We skip here because equal `dst` and `value` would
                // allocate just a single function local constant value
                // which our testcase is not prepared for.
                // Ideally we'd have yet another test for that case.
                continue;
            }
            test_fill_at_imm(dst, value);
        }
    }
}

fn testcase_fill_at_exact(dst: u64, len: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (func (param $value i32)
                (i32.const {dst})
                (local.get $value)
                (i32.const {len})
                (memory.fill)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_fill_at_exact16(dst: u64, len: u64) {
    testcase_fill_at_exact(dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill_exact(Reg::from(-1), Reg::from(0), u64imm16(len)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_to_exact16() {
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_fill_at_exact16(dst, len);
        }
    }
}

fn test_fill_at_exact(dst: u64, len: u64) {
    testcase_fill_at_exact(dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill(Reg::from(-1), Reg::from(0), Reg::from(-2)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([dst, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_exact() {
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
    for dst in values {
        for value in values {
            if dst == value {
                // We skip here because equal `dst` and `value` would
                // allocate just a single function local constant value
                // which our testcase is not prepared for.
                // Ideally we'd have yet another test for that case.
                continue;
            }
            test_fill_at_exact(dst, value);
        }
    }
}

fn testcase_fill_imm_exact(value: u64, len: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (func (param $dst i32)
                (local.get $dst)
                (i32.const {value})
                (i32.const {len})
                (memory.fill)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_fill_imm_exact16(value: u64, len: u64) {
    testcase_fill_imm_exact(value, len)
        .expect_func_instrs([
            Instruction::memory_fill_imm_exact(Reg::from(0), value as u8, u64imm16(len)),
            Instruction::memory_index(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_imm_exact16() {
    let len_values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    let values = [
        0,
        1,
        42,
        u64::from(u16::MAX) - 1,
        u64::from(u16::MAX),
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
    for value in values {
        for len in len_values {
            test_fill_imm_exact16(value, len);
        }
    }
}

fn test_fill_imm_exact(value: u64, len: u64) {
    testcase_fill_imm_exact(value, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill_imm(Reg::from(0), value as u8, Reg::from(-1)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_imm_exact() {
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
    for dst in values {
        for value in values {
            if dst == value {
                // We skip here because equal `dst` and `value` would
                // allocate just a single function local constant value
                // which our testcase is not prepared for.
                // Ideally we'd have yet another test for that case.
                continue;
            }
            test_fill_imm_exact(dst, value);
        }
    }
}

fn testcase_fill_at_imm_exact(dst: u64, value: u64, len: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (func
                (i32.const {dst})
                (i32.const {value})
                (i32.const {len})
                (memory.fill)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_fill_at_imm_exact16(dst: u64, value: u64, len: u64) {
    testcase_fill_at_imm_exact(dst, value, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill_imm_exact(Reg::from(-1), value as u8, u64imm16(len)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_imm_exact16() {
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for value in values {
            for len in values {
                test_fill_at_imm_exact16(dst, value, len);
            }
        }
    }
}

fn test_fill_at_imm_exact(dst: u64, value: u64, len: u64) {
    testcase_fill_at_imm_exact(dst, value, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_fill_imm(Reg::from(-1), value as u8, Reg::from(-2)),
                Instruction::memory_index(0),
                Instruction::Return,
            ])
            .consts([dst, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_imm_exact() {
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
    for dst in values {
        for value in values {
            for len in values {
                if dst == value || value == len || dst == len {
                    // We skip here because equal `dst` and `value` would
                    // allocate just a single function local constant value
                    // which our testcase is not prepared for.
                    // Ideally we'd have yet another test for that case.
                    continue;
                }
                test_fill_at_imm_exact(dst, value, len);
            }
        }
    }
}
