use super::*;
use crate::ir::index::Memory;

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    let wasm = r"
        (module
            (memory $m 10)
            (func (result i32)
                (memory.size $m)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::memory_size(Local::from(0), Memory::from(0)),
            Instruction::return_reg(Local::from(0)),
        ])
        .run();
}
