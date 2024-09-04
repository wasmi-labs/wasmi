//! Translation tests for all Wasm `store` instructions.

use super::*;
use crate::core::UntypedVal;

mod f32_store;
mod f64_store;
mod i32_store;
mod i32_store16;
mod i32_store8;
mod i64_store;
mod i64_store16;
mod i64_store32;
mod i64_store8;

use crate::core::TrapCode;
use core::fmt::Display;

fn test_store_for(
    wasm_op: WasmOp,
    offset: u32,
    make_instr: fn(ptr: Reg, offset: Const32<u32>) -> Instruction,
) {
    assert!(
        u16::try_from(offset).is_err(),
        "this test requires non-16 bit offsets but found {offset}"
    );
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32) (param $value {param_ty})
                local.get $ptr
                local.get $value
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(0), Const32::from(offset)),
            Instruction::Register(Reg::from(1)),
            Instruction::Return,
        ])
        .run();
}

fn test_store(wasm_op: WasmOp, make_instr: fn(ptr: Reg, offset: Const32<u32>) -> Instruction) {
    test_store_for(wasm_op, u32::from(u16::MAX) + 1, make_instr);
    test_store_for(wasm_op, u32::MAX - 1, make_instr);
    test_store_for(wasm_op, u32::MAX, make_instr);
}

fn test_store_offset16_for(
    wasm_op: WasmOp,
    offset: u16,
    make_instr: fn(ptr: Reg, offset: u16, value: Reg) -> Instruction,
) {
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32) (param $value {param_ty})
                local.get $ptr
                local.get $value
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(0), offset, Reg::from(1)),
            Instruction::Return,
        ])
        .run();
}

fn test_store_offset16(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset: u16, value: Reg) -> Instruction,
) {
    test_store_offset16_for(wasm_op, 0, make_instr);
    test_store_offset16_for(wasm_op, u16::MAX - 1, make_instr);
    test_store_offset16_for(wasm_op, u16::MAX, make_instr);
}

fn test_store_offset16_imm_for<T>(
    wasm_op: WasmOp,
    offset: u16,
    value: T,
    make_instr: fn(ptr: Reg, offset: u16, value: Reg) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32)
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func(
            ExpectedFunc::new([
                make_instr(Reg::from(0), offset, Reg::from(-1)),
                Instruction::Return,
            ])
            .consts([value]),
        )
        .run();
}

fn test_store_offset16_imm<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(ptr: Reg, offset: u16, value: Reg) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    test_store_offset16_imm_for(wasm_op, 0, value, make_instr);
    test_store_offset16_imm_for(wasm_op, u16::MAX - 1, value, make_instr);
    test_store_offset16_imm_for(wasm_op, u16::MAX, value, make_instr);
}

fn test_store_offset16_imm16_for<T>(
    wasm_op: WasmOp,
    offset: u16,
    value: T,
    make_instr: fn(ptr: Reg, offset: u16, value: T) -> Instruction,
) where
    T: Copy,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32)
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([make_instr(Reg::from(0), offset, value), Instruction::Return])
        .run();
}

fn test_store_offset16_imm16<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(ptr: Reg, offset: u16, value: T) -> Instruction,
) where
    T: Copy,
    DisplayWasm<T>: Display,
{
    test_store_offset16_imm16_for(wasm_op, 0, value, make_instr);
    test_store_offset16_imm16_for(wasm_op, u16::MAX - 1, value, make_instr);
    test_store_offset16_imm16_for(wasm_op, u16::MAX, value, make_instr);
}

fn test_store_imm_for<T>(
    wasm_op: WasmOp,
    offset: u32,
    value: T,
    make_instr: fn(ptr: Reg, offset: Const32<u32>) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    assert!(
        u16::try_from(offset).is_err(),
        "this test requires non-16 bit offsets but found {offset}"
    );
    let param_ty = wasm_op.param_ty();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32)
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func(
            ExpectedFunc::new([
                make_instr(Reg::from(0), Const32::from(offset)),
                Instruction::Register(Reg::from(-1)),
                Instruction::Return,
            ])
            .consts([value]),
        )
        .run();
}

fn test_store_imm<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(ptr: Reg, offset: Const32<u32>) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    test_store_imm_for(wasm_op, u32::from(u16::MAX) + 1, value, make_instr);
    test_store_imm_for(wasm_op, u32::MAX - 1, value, make_instr);
    test_store_imm_for(wasm_op, u32::MAX, value, make_instr);
}

