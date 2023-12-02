use super::*;
use crate::engine::{
    bytecode::{FuncIdx, RegisterSpan, SignatureIdx, TableIdx},
    regmach::CompiledFunc,
};

#[test]
#[cfg_attr(miri, ignore)]
fn imm() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32)
                (local.set 0 (i32.const 10))
                (local.get 0)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_tee() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32) (result i32)
                (local.tee 0 (i32.const 10))
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_result_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param $lhs i32) (param $rhs i32) (result i32)
                (local.tee $lhs
                    (i32.add
                        (local.get $lhs)
                        (local.get $rhs)
                    )
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_add(
                Register::from_i16(0),
                Register::from_i16(0),
                Register::from_i16(1),
            ),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_call_internal_result_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func $f (param i32) (result i32)
                (local.get 0)
            )
            (func (param i32) (result i32)
                (local.tee 0
                    (call $f (local.get 0))
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(Register::from_i16(0))])
        .expect_func_instrs([
            Instruction::call_internal(
                RegisterSpan::new(Register::from_i16(0)),
                CompiledFunc::from_u32(0),
            ),
            Instruction::register(0),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_call_imported_result_1() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "env" "f" (func $f (param i32) (result i32)))
            (func (param i32) (result i32)
                (local.tee 0
                    (call $f (local.get 0))
                )
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_imported(RegisterSpan::new(Register::from_i16(0)), FuncIdx::from(0)),
            Instruction::register(0),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_call_indirect_result_1() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32) (result i32)))
            (func (param $index i32) (param $value i32) (result i32)
                (local.tee 0
                    (call_indirect (type $type) (local.get $value) (local.get $index))
                )
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::call_indirect(
                RegisterSpan::new(Register::from_i16(0)),
                SignatureIdx::from(0),
            ),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::register(1),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn overwrite_select_result_1() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $condition i32) (result i32)
                (i32.const 10)
                (i32.const 20)
                (select (local.get $condition))
                (local.tee $condition)
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::select_imm32(Register::from_i16(0), 10_i32),
            Instruction::select_imm32(Register::from_i16(0), 20_i32),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn avoid_overwrite_result_1() {
    let wasm = wat2wasm(
        r"
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
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_add(
                Register::from_i16(2),
                Register::from_i16(0),
                Register::from_i16(1),
            ),
            Instruction::copy(Register::from_i16(0), Register::from_i16(2)),
            Instruction::return_reg(Register::from_i16(0)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn local_set_chain() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (param i32) (result i32)
                (local.set 0 (i32.const 10))
                (local.set 1 (local.get 0))
                (local.get 1)
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn local_tee_chain() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (param i32) (result i32)
                (local.tee 0 (i32.const 10))
                (local.tee 1)
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_result_1() {
    let wasm = wat2wasm(
        r#"
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
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(3), Register::from_i16(0)),
            Instruction::i32_add(
                Register::from_i16(0),
                Register::from_i16(0),
                Register::from_i16(1),
            ),
            Instruction::return_reg(Register::from_i16(3)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_result_2() {
    let wasm = wat2wasm(
        r#"
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
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(3), Register::from_i16(0)),
            Instruction::i32_add(
                Register::from_i16(0),
                Register::from_i16(0),
                Register::from_i16(1),
            ),
            Instruction::return_reg2(3, 3),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_0() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (local.set 0 (i32.const 10))
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::return_reg(Register::from_i16(1)),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_1() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32 i32)
                (local.get 0)
                (local.get 0)
                (local.set 0 (i32.const 10))
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::return_reg2(1, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_2() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.set 0 (i32.const 10))
                (local.get 0)
                (local.set 0 (i32.const 20))
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(3), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 20_i32),
            Instruction::return_reg2(3, 2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_3() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32) (result i32)
                (local.get 0)
                (local.set 0 (i32.const 10))
                (drop)
                (local.get 0)
                (local.set 0 (i32.const 20))
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 20_i32),
            Instruction::return_reg(1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_4() {
    let wasm = wat2wasm(
        r#"
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
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 10_i32),
            Instruction::copy(Register::from_i16(1), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 20_i32),
            Instruction::return_reg2(1, 1),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_5() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.set 2 (i32.const 11))
                (local.set 1 (i32.const 22))
                (local.set 0 (i32.const 33))
            )
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(5), Register::from_i16(2)),
            Instruction::copy_imm32(Register::from_i16(2), 11_i32),
            Instruction::copy(Register::from_i16(4), Register::from_i16(1)),
            Instruction::copy_imm32(Register::from_i16(1), 22_i32),
            Instruction::copy(Register::from_i16(3), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 33_i32),
            Instruction::return_reg3(3, 4, 5),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn preserve_multiple_6() {
    let wasm = wat2wasm(
        r#"
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
        )"#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(5), Register::from_i16(2)),
            Instruction::copy_imm32(Register::from_i16(2), 11_i32),
            Instruction::copy(Register::from_i16(4), Register::from_i16(0)),
            Instruction::copy_imm32(Register::from_i16(0), 22_i32),
            Instruction::copy(Register::from_i16(3), Register::from_i16(1)),
            Instruction::copy_imm32(Register::from_i16(1), 33_i32),
            Instruction::copy(Register::from_i16(5), Register::from_i16(1)),
            Instruction::copy_imm32(Register::from_i16(1), 44_i32),
            Instruction::return_reg3(4, 3, 5),
        ])
        .run()
}
