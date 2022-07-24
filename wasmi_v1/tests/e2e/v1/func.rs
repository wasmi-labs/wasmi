//! Tests for the `Func` type in `wasmi_v1`.

use assert_matches::assert_matches;
use wasmi_core::{Value, F32, F64};
use wasmi_v1::{errors::FuncError, Engine, Error, Func, Store};

fn test_setup() -> Store<()> {
    let engine = Engine::default();
    Store::new(&engine, ())
}

/// Asserts that `lhs` and `rhs` tuples are equal.
///
/// We need to define the following macro for the comparison of tuples with
/// 16 elements since `PartialEq` and `Debug` do not seem to be implemented
/// for tuples of that size.
macro_rules! assert_eq_tuple {
    ( $lhs:ident, $rhs:ident; $($n:tt),* $(,)? ) => {
        $(
            assert_eq!($lhs.$n, $rhs.$n);
        )*
    }
}

fn setup_add2() -> (Store<()>, Func) {
    let mut store = test_setup();
    // This host function represents a simple binary addition.
    let add2 = Func::wrap(&mut store, |lhs: i32, rhs: i32| lhs + rhs);
    (store, add2)
}

#[test]
fn dynamic_add2_works() {
    let (mut store, add2) = setup_add2();
    let result_add2 = {
        let mut result = [Value::I32(0)];
        add2.call(&mut store, &[Value::I32(1), Value::I32(2)], &mut result)
            .unwrap();
        result[0]
    };
    assert_eq!(result_add2, Value::I32(3));
}

#[test]
fn static_add2_works() {
    let (mut store, add2) = setup_add2();
    let typed_add2 = add2.typed::<(i32, i32), i32, _>(&mut store).unwrap();
    let result = typed_add2.call(&mut store, (1, 2)).unwrap();
    assert_eq!(result, 3);
}

fn setup_add3() -> (Store<()>, Func) {
    let mut store = test_setup();
    // This host function performance a three-way addition.
    let add3 = Func::wrap(&mut store, |v0: i32, v1: i32, v2: i32| v0 + v1 + v2);
    (store, add3)
}

#[test]
fn dynamic_add3_works() {
    let (mut store, add3) = setup_add3();
    let result_add3 = {
        let mut result = [Value::I32(0)];
        add3.call(
            &mut store,
            &[Value::I32(1), Value::I32(2), Value::I32(3)],
            &mut result,
        )
        .unwrap();
        result[0]
    };
    assert_eq!(result_add3, Value::I32(6));
}

#[test]
fn static_add3_works() {
    let (mut store, add3) = setup_add3();
    let typed_add3 = add3.typed::<(i32, i32, i32), i32, _>(&mut store).unwrap();
    let result = typed_add3.call(&mut store, (1, 2, 3)).unwrap();
    assert_eq!(result, 6);
}

fn setup_duplicate() -> (Store<()>, Func) {
    let mut store = test_setup();
    // This host function takes one `i32` argument and returns it twice.
    let duplicate = Func::wrap(&mut store, |value: i32| (value, value));
    (store, duplicate)
}

#[test]
fn dynamic_duplicate_works() {
    let (mut store, duplicate) = setup_duplicate();
    let result_duplicate = {
        let mut result = [Value::I32(0), Value::I32(0)];
        duplicate
            .call(&mut store, &[Value::I32(10)], &mut result)
            .unwrap();
        (result[0], result[1])
    };
    assert_eq!(result_duplicate, (Value::I32(10), Value::I32(10)));
}

#[test]
fn static_duplicate_works() {
    let (mut store, duplicate) = setup_duplicate();
    let typed_duplicate = duplicate.typed::<i32, (i32, i32), _>(&mut store).unwrap();
    let result = typed_duplicate.call(&mut store, 10).unwrap();
    assert_eq!(result, (10, 10));
}

fn setup_many_params() -> (Store<()>, Func) {
    let mut store = test_setup();
    // Function taking 16 arguments (maximum) and doing nothing.
    let func = Func::wrap(
        &mut store,
        |_0: i32,
         _1: i32,
         _2: i32,
         _3: i32,
         _4: i32,
         _5: i32,
         _6: i32,
         _7: i32,
         _8: i32,
         _9: i32,
         _10: i32,
         _11: i32,
         _12: i32,
         _13: i32,
         _14: i32,
         _15: i32| (),
    );
    (store, func)
}

