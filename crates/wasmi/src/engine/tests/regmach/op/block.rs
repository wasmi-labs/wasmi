use super::*;

#[test]
fn empty_block() {
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

#[test]
fn nested_empty_block() {
    let wasm = wat2wasm(
        r"
        (module
            (func (block (block)))
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::Return])
        .run()
}

#[test]
fn identity_block_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (block (param i32) (result i32))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::return_reg(Register::from_u16(0))])
        .run()
}

#[test]
fn nested_identity_block_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (block (param i32) (result i32)
                    (block (param i32) (result i32))
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::return_reg(Register::from_u16(0))])
        .run()
}
