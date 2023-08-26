use super::*;
use crate::{
    core::ValueType,
    engine::tests::regmach::{display_wasm::DisplayValueType, wasm_type::WasmType},
    ExternRef,
    FuncRef,
};

fn test_fill(ty: ValueType) {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::table_fill(
                Register::from_i16(0),
                Register::from_i16(2),
                Register::from_i16(1),
            ),
            Instruction::table_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill() {
    test_fill(ValueType::FuncRef);
    test_fill(ValueType::ExternRef);
}

fn testcase_fill_exact(ty: ValueType, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_fill_exact16(ty: ValueType, len: u32) {
    testcase_fill_exact(ty, len)
        .expect_func_instrs([
            Instruction::table_fill_exact(
                Register::from_i16(0),
                u32imm16(len),
                Register::from_i16(1),
            ),
            Instruction::table_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_exact16() {
    fn test_for(len: u32) {
        test_fill_exact16(ValueType::FuncRef, len);
        test_fill_exact16(ValueType::ExternRef, len);
    }
    test_for(0);
    test_for(1);
    test_for(42);
    test_for(u32::from(u16::MAX));
}

fn test_fill_exact(ty: ValueType, len: u32) {
    testcase_fill_exact(ty, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(
                    Register::from_i16(0),
                    Register::from_i16(-1),
                    Register::from_i16(1),
                ),
                Instruction::table_idx(0),
                Instruction::Return,
            ])
            .consts([len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_exact() {
    fn test_for(len: u32) {
        test_fill_exact(ValueType::FuncRef, len);
        test_fill_exact(ValueType::ExternRef, len);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX);
}

fn testcase_fill_at(ty: ValueType, dst: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_fill_at16(ty: ValueType, dst: u32) {
    testcase_fill_at(ty, dst)
        .expect_func_instrs([
            Instruction::table_fill_at(u32imm16(dst), Register::from_i16(1), Register::from_i16(0)),
            Instruction::table_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at16() {
    fn test_for(dst: u32) {
        test_fill_at16(ValueType::FuncRef, dst);
        test_fill_at16(ValueType::ExternRef, dst);
    }
    test_for(0);
    test_for(u32::from(u16::MAX));
}

fn test_fill_at(ty: ValueType, dst: u32) {
    testcase_fill_at(ty, dst)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(
                    Register::from_i16(-1),
                    Register::from_i16(1),
                    Register::from_i16(0),
                ),
                Instruction::table_idx(0),
                Instruction::Return,
            ])
            .consts([dst]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at() {
    fn test_for(dst: u32) {
        test_fill_at(ValueType::FuncRef, dst);
        test_fill_at(ValueType::ExternRef, dst);
    }
    test_for(u32::from(u16::MAX) + 1);
    test_for(u32::MAX);
}

fn testcase_fill_at_exact(ty: ValueType, dst: u32, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_fill_at_exact16(ty: ValueType, dst: u32, len: u32) {
    testcase_fill_at_exact(ty, dst, len)
        .expect_func_instrs([
            Instruction::table_fill_at_exact(u32imm16(dst), u32imm16(len), Register::from_i16(0)),
            Instruction::table_idx(0),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_exact16() {
    fn test_for(dst: u32, len: u32) {
        test_fill_at_exact16(ValueType::FuncRef, dst, len);
        test_fill_at_exact16(ValueType::ExternRef, dst, len);
    }
    let values = [0, 1, u32::from(u16::MAX) - 1, u32::from(u16::MAX)];
    for dst in values {
        for len in values {
            test_for(dst, len);
        }
    }
}

fn test_fill_at_exact(ty: ValueType, dst: u32, len: u32) {
    testcase_fill_at_exact(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(
                    Register::from_i16(-1),
                    Register::from_i16(-2),
                    Register::from_i16(0),
                ),
                Instruction::table_idx(0),
                Instruction::Return,
            ])
            .consts([dst, len]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn fill_at_exact() {
    fn test_for(dst: u32, len: u32) {
        test_fill_at_exact(ValueType::FuncRef, dst, len);
        test_fill_at_exact(ValueType::ExternRef, dst, len);
    }
    let values = [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX];
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

fn testcase_fill_at_exact_imm(ty: ValueType, dst: u32, len: u32) -> TranslationTest {
    let display_ty = DisplayValueType::from(ty);
    let ref_ty = match ty {
        ValueType::FuncRef => "func",
        ValueType::ExternRef => "extern",
        _ => panic!("invalid Wasm reftype"),
    };
    let wasm = wat2wasm(&format!(
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
    ));
    TranslationTest::new(wasm)
}

fn test_fill_at_exact_imm(ty: ValueType, dst: u32, len: u32) {
    testcase_fill_at_exact_imm(ty, dst, len)
        .expect_func(
            ExpectedFunc::new([
                Instruction::table_fill(
                    Register::from_i16(-1),
                    Register::from_i16(-2),
                    Register::from_i16(-3),
                ),
                Instruction::table_idx(0),
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
        test_fill_at_exact_imm(ValueType::FuncRef, dst, len);
        test_fill_at_exact_imm(ValueType::ExternRef, dst, len);
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
