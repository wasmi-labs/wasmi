//! Translation tests for all Wasm `store` instructions.

use super::*;
use crate::{
    core::UntypedVal,
    engine::translator::utils::Wrap,
    ir::{AnyConst16, Offset16, Offset64, Offset64Lo},
};
use std::vec;

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
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    memory_index: MemIdx,
    index_ty: IndexType,
    offset: u64,
) {
    let offset = offset.into();
    assert!(
        u16::try_from(offset).is_err() || !memory_index.is_default(),
        "this test requires non-16 bit offsets or non-default memory \
        but found: offset={offset}, memory={memory_index}"
    );
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (param $ptr {index_ty}) (param $value {param_ty})
                local.get $ptr
                local.get $value
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let (offset_hi, offset_lo) = Offset64::split(offset);
    TranslationTest::new(&wasm)
        .expect_func_instrs(iter_filter_opts![
            make_instr(Reg::from(0), offset_lo),
            Instruction::register_and_offset_hi(Reg::from(1), offset_hi),
            memory_index.instr(),
            Instruction::Return,
        ])
        .run();
}

fn test_store(wasm_op: WasmOp, make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction) {
    // Case: offsets that cannot be 16-bit encoded:
    [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ]
    .into_iter()
    .for_each(|offset| {
        test_store_for(wasm_op, make_instr, MemIdx(0), IndexType::Memory32, offset);
        test_store_for(wasm_op, make_instr, MemIdx(1), IndexType::Memory32, offset);
        test_store_for(wasm_op, make_instr, MemIdx(0), IndexType::Memory64, offset);
        test_store_for(wasm_op, make_instr, MemIdx(1), IndexType::Memory64, offset);
    });
    // Case: 64-bit offsets and `memory64`:
    [u64::from(u32::MAX) + 1, u64::MAX - 1, u64::MAX]
        .into_iter()
        .for_each(|offset| {
            test_store_for(wasm_op, make_instr, MemIdx(0), IndexType::Memory64, offset);
            test_store_for(wasm_op, make_instr, MemIdx(1), IndexType::Memory64, offset);
        });
    // Case: 16-bit offsets but non-default memory index:
    [0, 1, u64::from(u16::MAX) - 1, u64::from(u16::MAX)]
        .into_iter()
        .for_each(|offset| {
            test_store_for(wasm_op, make_instr, MemIdx(1), IndexType::Memory32, offset);
            test_store_for(wasm_op, make_instr, MemIdx(1), IndexType::Memory64, offset);
        })
}

fn test_store_offset16_for(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
    index_ty: IndexType,
    offset: u16,
) {
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory {index_ty} 1)
            (func (param $ptr {index_ty}) (param $value {param_ty})
                local.get $ptr
                local.get $value
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(0), offset16(offset), Reg::from(1)),
            Instruction::Return,
        ])
        .run();
}

fn test_store_offset16(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
) {
    [0, 1, u16::MAX - 1, u16::MAX]
        .into_iter()
        .for_each(|offset| {
            test_store_offset16_for(wasm_op, make_instr, IndexType::Memory32, offset);
            test_store_offset16_for(wasm_op, make_instr, IndexType::Memory64, offset);
        })
}

fn test_store_offset16_imm_for<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
    index_ty: IndexType,
    offset: u16,
    value: T,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory {index_ty} 1)
            (func (param $ptr {index_ty})
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new([
                make_instr(Reg::from(0), offset16(offset), Reg::from(-1)),
                Instruction::Return,
            ])
            .consts([value]),
        )
        .run();
}

fn test_store_offset16_imm<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(ptr: Reg, offset: Offset16, value: Reg) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    [0, 1, u16::MAX - 1, u16::MAX]
        .into_iter()
        .for_each(|offset| {
            test_store_offset16_imm_for(wasm_op, make_instr, IndexType::Memory32, offset, value);
            test_store_offset16_imm_for(wasm_op, make_instr, IndexType::Memory64, offset, value);
        })
}

