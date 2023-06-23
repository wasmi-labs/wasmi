use super::*;
use crate::{
    core::ValueType,
    engine::tests::regmach::display_wasm::{DisplayValue, DisplayValueType},
    ExternRef,
    FuncRef,
    Value,
};
use wasmi_core::{UntypedValue, F32, F64};

/// Returns the `return` instruction to return a single [`Value`].
///
/// Also adds an `expect_const` case to [`TranslationTest`] if necessary.
fn return_for_value(testcase: &mut TranslationTest, value: Value) -> Instruction {
    fn return_cref(testcase: &mut TranslationTest, value: UntypedValue) -> Instruction {
        testcase.expect_const(ConstRef::from_u32(0), value);
        Instruction::return_cref(0)
    }
    match value {
        Value::I32(value) => Instruction::return_imm32(value),
        Value::I64(value) => {
            if let Ok(value) = i32::try_from(value) {
                return Instruction::return_i64imm32(value);
            }
            return_cref(testcase, value.into())
        }
        Value::F32(value) => Instruction::return_imm32(value),
        Value::F64(_value) => return_cref(testcase, value.into()),
        Value::FuncRef(_value) => return_cref(testcase, value.into()),
        Value::ExternRef(_value) => return_cref(testcase, value.into()),
    }
}

/// Returns the instruction parameter to hold a single [`Value`].
///
/// Also adds an `expect_const` case to [`TranslationTest`] if necessary.
fn param_for_value(
    testcase: &mut TranslationTest,
    value: Value,
    allow_i64_opt: bool,
) -> Instruction {
    fn cref(testcase: &mut TranslationTest, value: UntypedValue) -> Instruction {
        let cref = ConstRef::from_u32(0);
        testcase.expect_const(cref, value);
        Instruction::ConstRef(cref)
    }
    match value {
        Value::I32(value) => Instruction::Const32(value.into()),
        Value::I64(value) => {
            if allow_i64_opt {
                if let Ok(value) = i32::try_from(value) {
                    return Instruction::Const32(value.into());
                }
            }
            cref(testcase, value.into())
        }
        Value::F32(value) => Instruction::Const32(value.into()),
        Value::F64(_value) => cref(testcase, value.into()),
        Value::FuncRef(_value) => cref(testcase, value.into()),
        Value::ExternRef(_value) => cref(testcase, value.into()),
    }
}

fn test_reg(result_ty: ValueType) {
    let display_ty = DisplayValueType::from(result_ty);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $condition i32) (param $lhs {display_ty}) (param $rhs {display_ty}) (result {display_ty})
                local.get $lhs
                local.get $rhs
                local.get $condition
                select (result {display_ty})
            )
        )
    "#,
    ));
    let condition = Register::from_u16(0);
    let lhs = Register::from_u16(1);
    let rhs = Register::from_u16(2);
    let result = Register::from_u16(3);
    TranslationTest::new(wasm)
        .expect_func([
            Instruction::select(result, condition, lhs),
            Instruction::Register(rhs),
            Instruction::return_reg(result),
        ])
        .run();
}

#[test]
fn reg() {
    test_reg(ValueType::I32);
    test_reg(ValueType::I64);
    test_reg(ValueType::F32);
    test_reg(ValueType::F64);
    test_reg(ValueType::FuncRef);
    test_reg(ValueType::ExternRef);
}

fn test_same_reg(result_ty: ValueType) {
    let display_ty = DisplayValueType::from(result_ty);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $condition i32) (param $input {display_ty}) (result {display_ty})
                local.get $input
                local.get $input
                local.get $condition
                select (result {display_ty})
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([Instruction::return_reg(Register::from_u16(1))])
        .run();
}

#[test]
fn same_reg() {
    test_same_reg(ValueType::I32);
    test_same_reg(ValueType::I64);
    test_same_reg(ValueType::F32);
    test_same_reg(ValueType::F64);
    test_same_reg(ValueType::FuncRef);
    test_same_reg(ValueType::ExternRef);
}

fn test_same_imm(input: Value) {
    let display_ty = DisplayValueType::from(input.ty());
    let display_input = DisplayValue::from(input.clone());
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $condition i32) (param $input {display_ty}) (result {display_ty})
                {display_ty}.const {display_input}
                {display_ty}.const {display_input}
                local.get $condition
                select (result {display_ty})
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    let return_instr = return_for_value(&mut testcase, input);
    testcase.expect_func([return_instr]).run();
}

#[test]
fn same_imm() {
    test_same_imm(Value::I32(42));
    test_same_imm(Value::I64(42));
    test_same_imm(Value::F32(F32::from(42.5)));
    test_same_imm(Value::F64(F64::from(42.5)));
}

