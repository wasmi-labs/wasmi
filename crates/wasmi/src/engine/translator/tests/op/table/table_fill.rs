use super::*;
use crate::ValType;

fn test_fill(ty: ValType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $dst i32) (param $value {display_ty}) (param $len i32)
                (local.get $dst)
                (local.get $value)
                (local.get $len)
                (table.fill $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            Instruction::table_fill(Reg::from(0), Reg::from(2), Reg::from(1)),
            Instruction::table_index(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill() {
    test_fill(ValType::FuncRef);
    test_fill(ValType::ExternRef);
}

fn testcase_fill_exact(ty: ValType, len: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $dst i32) (param $value {display_ty})
                (local.get $dst)
                (local.get $value)
                (i32.const {len})
                (table.fill $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_fill_exact16(ty: ValType, len: u64) {
    testcase_fill_exact(ty, len)
        .expect_func_instrs([
            Instruction::table_fill_imm(Reg::from(0), u64imm16(len), Reg::from(1)),
            Instruction::table_index(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_exact16() {
    fn test_for(len: u64) {
        test_fill_exact16(ValType::FuncRef, len);
        test_fill_exact16(ValType::ExternRef, len);
    }
    test_for(0);
    test_for(1);
    test_for(42);
    test_for(u64::from(u16::MAX));
}

fn test_fill_exact(ty: ValType, len: u64) {
    testcase_fill_exact(ty, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(Reg::from(0), Reg::from(-1), Reg::from(1)),
                Instruction::table_index(0),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_exact() {
    fn test_for(len: u64) {
        test_fill_exact(ValType::FuncRef, len);
        test_fill_exact(ValType::ExternRef, len);
    }
    test_for(u64::from(u16::MAX) + 1);
    test_for(u64::from(u32::MAX));
}

fn testcase_fill_at(ty: ValType, dst: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty}) (param $len i32)
                (i32.const {dst})
                (local.get $value)
                (local.get $len)
                (table.fill $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_fill_at16(ty: ValType, dst: u64) {
    testcase_fill_at(ty, dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(Reg::from(-1), Reg::from(1), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at16() {
    fn test_for(dst: u64) {
        test_fill_at16(ValType::FuncRef, dst);
        test_fill_at16(ValType::ExternRef, dst);
    }
    test_for(0);
    test_for(u64::from(u16::MAX));
}

fn test_fill_at(ty: ValType, dst: u64) {
    testcase_fill_at(ty, dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(Reg::from(-1), Reg::from(1), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at() {
    fn test_for(dst: u64) {
        test_fill_at(ValType::FuncRef, dst);
        test_fill_at(ValType::ExternRef, dst);
    }
    test_for(u64::from(u16::MAX) + 1);
    test_for(u64::from(u32::MAX));
}

fn testcase_fill_at_exact(ty: ValType, dst: u64, len: u64) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func (param $value {display_ty})
                (i32.const {dst})
                (local.get $value)
                (i32.const {len})
                (table.fill $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_fill_at_exact16(ty: ValType, dst: u64, len: u64) {
    testcase_fill_at_exact(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill_imm(Reg::from(-1), u64imm16(len), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_exact16() {
    fn test_for(dst: u64, len: u64) {
        test_fill_at_exact16(ValType::FuncRef, dst, len);
        test_fill_at_exact16(ValType::ExternRef, dst, len);
    }
    let values = [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_for(dst, len);
        }
    }
}

fn test_fill_at_exact(ty: ValType, dst: u64, len: u64) {
    testcase_fill_at_exact(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(Reg::from(-1), Reg::from(-2), Reg::from(0)),
                Instruction::table_index(0),
                Instruction::Return,
            ])
            .consts([dst, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_exact() {
    fn test_for(dst: u64, len: u64) {
        test_fill_at_exact(ValType::FuncRef, dst, len);
        test_fill_at_exact(ValType::ExternRef, dst, len);
    }
    let values = [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ];
    for dst in values {
        for len in values {
            if dst == len {
                // We skip here because equal `dst` and `len` would
                // allocate just a single function local constant value
                // which our testcase is not prepared for.
                // Ideally we'd have yet another test for that case.
                continue;
            }
            test_for(dst, len);
        }
    }
}

fn testcase_fill_at_exact_imm(ty: ValType, dst: u32, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let ref_ty = match ty {
        ValType::FuncRef => "func",
        ValType::ExternRef => "extern",
        _ => panic!("invalid Wasm reftype"),
    };
    let wasm = format!(
        r"
        (module
            (table $t 10 {display_ty})
            (func
                (i32.const {dst})
                (ref.null {ref_ty})
                (i32.const {len})
                (table.fill $t)
            )
        )",
    );
    TranslationTest::new(&wasm)
}

fn test_fill_at_exact_imm(ty: ValType, dst: u32, len: u32) {
    testcase_fill_at_exact_imm(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(Reg::from(-1), Reg::from(-2), Reg::from(-3)),
                Instruction::table_index(0),
                Instruction::Return,
            ])
            .consts([dst, len, 0]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_exact_exact() {
    fn test_for(dst: u32, len: u32) {
        test_fill_at_exact_imm(ValType::FuncRef, dst, len);
        test_fill_at_exact_imm(ValType::ExternRef, dst, len);
    }
    let values = [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX];
    for dst in values {
        for src in values {
            if dst == src {
                // We skip here because equal `dst` and `len` would
                // allocate just a single function local constant value
                // which our testcase is not prepared for.
                // Ideally we'd have yet another test for that case.
                continue;
            }
            test_for(dst, src);
        }
    }
}