fn test_store_offset16_imm16_for<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset: Offset16, value: T) -> Instruction,
    index_ty: IndexType,
    offset: u16,
    value: T,
) where
    T: Copy,
    DisplayWasm<T>: Display,
{
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory {index_ty} 1)
            (func (param $ptr {index_ty})
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(0), offset16(offset), value),
            Instruction::Return,
        ])
        .run();
}

fn test_store_offset16_imm16<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset: Offset16, value: T) -> Instruction,
    value: T,
) where
    T: Copy,
    DisplayWasm<T>: Display,
{
    [0, 1, u16::MAX - 1, u16::MAX]
        .into_iter()
        .for_each(|offset| {
            test_store_offset16_imm16_for(wasm_op, make_instr, IndexType::Memory32, offset, value);
            test_store_offset16_imm16_for(wasm_op, make_instr, IndexType::Memory64, offset, value);
        })
}

fn test_store_wrap_offset16_imm_for<Src, Wrapped, Field>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset: Offset16, value: Field) -> Instruction,
    index_ty: IndexType,
    offset: u16,
    value: Src,
) where
    Src: Copy + Wrap<Wrapped>,
    Field: TryFrom<Wrapped>,
    DisplayWasm<Src>: Display,
{
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory {index_ty} 1)
            (func (param $ptr {index_ty})
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    let value = Field::try_from(value.wrap()).ok().unwrap();
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(0), offset16(offset), value),
            Instruction::Return,
        ])
        .run();
}

fn test_store_wrap_offset16_imm<Src, Wrapped, Field>(
    wasm_op: WasmOp,
    value: Src,
    make_instr: fn(ptr: Reg, offset: Offset16, value: Field) -> Instruction,
) where
    Src: Copy + Wrap<Wrapped>,
    Field: TryFrom<Wrapped>,
    DisplayWasm<Src>: Display,
{
    [0, 1, u16::MAX - 1, u16::MAX]
        .into_iter()
        .for_each(|offset| {
            test_store_wrap_offset16_imm_for(
                wasm_op,
                make_instr,
                IndexType::Memory32,
                offset,
                value,
            );
            test_store_wrap_offset16_imm_for(
                wasm_op,
                make_instr,
                IndexType::Memory64,
                offset,
                value,
            );
        })
}

fn test_store_wrap_imm_for<Src, Wrapped, Field>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    index_ty: IndexType,
    memory_index: MemIdx,
    offset: u64,
    value: Src,
) where
    Src: Copy + Into<UntypedVal> + Wrap<Wrapped>,
    Field: TryFrom<Wrapped> + Into<AnyConst16>,
    DisplayWasm<Src>: Display,
{
    assert!(
        u16::try_from(offset).is_err() || !memory_index.is_default(),
        "this test requires non-16 bit offsets or non-default memory \
        but found: offset={offset}, memory={memory_index}"
    );
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (param $ptr {index_ty})
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let (offset_hi, offset_lo) = Offset64::split(offset);
    let value = Field::try_from(value.wrap()).ok().unwrap();
    TranslationTest::new(&wasm)
        .expect_func_instrs(iter_filter_opts![
            make_instr(Reg::from(0), offset_lo),
            Instruction::imm16_and_offset_hi(value, offset_hi),
            memory_index.instr(),
            Instruction::Return,
        ])
        .run();
}

fn test_store_wrap_imm<Src, Wrapped, Field>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    value: Src,
) where
    Src: Copy + Into<UntypedVal> + Wrap<Wrapped>,
    Field: TryFrom<Wrapped> + Into<AnyConst16>,
    DisplayWasm<Src>: Display,
{
    for offset in [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ] {
        for mem_idx in [0, 1].map(MemIdx) {
            for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                test_store_wrap_imm_for::<Src, Wrapped, Field>(
                    wasm_op, make_instr, index_ty, mem_idx, offset, value,
                );
            }
        }
    }
    for offset in [u64::from(u32::MAX) + 1, u64::MAX - 1, u64::MAX] {
        for mem_idx in [0, 1].map(MemIdx) {
            test_store_wrap_imm_for::<Src, Wrapped, Field>(
                wasm_op,
                make_instr,
                IndexType::Memory64,
                mem_idx,
                offset,
                value,
            );
        }
    }
}

