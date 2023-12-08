use super::*;

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    let wasm = wat2wasm(
        r"
        (module
            (memory $m 10)
            (func (param $delta i32) (result i32)
                (local.get $delta)
                (memory.grow $m)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_grow(Register::from_i16(1), Register::from_i16(0)),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run();
}

fn test_imm16(delta: u32) {
    assert!(1 <= delta && delta <= u32::from(u16::MAX));
    let wasm = wat2wasm(&format!(
        r"
        (module
            (memory $m 10)
            (func (result i32)
                (i32.const {delta})
                (memory.grow $m)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_grow_by(Register::from_i16(0), u32imm16(delta)),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm16() {
    test_imm16(1);
    test_imm16(42);
    test_imm16(u32::from(u16::MAX) - 1);
    test_imm16(u32::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_zero() {
    let wasm =
        wat2wasm(
            r"
        (module
            (memory $m 10)
            (func (result i32)
                (i32.const 0)
                (memory.grow $m)
            )
        )",
        );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_size(Register::from_i16(0)),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run();
}

fn test_imm(delta: u32) {
    let wasm = wat2wasm(&format!(
        r"
        (module
            (memory $m 10)
            (func (result i32)
                (i32.const {delta})
                (memory.grow $m)
            )
        )",
    ));
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::memory_grow(Register::from_i16(0), Register::from_i16(-1)),
                Instruction::return_reg(Register::from_i16(0)),
            ])
            .consts([delta]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    test_imm(u32::from(u16::MAX) + 1);
    test_imm(u32::MAX - 1);
    test_imm(u32::MAX);
}
