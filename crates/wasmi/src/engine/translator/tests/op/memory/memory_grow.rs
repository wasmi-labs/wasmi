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
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_grow(Reg::from(1), Reg::from(0)),
            Instruction::memory_index(0),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run();
}

fn test_imm32(index_ty: IndexType, memory_index: MemIdx, delta: u32) {
    assert!(delta != 0);
    let index_ty = index_ty.wat();
    let wasm = &format!(
        r"
        (module
            (memory $mem0 {index_ty} 1)
            (memory $mem1 {index_ty} 1)
            (func (result {index_ty})
                {index_ty}.const {delta}
                memory.grow {memory_index}
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs(iter_filter_opts![
            Instruction::memory_grow_imm(Reg::from(0), delta),
            Instruction::memory_index(memory_index.0),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm32() {
    for delta in [
        1,
        42,
        u32::from(u16::MAX) - 1,
        u32::from(u16::MAX),
        u32::from(u16::MAX) + 1,
        u32::MAX - 1,
        u32::MAX,
    ] {
        for mem_idx in [0, 1].map(MemIdx) {
            for index_ty in [IndexType::Memory32, IndexType::Memory64] {
                test_imm32(index_ty, mem_idx, delta)
            }
        }
    }
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
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_size(Reg::from(0), Memory::from(0)),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run();
}
