use super::*;
use crate::ir::index::Memory;

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    let wasm = r"
        (module
            (memory $m 10)
            (func (param $delta i32) (result i32)
                (local.get $delta)
                (memory.grow $m)
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::memory_grow(Reg::from(1), Reg::from(0)),
            Instruction::memory_index(0),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

fn test_imm16(delta: u32) {
    assert!(delta != 0);
    let wasm = &format!(
        r"
        (module
            (memory $m 10)
            (func (result i32)
                (i32.const {delta})
                (memory.grow $m)
            )
        )",
    );
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::memory_grow_by(Reg::from(0), delta),
            Instruction::memory_index(0),
            Instruction::return_reg(Reg::from(0)),
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
    test_imm16(u32::from(u16::MAX) + 1);
    test_imm16(u32::MAX - 1);
    test_imm16(u32::MAX);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_zero() {
    let wasm = r"
        (module
            (memory $m 10)
            (func (result i32)
                (i32.const 0)
                (memory.grow $m)
            )
        )";
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::memory_size(Reg::from(0), Memory::from(0)),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run();
}
