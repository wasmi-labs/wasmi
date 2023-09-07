use super::*;
use crate::{core::ValueType, engine::tests::regmach::display_wasm::DisplayValueType};

fn test_init(ty: ValueType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_init(
                Register::from_i16(0),
                Register::from_i16(1),
                Register::from_i16(2),
            ),
            Instruction::table_idx(0),
            Instruction::elem_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init() {
    test_init(ValueType::FuncRef);
    test_init(ValueType::ExternRef);
}

fn testcase_init_exact(ty: ValueType, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_init_exact16(ty: ValueType, len: u32) {
    testcase_init_exact(ty, len)
        .expect_func_instrs([
            Instruction::table_init_exact(
                Register::from_i16(0),
                Register::from_i16(1),
                u32imm16(len),
            ),
            Instruction::table_idx(0),
            Instruction::elem_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_exact16() {
    fn test_for(len: u32) {
        test_init_exact16(ValueType::FuncRef, len);
        test_init_exact16(ValueType::ExternRef, len);
    }
    test_for(0);
    test_for(1);
    test_for(42);
    test_for(u32::from(u16::MAX));
}

fn test_init_exact(ty: ValueType, len: u32) {
    testcase_init_exact(ty, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    Register::from_i16(-1),
                ),
                Instruction::table_idx(0),
                Instruction::elem_idx(0),
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
        test_init_exact(ValueType::FuncRef, len);
        test_init_exact(ValueType::ExternRef, len);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX);
}

fn testcase_init_from(ty: ValueType, src: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_init_from16(ty: ValueType, src: u32) {
    testcase_init_from(ty, src)
        .expect_func_instrs([
            Instruction::table_init_from(
                Register::from_i16(0),
                u32imm16(src),
                Register::from_i16(1),
            ),
            Instruction::table_idx(0),
            Instruction::elem_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from16() {
    fn test_for(src: u32) {
        test_init_from16(ValueType::FuncRef, src);
        test_init_from16(ValueType::ExternRef, src);
    }
    test_for(0);
    test_for(u32::from(u16::MAX));
}

fn test_init_from(ty: ValueType, src: u32) {
    testcase_init_from(ty, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(
                    Register::from_i16(0),
                    Register::from_i16(-1),
                    Register::from_i16(1),
                ),
                Instruction::table_idx(0),
                Instruction::elem_idx(0),
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
        test_init_from(ValueType::FuncRef, src);
        test_init_from(ValueType::ExternRef, src);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX);
}

fn testcase_init_to(ty: ValueType, dst: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_init_to16(ty: ValueType, dst: u32) {
    testcase_init_to(ty, dst)
        .expect_func_instrs([
            Instruction::table_init_to(u32imm16(dst), Register::from_i16(0), Register::from_i16(1)),
            Instruction::table_idx(0),
            Instruction::elem_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to16() {
    fn test_for(dst: u32) {
        test_init_to16(ValueType::FuncRef, dst);
        test_init_to16(ValueType::ExternRef, dst);
    }
    test_for(0);
    test_for(u32::from(u16::MAX));
}

fn test_init_to(ty: ValueType, dst: u32) {
    testcase_init_to(ty, dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(
                    Register::from_i16(-1),
                    Register::from_i16(0),
                    Register::from_i16(1),
                ),
                Instruction::table_idx(0),
                Instruction::elem_idx(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to() {
    fn test_for(dst: u32) {
        test_init_to(ValueType::FuncRef, dst);
        test_init_to(ValueType::ExternRef, dst);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX);
}

fn testcase_init_from_to(ty: ValueType, dst: u32, src: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_init_from_to16(ty: ValueType, dst: u32, src: u32) {
    testcase_init_from_to(ty, dst, src)
        .expect_func_instrs([
            Instruction::table_init_from_to(u32imm16(dst), u32imm16(src), Register::from_i16(0)),
            Instruction::table_idx(0),
            Instruction::elem_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to16() {
    fn test_for(dst: u32, src: u32) {
        test_init_from_to16(ValueType::FuncRef, dst, src);
        test_init_from_to16(ValueType::ExternRef, dst, src);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for src in values {
            test_for(dst, src);
        }
    }
}

fn test_init_from_to(ty: ValueType, dst: u32, src: u32) {
    testcase_init_from_to(ty, dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(
                    Register::from_i16(-1),
                    Register::from_i16(-2),
                    Register::from_i16(0),
                ),
                Instruction::table_idx(0),
                Instruction::elem_idx(0),
                Instruction::Return,
            ])
            .consts([dst, src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to() {
    fn test_for(dst: u32, src: u32) {
        test_init_from_to(ValueType::FuncRef, dst, src);
        test_init_from_to(ValueType::ExternRef, dst, src);
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

fn testcase_init_to_exact(ty: ValueType, dst: u32, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_init_to_exact16(ty: ValueType, dst: u32, len: u32) {
    testcase_init_to_exact(ty, dst, len)
        .expect_func_instrs([
            Instruction::table_init_to_exact(u32imm16(dst), Register::from_i16(0), u32imm16(len)),
            Instruction::table_idx(0),
            Instruction::elem_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to_exact16() {
    fn test_for(dst: u32, len: u32) {
        test_init_to_exact16(ValueType::FuncRef, dst, len);
        test_init_to_exact16(ValueType::ExternRef, dst, len);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_for(dst, len);
        }
    }
}

fn test_init_to_exact(ty: ValueType, dst: u32, len: u32) {
    testcase_init_to_exact(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(
                    Register::from_i16(-1),
                    Register::from_i16(0),
                    Register::from_i16(-2),
                ),
                Instruction::table_idx(0),
                Instruction::elem_idx(0),
                Instruction::Return,
            ])
            .consts([dst, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_to_exact() {
    fn test_for(dst: u32, len: u32) {
        test_init_to_exact(ValueType::FuncRef, dst, len);
        test_init_to_exact(ValueType::ExternRef, dst, len);
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

fn testcase_init_from_exact(ty: ValueType, src: u32, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_init_from_exact16(ty: ValueType, src: u32, len: u32) {
    testcase_init_from_exact(ty, src, len)
        .expect_func_instrs([
            Instruction::table_init_from_exact(Register::from_i16(0), u32imm16(src), u32imm16(len)),
            Instruction::table_idx(0),
            Instruction::elem_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_exact16() {
    fn test_for(dst: u32, len: u32) {
        test_init_from_exact16(ValueType::FuncRef, dst, len);
        test_init_from_exact16(ValueType::ExternRef, dst, len);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_for(dst, len);
        }
    }
}

fn test_init_from_exact(ty: ValueType, src: u32, len: u32) {
    testcase_init_from_exact(ty, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(
                    Register::from_i16(0),
                    Register::from_i16(-1),
                    Register::from_i16(-2),
                ),
                Instruction::table_idx(0),
                Instruction::elem_idx(0),
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
        test_init_from_exact(ValueType::FuncRef, src, len);
        test_init_from_exact(ValueType::ExternRef, src, len);
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

fn testcase_init_from_to_exact(ty: ValueType, dst: u32, src: u32, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_init_from_to_exact16(ty: ValueType, dst: u32, src: u32, len: u32) {
    testcase_init_from_to_exact(ty, dst, src, len)
        .expect_func_instrs([
            Instruction::table_init_from_to_exact(u32imm16(dst), u32imm16(src), u32imm16(len)),
            Instruction::table_idx(0),
            Instruction::elem_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to_exact16() {
    fn test_for(dst: u32, src: u32, len: u32) {
        test_init_from_to_exact16(ValueType::FuncRef, dst, src, len);
        test_init_from_to_exact16(ValueType::ExternRef, dst, src, len);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for src in values {
            for len in values {
                test_for(dst, src, len);
            }
        }
    }
}

fn test_init_from_to_exact(ty: ValueType, dst: u32, src: u32, len: u32) {
    testcase_init_from_to_exact(ty, dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_init(
                    Register::from_i16(-1),
                    Register::from_i16(-2),
                    Register::from_i16(-3),
                ),
                Instruction::table_idx(0),
                Instruction::elem_idx(0),
                Instruction::Return,
            ])
            .consts([dst, src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn init_from_to_exact() {
    fn test_for(dst: u32, src: u32, len: u32) {
        test_init_from_to_exact(ValueType::FuncRef, dst, src, len);
        test_init_from_to_exact(ValueType::ExternRef, dst, src, len);
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
                test_for(dst, src, len);
            }
        }
    }
}
