use super::*;

#[test]
#[cfg_attr(miri, ignore)]
fn copy() {
    let wasm = r"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $dst i32) (param $src i32) (param $len i32)
                (local.get $dst)
                (local.get $src)
                (local.get $len)
                (memory.copy $mem0 $mem1)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_copy(Reg::from(0), Reg::from(1), Reg::from(2)),
            Instruction::memory_index(0),
            Instruction::memory_index(1),
            Instruction::Return,
        ])
        .run()
}

fn testcase_copy_exact(len: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $dst i32) (param $src i32)
                (local.get $dst)
                (local.get $src)
                (i32.const {len})
                (memory.copy $mem0 $mem1)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_exact16(len: u64) {
    testcase_copy_exact(len)
        .expect_func_instrs([
            Instruction::memory_copy_imm(Reg::from(0), Reg::from(1), u64imm16(len)),
            Instruction::memory_index(0),
            Instruction::memory_index(1),
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
    test_copy_exact16(u64::from(u16::MAX));
}

fn test_copy_exact(len: u64) {
    testcase_copy_exact(len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(0), Reg::from(1), Reg::from(-1)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_exact() {
    test_copy_exact(u64::from(u16::MAX) + 1);
    test_copy_exact(u64::from(u32::MAX));
}

fn testcase_copy_from(src: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $dst i32) (param $len i32)
                (local.get $dst)
                (i32.const {src})
                (local.get $len)
                (memory.copy $mem0 $mem1)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_from16(src: u64) {
    testcase_copy_from(src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from16() {
    test_copy_from16(0);
    test_copy_from16(u64::from(u16::MAX));
}

fn test_copy_from(src: u64) {
    testcase_copy_from(src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from() {
    test_copy_from(u64::from(u16::MAX) + 1);
    test_copy_from(u64::from(u32::MAX));
}

fn testcase_copy_to(dst: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $src i32) (param $len i32)
                (i32.const {dst})
                (local.get $src)
                (local.get $len)
                (memory.copy $mem0 $mem1)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_to16(dst: u64) {
    testcase_copy_to(dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to16() {
    test_copy_to16(0);
    test_copy_to16(u64::from(u16::MAX));
}

fn test_copy_to(dst: u64) {
    testcase_copy_to(dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to() {
    test_copy_to(u64::from(u16::MAX) + 1);
    test_copy_to(u64::from(u32::MAX));
}

fn testcase_copy_from_to(dst: u64, src: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $len i32)
                (i32.const {dst})
                (i32.const {src})
                (local.get $len)
                (memory.copy $mem0 $mem1)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_from_to16(dst: u64, src: u64) {
    testcase_copy_from_to(dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([dst, src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to16() {
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for src in values {
            if dst == src {
                continue;
            }
            test_copy_from_to16(dst, src);
        }
    }
}

fn test_copy_from_to(dst: u64, src: u64) {
    testcase_copy_from_to(dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([dst, src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to() {
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
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

fn testcase_copy_to_exact(dst: u64, len: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $src i32)
                (i32.const {dst})
                (local.get $src)
                (i32.const {len})
                (memory.copy $mem0 $mem1)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_to_exact16(dst: u64, len: u64) {
    testcase_copy_to_exact(dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy_imm(Reg::from(-1), Reg::from(0), u64imm16(len)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to_exact16() {
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_copy_to_exact16(dst, len);
        }
    }
}

fn test_copy_to_exact(dst: u64, len: u64) {
    testcase_copy_to_exact(dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(-1), Reg::from(0), Reg::from(-2)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([dst, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to_exact() {
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
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

fn testcase_copy_from_exact(src: u64, len: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $dst i32)
                (local.get $dst)
                (i32.const {src})
                (i32.const {len})
                (memory.copy $mem0 $mem1)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_from_exact16(src: u64, len: u64) {
    testcase_copy_from_exact(src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy_imm(Reg::from(0), Reg::from(-1), u64imm16(len)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_exact16() {
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_copy_from_exact16(dst, len);
        }
    }
}

fn test_copy_from_exact(src: u64, len: u64) {
    testcase_copy_from_exact(src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(0), Reg::from(-1), Reg::from(-2)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_exact() {
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
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

fn testcase_copy_from_to_exact(dst: u64, src: u64, len: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func
                (i32.const {dst})
                (i32.const {src})
                (i32.const {len})
                (memory.copy $mem0 $mem1)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_from_to_exact16(dst: u64, src: u64, len: u64) {
    testcase_copy_from_to_exact(dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy_imm(Reg::from(-1), Reg::from(-2), u64imm16(len)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([dst, src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to_exact16() {
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for src in values {
            if dst == src {
                continue;
            }
            for len in values {
                test_copy_from_to_exact16(dst, src, len);
            }
        }
    }
}

fn test_copy_from_to_exact(dst: u64, src: u64, len: u64) {
    testcase_copy_from_to_exact(dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_copy(Reg::from(-1), Reg::from(-2), Reg::from(-3)),
                Instruction::memory_index(0),
                Instruction::memory_index(1),
                Instruction::Return,
            ])
            .consts([dst, src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to_exact() {
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
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
