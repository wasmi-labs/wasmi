//! Translation tests for all Wasm `load` instructions.

use super::*;
use crate::{
    core::TrapCode,
    ir::{Offset16, Offset64, Offset64Lo},
};
use core::fmt;

/// Adjusts a translation test to use memories with that specified index type.
#[derive(Copy, Clone)]
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

/// Macro that turns an iterator over `Option<T>` into an iterator over `T`.
///
/// - Filters out all the `None` items yielded by the input iterator.
/// - Allows to specify `Some` items as just `T` as convenience.
macro_rules! iter_filter_opts {
    [ $($item:expr),* $(,)? ] => {{
        [ $( ::core::option::Option::from($item) ),* ].into_iter().filter_map(|x| x)
    }};
}

/// Convenience type to create Wat memories with a tagged memory index.
pub struct MemIdx(u32);

impl fmt::Display for MemIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "$mem{}", self.0)
    }
}

impl MemIdx {
    /// Returns the `$mem{n}` memory index used by some Wasm memory instructions.
    fn instr(self) -> Option<Instruction> {
        match self.0 {
            0 => None,
            n => Some(Instruction::memory_index(n)),
        }
    }
}

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
                {wasm_op} {memory_index} offset={offset}
            )
        )
    "#
    );
    let address = ptr
        .checked_add(offset)
        .expect("ptr+offset must be valid in this testcase");
    let address = u32::try_from(address).expect("ptr+offset must fit into a `u32` value");
    TranslationTest::new(&wasm)
        .expect_func_instrs(iter_filter_opts![
            make_instr_at(Reg::from(0), address),
            memory_index.instr(),
            Instruction::return_reg(0),
        ])
        .run();
}

/// Asserts that `ptr+offset` overflow either `i32` or `i64` depending on `index_ty`.
fn assert_overflowing_ptr_offset(index_ty: IndexType, ptr: u64, offset: u64) {
    match index_ty {
        IndexType::Memory32 => {
            let Ok(ptr32) = u32::try_from(ptr) else {
                panic!("ptr must be a 32-bit value but found: {ptr}");
            };
            let Ok(offset32) = u32::try_from(offset) else {
                panic!("offset must be a 32-bit value but found: {offset}");
            };
            assert!(
                ptr32.checked_add(offset32).is_none(),
                "ptr+offset must overflow in this testcase (32-bit)"
            );
        }
        IndexType::Memory64 => {
            assert!(
                ptr.checked_add(offset).is_none(),
                "ptr+offset must overflow in this testcase (64-bit)"
            );
        }
    }
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
        fn reg_mem0() {
            [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX]
                .into_iter()
                .for_each(|offset| {
                    test_load(WASM_OP, IndexType::Memory32, MemIdx(0), $make_instr, offset);
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
                test_load(WASM_OP, IndexType::Memory64, MemIdx(0), $make_instr, offset);
            })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn reg() {
            [u32::from(u16::MAX) + 1, u32::MAX - 1, u32::MAX]
                .into_iter()
                .for_each(|offset| {
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
                    test_load_at(WASM_OP, index_ty, MemIdx(0), $make_instr_at, ptr, offset);
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
                    test_load_at(WASM_OP, index_ty, MemIdx(1), $make_instr_at, ptr, offset);
                }
            })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at_overflow_mem0() {
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
                test_load_at_overflow(WASM_OP, index_ty, MemIdx(1), ptr, offset);
            })
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at_fallback_mem0() {
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
