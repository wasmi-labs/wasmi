//! Translation tests for all Wasm `load` instructions.

use super::*;
use crate::{
    ir::{Address32, Offset16, Offset64, Offset64Lo},
    TrapCode,
};

fn test_load(
    wasm_op: WasmOp,
    index_ty: IndexType,
    memory_index: MemIdx,
    make_instr: fn(result: Reg, offset_lo: Offset64Lo) -> Instruction,
    offset: impl Into<u64>,
) {
    let offset = offset.into();
    assert!(
        offset > u64::from(u16::MAX),
        "offset must not be 16-bit encodable in this testcase but found: {offset}"
    );
    let index_ty = index_ty.wat();
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (param $ptr {index_ty}) (result {result_ty})
                local.get $ptr
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let (offset_hi, offset_lo) = Offset64::split(offset);
    TranslationTest::new(&wasm)
        .expect_func_instrs(iter_filter_opts![
            make_instr(Reg::from(1), offset_lo),
            Instruction::register_and_offset_hi(Reg::from(0), offset_hi),
            memory_index.instr(),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

fn test_load_offset16(
    wasm_op: WasmOp,
    index_ty: IndexType,
    offset: u16,
    make_instr_offset16: fn(result: Reg, ptr: Reg, offset: Offset16) -> Instruction,
) {
    let result_ty = wasm_op.result_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory {index_ty} 1)
            (func (param $ptr {index_ty}) (result {result_ty})
                local.get $ptr
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            make_instr_offset16(Reg::from(1), Reg::from(0), offset16(offset)),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

fn test_load_at(
    wasm_op: WasmOp,
    index_ty: IndexType,
    memory_index: MemIdx,
    make_instr_at: fn(result: Reg, address: Address32) -> Instruction,
    ptr: u64,
    offset: u64,
) {
    let result_ty = wasm_op.result_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (result {result_ty})
                {index_ty}.const {ptr}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let address = effective_address32(ptr, offset);
    TranslationTest::new(&wasm)
        .expect_func_instrs(iter_filter_opts![
            make_instr_at(Reg::from(0), address),
            memory_index.instr(),
            Instruction::return_reg(0),
        ])
        .run();
}

fn test_load_at_overflow(
    wasm_op: WasmOp,
    index_ty: IndexType,
    memory_index: MemIdx,
    ptr: u64,
    offset: u64,
) {
    let result_ty = wasm_op.result_ty();
    let index_repr = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 {index_repr} 1)
            (memory $mem1 {index_repr} 1)
            (func (result {result_ty})
                {index_repr}.const {ptr}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    assert_overflowing_ptr_offset(index_ty, ptr, offset);
    TranslationTest::new(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

fn test_load_at_fallback(
    wasm_op: WasmOp,
    memory_index: MemIdx,
    make_instr: fn(result: Reg, offset_lo: Offset64Lo) -> Instruction,
    ptr: u64,
    offset: u64,
) {
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 i64 1)
            (memory $mem1 i64 1)
            (func (result {result_ty})
                i64.const {ptr}
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let Some(address64) = ptr.checked_add(offset) else {
        panic!("ptr+offset must be a valid 64-bit result but found: ptr={ptr}, offset={offset}")
    };
    if u32::try_from(address64).is_ok() {
        panic!("ptr+offset must not fit into a `u32` value but found: ptr={ptr}, offset={offset}")
    }
    let (offset_hi, offset_lo) = Offset64::split(address64);
    TranslationTest::new(&wasm)
        .expect_func(
            ExpectedFunc::new(iter_filter_opts![
                make_instr(Reg::from(0), offset_lo),
                Instruction::register_and_offset_hi(Reg::from(-1), offset_hi),
                memory_index.instr(),
                Instruction::return_reg(Reg::from(0)),
            ])
            .consts([0_u64]),
        )
        .run();
}

macro_rules! generate_tests {
    ( $wasm_op:ident, $make_instr:expr, $make_instr_offset16:expr, $make_instr_at:expr ) => {
        #[test]
        #[cfg_attr(miri, ignore)]
        fn reg() {
            [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX]
                .into_iter()
                .for_each(|offset| {
                    test_load(WASM_OP, IndexType::Memory32, MemIdx(0), $make_instr, offset);
                    test_load(WASM_OP, IndexType::Memory32, MemIdx(1), $make_instr, offset)
                })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn reg_memory64() {
            [
                u64::from(u16::MAX) + 1,
                u64::from(u32::MAX) - 1,
                u64::from(u32::MAX),
                u64::from(u32::MAX) + 1,
                u64::MAX - 1,
                u64::MAX,
            ]
            .into_iter()
            .for_each(|offset| {
                test_load(WASM_OP, IndexType::Memory64, MemIdx(0), $make_instr, offset);
                test_load(WASM_OP, IndexType::Memory64, MemIdx(1), $make_instr, offset)
            })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn offset16() {
            [0, 1, u16::MAX - 1, u16::MAX]
                .into_iter()
                .for_each(|offset| {
                    for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                        test_load_offset16(WASM_OP, index_ty, offset, $make_instr_offset16);
                    }
                })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at() {
            [
                (0, 0),
                (42, 5),
                (u64::from(u32::MAX), 0),
                (0, u64::from(u32::MAX)),
            ]
            .into_iter()
            .for_each(|(ptr, offset)| {
                for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                    test_load_at(WASM_OP, index_ty, MemIdx(0), $make_instr_at, ptr, offset);
                    test_load_at(WASM_OP, index_ty, MemIdx(1), $make_instr_at, ptr, offset);
                }
            })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at_overflow() {
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
                test_load_at_overflow(WASM_OP, index_ty, MemIdx(0), ptr, offset);
                test_load_at_overflow(WASM_OP, index_ty, MemIdx(1), ptr, offset);
            })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at_fallback() {
            [
                (u64::from(u32::MAX), 1),
                (1, u64::from(u32::MAX)),
                (1, u64::MAX - 1),
                (u64::MAX - 1, 1),
                (0, u64::MAX),
                (u64::MAX, 0),
            ]
            .into_iter()
            .for_each(|(ptr, offset)| {
                test_load_at_fallback(WASM_OP, MemIdx(0), $make_instr, ptr, offset);
                test_load_at_fallback(WASM_OP, MemIdx(1), $make_instr, ptr, offset);
            })
        }
    };
}

mod i32_load {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I32, "load");

    generate_tests!(
        WASM_OP,
        Instruction::load32,
        Instruction::load32_offset16,
        Instruction::load32_at
    );
}

mod i32_load8_s {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I32, "load8_s");

    generate_tests!(
        WASM_OP,
        Instruction::i32_load8_s,
        Instruction::i32_load8_s_offset16,
        Instruction::i32_load8_s_at
    );
}

mod i32_load8_u {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I32, "load8_u");

    generate_tests!(
        WASM_OP,
        Instruction::i32_load8_u,
        Instruction::i32_load8_u_offset16,
        Instruction::i32_load8_u_at
    );
}

mod i32_load16_s {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I32, "load16_s");

    generate_tests!(
        WASM_OP,
        Instruction::i32_load16_s,
        Instruction::i32_load16_s_offset16,
        Instruction::i32_load16_s_at
    );
}

mod i32_load16_u {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I32, "load16_u");

    generate_tests!(
        WASM_OP,
        Instruction::i32_load16_u,
        Instruction::i32_load16_u_offset16,
        Instruction::i32_load16_u_at
    );
}

mod i64_load {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I64, "load");

    generate_tests!(
        WASM_OP,
        Instruction::load64,
        Instruction::load64_offset16,
        Instruction::load64_at
    );
}

mod i64_load8_s {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I64, "load8_s");

    generate_tests!(
        WASM_OP,
        Instruction::i64_load8_s,
        Instruction::i64_load8_s_offset16,
        Instruction::i64_load8_s_at
    );
}

