use super::*;
use crate::{engine::EngineFunc, ir::LocalSpan};

#[test]
#[cfg_attr(miri, ignore)]
fn no_params() {
    let wasm = r#"
        (module
            (func $f)
            (func
                (call $f)
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::Return])
        .expect_func_instrs([
            Instruction::call_internal_0(LocalSpan::new(Local::from(0)), EngineFunc::from_u32(0)),
            Instruction::Return,
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_param_reg() {
    let wasm = r#"
        (module
            (func $f (param i32) (result i32)
                (local.get 0)
            )
            (func (param i32) (result i32)
                (call $f (local.get 0))
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(Local::from(0))])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(1)), EngineFunc::from_u32(0)),
            Instruction::local(0),
            Instruction::return_reg(Local::from(1)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_param_imm() {
    let wasm = r#"
        (module
            (func $f (param i32) (result i32)
                (local.get 0)
            )
            (func (result i32)
                (call $f (i32.const 10))
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_reg(0)])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(LocalSpan::new(Local::from(0)), EngineFunc::from_u32(0)),
                Instruction::local(-1),
                Instruction::return_reg(0),
            ])
            .consts([10_i32]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_reg() {
    let wasm = r#"
        (module
            (func $f (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
            )
            (func (param i32 i32) (result i32 i32)
                (call $f (local.get 0) (local.get 1))
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_loc2_ext(0, 1)])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(2)), EngineFunc::from_u32(0)),
            Instruction::local2_ext(0, 1),
            Instruction::return_loc2_ext(2, 3),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_local_lhs() {
    let wasm = r#"
        (module
            (func $f (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
            )
            (func (param i32 i32) (result i32 i32)
                (call $f (local.get 1) (local.get 0))
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_loc2_ext(0, 1)])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(2)), EngineFunc::from_u32(0)),
            Instruction::local2_ext(1, 0),
            Instruction::return_loc2_ext(2, 3),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_params_imm() {
    let wasm = r#"
        (module
            (func $f (param i32 i32) (result i32 i32)
                (local.get 0)
                (local.get 1)
            )
            (func (result i32 i32)
                (call $f (i32.const 10) (i32.const 20))
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_loc2_ext(0, 1)])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(LocalSpan::new(Local::from(0)), EngineFunc::from_u32(0)),
                Instruction::local2_ext(-1, -2),
                Instruction::return_loc2_ext(0, 1),
            ])
            .consts([10_i32, 20]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_reg() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
            )
            (func (param i32 i32 i32) (result i32 i32 i32)
                (call $f (local.get 0) (local.get 1) (local.get 2))
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_loc3_ext(0, 1, 2)])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(3)), EngineFunc::from_u32(0)),
            Instruction::local3_ext(0, 1, 2),
            Instruction::return_loc3_ext(3, 4, 5),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_local_lhs() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
            )
            (func (param i32 i32 i32) (result i32 i32 i32)
                (call $f (local.get 2) (local.get 1) (local.get 0))
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_loc3_ext(0, 1, 2)])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(3)), EngineFunc::from_u32(0)),
            Instruction::local3_ext(2, 1, 0),
            Instruction::return_loc3_ext(3, 4, 5),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn three_params_imm() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32) (result i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
            )
            (func (result i32 i32 i32)
                (call $f (i32.const 10) (i32.const 20) (i32.const 30))
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_loc3_ext(0, 1, 2)])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(LocalSpan::new(Local::from(0)), EngineFunc::from_u32(0)),
                Instruction::local3_ext(-1, -2, -3),
                Instruction::return_loc3_ext(0, 1, 2),
            ])
            .consts([10_i32, 20, 30]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params7_reg() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
            )
            (func (param i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (local.get 0)
                    (local.get 1)
                    (local.get 2)
                    (local.get 3)
                    (local.get 4)
                    (local.get 5)
                    (local.get 6)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 7))])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(7)), EngineFunc::from_u32(0)),
            Instruction::local_list_ext(0, 1, 2),
            Instruction::local_list_ext(3, 4, 5),
            Instruction::local(6),
            Instruction::return_span(bspan(7, 7)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params7_local_lhs() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
            )
            (func (param i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (local.get 6)
                    (local.get 5)
                    (local.get 4)
                    (local.get 3)
                    (local.get 2)
                    (local.get 1)
                    (local.get 0)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 7))])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(7)), EngineFunc::from_u32(0)),
            Instruction::local_list_ext(6, 5, 4),
            Instruction::local_list_ext(3, 2, 1),
            Instruction::local(0),
            Instruction::return_span(bspan(7, 7)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params7_imm() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
            )
            (func (result i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (i32.const 10)
                    (i32.const 20)
                    (i32.const 30)
                    (i32.const 40)
                    (i32.const 50)
                    (i32.const 60)
                    (i32.const 70)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 7))])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(LocalSpan::new(Local::from(0)), EngineFunc::from_u32(0)),
                Instruction::local_list_ext(-1, -2, -3),
                Instruction::local_list_ext(-4, -5, -6),
                Instruction::local(-7),
                Instruction::return_span(bspan(0, 7)),
            ])
            .consts([10, 20, 30, 40, 50, 60, 70]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params8_reg() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
            )
            (func (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (local.get 0)
                    (local.get 1)
                    (local.get 2)
                    (local.get 3)
                    (local.get 4)
                    (local.get 5)
                    (local.get 6)
                    (local.get 7)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 8))])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(8)), EngineFunc::from_u32(0)),
            Instruction::local_list_ext(0, 1, 2),
            Instruction::local_list_ext(3, 4, 5),
            Instruction::local2_ext(6, 7),
            Instruction::return_span(bspan(8, 8)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params8_local_lhs() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
            )
            (func (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (local.get 7)
                    (local.get 6)
                    (local.get 5)
                    (local.get 4)
                    (local.get 3)
                    (local.get 2)
                    (local.get 1)
                    (local.get 0)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 8))])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(8)), EngineFunc::from_u32(0)),
            Instruction::local_list_ext(7, 6, 5),
            Instruction::local_list_ext(4, 3, 2),
            Instruction::local2_ext(1, 0),
            Instruction::return_span(bspan(8, 8)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params8_imm() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
            )
            (func (result i32 i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (i32.const 10)
                    (i32.const 20)
                    (i32.const 30)
                    (i32.const 40)
                    (i32.const 50)
                    (i32.const 60)
                    (i32.const 70)
                    (i32.const 80)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 8))])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(LocalSpan::new(Local::from(0)), EngineFunc::from_u32(0)),
                Instruction::local_list_ext(-1, -2, -3),
                Instruction::local_list_ext(-4, -5, -6),
                Instruction::local2_ext(-7, -8),
                Instruction::return_span(bspan(0, 8)),
            ])
            .consts([10, 20, 30, 40, 50, 60, 70, 80]),
        )
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params9_reg() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
                (local.get 8)
            )
            (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (local.get 0)
                    (local.get 1)
                    (local.get 2)
                    (local.get 3)
                    (local.get 4)
                    (local.get 5)
                    (local.get 6)
                    (local.get 7)
                    (local.get 8)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 9))])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(9)), EngineFunc::from_u32(0)),
            Instruction::local_list_ext(0, 1, 2),
            Instruction::local_list_ext(3, 4, 5),
            Instruction::local3_ext(6, 7, 8),
            Instruction::return_span(bspan(9, 9)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params9_local_lhs() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
                (local.get 8)
            )
            (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (local.get 8)
                    (local.get 7)
                    (local.get 6)
                    (local.get 5)
                    (local.get 4)
                    (local.get 3)
                    (local.get 2)
                    (local.get 1)
                    (local.get 0)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 9))])
        .expect_func_instrs([
            Instruction::call_internal(LocalSpan::new(Local::from(9)), EngineFunc::from_u32(0)),
            Instruction::local_list_ext(8, 7, 6),
            Instruction::local_list_ext(5, 4, 3),
            Instruction::local3_ext(2, 1, 0),
            Instruction::return_span(bspan(9, 9)),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn params9_imm() {
    let wasm = r#"
        (module
            (func $f (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (local.get 3)
                (local.get 4)
                (local.get 5)
                (local.get 6)
                (local.get 7)
                (local.get 8)
            )
            (func (result i32 i32 i32 i32 i32 i32 i32 i32 i32)
                (call $f
                    (i32.const 10)
                    (i32.const 20)
                    (i32.const 30)
                    (i32.const 40)
                    (i32.const 50)
                    (i32.const 60)
                    (i32.const 70)
                    (i32.const 80)
                    (i32.const 90)
                )
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_span(bspan(0, 9))])
        .expect_func(
            ExpectedFunc::new([
                Instruction::call_internal(LocalSpan::new(Local::from(0)), EngineFunc::from_u32(0)),
                Instruction::local_list_ext(-1, -2, -3),
                Instruction::local_list_ext(-4, -5, -6),
                Instruction::local3_ext(-7, -8, -9),
                Instruction::return_span(bspan(0, 9)),
            ])
            .consts([10, 20, 30, 40, 50, 60, 70, 80, 90]),
        )
        .run();
}
