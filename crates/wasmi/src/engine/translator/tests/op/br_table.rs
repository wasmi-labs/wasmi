use super::*;
use crate::engine::bytecode::{BranchOffset, GlobalIdx, RegisterSpan};

#[test]
#[cfg_attr(miri, ignore)]
fn reg_len_targets_1() {
    let wasm =
        wat2wasm(
            r"
        (module
            (func (param $index i32) (result i32)
                (block
                    (br_table 0 (local.get $index))
                )
                (return (i32.const 10))
            )
        )",
        );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch(BranchOffset::from(1)),
            Instruction::return_imm32(10_i32),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_params_0() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param $index i32) (result i32)
                (block
                    (block
                        (block
                            (block
                                (br_table 3 2 1 0 (local.get $index))
                            )
                            (return (i32.const 10))
                        )
                        (return (i32.const 20))
                    )
                    (return (i32.const 30))
                )
                (return (i32.const 40))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_table(Register::from_i16(0), 4),
            Instruction::branch(BranchOffset::from(7)),
            Instruction::branch(BranchOffset::from(5)),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::return_imm32(10),
            Instruction::return_imm32(20),
            Instruction::return_imm32(30),
            Instruction::return_imm32(40),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_params_0_return() {
    let wasm = wat2wasm(
        r"
        (module
            (global $g (mut i32) (i32.const 0))
            (func (param $index i32)
                (block
                    (block
                        (block
                            (block
                                (br_table 4 3 2 1 0 (local.get $index))
                            )
                            (return (global.set $g (i32.const 10)))
                        )
                        (return (global.set $g (i32.const 20)))
                    )
                    (return (global.set $g (i32.const 30)))
                )
                (return (global.set $g (i32.const 40)))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_table(Register::from_i16(0), 5),
            Instruction::Return,
            Instruction::branch(BranchOffset::from(10)),
            Instruction::branch(BranchOffset::from(7)),
            Instruction::branch(BranchOffset::from(4)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::global_set_i32imm16(GlobalIdx::from(0), 10),
            Instruction::Return,
            Instruction::global_set_i32imm16(GlobalIdx::from(0), 20),
            Instruction::Return,
            Instruction::global_set_i32imm16(GlobalIdx::from(0), 30),
            Instruction::Return,
            Instruction::global_set_i32imm16(GlobalIdx::from(0), 40),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_params_1_return() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param $index i32) (param $value i32) (result i32)
                (block (result i32)
                    (block (result i32)
                        (block (result i32)
                            (block (result i32)
                                (local.get $value) ;; param to br_table targets
                                (br_table 4 3 2 1 0 (local.get $index))
                            )
                            (return (i32.add (i32.const 10)))
                        )
                        (return (i32.add (i32.const 20)))
                    )
                    (return (i32.add (i32.const 30)))
                )
                (return (i32.add (i32.const 40)))
            )
        )",
    );
    let index = Register::from_i16(0);
    let value = Register::from_i16(1);
    let result = Register::from_i16(2);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(result, value),
            Instruction::branch_table(index, 5),
            Instruction::return_reg(result),
            Instruction::branch(BranchOffset::from(10)),
            Instruction::branch(BranchOffset::from(7)),
            Instruction::branch(BranchOffset::from(4)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::i32_add_imm16(result, result, 10),
            Instruction::return_reg(result),
            Instruction::i32_add_imm16(result, result, 20),
            Instruction::return_reg(result),
            Instruction::i32_add_imm16(result, result, 30),
            Instruction::return_reg(result),
            Instruction::i32_add_imm16(result, result, 40),
            Instruction::return_reg(result),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_params_1_pass() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param $index i32) (param $value i32) (result i32)
                (block (result i32 i32)
                    (block (result i32)
                        (block (result i32)
                            (block (result i32)
                                (local.get $value) ;; param to br_table targets
                                (br_table 2 1 0 (local.get $index))
                            )
                            (i32.const 10)
                            (br 2)
                        )
                        (i32.const 20)
                        (br 1)
                    )
                    (i32.const 30)
                    (br 0)
                )
                (i32.add)
            )
        )",
    );
    let index = Register::from_i16(0);
    let value = Register::from_i16(1);
    let result = Register::from_i16(2);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(result, value),
            Instruction::branch_table(index, 3),
            Instruction::branch(BranchOffset::from(7)),
            Instruction::branch(BranchOffset::from(4)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::copy_imm32(Register::from_i16(3), 10_i32),
            Instruction::branch(BranchOffset::from(5)),
            Instruction::copy_imm32(Register::from_i16(3), 20_i32),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::copy_imm32(Register::from_i16(3), 30_i32),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::i32_add(result, result, Register::from_i16(3)),
            Instruction::return_reg(result),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_params_2_ops() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param $index i32) (param $lhs i32) (param $rhs i32) (result i32)
                (block (result i32 i32)
                    (block (result i32 i32)
                        (block (result i32 i32)
                            (local.get $lhs) ;; param to br_table targets
                            (local.get $rhs) ;; param to br_table targets
                            (br_table 2 1 0 (local.get $index))
                        )
                        (return (i32.add))
                    )
                    (return (i32.sub))
                )
                (return (i32.mul))
            )
        )",
    );
    let index = Register::from_i16(0);
    let lhs = Register::from_i16(1);
    let result = Register::from_i16(3);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy2(RegisterSpan::new(result), lhs, lhs.next()),
            Instruction::branch_table(index, 3),
            Instruction::branch(BranchOffset::from(7)),
            Instruction::branch(BranchOffset::from(4)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::i32_add(result, result, Register::from_i16(4)),
            Instruction::return_reg(result),
            Instruction::i32_sub(result, result, Register::from_i16(4)),
            Instruction::return_reg(result),
            Instruction::i32_mul(result, result, Register::from_i16(4)),
            Instruction::return_reg(result),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_params_2_return() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param $index i32) (param $lhs i32) (param $rhs i32) (result i32 i32)
                (block (result i32 i32)
                    (block (result i32 i32)
                        (block (result i32 i32)
                            (local.get $lhs) ;; param to br_table targets
                            (local.get $rhs) ;; param to br_table targets
                            (br_table 3 2 1 0 (local.get $index))
                        )
                        (return (i32.add) (i32.const 0))
                    )
                    (return (i32.sub) (i32.const 1))
                )
                (return (i32.mul) (i32.const 2))
            )
        )",
    );
    let index = Register::from_i16(0);
    let lhs = Register::from_i16(1);
    let result = Register::from_i16(3);
    let result2 = result.next();
    let results = RegisterSpan::new(result).iter(2);
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::copy2(RegisterSpan::new(result), lhs, lhs.next()),
                Instruction::branch_table(index, 4),
                Instruction::return_span(results),
                Instruction::branch(BranchOffset::from(7)),
                Instruction::branch(BranchOffset::from(4)),
                Instruction::branch(BranchOffset::from(1)),
                Instruction::i32_add(result, result, result2),
                Instruction::return_reg2(3, -1),
                Instruction::i32_sub(result, result, result2),
                Instruction::return_reg2(3, -2),
                Instruction::i32_mul(result, result, result2),
                Instruction::return_reg2(3, -3),
            ])
            .consts([0_i32, 1, 2]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_params_1_diff() {
    // Test that uses `br_table` with targets that do not share
    // common branch parameters. We achieve this by interleaving
    // dynamic register allocations via `global.get`.
    //
    // This way the translator is forced to generated less optimized bytecode.
    let wasm = wat2wasm(
        r"
        (module
            (global $g (mut i32) (i32.const 0))
            (func (param $index i32) (param $input i32) (result i32)
                (block (result i32)
                    (block (result i32)
                        (global.get $g) ;; allocates a dynamic register
                        (block (result i32)
                            (local.get $input) ;; param to br_table targets
                            (br_table 3 2 1 1 2 3 0 (local.get $index))
                        )
                        (return (i32.add))
                    )
                    (return (i32.sub (i32.const 10)))
                )
                (return (i32.mul (i32.const 10)))
            )
        )",
    );
    let index = Register::from_i16(0);
    let input = Register::from_i16(1);
    let result = Register::from_i16(2);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_get(result, GlobalIdx::from(0)),
            Instruction::branch_table(index, 7),
            Instruction::branch(BranchOffset::from(13)),
            Instruction::branch(BranchOffset::from(10)),
            Instruction::branch(BranchOffset::from(7)),
            Instruction::branch(BranchOffset::from(6)),
            Instruction::branch(BranchOffset::from(7)),
            Instruction::branch(BranchOffset::from(8)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::copy(Register::from_i16(3), input),
            Instruction::branch(BranchOffset::from(6)),
            Instruction::copy(Register::from_i16(2), input),
            Instruction::branch(BranchOffset::from(6)),
            Instruction::copy(Register::from_i16(2), input),
            Instruction::branch(BranchOffset::from(6)),
            Instruction::return_reg(input),
            Instruction::i32_add(
                Register::from_i16(2),
                Register::from_i16(2),
                Register::from_i16(3),
            ),
            Instruction::return_reg(result),
            Instruction::i32_sub_imm16(result, result, 10),
            Instruction::return_reg(result),
            Instruction::i32_mul_imm16(result, result, 10),
            Instruction::return_reg(result),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_params_2_diff() {
    // Test that uses `br_table` with targets that do not share
    // common branch parameters. We achieve this by interleaving
    // dynamic register allocations via `global.get`.
    //
    // This way the translator is forced to generated less optimized bytecode.
    let wasm = wat2wasm(
        r"
        (module
            (global $g (mut i32) (i32.const 0))
            (func (param $index i32) (param $lhs i32) (param $rhs i32) (result i32)
                (block (result i32 i32)
                    (block (result i32 i32)
                        (global.get $g) ;; allocates a dynamic register
                        (block (result i32 i32)
                            (local.get $lhs) ;; param to br_table targets
                            (local.get $rhs) ;; param to br_table targets
                            (br_table 2 1 1 2 0 (local.get $index))
                        )
                        (i32.add)
                        (drop) ;; drop `global.get` again
                        (return)
                    )
                    (return (i32.sub))
                )
                (return (i32.mul))
            )
        )",
    );
    let index = Register::from_i16(0);
    let lhs = Register::from_i16(1);
    let result = Register::from_i16(3);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_get(result, GlobalIdx::from(0)),
            Instruction::branch_table(index, 5),
            Instruction::branch(BranchOffset::from(9)),
            Instruction::branch(BranchOffset::from(6)),
            Instruction::branch(BranchOffset::from(5)),
            Instruction::branch(BranchOffset::from(6)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::copy2(RegisterSpan::new(Register::from_i16(4)), lhs, lhs.next()),
            Instruction::branch(BranchOffset::from(5)),
            Instruction::copy2(RegisterSpan::new(result), lhs, lhs.next()),
            Instruction::branch(BranchOffset::from(5)),
            Instruction::copy2(RegisterSpan::new(result), lhs, lhs.next()),
            Instruction::branch(BranchOffset::from(5)),
            Instruction::i32_add(
                Register::from_i16(4),
                Register::from_i16(4),
                Register::from_i16(5),
            ),
            Instruction::return_reg(result),
            Instruction::i32_sub(result, result, Register::from_i16(4)),
            Instruction::return_reg(result),
            Instruction::i32_mul(result, result, Register::from_i16(4)),
            Instruction::return_reg(result),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_params_0() {
    fn test_with(index: u32) {
        let targets: [i32; 3] = [30, 20, 10];
        let clamped_index = (index as usize).min(targets.len() - 1);
        let chosen = targets[clamped_index];
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (result i32)
                    (block
                        (block
                            (block
                                (br_table 2 1 0 (i32.const {index}))
                            )
                            (return (i32.const 10))
                        )
                        (return (i32.const 20))
                    )
                    (return (i32.const 30))
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::branch(BranchOffset::from(1)),
                Instruction::return_imm32(chosen),
            ])
            .run()
    }
    test_with(0);
    test_with(1);
    test_with(2);
    test_with(3);
    test_with(1000);
}
