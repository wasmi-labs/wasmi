use super::*;
use crate::engine::{
    bytecode::{GlobalIdx, SignatureIdx, TableIdx},
    CompiledFunc,
    RegisterSpan,
};

#[test]
#[cfg_attr(miri, ignore)]
fn no_params_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func))
            (func (param $index i32)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
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
        let wasm = wat2wasm(&format!(
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
        ));
        TranslationTest::new(wasm)
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
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32) (result i32)))
            (func (param $index i32) (param i32) (result i32)
                (local.get 1)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(1)).iter(1), 1),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_reg_param_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
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
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(u32imm16(index), TableIdx::from(0)),
                Instruction::call_params(RegisterSpan::new(Register::from_i16(1)).iter(1), 1),
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
fn one_imm_param_reg() {
    let wasm = wat2wasm(
        r#"
        (module
            (import "" "table" (table $table 10 funcref))
            (type $type (func (param i32) (result i32)))
            (func (param $index i32) (result i32)
                (i32.const 10)
                (local.get $index)
                (return_call_indirect (type $type))
            )
        )
    "#,
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(1), 10),
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::call_params(RegisterSpan::new(Register::from_i16(1)).iter(1), 1),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn one_imm_param_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
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
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::copy_imm32(Register::from_i16(1), 10),
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(u32imm16(index), TableIdx::from(0)),
                Instruction::call_params(RegisterSpan::new(Register::from_i16(1)).iter(1), 1),
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
fn two_reg_params_reg() {
    let wasm = wat2wasm(
        r#"
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
    "#,
    );
    let params = RegisterSpan::new(Register::from_i16(1)).iter(2);
    let elem_index = Register::from_i16(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::call_params(params, 2),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_reg_params_reg_rev() {
    let wasm = wat2wasm(
        r#"
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
    "#,
    );
    let params = RegisterSpan::new(Register::from_i16(3)).iter(2);
    let elem_index = Register::from_i16(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy(Register::from_i16(3), Register::from_i16(2)),
            Instruction::copy(Register::from_i16(4), Register::from_i16(1)),
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::call_params(params, 2),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_imm_params_reg() {
    let wasm = wat2wasm(
        r#"
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
    "#,
    );
    let params = RegisterSpan::new(Register::from_i16(1)).iter(2);
    let elem_index = Register::from_i16(0);
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::copy_imm32(Register::from_i16(1), 10),
            Instruction::copy_imm32(Register::from_i16(2), 20),
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(elem_index, TableIdx::from(0)),
            Instruction::call_params(params, 2),
        ])
        .run();
}

#[test]
#[cfg_attr(miri, ignore)]
fn two_reg_params_imm16() {
    fn test_with(index: u32) {
        let wasm = wat2wasm(&format!(
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
        ));
        let params = RegisterSpan::new(Register::from_i16(0)).iter(2);
        let elem_index = u32imm16(index);
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                Instruction::call_params(params, 2),
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
        let wasm = wat2wasm(&format!(
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
        ));
        let params = RegisterSpan::new(Register::from_i16(2)).iter(2);
        let elem_index = u32imm16(index);
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::copy(Register::from_i16(2), Register::from_i16(1)),
                Instruction::copy(Register::from_i16(3), Register::from_i16(0)),
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                Instruction::call_params(params, 2),
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
        let wasm = wat2wasm(&format!(
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
        ));
        let params = RegisterSpan::new(Register::from_i16(0)).iter(2);
        let elem_index = u32imm16(index);
        TranslationTest::new(wasm)
            .expect_func_instrs([
                Instruction::copy_imm32(Register::from_i16(0), 10),
                Instruction::copy_imm32(Register::from_i16(1), 20),
                Instruction::return_call_indirect(SignatureIdx::from(0)),
                Instruction::call_indirect_params_imm16(elem_index, TableIdx::from(0)),
                Instruction::call_params(params, 2),
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
fn test_imm_params_dynamic_index() {
    let wasm = wat2wasm(
        r#"
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
        "#
    );
    let params = RegisterSpan::new(Register::from_i16(1)).iter(2);
    TranslationTest::new(wasm)
        .expect_func_instrs([Instruction::return_imm32(0_i32)])
        .expect_func_instrs([
            Instruction::global_get(Register::from_i16(0), GlobalIdx::from(0)),
            Instruction::copy_imm32(Register::from_i16(1), 10),
            Instruction::copy_imm32(Register::from_i16(2), 20),
            Instruction::return_call_indirect(SignatureIdx::from(0)),
            Instruction::call_indirect_params(Register::from_i16(0), TableIdx::from(0)),
            Instruction::call_params(params, 1),
        ])
        .run();
}
