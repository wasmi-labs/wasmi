//! Translation tests for all Wasm `store` instructions.

use super::*;
use crate::{
    core::UntypedVal,
    engine::translator::utils::Wrap,
    ir::{Address32, AnyConst16, Offset16, Offset64, Offset64Lo},
};

mod f32_store;
mod f64_store;
mod i32_store;
mod i32_store16;
mod i32_store8;
mod i64_store;
mod i64_store16;
mod i64_store32;
mod i64_store8;

use crate::TrapCode;
use core::fmt::Display;

fn test_store_for(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    memory_index: MemIdx,
    index_ty: IndexType,
    offset: u64,
) {
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
    make_instr: fn(value: Reg, address: Address32) -> Instruction,
    index_ty: IndexType,
    memory_index: MemIdx,
    ptr: u64,
    offset: u64,
) {
    let address = effective_address32(ptr, offset);
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (param $value {param_ty})
                {index_ty}.const {ptr}
                local.get $value
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs(iter_filter_opts![
            make_instr(Reg::from(0), address),
            memory_index.instr(),
            Instruction::Return,
        ])
        .run();
}

fn test_store_at(wasm_op: WasmOp, make_instr: fn(value: Reg, address: Address32) -> Instruction) {
    for (ptr, offset) in [
        (0, 0),
        (0, 1),
        (1, 0),
        (1, 1),
        (1000, 1000),
        (1, u64::from(u32::MAX) - 1),
        (u64::from(u32::MAX) - 1, 1),
        (0, u64::from(u32::MAX)),
        (u64::from(u32::MAX), 0),
    ] {
        for mem_idx in [0, 1].map(MemIdx) {
            for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                test_store_at_for(wasm_op, make_instr, index_ty, mem_idx, ptr, offset);
            }
        }
    }
}

fn test_store_at_overflow_for(
    wasm_op: WasmOp,
    index_ty: IndexType,
    memory_index: MemIdx,
    ptr: u64,
    offset: u64,
) {
    assert_overflowing_ptr_offset(index_ty, ptr, offset);
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (param $value {param_ty})
                {index_ty}.const {ptr}
                local.get $value
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

fn test_store_at_overflow(wasm_op: WasmOp) {
    [
        (IndexType::Memory32, u64::from(u32::MAX), 1),
        (IndexType::Memory32, 1, u64::from(u32::MAX)),
        (
            IndexType::Memory32,
            u64::from(u32::MAX),
            u64::from(u32::MAX),
        ),
        (IndexType::Memory64, u64::MAX, 1),
        (IndexType::Memory64, 1, u64::MAX),
        (IndexType::Memory64, u64::MAX, u64::MAX),
    ]
    .into_iter()
    .for_each(|(index_ty, ptr, offset)| {
        test_store_at_overflow_for(wasm_op, index_ty, MemIdx(0), ptr, offset);
        test_store_at_overflow_for(wasm_op, index_ty, MemIdx(1), ptr, offset);
    })
}

fn test_store_at_fallback_for(
    wasm_op: WasmOp,
    make_instr: fn(result: Reg, offset_lo: Offset64Lo) -> Instruction,
    memory_index: MemIdx,
    ptr: u64,
    offset: u64,
) {
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 i64 1)
            (memory $mem1 i64 1)
            (func (param $value {param_ty})
                i64.const {ptr}
                local.get $value
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let address = ptr.checked_add(offset).unwrap();
    let (offset_hi, offset_lo) = Offset64::split(address);
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new(iter_filter_opts![
                make_instr(Reg::from(-1), offset_lo),
                Instruction::register_and_offset_hi(Reg::from(0), offset_hi),
                memory_index.instr(),
                Instruction::Return,
            ])
            .consts([0_u64]),
        )
        .run();
}

fn test_store_at_fallback(
    wasm_op: WasmOp,
    make_instr: fn(result: Reg, offset_lo: Offset64Lo) -> Instruction,
) {
    [
        (u64::from(u32::MAX), 1),
        (1, u64::from(u32::MAX)),
        (u64::from(u32::MAX), u64::from(u32::MAX)),
        (u64::MAX - 1, 1),
        (1, u64::MAX - 1),
    ]
    .into_iter()
    .for_each(|(ptr, offset)| {
        test_store_at_fallback_for(wasm_op, make_instr, MemIdx(0), ptr, offset);
        test_store_at_fallback_for(wasm_op, make_instr, MemIdx(1), ptr, offset);
    })
}

