use super::*;
use crate::{
    engine::EngineFunc,
    ir::{BranchOffset, BranchOffset16, LocalSpan},
};

#[test]
#[cfg_attr(miri, ignore)]
fn simple_block_1() {
    let wasm = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                (block
                    (br_if 0 (local.get 1))
                    (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 0),
            Instruction::branch_i32_ne_imm16(Local::from(1), 0, BranchOffset16::from(2)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_block_2() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32) (result i32 i32)
                local.get 0
                local.get 1
                (block
                    (br_if 0 (local.get 2))
                    (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                    (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(LocalSpan::new(Local::from(3)), 0, 1),
            Instruction::branch_i32_ne_imm16(Local::from(2), 0, BranchOffset16::from(3)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::return_reg2_ext(3, 4),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_block_3_span() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32 i32) (result i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                (block
                    (br_if 0 (local.get 3))
                    (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                    (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                    (local.set 2 (i32.const 30)) ;; overwrites (local 2) conditionally
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span_non_overlapping(
                LocalSpan::new(Local::from(4)),
                LocalSpan::new(Local::from(0)),
                3_u16,
            ),
            Instruction::branch_i32_ne_imm16(Local::from(3), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy_imm32(Local::from(2), 30_i32),
            Instruction::return_reg3_ext(4, 5, 6),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_block_3_many() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32 i32) (result i32 i32 i32)
                local.get 2
                local.get 1
                local.get 0
                (block
                    (br_if 0 (local.get 3))
                    (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                    (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                    (local.set 2 (i32.const 30)) ;; overwrites (local 2) conditionally
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_many_non_overlapping_ext(LocalSpan::new(Local::from(4)), 2, 1),
            Instruction::register(0),
            Instruction::branch_i32_ne_imm16(Local::from(3), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy_imm32(Local::from(2), 30_i32),
            Instruction::return_reg3_ext(4, 5, 6),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_block_4_params_2() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32 i32 i32) (result i32 i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                local.get 3
                (block (param i32 i32) (result i32 i32)
                    (br_if 0 (local.get 4))
                    (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                    (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                    (local.set 2 (i32.const 30)) ;; overwrites (local 2) conditionally
                    (local.set 3 (i32.const 40)) ;; overwrites (local 3) conditionally
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span_non_overlapping(
                LocalSpan::new(Local::from(7)),
                LocalSpan::new(Local::from(0)),
                4_u16,
            ),
            Instruction::branch_i32_eq_imm16(Local::from(4), 0, BranchOffset16::from(3)),
            Instruction::copy2_ext(LocalSpan::new(Local::from(5)), 9, 10),
            Instruction::branch(BranchOffset::from(6)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy_imm32(Local::from(2), 30_i32),
            Instruction::copy_imm32(Local::from(3), 40_i32),
            Instruction::copy2_ext(LocalSpan::new(Local::from(5)), 9, 10),
            Instruction::return_many_ext(7, 8, 5),
            Instruction::register(6),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_block_30() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
                ;; Push 30 locals on the compilation stack.
                (local.get  0) (local.get  1) (local.get  2) (local.get  3) (local.get  4)
                (local.get  5) (local.get  6) (local.get  7) (local.get  8) (local.get  9)
                (local.get  0) (local.get  1) (local.get  2) (local.get  3) (local.get  4)
                (local.get  5) (local.get  6) (local.get  7) (local.get  8) (local.get  9)
                (local.get  0) (local.get  1) (local.get  2) (local.get  3) (local.get  4)
                (local.get  5) (local.get  6) (local.get  7) (local.get  8) (local.get  9)
                ;; Now all those previously pushed locals need to be preserved.
                (block
                    (br_if 0 (local.get 10))
                    (local.set 0 (i32.const  10)) ;; overwrites (local 0) conditionally
                    (local.set 1 (i32.const  20)) ;; overwrites (local 1) conditionally
                    (local.set 2 (i32.const  30)) ;; overwrites (local 2) conditionally
                    (local.set 3 (i32.const  40)) ;; overwrites (local 3) conditionally
                    (local.set 4 (i32.const  50)) ;; overwrites (local 4) conditionally
                    (local.set 5 (i32.const  60)) ;; overwrites (local 5) conditionally
                    (local.set 6 (i32.const  70)) ;; overwrites (local 6) conditionally
                    (local.set 7 (i32.const  80)) ;; overwrites (local 7) conditionally
                    (local.set 8 (i32.const  90)) ;; overwrites (local 8) conditionally
                    (local.set 9 (i32.const 100)) ;; overwrites (local 9) conditionally
                )
                ;; Drop 20 out of the 30 return values which still returns every local once.
                (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
                (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_many_non_overlapping_ext(LocalSpan::new(Local::from(11)), 9, 8),
            Instruction::register_list_ext(7, 6, 5),
            Instruction::register_list_ext(4, 3, 2),
            Instruction::register2_ext(1, 0),
            Instruction::branch_i32_ne_imm16(Local::from(10), 0, BranchOffset16::from(11)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy_imm32(Local::from(2), 30_i32),
            Instruction::copy_imm32(Local::from(3), 40_i32),
            Instruction::copy_imm32(Local::from(4), 50_i32),
            Instruction::copy_imm32(Local::from(5), 60_i32),
            Instruction::copy_imm32(Local::from(6), 70_i32),
            Instruction::copy_imm32(Local::from(7), 80_i32),
            Instruction::copy_imm32(Local::from(8), 90_i32),
            Instruction::copy_imm32(Local::from(9), 100_i32),
            Instruction::return_many_ext(20, 19, 18),
            Instruction::register_list_ext(17, 16, 15),
            Instruction::register_list_ext(14, 13, 12),
            Instruction::register(11),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_1() {
    let wasm = r#"
        (module
            (func (param i32 i32) (result i32)
                local.get 0
                (if (local.get 1)
                    (then
                        (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                    )
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(2, 0),
            Instruction::branch_i32_eq_imm16(Local::from(1), 0, BranchOffset16::from(2)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_2() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32) (result i32 i32)
                local.get 0
                local.get 1
                (if (local.get 2)
                    (then
                        (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                        (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                    )
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(LocalSpan::new(Local::from(3)), 0, 1),
            Instruction::branch_i32_eq_imm16(Local::from(2), 0, BranchOffset16::from(3)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::return_reg2_ext(3, 4),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_3_span() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32 i32) (result i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                (if (local.get 3)
                    (then
                        (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                        (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                        (local.set 2 (i32.const 30)) ;; overwrites (local 2) conditionally
                    )
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span_non_overlapping(
                LocalSpan::new(Local::from(4)),
                LocalSpan::new(Local::from(0)),
                3_u16,
            ),
            Instruction::branch_i32_eq_imm16(Local::from(3), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy_imm32(Local::from(2), 30_i32),
            Instruction::return_reg3_ext(4, 5, 6),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_3_many() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32 i32) (result i32 i32 i32)
                local.get 2
                local.get 1
                local.get 0
                (if (local.get 3)
                    (then
                        (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                        (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                        (local.set 2 (i32.const 30)) ;; overwrites (local 2) conditionally
                    )
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_many_non_overlapping_ext(LocalSpan::new(Local::from(4)), 2, 1),
            Instruction::register(0),
            Instruction::branch_i32_eq_imm16(Local::from(3), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy_imm32(Local::from(2), 30_i32),
            Instruction::return_reg3_ext(4, 5, 6),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn simple_if_4_params_2() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32 i32 i32) (result i32 i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                local.get 3
                (if (param i32 i32) (result i32 i32) (local.get 4)
                    (then
                        (local.set 0 (i32.const 10)) ;; overwrites (local 0) conditionally
                        (local.set 1 (i32.const 20)) ;; overwrites (local 1) conditionally
                        (local.set 2 (i32.const 30)) ;; overwrites (local 2) conditionally
                        (local.set 3 (i32.const 40)) ;; overwrites (local 3) conditionally
                    )
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_span_non_overlapping(
                LocalSpan::new(Local::from(7)),
                LocalSpan::new(Local::from(0)),
                4_u16,
            ),
            Instruction::branch_i32_eq_imm16(Local::from(4), 0, BranchOffset16::from(7)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy_imm32(Local::from(2), 30_i32),
            Instruction::copy_imm32(Local::from(3), 40_i32),
            Instruction::copy2_ext(LocalSpan::new(Local::from(5)), 9, 10),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy2_ext(LocalSpan::new(Local::from(5)), 9, 10),
            Instruction::return_many_ext(7, 8, 5),
            Instruction::register(6),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nested_block() {
    let wasm = r#"
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
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(LocalSpan::new(Local::from(4)), 0, 1),
            Instruction::branch_i32_ne_imm16(Local::from(2), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::branch_i32_ne_imm16(Local::from(3), 0, BranchOffset16::from(2)),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::return_reg2_ext(4, 5),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn nested_if() {
    let wasm = r#"
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
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2_ext(LocalSpan::new(Local::from(4)), 0, 1),
            Instruction::branch_i32_eq_imm16(Local::from(2), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::branch_i32_eq_imm16(Local::from(3), 0, BranchOffset16::from(2)),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::return_reg2_ext(4, 5),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn expr_block() {
    let wasm = r#"
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
        "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(3, 1),
            Instruction::branch_i32_eq_imm16(Local::from(0), 0, BranchOffset16::from(3)),
            Instruction::copy_imm32(Local::from(2), 10_i32),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy_imm32(Local::from(2), 30_i32),
            Instruction::i32_add(Local::from(2), Local::from(3), Local::from(2)),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn expr_if() {
    let wasm = r#"
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
        "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(4, 0),
            Instruction::branch_i32_eq_imm16(Local::from(1), 0, BranchOffset16::from(4)),
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(3), 20_i32),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy_imm32(Local::from(3), 30_i32),
            Instruction::i32_add(Local::from(3), Local::from(4), Local::from(3)),
            Instruction::return_reg(3),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn invalid_preservation_slot_reuse_1() {
    let wasm = r#"
        (module
            (func (param i32 i32)
                (local.get 1) ;; preserved for (local.tee 1)
                (local.get 0) ;; preserved for (local.tee 0)
                (local.tee 0 (i32.popcnt (local.get 0)))
                (i32.add)
                (local.set 1)
                (drop)
            )
          )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(3, 0),
            Instruction::i32_popcnt(Local::from(0), Local::from(0)),
            Instruction::i32_add(Local::from(2), Local::from(3), Local::from(0)),
            Instruction::copy(3, 1),
            Instruction::copy(1, 2),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn invalid_preservation_slot_reuse_2() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32) (result i32)
                (i32.const 20)
            )
            (func (param i32 i32)
                (local.get 1) ;; preserved after (local.tee 1)
                (local.get 1) ;; ^
                (local.get 0) ;; preserved after (local.tee 0)
                (local.tee 0 (i32.popcnt (local.get 0)))
                (call $f)
                (local.set 1)
                (drop)
            )
          )
    "#;
    TranslationTest::new(wasm)
        .expect_func(ExpectedFunc::new([Instruction::return_imm32(20_i32)]))
        .expect_func(ExpectedFunc::new([
            Instruction::copy(3, 0),
            Instruction::i32_popcnt(Local::from(0), Local::from(0)),
            Instruction::call_internal(LocalSpan::new(Local::from(2)), EngineFunc::from_u32(0)),
            Instruction::register3_ext(1, 3, 0),
            Instruction::copy(3, 1),
            Instruction::copy(1, 2),
            Instruction::Return,
        ]))
        .run()
}

#[test]
fn concat_local_tee_pair() {
    let wasm = r"
        (module
            (func (result i32)
                (local i32 i32)
            
                (local.set 0 (i32.const 10))
                (local.set 1 (i32.const 20))

                local.get 1
                local.get 0

                (local.set 0 (i32.const 10))

                local.tee 1
                i32.add
            )
        )
    ";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy_imm32(Local::from(1), 20_i32),
            Instruction::copy(Local::from(4), Local::from(0)), // preserved
            Instruction::copy_imm32(Local::from(0), 10_i32),
            Instruction::copy(Local::from(3), Local::from(1)), // preserved
            Instruction::copy(Local::from(1), Local::from(4)),
            Instruction::i32_add(Local::from(2), Local::from(3), Local::from(1)),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
fn loop_iter_1() {
    let wasm = r#"
        (module
            (func (export "test") (param i32) (result i32)
                (local.set 0 (i32.const 0))
                (local.get 0)
                (loop $continue
                    (if (i32.eqz (local.get 0))
                        (then
                            (local.set 0 (i32.const 1))
                            (br $continue)
                        )
                    )
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(0, 0_i32),
            Instruction::copy(2, 0),
            Instruction::branch_i32_ne_imm16(0, 0_i16, BranchOffset16::from(3)),
            Instruction::copy_imm32(0, 1),
            Instruction::branch(BranchOffset::from(-2)),
            Instruction::return_reg(2),
        ])
        .run()
}
