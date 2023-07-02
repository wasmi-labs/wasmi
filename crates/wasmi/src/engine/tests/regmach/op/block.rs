use super::*;

#[test]
fn simple_block() {
    let wasm = wat2wasm(
        r"
        (module
            (func (block))
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::Return])
        .run()
}
