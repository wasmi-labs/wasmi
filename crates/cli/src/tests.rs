use super::*;
use core::borrow::Borrow;
use wasmi::ValType;

fn assert_display(func_type: impl Borrow<FuncType>, expected: &str) {
    assert_eq!(
        format!("{}", DisplayFuncType::from(func_type.borrow())),
        String::from(expected),
    );
}

macro_rules! func_ty {
    ($params:expr, $results:expr $(,)?) => {{ FuncType::new($params, $results) }};
}

#[test]
fn display_0in_0out() {
    assert_display(func_ty!([], []), "fn()");
}

#[test]
fn display_1in_0out() {
    assert_display(func_ty!([ValType::I32], []), "fn(i32)");
}

#[test]
fn display_0in_1out() {
    assert_display(func_ty!([], [ValType::I32]), "fn() -> i32");
}

#[test]
fn display_1in_1out() {
    assert_display(func_ty!([ValType::I32], [ValType::I32]), "fn(i32) -> i32");
}

#[test]
fn display_4in_0out() {
    assert_display(
        func_ty!([ValType::I32, ValType::I64, ValType::F32, ValType::F64], []),
        "fn(i32, i64, f32, f64)",
    );
}

#[test]
fn display_0in_4out() {
    assert_display(
        func_ty!([], [ValType::I32, ValType::I64, ValType::F32, ValType::F64]),
        "fn() -> (i32, i64, f32, f64)",
    );
}

#[test]
fn display_4in_4out() {
    assert_display(
        func_ty!(
            [ValType::I32, ValType::I64, ValType::F32, ValType::F64],
            [ValType::I32, ValType::I64, ValType::F32, ValType::F64],
        ),
        "fn(i32, i64, f32, f64) -> (i32, i64, f32, f64)",
    );
}
