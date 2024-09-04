use super::*;

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
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::memory_size(Reg::from(0)),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run();
}
