use super::*;
use crate::ir::index::{FuncType, Global, Table};

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
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect_0(FuncType::from(0)),
            Instruction::call_indirect_params(Local::from(0), Table::from(0)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn no_params_imm16() {
    fn test_with(index: u64) {
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
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect_0_imm16(FuncType::from(0)),
                Instruction::call_indirect_params_imm16(u64imm16(index), Table::from(0)),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u64::from(u16::MAX) - 1);
    test_with(u64::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_local_param_reg() {
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
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(FuncType::from(0)),
            Instruction::call_indirect_params(Local::from(0), Table::from(0)),
            Instruction::local(1),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_local_param_imm16() {
    fn test_with(index: u64) {
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
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect_imm16(FuncType::from(0)),
                Instruction::call_indirect_params_imm16(u64imm16(index), Table::from(0)),
                Instruction::local(1),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u64::from(u16::MAX) - 1);
    test_with(u64::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_local_param_imm() {
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
        TranslationTest::new(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect(FuncType::from(0)),
                    Instruction::call_indirect_params(Local::from(-1), Table::from(0)),
                    Instruction::local(1),
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
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(FuncType::from(0)),
                Instruction::call_indirect_params(Local::from(0), Table::from(0)),
                Instruction::local(-1),
            ])
            .consts([10_i32]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_imm_param_imm16() {
    fn test_with(index: u64) {
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
        TranslationTest::new(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect_imm16(FuncType::from(0)),
                    Instruction::call_indirect_params_imm16(u64imm16(index), Table::from(0)),
                    Instruction::local(-1),
                ])
                .consts([10_i32]),
            )
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u64::from(u16::MAX) - 1);
    test_with(u64::from(u16::MAX));
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
        TranslationTest::new(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect(FuncType::from(0)),
                    Instruction::call_indirect_params(Local::from(-1), Table::from(0)),
                    Instruction::local(-2),
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
fn two_local_params_reg() {
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
    let elem_index = Local::from(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(FuncType::from(0)),
            Instruction::call_indirect_params(elem_index, Table::from(0)),
            Instruction::local2_ext(1, 2),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_local_params_local_lhs() {
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
    let elem_index = Local::from(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(FuncType::from(0)),
            Instruction::call_indirect_params(elem_index, Table::from(0)),
            Instruction::local2_ext(2, 1),
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
    let elem_index = Local::from(0);
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(FuncType::from(0)),
                Instruction::call_indirect_params(elem_index, Table::from(0)),
                Instruction::local2_ext(-1, -2),
            ])
            .consts([10_i32, 20_i32]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_local_params_imm16() {
    fn test_with(index: u64) {
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
        let elem_index = u64imm16(index);
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect_imm16(FuncType::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, Table::from(0)),
                Instruction::local2_ext(0, 1),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u64::from(u16::MAX) - 1);
    test_with(u64::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_local_params_lhs_imm16() {
    fn test_with(index: u64) {
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
        let elem_index = u64imm16(index);
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect_imm16(FuncType::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, Table::from(0)),
                Instruction::local2_ext(1, 0),
            ])
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u64::from(u16::MAX) - 1);
    test_with(u64::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_imm_params_imm16() {
    fn test_with(index: u64) {
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
        let elem_index = u64imm16(index);
        TranslationTest::new(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect_imm16(FuncType::from(0)),
                    Instruction::call_indirect_params_imm16(elem_index, Table::from(0)),
                    Instruction::local2_ext(-1, -2),
                ])
                .consts([10_i32, 20_i32]),
            )
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u64::from(u16::MAX) - 1);
    test_with(u64::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_local_params_reg() {
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
    let elem_index = Local::from(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(FuncType::from(0)),
            Instruction::call_indirect_params(elem_index, Table::from(0)),
            Instruction::local3_ext(1, 2, 3),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_local_params_local_lhs() {
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
    let elem_index = Local::from(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(FuncType::from(0)),
            Instruction::call_indirect_params(elem_index, Table::from(0)),
            Instruction::local3_ext(3, 2, 1),
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
    let elem_index = Local::from(0);
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(FuncType::from(0)),
                Instruction::call_indirect_params(elem_index, Table::from(0)),
                Instruction::local3_ext(-1, -2, -3),
            ])
            .consts([10_i32, 20, 30]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_imm_params_imm16() {
    fn test_with(index: u64) {
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
        let elem_index = u64imm16(index);
        TranslationTest::new(&wasm)
            .expect_func(
                ExpectedFunc::new([
                    Instruction::return_call_indirect_imm16(FuncType::from(0)),
                    Instruction::call_indirect_params_imm16(elem_index, Table::from(0)),
                    Instruction::local3_ext(-1, -2, -3),
                ])
                .consts([10_i32, 20, 30]),
            )
            .run();
    }

    test_with(0);
    test_with(1);
    test_with(u64::from(u16::MAX) - 1);
    test_with(u64::from(u16::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn params7_local_index_local() {
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
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(FuncType::from(0)),
            Instruction::call_indirect_params(Local::from(0), Table::from(0)),
            Instruction::local_list_ext(1, 2, 3),
            Instruction::local_list_ext(4, 5, 6),
            Instruction::local(7),
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
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(FuncType::from(0)),
                Instruction::call_indirect_params(Local::from(0), Table::from(0)),
                Instruction::local_list_ext(-1, -2, -3),
                Instruction::local_list_ext(-4, -5, -6),
                Instruction::local(-7),
            ])
            .consts([10_i32, 20, 30, 40, 50, 60, 70]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params8_local_index_local() {
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
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(FuncType::from(0)),
            Instruction::call_indirect_params(Local::from(0), Table::from(0)),
            Instruction::local_list_ext(1, 2, 3),
            Instruction::local_list_ext(4, 5, 6),
            Instruction::local2_ext(7, 8),
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
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(FuncType::from(0)),
                Instruction::call_indirect_params(Local::from(0), Table::from(0)),
                Instruction::local_list_ext(-1, -2, -3),
                Instruction::local_list_ext(-4, -5, -6),
                Instruction::local2_ext(-7, -8),
            ])
            .consts([10_i32, 20, 30, 40, 50, 60, 70, 80]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params9_local_index_local() {
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
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(FuncType::from(0)),
            Instruction::call_indirect_params(Local::from(0), Table::from(0)),
            Instruction::local_list_ext(1, 2, 3),
            Instruction::local_list_ext(4, 5, 6),
            Instruction::local3_ext(7, 8, 9),
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
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([
                Instruction::return_call_indirect(FuncType::from(0)),
                Instruction::call_indirect_params(Local::from(0), Table::from(0)),
                Instruction::local_list_ext(-1, -2, -3),
                Instruction::local_list_ext(-4, -5, -6),
                Instruction::local3_ext(-7, -8, -9),
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
                    (global.get $g) ;; index on dynamic local space
                )
            )
        )
        "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_imm32(0_i32)])
        .expect_func(
            ExpectedFunc::new([
                Instruction::global_get(Local::from(0), Global::from(0)),
                Instruction::return_call_indirect(FuncType::from(0)),
                Instruction::call_indirect_params(Local::from(0), Table::from(0)),
                Instruction::local2_ext(-1, -2),
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
                    (global.get $g1) ;; index on dynamic local space
                )
            )
        )
        "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_imm32(0_i32)])
        .expect_func(
            ExpectedFunc::new([
                Instruction::global_get(Local::from(0), Global::from(0)),
                Instruction::global_get(Local::from(1), Global::from(1)),
                Instruction::return_call_indirect(FuncType::from(0)),
                Instruction::call_indirect_params(Local::from(1), Table::from(0)),
                Instruction::local2_ext(0, -1),
            ])
            .consts([20_i32]),
        )
        .run();
}