type I32x16 = (
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
);

/// Returns a `(i32, ...)` tuple with 16 elements that have ascending values.
///
/// This is required as input or output of many of the following tests.
fn ascending_tuple() -> I32x16 {
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15)
}

#[test]
fn dynamic_many_params_works() {
    let (mut store, func) = setup_many_params();
    func.call(
        &mut store,
        &[
            Value::I32(0),
            Value::I32(1),
            Value::I32(2),
            Value::I32(3),
            Value::I32(4),
            Value::I32(5),
            Value::I32(6),
            Value::I32(7),
            Value::I32(8),
            Value::I32(9),
            Value::I32(10),
            Value::I32(11),
            Value::I32(12),
            Value::I32(13),
            Value::I32(14),
            Value::I32(15),
        ],
        &mut [],
    )
    .unwrap();
}

#[test]
fn static_many_params_works() {
    let (mut store, func) = setup_many_params();
    let typed_func = func.typed::<I32x16, (), _>(&mut store).unwrap();
    let inputs = ascending_tuple();
    let result = typed_func.call(&mut store, inputs);
    assert_matches!(result, Ok(()));
}

fn setup_many_results() -> (Store<()>, Func) {
    let mut store = test_setup();
    // Function taking 16 arguments (maximum) and doing nothing.
    let func = Func::wrap(&mut store, || ascending_tuple());
    (store, func)
}

#[test]
fn dynamic_many_results_works() {
    let (mut store, func) = setup_many_results();
    let mut results = [Value::I32(0); 16];
    func.call(&mut store, &[], &mut results).unwrap();
    let mut i = 0;
    let expected = [0; 16].map(|_| {
        let value = Value::I32(i as _);
        i += 1;
        value
    });
    assert_eq!(results, expected)
}

