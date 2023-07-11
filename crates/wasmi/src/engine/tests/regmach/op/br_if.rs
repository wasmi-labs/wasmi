use super::*;
use crate::{
    engine::{
        bytecode::BranchOffset,
        tests::regmach::{display_wasm::DisplayValueType, wasm_type::WasmType},
    },
    Value,
};
use core::fmt::Display;

#[test]
fn consteval_return() {
    fn test_for(condition: bool) {
        let condition = DisplayWasm::from(i32::from(condition));
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32)
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func([Instruction::Return])
            .run()
    }
    test_for(true);
    test_for(false);
}

#[test]
fn consteval_return_1() {
    fn test_for(condition: bool) {
        let expected = match condition {
            true => Register::from_u16(0),
            false => Register::from_u16(1),
        };
        let condition = DisplayWasm::from(i32::from(condition));
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32 i32) (result i32)
                    (local.get 0)
                    (i32.const {condition}) ;; br_if condition
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func([Instruction::return_reg(expected)])
            .run()
    }
    test_for(true);
    test_for(false);
}

#[test]
fn consteval_branch_always() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (block (param i32) (result i32)
                    (i32.const 1) ;; br_if condition: true
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::branch(BranchOffset::from(1)),
            Instruction::return_reg(Register::from_u16(2)),
        ])
        .run()
}

#[test]
fn consteval_branch_never() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (block (param i32) (result i32)
                    (i32.const 0) ;; br_if condition: false
                    (br_if 0)
                    (drop)
                    (local.get 1)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([Instruction::return_reg(Register::from_u16(1))])
        .run()
}

#[test]
fn return_if_results_0() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (local.get 0)
                (br_if 0)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::return_nez(Register::from_u16(0)),
            Instruction::Return,
        ])
        .run()
}

#[test]
fn return_if_results_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (br_if 0)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::return_nez_reg(Register::from_u16(1), Register::from_u16(0)),
            Instruction::return_reg(Register::from_u16(0)),
        ])
        .run()
}

#[test]
fn return_if_results_1_imm() {
    fn test_for<T>(returned_value: T)
    where
        T: WasmType,
        DisplayWasm<T>: Display,
    {
        let display_ty = DisplayValueType::from(<T as WasmType>::VALUE_TYPE);
        let display_value = DisplayWasm::from(returned_value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32) (result {display_ty})
                    ({display_ty}.const {display_value})
                    (local.get 0) ;; br_if condition
                    (br_if 0)
                )
            )",
        ));
        let cref = ConstRef::from_u32(0);
        TranslationTest::new(wasm)
            .expect_func([
                Instruction::return_nez_imm(Register::from_u16(0), cref),
                Instruction::return_imm(cref),
            ])
            .expect_const(cref, returned_value)
            .run()
    }

    test_for::<i64>(i64::from(i32::MIN) - 1);
    test_for::<i64>(i64::from(i32::MAX) + 1);
    test_for::<i64>(i64::MIN);
    test_for::<i64>(i64::MAX);

    test_for::<f64>(0.0);
    test_for::<f64>(1.0);
    test_for::<f64>(-1.0);
    test_for::<f64>(42.25);
    test_for::<f64>(f64::NAN);
}

#[test]
fn return_if_results_1_imm32() {
    fn test_for<T>(returned_value: T)
    where
        T: WasmType + Into<Const32>,
        DisplayWasm<T>: Display,
    {
        let display_ty = DisplayValueType::from(<T as WasmType>::VALUE_TYPE);
        let display_value = DisplayWasm::from(returned_value);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32) (result {display_ty})
                    ({display_ty}.const {display_value})
                    (local.get 0) ;; br_if condition
                    (br_if 0)
                )
            )",
        ));
        let const32: Const32 = returned_value.into();
        TranslationTest::new(wasm)
            .expect_func([
                Instruction::return_nez_imm32(Register::from_u16(0), const32),
                Instruction::return_imm32(const32),
            ])
            .run()
    }
    test_for::<i32>(0);
    test_for::<i32>(1);
    test_for::<i32>(-1);
    test_for::<i32>(42);
    test_for::<f32>(0.0);
    test_for::<f32>(5.5);
    test_for::<f32>(42.25);
    test_for::<f32>(f32::NAN);
}

