use super::*;

#[test]
#[cfg_attr(miri, ignore)]
fn init() {
    let wasm = r"
        (module
            (memory 1)
            (data $d (i32.const 0))
            (func (param $dst i32) (param $src i32) (param $len i32)
                (local.get $dst)
                (local.get $src)
                (local.get $len)
                (memory.init $d)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_init(Reg::from(0), Reg::from(1), Reg::from(2)),
            Instruction::memory_index(0),
            Instruction::data_index(0),
            Instruction::Return,
        ])
        .run()
}

fn testcase_init_exact(len: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (data $d (i32.const 0))
            (func (param $dst i32) (param $src i32)
                (local.get $dst)
                (local.get $src)
                (i32.const {len})
                (memory.init $d)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_exact16(len: u32) {
    testcase_init_exact(len)
        .expect_func_instrs([
            Instruction::memory_init_imm(Reg::from(0), Reg::from(1), u32imm16(len)),
            Instruction::memory_index(0),
            Instruction::data_index(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_exact16() {
    test_copy_exact16(0);
    test_copy_exact16(1);
    test_copy_exact16(42);
    test_copy_exact16(u32::from(u16::MAX));
}

fn test_copy_exact(len: u32) {
    testcase_init_exact(len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(0), Reg::from(1), Reg::from(-1)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_exact() {
    test_copy_exact(u32::from(u16::MAX) + 1);
    test_copy_exact(u32::MAX);
}

fn testcase_init_from(src: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (data $d (i32.const 0))
            (func (param $dst i32) (param $len i32)
                (local.get $dst)
                (i32.const {src})
                (local.get $len)
                (memory.init $d)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_from16(src: u32) {
    testcase_init_from(src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from16() {
    test_copy_from16(0);
    test_copy_from16(u32::from(u16::MAX));
}

fn test_copy_from(src: u32) {
    testcase_init_from(src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from() {
    test_copy_from(u32::from(u16::MAX) + 1);
    test_copy_from(u32::MAX);
}

fn testcase_init_to(dst: u64) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (data $d (i32.const 0))
            (func (param $src i32) (param $len i32)
                (i32.const {dst})
                (local.get $src)
                (local.get $len)
                (memory.init $d)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_to16(dst: u64) {
    testcase_init_to(dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to16() {
    test_copy_to16(0);
    test_copy_to16(u64::from(u16::MAX));
}

fn test_copy_to(dst: u64) {
    testcase_init_to(dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to() {
    test_copy_to(u64::from(u16::MAX) + 1);
    test_copy_to(u64::from(u32::MAX));
}

fn testcase_init_from_to(dst: u64, src: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (data $d (i32.const 0))
            (func (param $len i32)
                (i32.const {dst})
                (i32.const {src})
                (local.get $len)
                (memory.init $d)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_from_to16(dst: u64, src: u32) {
    testcase_init_from_to(dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([dst, u64::from(src)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to16() {
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for src in values {
            if dst == src {
                continue;
            }
            test_copy_from_to16(u64::from(dst), src);
        }
    }
}

fn test_copy_from_to(dst: u64, src: u32) {
    testcase_init_from_to(dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([dst, u64::from(src)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to() {
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
            test_copy_from_to(u64::from(dst), src);
        }
    }
}

fn testcase_init_to_exact(dst: u64, len: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (data $d (i32.const 0))
            (func (param $src i32)
                (i32.const {dst})
                (local.get $src)
                (i32.const {len})
                (memory.init $d)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_to_exact16(dst: u64, len: u32) {
    testcase_init_to_exact(dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init_imm(Reg::from(-1), Reg::from(0), u32imm16(len)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to_exact16() {
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_copy_to_exact16(u64::from(dst), len);
        }
    }
}

fn test_copy_to_exact(dst: u64, len: u32) {
    testcase_init_to_exact(dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(-1), Reg::from(0), Reg::from(-2)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([dst, u64::from(len)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to_exact() {
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
            test_copy_to_exact(u64::from(dst), src);
        }
    }
}

fn testcase_init_from_exact(src: u32, len: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (data $d (i32.const 0))
            (func (param $dst i32)
                (local.get $dst)
                (i32.const {src})
                (i32.const {len})
                (memory.init $d)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_from_exact16(src: u32, len: u32) {
    testcase_init_from_exact(src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init_imm(Reg::from(0), Reg::from(-1), u32imm16(len)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_exact16() {
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_copy_from_exact16(dst, len);
        }
    }
}

fn test_copy_from_exact(src: u32, len: u32) {
    testcase_init_from_exact(src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(0), Reg::from(-1), Reg::from(-2)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_exact() {
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

fn testcase_init_from_to_exact(dst: u64, src: u32, len: u32) -> TranslationTest {
    let wasm = &format!(
        r"
        (module
            (memory 1)
            (data $d (i32.const 0))
            (func
                (i32.const {dst})
                (i32.const {src})
                (i32.const {len})
                (memory.init $d)
            )
        )",
    );
    TranslationTest::new(wasm)
}

fn test_copy_from_to_exact16(dst: u64, src: u32, len: u32) {
    testcase_init_from_to_exact(dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init_imm(Reg::from(-1), Reg::from(-2), u32imm16(len)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([dst, u64::from(src)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to_exact16() {
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for src in values {
            if dst == src {
                continue;
            }
            for len in values {
                test_copy_from_to_exact16(u64::from(dst), src, len);
            }
        }
    }
}

fn test_copy_from_to_exact(dst: u64, src: u32, len: u32) {
    testcase_init_from_to_exact(dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_init(Reg::from(-1), Reg::from(-2), Reg::from(-3)),
                Instruction::memory_index(0),
                Instruction::data_index(0),
                Instruction::Return,
            ])
            .consts([dst, u64::from(src), u64::from(len)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to_exact() {
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
                test_copy_from_to_exact(u64::from(dst), src, len);
            }
        }
    }
}