mod i64_load8_u {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I64, "load8_u");

    generate_tests!(
        WASM_OP,
        Instruction::i64_load8_u,
        Instruction::i64_load8_u_offset16,
        Instruction::i64_load8_u_at
    );
}

mod i64_load16_s {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I64, "load16_s");

    generate_tests!(
        WASM_OP,
        Instruction::i64_load16_s,
        Instruction::i64_load16_s_offset16,
        Instruction::i64_load16_s_at
    );
}

mod i64_load16_u {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I64, "load16_u");

    generate_tests!(
        WASM_OP,
        Instruction::i64_load16_u,
        Instruction::i64_load16_u_offset16,
        Instruction::i64_load16_u_at
    );
}

mod i64_load32_s {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I64, "load32_s");

    generate_tests!(
        WASM_OP,
        Instruction::i64_load32_s,
        Instruction::i64_load32_s_offset16,
        Instruction::i64_load32_s_at
    );
}

mod i64_load32_u {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::I64, "load32_u");

    generate_tests!(
        WASM_OP,
        Instruction::i64_load32_u,
        Instruction::i64_load32_u_offset16,
        Instruction::i64_load32_u_at
    );
}

mod f32_load {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::F32, "load");

    generate_tests!(
        WASM_OP,
        Instruction::load32,
        Instruction::load32_offset16,
        Instruction::load32_at
    );
}

mod f64_load {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::F64, "load");

    generate_tests!(
        WASM_OP,
        Instruction::load64,
        Instruction::load64_offset16,
        Instruction::load64_at
    );
}
