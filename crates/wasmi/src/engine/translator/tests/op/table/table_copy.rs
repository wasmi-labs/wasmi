use super::*;
use crate::core::ValType;

fn test_copy(ty: ValType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t1 10 {display_ty})
            (table $t2 10 {display_ty})
            (func (param $dst i32) (param $src i32) (param $len i32)
                (local.get $dst)
                (local.get $src)
                (local.get $len)
                (table.copy $t1 $t2)
            )
        )"
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_copy(Reg::from(0), Reg::from(1), Reg::from(2)),
            Instruction::table_index(0),
            Instruction::table_index(1),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy() {
    test_copy(ValType::FuncRef);
    test_copy(ValType::ExternRef);
}

fn testcase_copy_exact(ty: ValType, len: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t1 10 {display_ty})
            (table $t2 10 {display_ty})
            (func (param $dst i32) (param $src i32)
                (local.get $dst)
                (local.get $src)
                (i32.const {len})
                (table.copy $t1 $t2)
            )
        )"
    );
    TranslationTest::new(&wasm)
}

fn test_copy_exact16(ty: ValType, len: u64) {
    testcase_copy_exact(ty, len)
        .expect_func_instrs([
            Instruction::table_copy_exact(Reg::from(0), Reg::from(1), u64imm16(len)),
            Instruction::table_index(0),
            Instruction::table_index(1),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_exact16() {
    fn test_for(len: u64) {
        test_copy_exact16(ValType::FuncRef, len);
        test_copy_exact16(ValType::ExternRef, len);
    }
    test_for(0);
    test_for(1);
    test_for(42);
    test_for(u64::from(u16::MAX));
}

fn test_copy_exact(ty: ValType, len: u64) {
    testcase_copy_exact(ty, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(0), Reg::from(1), Reg::from(-1)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_exact() {
    fn test_for(len: u64) {
        test_copy_exact(ValType::FuncRef, len);
        test_copy_exact(ValType::ExternRef, len);
    }
    test_for(u64::from(u16::MAX) + 1);
    test_for(u64::from(u32::MAX));
}

fn testcase_copy_from(ty: ValType, src: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t1 10 {display_ty})
            (table $t2 10 {display_ty})
            (func (param $dst i32) (param $len i32)
                (local.get $dst)
                (i32.const {src})
                (local.get $len)
                (table.copy $t1 $t2)
            )
        )"
    );
    TranslationTest::new(&wasm)
}

fn test_copy_from16(ty: ValType, src: u64) {
    testcase_copy_from(ty, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from16() {
    fn test_for(src: u64) {
        test_copy_from16(ValType::FuncRef, src);
        test_copy_from16(ValType::ExternRef, src);
    }
    test_for(0);
    test_for(u64::from(u16::MAX));
}

fn test_copy_from(ty: ValType, src: u64) {
    testcase_copy_from(ty, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from() {
    fn test_for(src: u64) {
        test_copy_from(ValType::FuncRef, src);
        test_copy_from(ValType::ExternRef, src);
    }
    test_for(u64::from(u16::MAX) + 1);
    test_for(u64::from(u32::MAX));
}

fn testcase_copy_to(ty: ValType, dst: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t1 10 {display_ty})
            (table $t2 10 {display_ty})
            (func (param $src i32) (param $len i32)
                (i32.const {dst})
                (local.get $src)
                (local.get $len)
                (table.copy $t1 $t2)
            )
        )"
    );
    TranslationTest::new(&wasm)
}

fn test_copy_to16(ty: ValType, dst: u64) {
    testcase_copy_to(ty, dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to16() {
    fn test_for(dst: u64) {
        test_copy_to16(ValType::FuncRef, dst);
        test_copy_to16(ValType::ExternRef, dst);
    }
    test_for(0);
    test_for(u64::from(u16::MAX));
}

fn test_copy_to(ty: ValType, dst: u64) {
    testcase_copy_to(ty, dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(-1), Reg::from(0), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to() {
    fn test_for(dst: u64) {
        test_copy_to(ValType::FuncRef, dst);
        test_copy_to(ValType::ExternRef, dst);
    }
    test_for(u64::from(u16::MAX) + 1);
    test_for(u64::from(u32::MAX));
}

fn testcase_copy_from_to(ty: ValType, dst: u64, src: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t1 10 {display_ty})
            (table $t2 10 {display_ty})
            (func (param $len i32)
                (i32.const {dst})
                (i32.const {src})
                (local.get $len)
                (table.copy $t1 $t2)
            )
        )"
    );
    TranslationTest::new(&wasm)
}

fn test_copy_from_to16(ty: ValType, dst: u64, src: u64) {
    testcase_copy_from_to(ty, dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([dst, src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to16() {
    fn test_for(dst: u64, src: u64) {
        test_copy_from_to16(ValType::FuncRef, dst, src);
        test_copy_from_to16(ValType::ExternRef, dst, src);
    }
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for src in values {
            if dst == src {
                continue;
            }
            test_for(dst, src);
        }
    }
}

fn test_copy_from_to(ty: ValType, dst: u64, src: u64) {
    testcase_copy_from_to(ty, dst, src)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([dst, src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to() {
    fn test_for(dst: u64, src: u64) {
        test_copy_from_to(ValType::FuncRef, dst, src);
        test_copy_from_to(ValType::ExternRef, dst, src);
    }
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
            test_for(dst, src);
        }
    }
}

fn testcase_copy_to_exact(ty: ValType, dst: u64, len: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t1 10 {display_ty})
            (table $t2 10 {display_ty})
            (func (param $src i32)
                (i32.const {dst})
                (local.get $src)
                (i32.const {len})
                (table.copy $t1 $t2)
            )
        )"
    );
    TranslationTest::new(&wasm)
}

fn test_copy_to_exact16(ty: ValType, dst: u64, len: u64) {
    testcase_copy_to_exact(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy_exact(Reg::from(-1), Reg::from(0), u64imm16(len)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to_exact16() {
    fn test_for(dst: u64, len: u64) {
        test_copy_to_exact16(ValType::FuncRef, dst, len);
        test_copy_to_exact16(ValType::ExternRef, dst, len);
    }
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_for(dst, len);
        }
    }
}

fn test_copy_to_exact(ty: ValType, dst: u64, len: u64) {
    testcase_copy_to_exact(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(-1), Reg::from(0), Reg::from(-2)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([dst, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_to_exact() {
    fn test_for(dst: u64, len: u64) {
        test_copy_to_exact(ValType::FuncRef, dst, len);
        test_copy_to_exact(ValType::ExternRef, dst, len);
    }
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
            test_for(dst, src);
        }
    }
}

fn testcase_copy_from_exact(ty: ValType, src: u64, len: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t1 10 {display_ty})
            (table $t2 10 {display_ty})
            (func (param $dst i32)
                (local.get $dst)
                (i32.const {src})
                (i32.const {len})
                (table.copy $t1 $t2)
            )
        )"
    );
    TranslationTest::new(&wasm)
}

fn test_copy_from_exact16(ty: ValType, src: u64, len: u64) {
    testcase_copy_from_exact(ty, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy_exact(Reg::from(0), Reg::from(-1), u64imm16(len)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_exact16() {
    fn test_for(dst: u64, len: u64) {
        test_copy_from_exact16(ValType::FuncRef, dst, len);
        test_copy_from_exact16(ValType::ExternRef, dst, len);
    }
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_for(dst, len);
        }
    }
}

fn test_copy_from_exact(ty: ValType, src: u64, len: u64) {
    testcase_copy_from_exact(ty, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(0), Reg::from(-1), Reg::from(-2)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_exact() {
    fn test_for(src: u64, len: u64) {
        test_copy_from_exact(ValType::FuncRef, src, len);
        test_copy_from_exact(ValType::ExternRef, src, len);
    }
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
            test_for(dst, src);
        }
    }
}

fn testcase_copy_from_to_exact(ty: ValType, dst: u64, src: u64, len: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t1 10 {display_ty})
            (table $t2 10 {display_ty})
            (func
                (i32.const {dst})
                (i32.const {src})
                (i32.const {len})
                (table.copy $t1 $t2)
            )
        )"
    );
    TranslationTest::new(&wasm)
}

fn test_copy_from_to_exact16(ty: ValType, dst: u64, src: u64, len: u64) {
    testcase_copy_from_to_exact(ty, dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy_exact(Reg::from(-1), Reg::from(-2), u64imm16(len)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([dst, src]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to_exact16() {
    fn test_for(dst: u64, src: u64, len: u64) {
        test_copy_from_to_exact16(ValType::FuncRef, dst, src, len);
        test_copy_from_to_exact16(ValType::ExternRef, dst, src, len);
    }
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for src in values {
            if dst == src {
                continue;
            }
            for len in values {
                test_for(dst, src, len);
            }
        }
    }
}

fn test_copy_from_to_exact(ty: ValType, dst: u64, src: u64, len: u64) {
    testcase_copy_from_to_exact(ty, dst, src, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_copy(Reg::from(-1), Reg::from(-2), Reg::from(-3)),
                Instruction::table_index(0),
                Instruction::table_index(1),
                Instruction::Return,
            ])
            .consts([dst, src, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn copy_from_to_exact() {
    fn test_for(dst: u64, src: u64, len: u64) {
        test_copy_from_to_exact(ValType::FuncRef, dst, src, len);
        test_copy_from_to_exact(ValType::ExternRef, dst, src, len);
    }
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
                test_for(dst, src, len);
            }
        }
    }
}