fn test_reg_imm(rhs: Value) {
    let display_ty = DisplayValueType::from(rhs.ty());
    let display_rhs = DisplayValue::from(rhs.clone());
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $condition i32) (param $lhs {display_ty}) (result {display_ty})
                local.get $lhs
                {display_ty}.const {display_rhs}
                local.get $condition
                select (result {display_ty})
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    let result = Register::from_u16(2);
    let condition = Register::from_u16(0);
    let lhs = Register::from_u16(1);
    let select_instr = match rhs.ty() {
        ValueType::I32 | ValueType::F32 => Instruction::select_imm32_rhs(result, condition, lhs),
        ValueType::I64 | ValueType::F64 | ValueType::FuncRef | ValueType::ExternRef => {
            Instruction::select_imm_rhs(result, condition, lhs)
        }
    };
    let param_instr = param_for_value(&mut testcase, rhs, false);
    testcase
        .expect_func([select_instr, param_instr, Instruction::return_reg(result)])
        .run();
}

#[test]
fn reg_imm() {
    test_reg_imm(Value::I32(42));
    test_reg_imm(Value::I64(42));
    test_reg_imm(Value::F32(F32::from(42.5)));
    test_reg_imm(Value::F64(F64::from(42.5)));
}

fn test_imm_reg(lhs: Value) {
    let display_ty = DisplayValueType::from(lhs.ty());
    let display_lhs = DisplayValue::from(lhs.clone());
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $condition i32) (param $rhs {display_ty}) (result {display_ty})
                {display_ty}.const {display_lhs}
                local.get $rhs
                local.get $condition
                select (result {display_ty})
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    let result = Register::from_u16(2);
    let condition = Register::from_u16(0);
    let rhs = Register::from_u16(1);
    let select_instr = match lhs.ty() {
        ValueType::I32 | ValueType::F32 => Instruction::select_imm32_lhs(result, condition, rhs),
        ValueType::I64 | ValueType::F64 | ValueType::FuncRef | ValueType::ExternRef => {
            Instruction::select_imm_lhs(result, condition, rhs)
        }
    };
    let param_instr = param_for_value(&mut testcase, lhs, false);
    testcase
        .expect_func([select_instr, param_instr, Instruction::return_reg(result)])
        .run();
}

#[test]
fn imm_reg() {
    test_imm_reg(Value::I32(42));
    test_imm_reg(Value::I64(42));
    test_imm_reg(Value::F32(F32::from(42.5)));
    test_imm_reg(Value::F64(F64::from(42.5)));
}

fn test_imm(lhs: Value, rhs: Value) {
    assert_eq!(lhs.ty(), rhs.ty());
    assert_ne!(
        UntypedValue::from(lhs.clone()),
        UntypedValue::from(rhs.clone()),
        "testcase required both `lhs` and `rhs` to not be equal"
    );
    let display_ty = DisplayValueType::from(lhs.ty());
    let display_lhs = DisplayValue::from(lhs.clone());
    let display_rhs = DisplayValue::from(rhs.clone());
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $condition i32) (result {display_ty})
                {display_ty}.const {display_lhs}
                {display_ty}.const {display_rhs}
                local.get $condition
                select (result {display_ty})
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    let result = Register::from_u16(1);
    let condition = Register::from_u16(0);
    match (lhs, rhs) {
        (Value::I32(lhs), Value::I32(rhs)) => {
            testcase.expect_func([
                Instruction::select_imm32(result, Const32::from(lhs)),
                Instruction::select_imm32(condition, Const32::from(rhs)),
                Instruction::return_reg(result),
            ]);
        }
        (Value::I64(lhs), Value::I64(rhs)) => {
            testcase.expect_func([
                Instruction::select_imm(result, ConstRef::from_u32(0)),
                Instruction::select_imm(condition, ConstRef::from_u32(1)),
                Instruction::return_reg(result),
            ]);
            testcase.expect_const(ConstRef::from_u32(0), lhs);
            testcase.expect_const(ConstRef::from_u32(1), rhs);
        }
        (Value::F32(lhs), Value::F32(rhs)) => {
            testcase.expect_func([
                Instruction::select_imm32(result, Const32::from(lhs)),
                Instruction::select_imm32(condition, Const32::from(rhs)),
                Instruction::return_reg(result),
            ]);
        }
        (Value::F64(lhs), Value::F64(rhs)) => {
            testcase.expect_func([
                Instruction::select_imm(result, ConstRef::from_u32(0)),
                Instruction::select_imm(condition, ConstRef::from_u32(1)),
                Instruction::return_reg(result),
            ]);
            testcase.expect_const(ConstRef::from_u32(0), lhs);
            testcase.expect_const(ConstRef::from_u32(1), rhs);
        }
        _ => unreachable!(),
    };
    testcase.run();
}

#[test]
fn imm() {
    test_imm(Value::I32(42), Value::I32(5));
    test_imm(Value::I64(42), Value::I64(5));
    test_imm(Value::F32(F32::from(42.5)), Value::F32(F32::from(5.0)));
    test_imm(Value::F64(F64::from(42.5)), Value::F64(F64::from(5.0)));
}