fn test_store_imm_for<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    index_ty: IndexType,
    memory_index: MemIdx,
    offset: impl Into<u64>,
    value: T,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let offset = offset.into();
    assert!(
        u16::try_from(offset).is_err() || !memory_index.is_default(),
        "this test requires non-16 bit offsets or non-default memory \
        but found: offset={offset}, memory={memory_index}"
    );
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (param $ptr {index_ty})
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let (offset_hi, offset_lo) = Offset64::split(offset);
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new(iter_filter_opts![
                make_instr(Reg::from(0), offset_lo),
                Instruction::register_and_offset_hi(Reg::from(-1), offset_hi),
                memory_index.instr(),
                Instruction::Return,
            ])
            .consts([value]),
        )
        .run();
}

fn test_store_imm<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    value: T,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    for offset in [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX] {
        for mem_idx in [0, 1].map(MemIdx) {
            for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                test_store_imm_for::<T>(wasm_op, make_instr, index_ty, mem_idx, offset, value);
            }
        }
    }
    for offset in [u64::from(u32::MAX) + 1, u64::MAX - 1, u64::MAX] {
        for mem_idx in [0, 1].map(MemIdx) {
            test_store_imm_for::<T>(
                wasm_op,
                make_instr,
                IndexType::Memory64,
                mem_idx,
                offset,
                value,
            );
        }
    }
}

fn test_store_imm16_for<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    index_ty: IndexType,
    memory_index: MemIdx,
    value: T,
    offset: impl Into<u64>,
) where
    T: Copy + TryInto<AnyConst16>,
    DisplayWasm<T>: Display,
{
    let offset = offset.into();
    assert!(
        u16::try_from(offset).is_err() || !memory_index.is_default(),
        "this test requires non-16 bit offsets or non-default memory \
        but found: offset={offset}, memory={memory_index}"
    );
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let display_value = DisplayWasm::from(value);
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (param $ptr {index_ty})
                local.get $ptr
                {param_ty}.const {display_value}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let (offset_hi, offset_lo) = Offset64::split(offset);
    let value = value.try_into().ok().unwrap();
    TranslationTest::new(&wasm)
        .expect_func_instrs(iter_filter_opts![
            make_instr(Reg::from(0), offset_lo),
            Instruction::imm16_and_offset_hi(value, offset_hi),
            memory_index.instr(),
            Instruction::Return,
        ])
        .run();
}

fn test_store_imm16<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    value: T,
) where
    T: Copy + TryInto<AnyConst16>,
    DisplayWasm<T>: Display,
{
    for offset in [
        u64::from(u16::MAX) + 1,
        u64::from(u32::MAX) - 1,
        u64::from(u32::MAX),
    ] {
        for mem_idx in [0, 1].map(MemIdx) {
            for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                test_store_imm16_for(wasm_op, make_instr, index_ty, mem_idx, value, offset);
            }
        }
    }
    for offset in [u64::from(u32::MAX) + 1, u64::MAX - 1, u64::MAX] {
        for mem_idx in [0, 1].map(MemIdx) {
            test_store_imm16_for(
                wasm_op,
                make_instr,
                IndexType::Memory64,
                mem_idx,
                value,
                offset,
            );
        }
    }
}

fn test_store_at_for(
    wasm_op: WasmOp,
    ptr: u32,
    offset: u32,
    make_instr: fn(value: Reg, address: u32) -> Instruction,
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
    TranslationTest::new(&wasm)
        .expect_func_instrs([make_instr(Reg::from(0), address), Instruction::Return])
        .run();
}