fn test_store_at_for(
    wasm_op: WasmOp,
    ptr: u32,
    offset: u32,
    make_instr: fn(address: Const32<u32>, value: Reg) -> Instruction,
) {
    let address = ptr
        .checked_add(offset)
        .expect("testcase requires valid ptr+offset address");
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $value {param_ty})
                i32.const {ptr}
                local.get $value
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            make_instr(Const32::from(address), Reg::from(0)),
            Instruction::Return,
        ])
        .run();
}

fn test_store_at(
    wasm_op: WasmOp,
    make_instr: fn(address: Const32<u32>, value: Reg) -> Instruction,
) {
    test_store_at_for(wasm_op, 0, 0, make_instr);
    test_store_at_for(wasm_op, 0, 1, make_instr);
    test_store_at_for(wasm_op, 1, 0, make_instr);
    test_store_at_for(wasm_op, 1, 1, make_instr);
    test_store_at_for(wasm_op, 1000, 1000, make_instr);
    test_store_at_for(wasm_op, 1, u32::MAX - 1, make_instr);
    test_store_at_for(wasm_op, u32::MAX - 1, 1, make_instr);
    test_store_at_for(wasm_op, 0, u32::MAX, make_instr);
    test_store_at_for(wasm_op, u32::MAX, 0, make_instr);
}

fn test_store_at_overflow_for(wasm_op: WasmOp, ptr: u32, offset: u32) {
    assert!(
        ptr.checked_add(offset).is_none(),
        "testcase expects overflowing ptr+offset address"
    );
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $value {param_ty})
                i32.const {ptr}
                local.get $value
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

fn test_store_at_overflow(wasm_op: WasmOp) {
    test_store_at_overflow_for(wasm_op, 1, u32::MAX);
    test_store_at_overflow_for(wasm_op, u32::MAX, 1);
    test_store_at_overflow_for(wasm_op, u32::MAX, u32::MAX);
}

fn test_store_at_imm_for<T>(
    wasm_op: WasmOp,
    ptr: u32,
    offset: u32,
    value: T,
    make_instr: fn(address: Const32<u32>, value: Reg) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let address = ptr
        .checked_add(offset)
        .expect("testcase requires valid ptr+offset address");
    let display_value = DisplayWasm::from(value);
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func
                i32.const {ptr}
                {param_ty}.const {display_value}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func(
            ExpectedFunc::new([
                make_instr(Const32::from(address), Reg::from(-1)),
                Instruction::Return,
            ])
            .consts([value]),
        )
        .run();
}

fn test_store_at_imm<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(address: Const32<u32>, value: Reg) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    test_store_at_imm_for(wasm_op, 0, 0, value, make_instr);
    test_store_at_imm_for(wasm_op, 0, 1, value, make_instr);
    test_store_at_imm_for(wasm_op, 1, 0, value, make_instr);
    test_store_at_imm_for(wasm_op, 1, 1, value, make_instr);
    test_store_at_imm_for(wasm_op, 1000, 1000, value, make_instr);
    test_store_at_imm_for(wasm_op, 1, u32::MAX - 1, value, make_instr);
    test_store_at_imm_for(wasm_op, u32::MAX - 1, 1, value, make_instr);
    test_store_at_imm_for(wasm_op, 0, u32::MAX, value, make_instr);
    test_store_at_imm_for(wasm_op, u32::MAX, 0, value, make_instr);
}

fn test_store_at_imm_overflow_for<T>(wasm_op: WasmOp, ptr: u32, offset: u32, value: T)
where
    T: Copy,
    DisplayWasm<T>: Display,
{
    assert!(
        ptr.checked_add(offset).is_none(),
        "testcase expects overflowing ptr+offset address"
    );
    let display_value = DisplayWasm::from(value);
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func
                i32.const {ptr}
                {param_ty}.const {display_value}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

fn test_store_at_imm_overflow<T>(wasm_op: WasmOp, value: T)
where
    T: Copy,
    DisplayWasm<T>: Display,
{
    test_store_at_imm_overflow_for(wasm_op, 1, u32::MAX, value);
    test_store_at_imm_overflow_for(wasm_op, u32::MAX, 1, value);
    test_store_at_imm_overflow_for(wasm_op, u32::MAX, u32::MAX, value);
}