fn test_store_at_imm_for<T>(
    wasm_op: WasmOp,
    make_instr: fn(value: Reg, address: Address32) -> Instruction,
    index_ty: IndexType,
    memory_index: MemIdx,
    ptr: u64,
    offset: u64,
    value: T,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let address = effective_address32(ptr, offset);
    let display_value = DisplayWasm::from(value);
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func
                {index_ty}.const {ptr}
                {param_ty}.const {display_value}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new(iter_filter_opts![
                make_instr(Reg::from(-1), address),
                memory_index.instr(),
                Instruction::Return,
            ])
            .consts([value]),
        )
        .run();
}

fn test_store_at_imm<T>(
    wasm_op: WasmOp,
    value: T,
    make_instr: fn(value: Reg, address: Address32) -> Instruction,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    [
        (0, 0),
        (0, 1),
        (1, 0),
        (1, 1),
        (1000, 1000),
        (1, u64::from(u32::MAX) - 1),
        (u64::from(u32::MAX) - 1, 1),
        (0, u64::from(u32::MAX)),
        (u64::from(u32::MAX), 0),
    ]
    .into_iter()
    .for_each(|(ptr, offset)| {
        for mem_idx in [0, 1].map(MemIdx) {
            for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                test_store_at_imm_for(wasm_op, make_instr, index_ty, mem_idx, ptr, offset, value);
            }
        }
    })
}

fn test_store_wrap_at_imm_for<Src, Wrapped, Field>(
    wasm_op: WasmOp,
    make_instr: fn(value: Field, address: Address32) -> Instruction,
    index_ty: IndexType,
    memory_index: MemIdx,
    ptr: u64,
    offset: u64,
    value: Src,
) where
    Src: Copy + Wrap<Wrapped>,
    Field: TryFrom<Wrapped> + Into<AnyConst16>,
    DisplayWasm<Src>: Display,
{
    let address = effective_address32(ptr, offset);
    let display_value = DisplayWasm::from(value);
    let param_ty = wasm_op.param_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func
                {index_ty}.const {ptr}
                {param_ty}.const {display_value}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let value = Field::try_from(value.wrap()).ok().unwrap();
    TranslationTest::new(&wasm)
        .expect_func_instrs(iter_filter_opts![
            make_instr(value, address),
            memory_index.instr(),
            Instruction::Return,
        ])
        .run();
}

fn test_store_wrap_at_imm<Src, Wrapped, Field>(
    wasm_op: WasmOp,
    make_instr: fn(value: Field, address: Address32) -> Instruction,
    value: Src,
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
        (1, u64::from(u32::MAX) - 1),
        (u64::from(u32::MAX) - 1, 1),
        (0, u64::from(u32::MAX)),
        (u64::from(u32::MAX), 0),
    ];
    for (ptr, offset) in ptrs_and_offsets {
        for mem_idx in [0, 1].map(MemIdx) {
            for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                test_store_wrap_at_imm_for::<Src, Wrapped, Field>(
                    wasm_op, make_instr, index_ty, mem_idx, ptr, offset, value,
                );
            }
        }
    }
}

fn test_store_at_imm_overflow_for<T>(
    wasm_op: WasmOp,
    memory_index: MemIdx,
    ptr: u32,
    offset: u32,
    value: T,
) where
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
                {wasm_op} {memory_index} offset={offset}
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
        test_store_at_imm_overflow_for(wasm_op, MemIdx(0), ptr, offset, value);
        test_store_at_imm_overflow_for(wasm_op, MemIdx(1), ptr, offset, value);
    }
}