#[test]
fn static_many_results_works() {
    let (mut store, func) = setup_many_results();
    let typed_func = func.typed::<(), I32x16, _>(&mut store).unwrap();
    let result = typed_func.call(&mut store, ()).unwrap();
    let expected = ascending_tuple();
    assert_eq_tuple!(result, expected; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
}

fn setup_many_params_many_results() -> (Store<()>, Func) {
    let mut store = test_setup();
    // Function taking 16 arguments (maximum) and doing nothing.
    let func = Func::wrap(
        &mut store,
        |v0: i32,
         v1: i32,
         v2: i32,
         v3: i32,
         v4: i32,
         v5: i32,
         v6: i32,
         v7: i32,
         v8: i32,
         v9: i32,
         v10: i32,
         v11: i32,
         v12: i32,
         v13: i32,
         v14: i32,
         v15: i32| {
            (
                v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15,
            )
        },
    );
    (store, func)
}

#[test]
fn dynamic_many_params_many_results_works() {
    let (mut store, func) = setup_many_params_many_results();
    let mut results = [Value::I32(0); 16];
    let inputs = [
        Value::I32(0),
        Value::I32(1),
        Value::I32(2),
        Value::I32(3),
        Value::I32(4),
        Value::I32(5),
        Value::I32(6),
        Value::I32(7),
        Value::I32(8),
        Value::I32(9),
        Value::I32(10),
        Value::I32(11),
        Value::I32(12),
        Value::I32(13),
        Value::I32(14),
        Value::I32(15),
    ];
    func.call(&mut store, &inputs, &mut results).unwrap();
    assert_eq!(&results, &inputs)
}

#[test]
fn static_many_params_many_results_works() {
    let (mut store, func) = setup_many_params_many_results();
    let typed_func = func.typed::<I32x16, I32x16, _>(&mut store).unwrap();
    let inputs = ascending_tuple();
    let result = typed_func.call(&mut store, inputs).unwrap();
    assert_eq_tuple!(result, inputs; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
}

#[test]
fn dynamic_many_types_works() {
    let mut store = test_setup();
    // Function taking no arguments and returning 16 results as tuple (maximum).
    let func = Func::wrap(
        &mut store,
        |v0: i32, v1: u32, v2: i64, v3: u64, v4: F32, v5: F64| (v0, v1, v2, v3, v4, v5),
    );
    let mut results = [Value::I32(0); 6];
    let inputs = [
        Value::I32(0),
        Value::I32(1),
        Value::I64(2),
        Value::I64(3),
        Value::F32(4.0.into()),
        Value::F64(5.0.into()),
    ];
    func.call(&mut store, &inputs, &mut results).unwrap();
    assert_eq!(&results, &inputs)
}

#[test]
fn static_many_types_works() {
    let mut store = test_setup();
    // Function taking no arguments and returning 16 results as tuple (maximum).
    let func = Func::wrap(
        &mut store,
        |v0: i32, v1: u32, v2: i64, v3: u64, v4: F32, v5: F64| (v0, v1, v2, v3, v4, v5),
    );
    let typed_func = func
        .typed::<(i32, u32, i64, u64, F32, F64), (i32, u32, i64, u64, F32, F64), _>(&mut store)
        .unwrap();
    let inputs = (0, 1, 2, 3, 4.0.into(), 5.0.into());
    let result = typed_func.call(&mut store, inputs).unwrap();
    assert_eq!(result, inputs);
}

#[test]
fn dynamic_type_check_works() {
    let mut store = test_setup();
    let identity = Func::wrap(&mut store, |value: i32| value);
    let mut result = Value::I32(0);
    // Case: Too few inputs given to function.
    assert_matches!(
        identity.call(&mut store, &[], core::slice::from_mut(&mut result)),
        Err(Error::Func(FuncError::MismatchingParameters { .. }))
    );
    // Case: Too many inputs given to function.
    assert_matches!(
        identity.call(
            &mut store,
            &[Value::I32(0), Value::I32(1)],
            core::slice::from_mut(&mut result)
        ),
        Err(Error::Func(FuncError::MismatchingParameters { .. }))
    );
    // Case: Too few outputs given to function.
    assert_matches!(
        identity.call(&mut store, &[Value::I32(0)], &mut [],),
        Err(Error::Func(FuncError::MismatchingResults { .. }))
    );
    // Case: Too many outputs given to function.
    assert_matches!(
        identity.call(
            &mut store,
            &[Value::I32(0)],
            &mut [Value::I32(0), Value::I32(1)],
        ),
        Err(Error::Func(FuncError::MismatchingResults { .. }))
    );
    // Case: Mismatching type given as input to function.
    for input in &[
        Value::I64(0),
        Value::F32(0.0.into()),
        Value::F64(0.0.into()),
    ] {
        assert_matches!(
            identity.call(
                &mut store,
                core::slice::from_ref(input),
                core::slice::from_mut(&mut result)
            ),
            Err(Error::Func(FuncError::MismatchingParameters { .. }))
        );
    }
    // Case: Allow for incorrect result type.
    //
    // The result type will be overwritten anyways.
    assert_matches!(
        identity.call(&mut store, &[Value::I32(0)], &mut [Value::I64(0)]),
        Ok(_)
    );
}

#[test]
fn static_type_check_works() {
    let mut store = test_setup();
    let identity = Func::wrap(&mut store, |value: i32| value);
    // Case: Too few inputs given to function.
    assert_matches!(
        identity.typed::<(), i32, _>(&mut store),
        Err(Error::Func(FuncError::MismatchingParameters { .. }))
    );
    // Case: Too many inputs given to function.
    assert_matches!(
        identity.typed::<(i32, i32), i32, _>(&mut store),
        Err(Error::Func(FuncError::MismatchingParameters { .. }))
    );
    // Case: Too few results given to function.
    assert_matches!(
        identity.typed::<i32, (), _>(&mut store),
        Err(Error::Func(FuncError::MismatchingResults { .. }))
    );
    // Case: Too many results given to function.
    assert_matches!(
        identity.typed::<i32, (i32, i32), _>(&mut store),
        Err(Error::Func(FuncError::MismatchingResults { .. }))
    );
    // Case: Mismatching type given as input to function.
    assert_matches!(
        identity.typed::<i64, i32, _>(&mut store),
        Err(Error::Func(FuncError::MismatchingParameters { .. }))
    );
    // Case: Mismatching type given as output of function.
    assert_matches!(
        identity.typed::<i32, i64, _>(&mut store),
        Err(Error::Func(FuncError::MismatchingResults { .. }))
    );
}