fn test_store_at(wasm_op: WasmOp, make_instr: fn(value: Reg, address: u32) -> Instruction) {
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

fn test_store_at_overflow_for(wasm_op: WasmOp, mem_idx: u32, ptr: u32, offset: u32) {
    assert!(
        ptr.checked_add(offset).is_none(),
        "testcase expects overflowing ptr+offset address"
    );
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $value {param_ty})
                i32.const {ptr}
                local.get $value
                {wasm_op} $mem{mem_idx} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

fn test_store_at_overflow(wasm_op: WasmOp) {
    let ptrs_and_offsets = [(1, u32::MAX), (u32::MAX, 1), (u32::MAX, u32::MAX)];
    for (ptr, offset) in ptrs_and_offsets {
        test_store_at_overflow_for(wasm_op, 0, ptr, offset);
        test_store_at_overflow_for(wasm_op, 1, ptr, offset);
    }
}

fn test_store_at_imm_for<T>(
    wasm_op: WasmOp,
    ptr: u32,
    offset: u32,
    value: T,
    make_instr: fn(value: Reg, address: u32) -> Instruction,
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
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new([make_instr(Reg::from(-1), address), Instruction::Return])
                .consts([value]),
        )
        .run();
}

fn test_store_at_imm<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(value: Reg, address: u32) -> Instruction,
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

fn test_store_wrap_at_imm_for<Src, Wrapped, Field>(
    wasm_op: WasmOp,
    mem_idx: u32,
    ptr: u32,
    offset: u32,
    value: Src,
    make_instr: fn(value: Field, address: u32) -> Instruction,
) where
    Src: Copy + Into<UntypedVal> + Wrap<Wrapped>,
    Field: TryFrom<Wrapped> + Into<AnyConst16>,
    DisplayWasm<Src>: Display,
{
    let address = ptr
        .checked_add(offset)
        .expect("testcase requires valid ptr+offset address");
    let display_value = DisplayWasm::from(value);
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func
                i32.const {ptr}
                {param_ty}.const {display_value}
                {wasm_op} $mem{mem_idx} offset={offset}
            )
        )
    "#
    );
    let value = Field::try_from(value.wrap()).ok().unwrap();
    let mut instrs = vec![make_instr(value, address)];
    if mem_idx != 0 {
        instrs.push(Instruction::memory_index(mem_idx));
    }
    instrs.push(Instruction::Return);
    TranslationTest::new(&wasm).expect_func_instrs(instrs).run();
}

fn test_store_wrap_at_imm<Src, Wrapped, Field>(
    wasm_op: WasmOp,
    value: Src,
    make_instr: fn(value: Field, address: u32) -> Instruction,
) where
    Src: Copy + Into<UntypedVal> + Wrap<Wrapped>,
    Field: TryFrom<Wrapped> + Into<AnyConst16>,
    DisplayWasm<Src>: Display,
{
    let ptrs_and_offsets = [
        (0, 0),
        (0, 1),
        (1, 0),
        (1, 1),
        (1000, 1000),
        (1, u32::MAX - 1),
        (u32::MAX - 1, 1),
        (0, u32::MAX),
        (u32::MAX, 0),
    ];
    for (ptr, offset) in ptrs_and_offsets {
        test_store_wrap_at_imm_for::<Src, Wrapped, Field>(
            wasm_op, 0, ptr, offset, value, make_instr,
        );
        test_store_wrap_at_imm_for::<Src, Wrapped, Field>(
            wasm_op, 1, ptr, offset, value, make_instr,
        );
    }
}

fn test_store_at_imm_overflow_for<T>(wasm_op: WasmOp, mem_idx: u8, ptr: u32, offset: u32, value: T)
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
            (memory $mem0 1)
            (memory $mem1 1)
            (func
                i32.const {ptr}
                {param_ty}.const {display_value}
                {wasm_op} $mem{mem_idx} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

fn test_store_at_imm_overflow<T>(wasm_op: WasmOp, value: T)
where
    T: Copy,
    DisplayWasm<T>: Display,
{
    let ptrs_and_offsets = [(1, u32::MAX), (u32::MAX, 1), (u32::MAX, u32::MAX)];
    for (ptr, offset) in ptrs_and_offsets {
        test_store_at_imm_overflow_for(wasm_op, 0, ptr, offset, value);
        test_store_at_imm_overflow_for(wasm_op, 1, ptr, offset, value);
    }
}
