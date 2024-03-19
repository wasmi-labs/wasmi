use super::*;
use crate::engine::bytecode::{GlobalIdx, SignatureIdx, TableIdx};

#[test]
#[cfg_attr(miri, ignore)]
fn no_params_reg() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func))
            (func (param $index i32)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect_0(SignatureIdx::from(0)),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn no_params_imm16() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func))
                (func (param $index i32)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect_0(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(u32imm16(index), TableIdx::from(0)),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_reg_param_reg() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32) (result i32)))
            (func (param $index i32) (param i32) (result i32)
                (local.get 1)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::register(1),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_reg_param_imm16() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32) (result i32)))
                (func (param $index i32) (param i32) (result i32)
                    (local.get 1)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(u32imm16(index), TableIdx::from(0)),
                Instruction::register(1),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_reg_param_imm() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32) (result i32)))
                (func (param $index i32) (param i32) (result i32)
                    (local.get 1)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        TranslationTest::from_wat(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect(SignatureIdx::from(0)),
                    Instruction::call_indirect_params(Register::from_i16(-1), TableIdx::from(0)),
                    Instruction::register(1),
                ])
                .consts([index]),
            )
            .run();
    }

    test_with(u32::from(u16::MAX) + 1);
    test_with(u32::MAX - 1);
    test_with(u32::MAX);
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_imm_param_reg() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32) (result i32)))
            (func (param $index i32) (result i32)
                (i32.const 10)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
                Instruction::register(-1),
            ])
            .consts([10_i32]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_imm_param_imm16() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32) (result i32)))
                (func (param $index i32) (result i32)
                    (i32.const 10)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        TranslationTest::from_wat(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect(SignatureIdx::from(0)),
                    Instruction::call_indirect_params_imm16(u32imm16(index), TableIdx::from(0)),
                    Instruction::register(-1),
                ])
                .consts([10_i32]),
            )
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_imm_param_imm() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32) (result i32)))
                (func (param $index i32) (result i32)
                    (i32.const 10)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        TranslationTest::from_wat(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect(SignatureIdx::from(0)),
                    Instruction::call_indirect_params(Register::from_i16(-1), TableIdx::from(0)),
                    Instruction::register(-2),
                ])
                .consts([index as i32, 10]),
            )
            .run();
    }

    test_with(u32::from(u16::MAX) + 1);
    test_with(u32::MAX - 1);
    test_with(u32::MAX);
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_reg_params_reg() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32) (result i32 i32)))
            (func (param $index i32) (param i32 i32) (result i32 i32)
                (local.get 1)
                (local.get 2)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    let elem_index = Register::from_i16(0);
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::register2(1, 2),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_reg_params_reg_rev() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32) (result i32 i32)))
            (func (param $index i32) (param i32 i32) (result i32 i32)
                (local.get 2)
                (local.get 1)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    let elem_index = Register::from_i16(0);
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::register2(2, 1),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_imm_params_reg() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32) (result i32 i32)))
            (func (param $index i32) (result i32 i32)
                (i32.const 10)
                (i32.const 20)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    let elem_index = Register::from_i16(0);
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
                Instruction::register2(-1, -2),
            ])
            .consts([10_i32, 20_i32]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_reg_params_imm16() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32 i32) (result i32 i32)))
                (func (param i32 i32) (result i32 i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        let elem_index = u32imm16(index);
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                Instruction::register2(0, 1),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_reg_params_rev_imm16() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32 i32) (result i32 i32)))
                (func (param i32 i32) (result i32 i32)
                    (local.get 1)
                    (local.get 0)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        let elem_index = u32imm16(index);
        TranslationTest::from_wat(&wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                Instruction::register2(1, 0),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_imm_params_imm16() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32 i32) (result i32 i32)))
                (func (result i32 i32)
                    (i32.const 10)
                    (i32.const 20)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        let elem_index = u32imm16(index);
        TranslationTest::from_wat(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect(SignatureIdx::from(0)),
                    Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                    Instruction::register2(-1, -2),
                ])
                .consts([10_i32, 20_i32]),
            )
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_reg_params_reg() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32) (result i32 i32 i32)))
            (func (param $index i32) (param i32 i32 i32) (result i32 i32 i32)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    let elem_index = Register::from_i16(0);
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::register3(1, 2, 3),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_reg_params_reg_rev() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32) (result i32 i32 i32)))
            (func (param $index i32) (param i32 i32 i32) (result i32 i32 i32)
                (local.get 3)
                (local.get 2)
                (local.get 1)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    let elem_index = Register::from_i16(0);
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::register3(3, 2, 1),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_imm_params_reg() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32) (result i32 i32 i32)))
            (func (param $index i32) (result i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 30)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    let elem_index = Register::from_i16(0);
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
                Instruction::register3(-1, -2, -3),
            ])
            .consts([10_i32, 20, 30]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_imm_params_imm16() {
    fn test_with(index: u32) {
        let wasm = format!(
            r#"
            (module
                (import "" "table" (table $table 10 funcref))
                (type $type (func (param i32 i32 i32) (result i32 i32 i32)))
                (func (result i32 i32 i32)
                    (i32.const 10)
                    (i32.const 20)
                    (i32.const 30)
                    (i32.const {index})
                    (return_call_indirect (type $type))
                )
            )
        "#,
        );
        let elem_index = u32imm16(index);
        TranslationTest::from_wat(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect(SignatureIdx::from(0)),
                    Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                    Instruction::register3(-1, -2, -3),
                ])
                .consts([10_i32, 20, 30]),
            )
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u32::from(u16::MAX) - 1);
    test_with(u32::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn params7_reg_index_local() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32)))
            (func (param $index i32) (param i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::register_list(1, 2, 3),
            Instruction::register_list(4, 5, 6),
            Instruction::register(7),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params7_imm_index_local() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32)))
            (func (param $index i32) (result i32 i32 i32 i32 i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 30)
                (i32.const 40)
                (i32.const 50)
                (i32.const 60)
                (i32.const 70)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
                Instruction::register_list(-1, -2, -3),
                Instruction::register_list(-4, -5, -6),
                Instruction::register(-7),
            ])
            .consts([10_i32, 20, 30, 40, 50, 60, 70]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params8_reg_index_local() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)))
            (func (param $index i32) (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
                (local.get 8)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::register_list(1, 2, 3),
            Instruction::register_list(4, 5, 6),
            Instruction::register2(7, 8),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params8_imm_index_local() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)))
            (func (param $index i32) (result i32 i32 i32 i32 i32 i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 30)
                (i32.const 40)
                (i32.const 50)
                (i32.const 60)
                (i32.const 70)
                (i32.const 80)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
                Instruction::register_list(-1, -2, -3),
                Instruction::register_list(-4, -5, -6),
                Instruction::register2(-7, -8),
            ])
            .consts([10_i32, 20, 30, 40, 50, 60, 70, 80]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params9_reg_index_local() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)))
            (func (param $index i32) (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
                (local.get 8)
                (local.get 9)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::register_list(1, 2, 3),
            Instruction::register_list(4, 5, 6),
            Instruction::register3(7, 8, 9),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params9_imm_index_local() {
    let wasm = r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)))
            (func (param $index i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (i32.const 10)
                (i32.const 20)
                (i32.const 30)
                (i32.const 40)
                (i32.const 50)
                (i32.const 60)
                (i32.const 70)
                (i32.const 80)
                (i32.const 90)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#;
    TranslationTest::from_wat(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
                Instruction::register_list(-1, -2, -3),
                Instruction::register_list(-4, -5, -6),
                Instruction::register3(-7, -8, -9),
            ])
            .consts([10_i32, 20, 30, 40, 50, 60, 70, 80, 90]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_imm_params_dynamic_index() {
    let wasm = r#"
        (module
            (type $sig (func (param i32 i32) (result i32)))
            (table funcref (elem $f))
            (global $g (mut i32) (i32.const 0))
            (func $f (param i32 i32) (result i32)
                (i32.const 0)
            )
            (func (result i32)
                (return_call_indirect (type $sig)
                    (i32.const 10) (i32.const 20) ;; call params
                    (global.get $g) ;; index on dynamic register space
                )
            )
        )
        "#;
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([Instruction::return_imm32(0_i32)])
        .expect_func(
            ExpectedFunc::new([
                Instruction::global_get(Register::from_i16(0), GlobalIdx::from(0)),
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
                Instruction::register2(-1, -2),
            ])
            .consts([10_i32, 20_i32]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn regression_issue_768() {
    let wasm = r#"
        (module
            (type $sig (func (param i32 i32) (result i32)))
            (table funcref (elem $f))
            (global $g0 (mut i32) (i32.const 0))
            (global $g1 (mut i32) (i32.const 1))
            (func $f (param i32 i32) (result i32)
                (i32.const 0)
            )
            (func (result i32)
                (return_call_indirect (type $sig)
                    (global.get $g0) (i32.const 20) ;; call params
                    (global.get $g1) ;; index on dynamic register space
                )
            )
        )
        "#;
    TranslationTest::from_wat(wasm)
        .expect_func_instrs([Instruction::return_imm32(0_i32)])
        .expect_func(
            ExpectedFunc::new([
                Instruction::global_get(Register::from_i16(0), GlobalIdx::from(0)),
                Instruction::global_get(Register::from_i16(1), GlobalIdx::from(1)),
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params(Register::from_i16(1), TableIdx::from(0)),
                Instruction::register2(0, -1),
            ])
            .consts([20_i32]),
        )
        .run();
}
