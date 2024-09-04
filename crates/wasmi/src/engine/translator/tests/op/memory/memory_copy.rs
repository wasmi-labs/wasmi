use super::*;

#[test]
#[cfg_attr(miri, ignore)]
fn copy() {
    let wasm = r"
        (module
            (memory $m1 10)
            (func (param $dst i32) (param $src i32) (param $len i32)
                (local.get $dst)
                (local.get $src)
                (local.get $len)
                (memory.copy)
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::memory_copy(Reg::from_i16(0), Reg::from_i16(1), Reg::from_i16(2)),
            Instruction::Return,
        ])
        .run()
}

fn testcase_copy_exact(len: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $m1 10)
            (func (param $dst i32) (param $src i32)
                (local.get $dst)
                (local.get $src)
                (i32.const {len})
                (memory.copy)
            )
        )",
    );
    TranslationTest::from_wat(wasm)
}

fn test_copy_exact16(len: u32) {
    testcase_copy_exact(len)
        .expect_func_instrs([
            Instruction::memory_copy_exact(Reg::from_i16(0), Reg::from_i16(1), u32imm16(len)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_exact16() {
    test_copy_exact16(0);
    test_copy_exact16(1);
    test_copy_exact16(42);
    test_copy_exact16(u32::from(u16::MAX));
}

fn test_copy_exact(len: u32) {
    testcase_copy_exact(len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from_i16(0), Reg::from_i16(1), Reg::from_i16(-1)),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_exact() {
    test_copy_exact(u32::from(u16::MAX) + 1);
    test_copy_exact(u32::MAX);
}

fn testcase_copy_from(src: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $m1 10)
            (func (param $dst i32) (param $len i32)
                (local.get $dst)
                (i32.const {src})
                (local.get $len)
                (memory.copy)
            )
        )",
    );
    TranslationTest::from_wat(wasm)
}

fn test_copy_from16(src: u32) {
    testcase_copy_from(src)
        .expect_func_instrs([
            Instruction::memory_copy_from(Reg::from_i16(0), u32imm16(src), Reg::from_i16(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from16() {
    test_copy_from16(0);
    test_copy_from16(u32::from(u16::MAX));
}

fn test_copy_from(src: u32) {
    testcase_copy_from(src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from_i16(0), Reg::from_i16(-1), Reg::from_i16(1)),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from() {
    test_copy_from(u32::from(u16::MAX) + 1);
    test_copy_from(u32::MAX);
}

fn testcase_copy_to(dst: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $m1 10)
            (func (param $src i32) (param $len i32)
                (i32.const {dst})
                (local.get $src)
                (local.get $len)
                (memory.copy)
            )
        )",
    );
    TranslationTest::from_wat(wasm)
}

fn test_copy_to16(dst: u32) {
    testcase_copy_to(dst)
        .expect_func_instrs([
            Instruction::memory_copy_to(u32imm16(dst), Reg::from_i16(0), Reg::from_i16(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to16() {
    test_copy_to16(0);
    test_copy_to16(u32::from(u16::MAX));
}

fn test_copy_to(dst: u32) {
    testcase_copy_to(dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from_i16(-1), Reg::from_i16(0), Reg::from_i16(1)),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to() {
    test_copy_to(u32::from(u16::MAX) + 1);
    test_copy_to(u32::MAX);
}

fn testcase_copy_from_to(dst: u32, src: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $m1 10)
            (func (param $len i32)
                (i32.const {dst})
                (i32.const {src})
                (local.get $len)
                (memory.copy)
            )
        )",
    );
    TranslationTest::from_wat(wasm)
}

fn test_copy_from_to16(dst: u32, src: u32) {
    testcase_copy_from_to(dst, src)
        .expect_func_instrs([
            Instruction::memory_copy_from_to(u32imm16(dst), u32imm16(src), Reg::from_i16(0)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to16() {
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for src in values {
            test_copy_from_to16(dst, src);
        }
    }
}

fn test_copy_from_to(dst: u32, src: u32) {
    testcase_copy_from_to(dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from_i16(-1), Reg::from_i16(-2), Reg::from_i16(0)),
                Instruction::Return,
            ])
            .consts([dst, src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to() {
    let values = [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX];
    for dst in values {
        for src in values {
            if dst == src {
                // We skip here because equal `dst` and `src` would
                // allocate just a single function local constant value
                // which our testcase is not prepared for.
                // Ideally we'd have yet another test for that case.
                continue;
            }
            test_copy_from_to(dst, src);
        }
    }
}

fn testcase_copy_to_exact(dst: u32, len: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $m1 10)
            (func (param $src i32)
                (i32.const {dst})
                (local.get $src)
                (i32.const {len})
                (memory.copy)
            )
        )",
    );
    TranslationTest::from_wat(wasm)
}

fn test_copy_to_exact16(dst: u32, len: u32) {
    testcase_copy_to_exact(dst, len)
        .expect_func_instrs([
            Instruction::memory_copy_to_exact(u32imm16(dst), Reg::from_i16(0), u32imm16(len)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to_exact16() {
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_copy_to_exact16(dst, len);
        }
    }
}

fn test_copy_to_exact(dst: u32, len: u32) {
    testcase_copy_to_exact(dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from_i16(-1), Reg::from_i16(0), Reg::from_i16(-2)),
                Instruction::Return,
            ])
            .consts([dst, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to_exact() {
    let values = [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX];
    for dst in values {
        for src in values {
            if dst == src {
                // We skip here because equal `dst` and `src` would
                // allocate just a single function local constant value
                // which our testcase is not prepared for.
                // Ideally we'd have yet another test for that case.
                continue;
            }
            test_copy_to_exact(dst, src);
        }
    }
}

fn testcase_copy_from_exact(src: u32, len: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $m1 10)
            (func (param $dst i32)
                (local.get $dst)
                (i32.const {src})
                (i32.const {len})
                (memory.copy)
            )
        )",
    );
    TranslationTest::from_wat(wasm)
}

fn test_copy_from_exact16(src: u32, len: u32) {
    testcase_copy_from_exact(src, len)
        .expect_func_instrs([
            Instruction::memory_copy_from_exact(Reg::from_i16(0), u32imm16(src), u32imm16(len)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_exact16() {
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_copy_from_exact16(dst, len);
        }
    }
}

fn test_copy_from_exact(src: u32, len: u32) {
    testcase_copy_from_exact(src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from_i16(0), Reg::from_i16(-1), Reg::from_i16(-2)),
                Instruction::Return,
            ])
            .consts([src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_exact() {
    let values = [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX];
    for dst in values {
        for src in values {
            if dst == src {
                // We skip here because equal `dst` and `src` would
                // allocate just a single function local constant value
                // which our testcase is not prepared for.
                // Ideally we'd have yet another test for that case.
                continue;
            }
            test_copy_from_exact(dst, src);
        }
    }
}

fn testcase_copy_from_to_exact(dst: u32, src: u32, len: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $m1 10)
            (func
                (i32.const {dst})
                (i32.const {src})
                (i32.const {len})
                (memory.copy)
            )
        )",
    );
    TranslationTest::from_wat(wasm)
}

fn test_copy_from_to_exact16(dst: u32, src: u32, len: u32) {
    testcase_copy_from_to_exact(dst, src, len)
        .expect_func_instrs([
            Instruction::memory_copy_from_to_exact(u32imm16(dst), u32imm16(src), u32imm16(len)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to_exact16() {
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for src in values {
            for len in values {
                test_copy_from_to_exact16(dst, src, len);
            }
        }
    }
}

fn test_copy_from_to_exact(dst: u32, src: u32, len: u32) {
    testcase_copy_from_to_exact(dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from_i16(-1), Reg::from_i16(-2), Reg::from_i16(-3)),
                Instruction::Return,
            ])
            .consts([dst, src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to_exact() {
    let values = [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX];
    for dst in values {
        for src in values {
            for len in values {
                if dst == src || src == len || dst == len {
                    // We skip here because equal `dst` and `src` would
                    // allocate just a single function local constant value
                    // which our testcase is not prepared for.
                    // Ideally we'd have yet another test for that case.
                    continue;
                }
                test_copy_from_to_exact(dst, src, len);
            }
        }
    }
}
