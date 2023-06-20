use wasmi_core::TrapCode;

use super::*;

fn test_load(
    wasm_op: WasmOp,
    offset: u32,
    make_instr: fn(result: Register, ptr: Register) -> Instruction,
) {
    println!("-2");
    assert!(
        offset > u32::from(u16::MAX),
        "offset must not be 16-bit encodable in this testcase"
    );
    println!("-1");
    let result_ty = wasm_op.result_ty();
    println!("0");
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32) (result {result_ty})
                local.get $ptr
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    println!("1");
    TranslationTest::new(wasm)
        .expect_func([
            make_instr(Register::from_u16(1), Register::from_u16(0)),
            Instruction::const32(offset),
            Instruction::return_reg(Register::from_u16(1)),
        ])
        .run();
}

fn test_load_offset16(
    wasm_op: WasmOp,
    offset: u16,
    make_instr_offset16: fn(result: Register, ptr: Register, offset: Const16) -> Instruction,
) {
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $ptr i32) (result {result_ty})
                local.get $ptr
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([
            make_instr_offset16(
                Register::from_u16(1),
                Register::from_u16(0),
                Const16::from_u16(offset),
            ),
            Instruction::return_reg(Register::from_u16(1)),
        ])
        .run();
}

fn test_load_at(
    wasm_op: WasmOp,
    ptr: u32,
    offset: u32,
    make_instr_at: fn(result: Register, address: Const32) -> Instruction,
) {
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (result {result_ty})
                i32.const {ptr}
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    let address = ptr
        .checked_add(offset)
        .expect("ptr+offset must be valid in this testcase");
    TranslationTest::new(wasm)
        .expect_func([
            make_instr_at(Register::from_u16(0), Const32::from(address)),
            Instruction::return_reg(Register::from_u16(0)),
        ])
        .run();
}

fn test_load_at_overflow(wasm_op: WasmOp, ptr: u32, offset: u32) {
    let result_ty = wasm_op.result_ty();
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (result {result_ty})
                i32.const {ptr}
                {wasm_op} offset={offset}
            )
        )
    "#,
    ));
    assert!(
        ptr.checked_add(offset).is_none(),
        "ptr+offset must overflow in this testcase"
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::Trap(TrapCode::MemoryOutOfBounds)])
        .run();
}

macro_rules! generate_tests {
    ( $wasm_op:ident, $make_instr:expr, $make_instr_offset16:expr, $make_instr_at:expr ) => {
        #[test]
        fn reg() {
            test_load(WASM_OP, u32::MAX, $make_instr);
        }

        #[test]
        fn offset16() {
            test_load_offset16(WASM_OP, 0, $make_instr_offset16);
            test_load_offset16(WASM_OP, u16::MAX, $make_instr_offset16);
        }

        #[test]
        fn at() {
            test_load_at(WASM_OP, 42, 5, $make_instr_at);
            test_load_at(WASM_OP, u32::MAX, 0, $make_instr_at);
            test_load_at(WASM_OP, 0, u32::MAX, $make_instr_at);
        }

        #[test]
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
