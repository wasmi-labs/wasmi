//! Translation tests for all Wasm `store` instructions.

use super::*;

mod f32_store;
mod f64_store;
mod i32_store;
mod i32_store16;
mod i32_store8;
mod i64_store;
mod i64_store16;
mod i64_store32;
mod i64_store8;

use core::fmt::Display;
use wasmi_core::TrapCode;

/// Creates an [`Instruction::ConstRef`] with 0 index.
pub fn const_ref<T>(_value: T) -> Instruction {
    Instruction::ConstRef(ConstRef::from_u32(0))
}

fn test_store(
    wasm_op: WasmOp,
    offset: u32,
    make_instr: fn(ptr: Register, offset: Const32) -> Instruction,
) {
    let param_ty = wasm_op.param_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32) (param $value {param_ty})
                local.get $ptr
                local.get $value
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            make_instr(Register::from_u16(0), Const32::from(offset)),
            Instruction::Register(Register::from_u16(1)),
            Instruction::Return,
        ])
        .run();
}

fn test_store_imm<T>(
    wasm_op: WasmOp,
    offset: u32,
    value: T,
    make_instr: fn(ptr: Register, offset: Const32) -> Instruction,
    make_instr_param: fn(value: T) -> Instruction,
) where
    T: Copy + Display,
{
    let param_ty = wasm_op.param_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32)
                local.get $ptr
                {param_ty}.const {value}
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            make_instr(Register::from_u16(0), Const32::from(offset)),
            make_instr_param(value),
            Instruction::Return,
        ])
        .run();
}

fn test_store_at(
    wasm_op: WasmOp,
    ptr: u32,
    offset: u32,
    make_instr: fn(address: Const32, value: Register) -> Instruction,
) {
    let address = ptr
        .checked_add(offset)
        .expect("testcase requires valid ptr+offset address");
    let param_ty = wasm_op.param_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $value {param_ty})
                i32.const {ptr}
                local.get $value
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            make_instr(Const32::from(address), Register::from_u16(0)),
            Instruction::Return,
        ])
        .run();
}

fn test_store_at_overflow(wasm_op: WasmOp, ptr: u32, offset: u32) {
    assert!(
        ptr.checked_add(offset).is_none(),
        "testcase expects overflowing ptr+offset address"
    );
    let param_ty = wasm_op.param_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $value {param_ty})
                i32.const {ptr}
                local.get $value
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([Instruction::Trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

fn test_store_imm_at<T>(
    wasm_op: WasmOp,
    ptr: u32,
    offset: u32,
    value: T,
    make_instr: fn(address: Const32) -> Instruction,
    make_instr_param: fn(value: T) -> Instruction,
) where
    T: Copy + Display,
{
    let address = ptr
        .checked_add(offset)
        .expect("testcase requires valid ptr+offset address");
    let param_ty = wasm_op.param_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func
                i32.const {ptr}
                {param_ty}.const {value}
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            make_instr(Const32::from(address)),
            make_instr_param(value),
            Instruction::Return,
        ])
        .run();
}

fn test_store_imm_n_at<T>(
    wasm_op: WasmOp,
    ptr: u32,
    offset: u32,
    value: T,
    make_instr: fn(address: Const32, value: T) -> Instruction,
) where
    T: Copy + Display,
{
    let address = ptr
        .checked_add(offset)
        .expect("testcase requires valid ptr+offset address");
    let param_ty = wasm_op.param_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func
                i32.const {ptr}
                {param_ty}.const {value}
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            make_instr(Const32::from(address), value),
            Instruction::Return,
        ])
        .run();
}

fn test_store_imm_at_overflow<T>(wasm_op: WasmOp, ptr: u32, offset: u32, value: T)
where
    T: Copy + Display,
{
    assert!(
        ptr.checked_add(offset).is_none(),
        "testcase expects overflowing ptr+offset address"
    );
    let param_ty = wasm_op.param_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func
                i32.const {ptr}
                {param_ty}.const {value}
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([Instruction::Trap(TrapCode::MemoryOutOfBounds)])
        .run();
}
