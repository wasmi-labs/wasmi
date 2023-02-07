use super::*;
use core::borrow::Borrow;
use wasmi::core::ValueType;

fn assert_display(func_type: impl Borrow<FuncType>, expected: &str) {
    assert_eq!(
        format!("{}", DisplayFuncType::from(func_type.borrow())),
        String::from(expected),
    );
}

#[test]
fn display_0in_0out() {
    assert_display(FuncType::new([], []), "fn()");
}

#[test]
fn display_1in_0out() {
    assert_display(FuncType::new([ValueType::I32], []), "fn(i32)");
}

#[test]
fn display_0in_1out() {
    assert_display(FuncType::new([], [ValueType::I32]), "fn() -> i32");
}

#[test]
fn display_1in_1out() {
    assert_display(
        FuncType::new([ValueType::I32], [ValueType::I32]),
        "fn(i32) -> i32",
    );
}

#[test]
fn display_4in_0out() {
    assert_display(
        FuncType::new(
            [
                ValueType::I32,
                ValueType::I64,
                ValueType::F32,
                ValueType::F64,
            ],
            [],
        ),
        "fn(i32, i64, f32, f64)",
    );
}

#[test]
fn display_0in_4out() {
    assert_display(
        FuncType::new(
            [],
            [
                ValueType::I32,
                ValueType::I64,
                ValueType::F32,
                ValueType::F64,
            ],
        ),
        "fn() -> (i32, i64, f32, f64)",
    );
}

#[test]
fn display_4in_4out() {
    assert_display(
        FuncType::new(
            [
                ValueType::I32,
                ValueType::I64,
                ValueType::F32,
                ValueType::F64,
            ],
            [
                ValueType::I32,
                ValueType::I64,
                ValueType::F32,
                ValueType::F64,
            ],
        ),
        "fn(i32, i64, f32, f64) -> (i32, i64, f32, f64)",
    );
}
