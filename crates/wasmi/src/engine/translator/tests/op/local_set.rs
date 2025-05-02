use super::*;
use crate::{
    engine::EngineFunc,
    ir::{
        index::{Func, FuncType, Table},
        RegSpan,
    },
};

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    let wasm = r"
        (module
            (func (param i32) (result i32)
                (local.set 0 (i32.const 10))
                (local.get 0)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_tee() {
    let wasm = r"
        (module
            (func (param i32) (result i32)
                (local.tee 0 (i32.const 10))
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::return_imm32(10_i32),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_result_1() {
    let wasm = r"
        (module
            (func (param $lhs i32) (param $rhs i32) (result i32)
                (local.tee $lhs
                    (i32.add
                        (local.get $lhs)
                        (local.get $rhs)
                    )
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_add(Reg::from(0), Reg::from(0), Reg::from(1)),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_call_internal_result_1() {
    let wasm = r"
        (module
            (func $f (param i32) (result i32)
                (local.get 0)
            )
            (func (param i32) (result i32)
                (local.tee 0
                    (call $f (local.get 0))
                )
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(Reg::from(0))])
        .expect_func_instrs([
            Instruction::call_internal(RegSpan::new(Reg::from(0)), EngineFunc::from_u32(0)),
            Instruction::register(0),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_call_imported_result_1() {
    let wasm = r#"
        (module
            (import "env" "f" (func $f (param i32) (result i32)))
            (func (param i32) (result i32)
                (local.tee 0
                    (call $f (local.get 0))
                )
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_imported(RegSpan::new(Reg::from(0)), Func::from(0)),
            Instruction::register(0),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_call_indirect_result_1() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32) (result i32)))
            (func (param $index i32) (param $value i32) (result i32)
                (local.tee 0
                    (call_indirect (type $type) (local.get $value) (local.get $index))
                )
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_indirect(RegSpan::new(Reg::from(0)), FuncType::from(0)),
            Instruction::call_indirect_params(Reg::from(0), Table::from(0)),
            Instruction::register(1),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_select_result_1() {
    let wasm = r#"
        (module
            (func (param $condition i32) (result i32)
                (i32.const 10)
                (i32.const 20)
                (select (local.get $condition))
                (local.tee $condition)
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::select_i32_ne_imm16(Reg::from(0), Reg::from(0), 0_i16),
                Instruction::register2_ext(-1, -2),
                Instruction::return_reg(Reg::from(0)),
            ])
            .consts([10, 20]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn avoid_overwrite_result_1() {
    let wasm = r"
        (module
            (func (param $lhs i32) (param $rhs i32) (result i32)
                (block (result i32)
                    (i32.add
                        (local.get $lhs)
                        (local.get $rhs)
                    )
                )
                (local.tee $lhs)
            )
        )";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_add(Reg::from(2), Reg::from(0), Reg::from(1)),
            Instruction::copy(Reg::from(0), Reg::from(2)),
            Instruction::return_reg(Reg::from(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn local_set_chain() {
    let wasm = r#"
        (module
            (func (param i32) (param i32) (result i32)
                (local.set 0 (i32.const 10))
                (local.set 1 (local.get 0))
                (local.get 1)
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn local_tee_chain() {
    let wasm = r#"
        (module
            (func (param i32) (param i32) (result i32)
                (local.tee 0 (i32.const 10))
                (local.tee 1)
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy_imm32(Reg::from(1), 10_i32),
            Instruction::return_imm32(10_i32),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_result_1() {
    let wasm = r#"
        (module
            (func (param $lhs i32) (param $rhs i32) (result i32)
                (local.get 0)
                (local.set 0
                    (i32.add
                        (local.get $lhs)
                        (local.get $rhs)
                    )
                )
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(3), Reg::from(0)),
            Instruction::i32_add(Reg::from(0), Reg::from(0), Reg::from(1)),
            Instruction::return_reg(Reg::from(3)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_result_2() {
    let wasm = r#"
        (module
            (func (param $lhs i32) (param $rhs i32) (result i32 i32)
                (local.get 0)
                (local.get 0)
                (local.set 0
                    (i32.add
                        (local.get $lhs)
                        (local.get $rhs)
                    )
                )
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(3), Reg::from(0)),
            Instruction::i32_add(Reg::from(0), Reg::from(0), Reg::from(1)),
            Instruction::return_reg2_ext(3, 3),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_0() {
    let wasm = r#"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (local.set 0 (i32.const 10))
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::return_reg(Reg::from(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_1() {
    let wasm = r#"
        (module
            (func (param i32) (result i32 i32)
                (local.get 0)
                (local.get 0)
                (local.set 0 (i32.const 10))
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::return_reg2_ext(1, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_2() {
    let wasm = r#"
        (module
            (func (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.set 0 (i32.const 10))
                (local.get 0)
                (local.set 0 (i32.const 20))
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(3), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy(Reg::from(2), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 20_i32),
            Instruction::return_reg2_ext(3, 2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_3() {
    let wasm = r#"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (local.set 0 (i32.const 10))
                (drop)
                (local.get 0)
                (local.set 0 (i32.const 20))
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 20_i32),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_4() {
    let wasm = r#"
        (module
            (func (param i32) (result i32 i32)
                (local.get 0)
                (local.get 0)
                (local.set 0 (i32.const 10))
                (drop)
                (drop)
                (local.get 0)
                (local.get 0)
                (local.set 0 (i32.const 20))
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy(Reg::from(1), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 20_i32),
            Instruction::return_reg2_ext(1, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_5() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.set 2 (i32.const 11))
                (local.set 1 (i32.const 22))
                (local.set 0 (i32.const 33))
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(5), Reg::from(2)),
            Instruction::copy_imm32(Reg::from(2), 11_i32),
            Instruction::copy(Reg::from(4), Reg::from(1)),
            Instruction::copy_imm32(Reg::from(1), 22_i32),
            Instruction::copy(Reg::from(3), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 33_i32),
            Instruction::return_reg3_ext(3, 4, 5),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_6() {
    let wasm = r#"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.set 2 (i32.const 11))
                (local.set 0 (i32.const 22))
                (local.set 1 (i32.const 33))
                (drop)        ;; drops above (local.get 2)
                (local.get 1) ;; reuse dropped preservation slot
                (local.set 1 (i32.const 44))
            )
        )"#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Reg::from(5), Reg::from(2)),
            Instruction::copy_imm32(Reg::from(2), 11_i32),
            Instruction::copy(Reg::from(4), Reg::from(0)),
            Instruction::copy_imm32(Reg::from(0), 22_i32),
            Instruction::copy(Reg::from(3), Reg::from(1)),
            Instruction::copy_imm32(Reg::from(1), 33_i32),
            Instruction::copy(Reg::from(5), Reg::from(1)),
            Instruction::copy_imm32(Reg::from(1), 44_i32),
            Instruction::return_reg3_ext(4, 3, 5),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn merge_overwriting_local_set() {
    let wasm = r"
        (module
            (func (result i32)
                (local i32 i32)

                i32.const 10
                local.set 0
                i32.const 20
                local.set 1
            
                local.get 1
                local.tee 0
                local.tee 1
            )
        )
    ";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy_imm32(Reg::from(1), 20_i32),
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), 1, 1),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn merge_overwriting_local_set_lhs() {
    let wasm = r"
        (module
            (func (result i32)
                (local i32 i32)

                i32.const 10
                local.set 0
                i32.const 20
                local.set 1
            
                local.get 0
                local.tee 1
                local.tee 0
            )
        )
    ";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy_imm32(Reg::from(1), 20_i32),
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), 0, 0),
            Instruction::return_reg(0),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn merge_overwriting_local_set_3() {
    let wasm = r"
        (module
            (func (result i32)
                (local i32 i32 i32) 

                (local.set 0 (i32.const 10))
                (local.set 1 (i32.const 20))
                (local.set 2 (i32.const 30))

                local.get 2
                local.tee 0
                local.tee 1
            )
        )
    ";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy_imm32(Reg::from(1), 20_i32),
            Instruction::copy_imm32(Reg::from(2), 30_i32),
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), 2, 2),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn merge_overwriting_local_set_3_lhs() {
    let wasm = r"
        (module
            (func (result i32)
                (local i32 i32 i32) 

                (local.set 0 (i32.const 10))
                (local.set 1 (i32.const 20))
                (local.set 2 (i32.const 30))

                local.get 2
                local.tee 1
                local.tee 0
            )
        )
    ";
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Reg::from(0), 10_i32),
            Instruction::copy_imm32(Reg::from(1), 20_i32),
            Instruction::copy_imm32(Reg::from(2), 30_i32),
            Instruction::copy2_ext(RegSpan::new(Reg::from(0)), 2, 2),
            Instruction::return_reg(0),
        ])
        .run()
}
