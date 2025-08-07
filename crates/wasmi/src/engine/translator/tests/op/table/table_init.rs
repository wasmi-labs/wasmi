use super::*;
use crate::{core::UntypedVal, ValType};

fn test_init(ty: ValType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (elem $e {display_ty})
            (func (param $dst i32) (param $src i32) (param $len i32)
                (local.get $dst)
                (local.get $src)
                (local.get $len)
                (table.init $t $e)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_init(Reg::from(0), Reg::from(1), Reg::from(2)),
            Instruction::table_index(0),
            Instruction::elem_index(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init() {
    test_init(ValType::FuncRef);
    test_init(ValType::ExternRef);
}

fn testcase_init_exact(ty: ValType, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (elem $e {display_ty})
            (func (param $dst i32) (param $src i32)
                (local.get $dst)
                (local.get $src)
                (i32.const {len})
                (table.init $t $e)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_init_exact16(ty: ValType, len: u32) {
    testcase_init_exact(ty, len)
        .expect_func_instrs([
            Instruction::table_init_imm(Reg::from(0), Reg::from(1), u32imm16(len)),
            Instruction::table_index(0),
            Instruction::elem_index(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_exact16() {
    fn test_for(len: u32) {
        test_init_exact16(ValType::FuncRef, len);
        test_init_exact16(ValType::ExternRef, len);
    }
    test_for(0);
    test_for(1);
    test_for(42);
    test_for(u32::from(u16::MAX));
}

fn test_init_exact(ty: ValType, len: u32) {
    testcase_init_exact(ty, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(0), Reg::from(1), Reg::from(-1)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_exact() {
    fn test_for(len: u32) {
        test_init_exact(ValType::FuncRef, len);
        test_init_exact(ValType::ExternRef, len);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX);
}

fn testcase_init_from(ty: ValType, src: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (elem $e {display_ty})
            (func (param $dst i32) (param $len i32)
                (local.get $dst)
                (i32.const {src})
                (local.get $len)
                (table.init $t $e)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_init_from16(ty: ValType, src: u32) {
    testcase_init_from(ty, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from16() {
    fn test_for(src: u32) {
        test_init_from16(ValType::FuncRef, src);
        test_init_from16(ValType::ExternRef, src);
    }
    test_for(0);
    test_for(u32::from(u16::MAX));
}

fn test_init_from(ty: ValType, src: u32) {
    testcase_init_from(ty, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from() {
    fn test_for(src: u32) {
        test_init_from(ValType::FuncRef, src);
        test_init_from(ValType::ExternRef, src);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX);
}

fn testcase_init_to(ty: ValType, dst: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (elem $e {display_ty})
            (func (param $src i32) (param $len i32)
                (i32.const {dst})
                (local.get $src)
                (local.get $len)
                (table.init $t $e)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_init_to16(ty: ValType, dst: u64) {
    testcase_init_to(ty, dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to16() {
    fn test_for(dst: u64) {
        test_init_to16(ValType::FuncRef, dst);
        test_init_to16(ValType::ExternRef, dst);
    }
    test_for(0);
    test_for(u64::from(u16::MAX));
}

fn test_init_to(ty: ValType, dst: u64) {
    testcase_init_to(ty, dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to() {
    fn test_for(dst: u64) {
        test_init_to(ValType::FuncRef, dst);
        test_init_to(ValType::ExternRef, dst);
    }
    test_for(u64::from(u16::MAX) + 1);
    test_for(u64::from(u32::MAX));
}

fn testcase_init_from_to(ty: ValType, dst: u64, src: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (elem $e {display_ty})
            (func (param $len i32)
                (i32.const {dst})
                (i32.const {src})
                (local.get $len)
                (table.init $t $e)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_init_from_to16(ty: ValType, dst: u64, src: u32) {
    testcase_init_from_to(ty, dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([dst, u64::from(src)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to16() {
    fn test_for(dst: u64, src: u32) {
        test_init_from_to16(ValType::FuncRef, dst, src);
        test_init_from_to16(ValType::ExternRef, dst, src);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for src in values {
            if dst == src {
                continue;
            }
            test_for(u64::from(dst), src);
        }
    }
}

fn test_init_from_to(ty: ValType, dst: u64, src: u32) {
    testcase_init_from_to(ty, dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([UntypedVal::from(dst), UntypedVal::from(src)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to() {
    fn test_for(dst: u64, src: u32) {
        test_init_from_to(ValType::FuncRef, dst, src);
        test_init_from_to(ValType::ExternRef, dst, src);
    }
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
            test_for(u64::from(dst), src);
        }
    }
}

fn testcase_init_to_exact(ty: ValType, dst: u64, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (elem $e {display_ty})
            (func (param $src i32)
                (i32.const {dst})
                (local.get $src)
                (i32.const {len})
                (table.init $t $e)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_init_to_exact16(ty: ValType, dst: u64, len: u32) {
    testcase_init_to_exact(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init_imm(Reg::from(-1), Reg::from(0), u32imm16(len)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to_exact16() {
    fn test_for(dst: u64, len: u32) {
        test_init_to_exact16(ValType::FuncRef, dst, len);
        test_init_to_exact16(ValType::ExternRef, dst, len);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_for(u64::from(dst), len);
        }
    }
}

fn test_init_to_exact(ty: ValType, dst: u64, len: u32) {
    testcase_init_to_exact(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(-1), Reg::from(0), Reg::from(-2)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([UntypedVal::from(dst), UntypedVal::from(len)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to_exact() {
    fn test_for(dst: u64, len: u32) {
        test_init_to_exact(ValType::FuncRef, dst, len);
        test_init_to_exact(ValType::ExternRef, dst, len);
    }
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
            test_for(u64::from(dst), src);
        }
    }
}

fn testcase_init_from_exact(ty: ValType, src: u32, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (elem $e {display_ty})
            (func (param $dst i32)
                (local.get $dst)
                (i32.const {src})
                (i32.const {len})
                (table.init $t $e)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_init_from_exact16(ty: ValType, src: u32, len: u32) {
    testcase_init_from_exact(ty, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init_imm(Reg::from(0), Reg::from(-1), u32imm16(len)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_exact16() {
    fn test_for(dst: u32, len: u32) {
        test_init_from_exact16(ValType::FuncRef, dst, len);
        test_init_from_exact16(ValType::ExternRef, dst, len);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_for(dst, len);
        }
    }
}

fn test_init_from_exact(ty: ValType, src: u32, len: u32) {
    testcase_init_from_exact(ty, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(0), Reg::from(-1), Reg::from(-2)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_exact() {
    fn test_for(src: u32, len: u32) {
        test_init_from_exact(ValType::FuncRef, src, len);
        test_init_from_exact(ValType::ExternRef, src, len);
    }
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
            test_for(dst, src);
        }
    }
}

fn testcase_init_from_to_exact(ty: ValType, dst: u64, src: u32, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (elem $e {display_ty})
            (func
                (i32.const {dst})
                (i32.const {src})
                (i32.const {len})
                (table.init $t $e)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_init_from_to_exact16(ty: ValType, dst: u64, src: u32, len: u32) {
    testcase_init_from_to_exact(ty, dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init_imm(Reg::from(-1), Reg::from(-2), u32imm16(len)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([dst, u64::from(src)]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to_exact16() {
    fn test_for(dst: u64, src: u32, len: u32) {
        test_init_from_to_exact16(ValType::FuncRef, dst, src, len);
        test_init_from_to_exact16(ValType::ExternRef, dst, src, len);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for src in values {
            if dst == src {
                continue;
            }
            for len in values {
                test_for(u64::from(dst), src, len);
            }
        }
    }
}

fn test_init_from_to_exact(ty: ValType, dst: u64, src: u32, len: u32) {
    testcase_init_from_to_exact(ty, dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(Reg::from(-1), Reg::from(-2), Reg::from(-3)),
                Instruction::table_index(0),
                Instruction::elem_index(0),
                Instruction::Return,
            ])
            .consts([
                UntypedVal::from(dst),
                UntypedVal::from(src),
                UntypedVal::from(len),
            ]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to_exact() {
    fn test_for(dst: u64, src: u32, len: u32) {
        test_init_from_to_exact(ValType::FuncRef, dst, src, len);
        test_init_from_to_exact(ValType::ExternRef, dst, src, len);
    }
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
                test_for(u64::from(dst), src, len);
            }
        }
    }
}