fn test_const_condition_reg(condition: bool, result_ty: ValueType) {
    let display_ty = DisplayValueType::from(result_ty);
    let condition_i32 = i32::from(condition);
    let lhs = Register::from_u16(0);
    let rhs = Register::from_u16(1);
    let picked_reg = if condition { lhs } else { rhs };
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $lhs {display_ty}) (param $rhs {display_ty}) (result {display_ty})
                local.get $lhs
                local.get $rhs
                i32.const {condition_i32}
                select (result {display_ty})
            )
        )
    "#,
    ));
    TranslationTest::new(wasm)
        .expect_func([Instruction::return_reg(picked_reg)])
        .run();
}

#[test]
fn const_condition_reg() {
    fn test_with(condition: bool) {
        test_const_condition_reg(condition, ValueType::I32);
        test_const_condition_reg(condition, ValueType::I64);
        test_const_condition_reg(condition, ValueType::F32);
        test_const_condition_reg(condition, ValueType::F64);
        test_const_condition_reg(condition, ValueType::FuncRef);
        test_const_condition_reg(condition, ValueType::ExternRef);
    }
    test_with(true);
    test_with(false);
}

fn test_const_condition_reg_imm(condition: bool, rhs: Value) {
    let display_ty = DisplayValueType::from(rhs.ty());
    let display_rhs = DisplayValue::from(rhs.clone());
    let condition_i32 = i32::from(condition);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $lhs {display_ty}) (result {display_ty})
                local.get $lhs
                {display_ty}.const {display_rhs}
                i32.const {condition_i32}
                select (result {display_ty})
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    let picked_instr = if condition {
        Instruction::return_reg(Register::from_u16(0))
    } else {
        return_for_value(&mut testcase, rhs)
    };
    testcase.expect_func([picked_instr]).run();
}

#[test]
fn const_condition_reg_imm() {
    fn test_with(condition: bool) {
        test_const_condition_reg_imm(condition, Value::I32(42));
        test_const_condition_reg_imm(condition, Value::I64(42));
        test_const_condition_reg_imm(condition, Value::F32(F32::from(42.5)));
        test_const_condition_reg_imm(condition, Value::F64(F64::from(42.5)));
    }
    test_with(true);
    test_with(false);
}

fn test_const_condition_imm_reg(condition: bool, lhs: Value) {
    let display_ty = DisplayValueType::from(lhs.ty());
    let display_lhs = DisplayValue::from(lhs.clone());
    let condition_i32 = i32::from(condition);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (param $rhs {display_ty}) (result {display_ty})
                {display_ty}.const {display_lhs}
                local.get $rhs
                i32.const {condition_i32}
                select (result {display_ty})
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    let picked_instr = if !condition {
        Instruction::return_reg(Register::from_u16(0))
    } else {
        return_for_value(&mut testcase, lhs)
    };
    testcase.expect_func([picked_instr]).run();
}

#[test]
fn const_condition_imm_reg() {
    fn test_with(condition: bool) {
        test_const_condition_imm_reg(condition, Value::I32(42));
        test_const_condition_imm_reg(condition, Value::I64(42));
        test_const_condition_imm_reg(condition, Value::F32(F32::from(42.5)));
        test_const_condition_imm_reg(condition, Value::F64(F64::from(42.5)));
    }
    test_with(true);
    test_with(false);
}

fn test_const_condition_imm(condition: bool, lhs: Value, rhs: Value) {
    assert_eq!(lhs.ty(), rhs.ty());
    let display_ty = DisplayValueType::from(lhs.ty());
    let display_lhs = DisplayValue::from(lhs.clone());
    let display_rhs = DisplayValue::from(rhs.clone());
    let condition_i32 = i32::from(condition);
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (memory 1)
            (func (result {display_ty})
                {display_ty}.const {display_lhs}
                {display_ty}.const {display_rhs}
                i32.const {condition_i32}
                select (result {display_ty})
            )
        )
    "#,
    ));
    let mut testcase = TranslationTest::new(wasm);
    let picked_instr = if condition {
        return_for_value(&mut testcase, lhs)
    } else {
        return_for_value(&mut testcase, rhs)
    };
    testcase.expect_func([picked_instr]).run();
}

#[test]
fn const_condition_imm() {
    fn test_with(condition: bool) {
        test_const_condition_imm(condition, Value::I32(42), Value::I32(5));
        test_const_condition_imm(condition, Value::I64(42), Value::I64(5));
        test_const_condition_imm(
            condition,
            Value::F32(F32::from(42.5)),
            Value::F32(F32::from(5.0)),
        );
        test_const_condition_imm(
            condition,
            Value::F64(F64::from(42.5)),
            Value::F64(F64::from(5.0)),
        );
    }
    test_with(true);
    test_with(false);
}
