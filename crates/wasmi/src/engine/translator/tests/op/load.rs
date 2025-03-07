//! Translation tests for all Wasm `load` instructions.

use super::*;
use crate::{
    core::TrapCode,
    ir::{Offset16, Offset64, Offset64Lo},
};

/// Creates an [`Offset16`] from the given `offset`.
fn offset16(offset: u16) -> Offset16 {
    Offset16::try_from(u64::from(offset)).unwrap()
}

/// Adjusts a translation test to use memories with that specified index type.
enum IndexType {
    /// The 32-bit index type.
    ///
    /// This is WebAssembly's default.
    Memory32,
    /// The 64-bit index type.
    ///
    /// This got introduced by the Wasm `memory64` proposal.
    Memory64,
}

impl IndexType {
    /// Returns the `.wat` string reprensetation for the [`IndexType`] of a `memory` declaration.
    fn wat(self) -> &'static str {
        match self {
            Self::Memory32 => "i32",
            Self::Memory64 => "i64",
        }
    }
}

fn test_load_mem0(
    wasm_op: WasmOp,
    index_ty: IndexType,
    make_instr: fn(result: Reg, offset_lo: Offset64Lo) -> Instruction,
    offset: impl Into<u64>,
) {
    let offset = offset.into();
    assert!(
        offset > u64::from(u16::MAX),
        "offset must not be 16-bit encodable in this testcase"
    );
    let index_ty = index_ty.wat();
    let result_ty = wasm_op.result_ty();
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
    let (offset_hi, offset_lo) = Offset64::split(u64::from(offset));
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(1), offset_lo),
            Instruction::register_and_offset_hi(Reg::from(0), offset_hi),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

fn test_load(
    wasm_op: WasmOp,
    index_ty: IndexType,
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
                {wasm_op} $mem1 offset={offset}
            )
        )
    "#
    );
    let (offset_hi, offset_lo) = Offset64::split(u64::from(offset));
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(1), offset_lo),
            Instruction::register_and_offset_hi(Reg::from(0), offset_hi),
            Instruction::memory_index(1),
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

fn test_load_at_mem0(
    wasm_op: WasmOp,
    index_ty: IndexType,
    make_instr_at: fn(result: Reg, address: u32) -> Instruction,
    ptr: u64,
    offset: u64,
) {
    let result_ty = wasm_op.result_ty();
    let index_ty = index_ty.wat();
    let wasm = format!(
        r#"
        (module
            (memory {index_ty} 1)
            (func (result {result_ty})
                {index_ty}.const {ptr}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    let address = ptr
        .checked_add(offset)
        .expect("ptr+offset must be valid in this testcase");
    let address = u32::try_from(address).expect("ptr+offset must fit into a `u32` value");
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            make_instr_at(Reg::from(0), address),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run();
}

fn test_load_at(
    wasm_op: WasmOp,
    index_ty: IndexType,
    make_instr_at: fn(result: Reg, address: u32) -> Instruction,
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
                {wasm_op} $mem1 offset={offset}
            )
        )
    "#
    );
    let address = ptr
        .checked_add(offset)
        .expect("ptr+offset must be valid in this testcase");
    let address = u32::try_from(address).expect("ptr+offset must fit into a `u32` value");
    TranslationTest::new(&wasm)
        .expect_func_instrs([
            make_instr_at(Reg::from(0), address),
            Instruction::memory_index(1),
            Instruction::return_reg(0),
        ])
        .run();
}

fn test_load_at_overflow_mem0(wasm_op: WasmOp, ptr: u32, offset: u32) {
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (result {result_ty})
                i32.const {ptr}
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    assert!(
        ptr.checked_add(offset).is_none(),
        "ptr+offset must overflow in this testcase"
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

fn test_load_at_overflow(wasm_op: WasmOp, ptr: u32, offset: u32) {
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (result {result_ty})
                i32.const {ptr}
                {wasm_op} $mem1 offset={offset}
            )
        )
    "#
    );
    assert!(
        ptr.checked_add(offset).is_none(),
        "ptr+offset must overflow in this testcase"
    );
    TranslationTest::new(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

macro_rules! generate_tests {
    ( $wasm_op:ident, $make_instr:expr, $make_instr_offset16:expr, $make_instr_at:expr ) => {
        #[test]
        #[cfg_attr(miri, ignore)]
        fn reg_mem0() {
            [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX]
                .into_iter()
                .for_each(|offset| {
                    test_load_mem0(WASM_OP, IndexType::Memory32, $make_instr, offset);
                })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn reg_mem0_memory64() {
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
                test_load_mem0(WASM_OP, IndexType::Memory64, $make_instr, offset);
            })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn reg() {
            [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX]
                .into_iter()
                .for_each(|offset| test_load(WASM_OP, IndexType::Memory32, $make_instr, offset))
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
            .for_each(|offset| test_load(WASM_OP, IndexType::Memory64, $make_instr, offset))
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
        fn at_mem0() {
            [
                (0, 0),
                (42, 5),
                (u64::from(u32::MAX), 0),
                (0, u64::from(u32::MAX)),
            ]
            .into_iter()
            .for_each(|(ptr, offset)| {
                for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                    test_load_at_mem0(WASM_OP, index_ty, $make_instr_at, ptr, offset);
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
                    test_load_at(WASM_OP, index_ty, $make_instr_at, ptr, offset);
                }
            })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at_overflow_mem0() {
            test_load_at_overflow_mem0(WASM_OP, u32::MAX, 1);
            test_load_at_overflow_mem0(WASM_OP, 1, u32::MAX);
            test_load_at_overflow_mem0(WASM_OP, u32::MAX, u32::MAX);
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at_overflow() {
            test_load_at_overflow(WASM_OP, u32::MAX, 1);
            test_load_at_overflow(WASM_OP, 1, u32::MAX);
            test_load_at_overflow(WASM_OP, u32::MAX, u32::MAX);
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