fn test_store_at_imm16_fallback_for<T, Wrapped>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    memory_index: MemIdx,
    ptr: u64,
    offset: u64,
    value: T,
) where
    T: Copy + Wrap<Wrapped>,
    Wrapped: TryInto<AnyConst16>,
    DisplayWasm<T>: Display,
{
    assert!(
        u32::try_from(ptr.saturating_add(offset)).is_err(),
        "testcase expects overflowing 32-bit ptr+offset address"
    );
    let display_value = DisplayWasm::from(value);
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 i64 1)
            (memory $mem1 i64 1)
            (func
                i64.const {ptr}
                {param_ty}.const {display_value}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let address = ptr.checked_add(offset).unwrap();
    let (offset_hi, offset_lo) = Offset64::split(address);
    let value = value.wrap().try_into().ok().unwrap();
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new(iter_filter_opts![
                make_instr(Reg::from(-1), offset_lo),
                Instruction::imm16_and_offset_hi(value, offset_hi),
                memory_index.instr(),
                Instruction::Return,
            ])
            .consts([0_u64]),
        )
        .run();
}

fn test_store_wrap_at_imm16_fallback<T, Wrapped>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    value: T,
) where
    T: Copy + Wrap<Wrapped>,
    Wrapped: TryInto<AnyConst16>,
    DisplayWasm<T>: Display,
{
    let ptrs_and_offsets = [
        (1, u64::from(u32::MAX)),
        (u64::from(u32::MAX), 1),
        (u64::from(u32::MAX), u64::from(u32::MAX)),
        (0, u64::MAX),
        (u64::MAX, 0),
        (1, u64::MAX - 1),
        (u64::MAX - 1, 1),
    ];
    for (ptr, offset) in ptrs_and_offsets {
        for mem_idx in [0, 1].map(MemIdx) {
            test_store_at_imm16_fallback_for::<T, Wrapped>(
                wasm_op, make_instr, mem_idx, ptr, offset, value,
            );
        }
    }
}

fn test_store_at_imm16_fallback<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    value: T,
) where
    T: Copy + TryInto<AnyConst16>,
    DisplayWasm<T>: Display,
{
    test_store_wrap_at_imm16_fallback::<T, T>(wasm_op, make_instr, value)
}

fn test_store_at_imm_fallback_for<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    memory_index: MemIdx,
    ptr: u64,
    offset: u64,
    value: T,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    assert!(
        u32::try_from(ptr.saturating_add(offset)).is_err(),
        "testcase expects overflowing 32-bit ptr+offset address"
    );
    let display_value = DisplayWasm::from(value);
    let param_ty = wasm_op.param_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 i64 1)
            (memory $mem1 i64 1)
            (func
                i64.const {ptr}
                {param_ty}.const {display_value}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let address = ptr.checked_add(offset).unwrap();
    let (offset_hi, offset_lo) = Offset64::split(address);
    let (value_reg, value_const) = match value.into() == 0_u64.into() {
        true => {
            // Case: since this scheme always allocates a 0 as function constant value
            //       and address is zero the translator only uses a single register to
            //       represent both. (special case)
            (Reg::from(-1), None)
        }
        false => {
            // Case: address is non-zero so the translator uses 2 different registers
            //       to represent the zero'ed ptr value and the value. (common case)
            (Reg::from(-2), Some(value.into()))
        }
    };
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new(iter_filter_opts![
                make_instr(Reg::from(-1), offset_lo),
                Instruction::register_and_offset_hi(value_reg, offset_hi),
                memory_index.instr(),
                Instruction::Return,
            ])
            .consts(iter_filter_opts![UntypedVal::from(0_u64), value_const]),
        )
        .run();
}

fn test_store_at_imm_fallback<T>(
    wasm_op: WasmOp,
    make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
    value: T,
) where
    T: Copy + Into<UntypedVal>,
    DisplayWasm<T>: Display,
{
    let ptrs_and_offsets = [
        (1, u64::from(u32::MAX)),
        (u64::from(u32::MAX), 1),
        (u64::from(u32::MAX), u64::from(u32::MAX)),
        (0, u64::MAX),
        (u64::MAX, 0),
        (1, u64::MAX - 1),
        (u64::MAX - 1, 1),
    ];
    for (ptr, offset) in ptrs_and_offsets {
        for mem_idx in [0, 1].map(MemIdx) {
            test_store_at_imm_fallback_for::<T>(wasm_op, make_instr, mem_idx, ptr, offset, value);
        }
    }
}
