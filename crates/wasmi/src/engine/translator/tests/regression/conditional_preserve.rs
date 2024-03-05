use super::*;

#[test]
#[cfg_attr(miri, ignore)]
fn simple_block() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                (block
                    (br_if 0 (local.get 1))
                    (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                )
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 0),
            Instruction::branch_i32_ne_imm(Register::from_i16(1), 0, BranchOffset16::from(2)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32 i32 i32) (result i32)
                local.get 0
                (if (local.get 2)
                    (then
                        (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                    )
                )
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(3, 0),
            Instruction::branch_i32_eq_imm(Register::from_i16(2), 0, BranchOffset16::from(2)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::return_reg(3),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nested_block() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32 i32) (param $c0 i32) (param $c1 i32) (result i32 i32)
                local.get 0 ;; 1st return value
                local.get 1 ;; 2nd return value
                (block
                    (br_if 0 (local.get $c0))
                    (local.set 0 (i32.const 10)) ;; conditionally overwrites (local 0) on stack
                    (block
                        (br_if 1 (local.get $c1))
                        (local.set 1 (i32.const 20)) ;; conditionally overwrites (local 1) on stack
                    )
                )
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegisterSpan::new(Register::from_i16(4)), 1, 0),
            Instruction::branch_i32_ne_imm(Register::from_i16(2), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::branch_i32_ne_imm(Register::from_i16(3), 0, BranchOffset16::from(2)),
            Instruction::copy_imm32(Register::from_i16(1), 20_i32),
            Instruction::return_reg2(5, 4),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nested_if() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32 i32 i32 i32) (result i32 i32)
                local.get 0 ;; 1st return value
                local.get 1 ;; 2nd return value
                (if (local.get 2)
                    (then
                        (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                        (if (local.get 3)
                            (then
                                (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                            )
                        )
                    )
                )
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegisterSpan::new(Register::from_i16(4)), 1, 0),
            Instruction::branch_i32_eq_imm(Register::from_i16(2), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::branch_i32_eq_imm(Register::from_i16(3), 0, BranchOffset16::from(2)),
            Instruction::copy_imm32(Register::from_i16(1), 20_i32),
            Instruction::return_reg2(5, 4),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn expr_block() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32 i32) (result i32)
                (i32.add
                    (local.get 1)
                    (block (result i32)
                        (drop (br_if 0
                            (i32.const 10) ;; br_if return value
                            (local.get 0)  ;; br_if condition
                        ))
                        (local.set 1 (i32.const 20))
                        (i32.const 30)
                    )
                )
            )
        )
        "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(3, 1),
            Instruction::branch_i32_eq_imm(Register::from_i16(0), 0, BranchOffset16::from(3)),
            Instruction::copy_imm32(Register::from_i16(2), 10_i32),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::copy_imm32(Register::from_i16(1), 20_i32),
            Instruction::copy_imm32(Register::from_i16(2), 30_i32),
            Instruction::i32_add(
                Register::from_i16(2),
                Register::from_i16(3),
                Register::from_i16(2),
            ),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn expr_if() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32 i32 i32) (result i32)
                (i32.add
                    (local.get 0)
                    (if (result i32) (local.get 1)
                        (then
                            (local.set 0 (i32.const 10))
                            (i32.const 20)
                        )
                        (else
                            (i32.const 30)
                        )
                    )
                )
            )
        )
        "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(4, 0),
            Instruction::branch_i32_eq_imm(Register::from_i16(1), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy_imm32(Register::from_i16(3), 20_i32),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy_imm32(Register::from_i16(3), 30_i32),
            Instruction::i32_add(
                Register::from_i16(3),
                Register::from_i16(4),
                Register::from_i16(3),
            ),
            Instruction::return_reg(3),
        ])
        .run()
}