#[test]
fn return_if_results_1_i64imm32() {
    fn test_for(returned_value: i32) {
        let display_value = DisplayWasm::from(i64::from(returned_value));
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param i32) (result i64)
                    (i64.const {display_value})
                    (local.get 0) ;; br_if condition
                    (br_if 0)
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func([
                Instruction::return_nez_i64imm32(Register::from_u16(0), returned_value),
                Instruction::return_i64imm32(returned_value),
            ])
            .run()
    }

    test_for(0);
    test_for(1);
    test_for(-1);
    test_for(i32::MIN);
    test_for(i32::MAX);
}

#[test]
fn branch_if_results_0() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32)
                (local.get 0)
                (block (param i32)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_nez(Register::from_u16(0), BranchOffset::from(1)),
            Instruction::Return,
        ])
        .run()
}

#[test]
fn branch_if_results_1() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (block (param i32 i32) (result i32)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_eqz(Register::from_u16(1), BranchOffset::from(3)),
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy(Register::from_u16(2), Register::from_u16(0)),
            Instruction::return_reg(Register::from_u16(2)),
        ])
        .run()
}

/// Variant of the [`branch_if_results_1`] test where it is possible to avoid copies.
///
/// # Note
///
/// Copy elision is possible since the registers on top of the stack
/// are the same as the expected block results when translating the Wasm `br_if`.
/// We achieve this by using expressions as inputs such as `(i32.clz (local.get 0))`.
#[test]
fn branch_if_results_1_avoid_copy() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (i32.clz (local.get 0))
                (i32.ctz (local.get 1))
                (block (param i32 i32) (result i32)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::i32_clz(Register::from_u16(2), Register::from_u16(0)),
            Instruction::i32_ctz(Register::from_u16(3), Register::from_u16(1)),
            Instruction::branch_nez(Register::from_u16(3), BranchOffset::from(1)),
            Instruction::return_reg(Register::from_u16(2)),
        ])
        .run()
}

#[test]
fn branch_if_results_2() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32 i32) (result i32)
                (local.get 0)
                (local.get 1)
                (local.get 2)
                (block (param i32 i32 i32) (result i32 i32)
                    (br_if 0)
                )
                (i32.add)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::branch_eqz(Register::from_u16(2), BranchOffset::from(4)),
            Instruction::copy(Register::from_u16(3), Register::from_u16(0)),
            Instruction::copy(Register::from_u16(4), Register::from_u16(1)),
            Instruction::branch(BranchOffset::from(3)),
            Instruction::copy(Register::from_u16(3), Register::from_u16(0)),
            Instruction::copy(Register::from_u16(4), Register::from_u16(1)),
            Instruction::i32_add(
                Register::from_u16(3),
                Register::from_u16(3),
                Register::from_u16(4),
            ),
            Instruction::return_reg(Register::from_u16(3)),
        ])
        .run()
}

/// Variant of the [`branch_if_results_2`] test where it is possible to avoid copies.
///
/// # Note
///
/// Read the docs on [`branch_if_results_1_avoid_copy`] test for more information.
#[test]
fn branch_if_results_2_avoid_copy() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32 i32) (result i32)
                (i32.clz (local.get 0))
                (i32.ctz (local.get 1))
                (local.get 2)
                (block (param i32 i32 i32) (result i32 i32)
                    (br_if 0)
                )
                (i32.add)
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::i32_clz(Register::from_u16(3), Register::from_u16(0)),
            Instruction::i32_ctz(Register::from_u16(4), Register::from_u16(1)),
            Instruction::branch_nez(Register::from_u16(2), BranchOffset::from(1)),
            Instruction::i32_add(
                Register::from_u16(3),
                Register::from_u16(3),
                Register::from_u16(4),
            ),
            Instruction::return_reg(Register::from_u16(3)),
        ])
        .run()
}
