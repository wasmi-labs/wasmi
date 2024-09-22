//! Translation tests for all Wasm `load` instructions.

use super::*;
use crate::{core::TrapCode, ir::index::Memory};

fn test_load_mem0(
    wasm_op: WasmOp,
    make_instr: fn(result: Reg, memory: Memory) -> Instruction,
    offset: u32,
) {
    assert!(
        offset > u32::from(u16::MAX),
        "offset must not be 16-bit encodable in this testcase"
    );
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32) (result {result_ty})
                local.get $ptr
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(1), Memory::from(0)),
            Instruction::register_and_imm32(Reg::from(0), offset),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

fn test_load(
    wasm_op: WasmOp,
    make_instr: fn(result: Reg, memory: Memory) -> Instruction,
    offset: u32,
) {
    assert!(
        offset > u32::from(u16::MAX),
        "offset must not be 16-bit encodable in this testcase"
    );
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (memory $mem0 1)
            (memory $mem1 1)
            (func (param $ptr i32) (result {result_ty})
                local.get $ptr
                {wasm_op} $mem1 offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            make_instr(Reg::from(1), Memory::from(1)),
            Instruction::register_and_imm32(Reg::from(0), offset),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

fn test_load_offset16(
    wasm_op: WasmOp,
    offset: u16,
    make_instr_offset16: fn(result: Reg, ptr: Reg, offset: Const16<u32>) -> Instruction,
) {
    let result_ty = wasm_op.result_ty();
    let wasm = format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32) (result {result_ty})
                local.get $ptr
                {wasm_op} offset={offset}
            )
        )
    "#
    );
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            make_instr_offset16(Reg::from(1), Reg::from(0), <Const16<u32>>::from(offset)),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

fn test_load_at_mem0(
    wasm_op: WasmOp,
    make_instr_at: fn(result: Reg, address: u32) -> Instruction,
    ptr: u32,
    offset: u32,
) {
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
    let address = ptr
        .checked_add(offset)
        .expect("ptr+offset must be valid in this testcase");
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([
            make_instr_at(Reg::from(0), address),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run();
}

fn test_load_at(
    wasm_op: WasmOp,
    make_instr_at: fn(result: Reg, address: u32) -> Instruction,
    ptr: u32,
    offset: u32,
) {
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
    let address = ptr
        .checked_add(offset)
        .expect("ptr+offset must be valid in this testcase");
    TranslationTest::from_wat(&wasm)
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
    TranslationTest::from_wat(&wasm)
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
    TranslationTest::from_wat(&wasm)
        .expect_func_instrs([Instruction::trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

macro_rules! generate_tests {
    ( $wasm_op:ident, $make_instr:expr, $make_instr_offset16:expr, $make_instr_at:expr ) => {
        #[test]
        #[cfg_attr(miri, ignore)]
        fn reg_mem0() {
            test_load_mem0(WASM_OP, $make_instr, u32::from(u16::MAX) + 1);
            test_load_mem0(WASM_OP, $make_instr, u32::MAX - 1);
            test_load_mem0(WASM_OP, $make_instr, u32::MAX);
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn reg() {
            test_load(WASM_OP, $make_instr, u32::from(u16::MAX) + 1);
            test_load(WASM_OP, $make_instr, u32::MAX - 1);
            test_load(WASM_OP, $make_instr, u32::MAX);
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn offset16() {
            test_load_offset16(WASM_OP, 0, $make_instr_offset16);
            test_load_offset16(WASM_OP, u16::MAX, $make_instr_offset16);
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at_mem0() {
            test_load_at_mem0(WASM_OP, $make_instr_at, 42, 5);
            test_load_at_mem0(WASM_OP, $make_instr_at, u32::MAX, 0);
            test_load_at_mem0(WASM_OP, $make_instr_at, 0, u32::MAX);
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn at() {
            test_load_at(WASM_OP, $make_instr_at, 42, 5);
            test_load_at(WASM_OP, $make_instr_at, u32::MAX, 0);
            test_load_at(WASM_OP, $make_instr_at, 0, u32::MAX);
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
        Instruction::i32_load,
        Instruction::i32_load_offset16,
        Instruction::i32_load_at
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
        Instruction::i64_load,
        Instruction::i64_load_offset16,
        Instruction::i64_load_at
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
        Instruction::f32_load,
        Instruction::f32_load_offset16,
        Instruction::f32_load_at
    );
}

mod f64_load {
    use super::*;

    const WASM_OP: WasmOp = WasmOp::load(WasmType::F64, "load");

    generate_tests!(
        WASM_OP,
        Instruction::f64_load,
        Instruction::f64_load_offset16,
        Instruction::f64_load_at
    );
}
