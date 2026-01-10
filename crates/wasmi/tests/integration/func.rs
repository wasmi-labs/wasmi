//! Tests for the `Func` type in Wasmi.

use assert_matches::assert_matches;
use core::slice;
use wasmi::{
    Engine,
    Func,
    FuncType,
    Store,
    Val,
    errors::{ErrorKind, FuncError},
};
use wasmi_core::{F32, F64, ValType};

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

// Returns a Wasm store and two binary addition [`Func`] instances.
fn setup_add2() -> (Store<()>, Func, Func) {
    let mut store = test_setup();
    let add2 = Func::wrap(&mut store, |lhs: i32, rhs: i32| lhs + rhs);
    let add2_dyn = Func::new(
        &mut store,
        FuncType::new([ValType::I32, ValType::I32], [ValType::I32]),
        |_caller, inputs: &[Val], results: &mut [Val]| {
            assert_eq!(inputs.len(), 2);
            assert_eq!(results.len(), 1);
            let lhs = &inputs[0].i32().unwrap();
            let rhs = &inputs[1].i32().unwrap();
            results[0] = (lhs + rhs).into();
            Ok(())
        },
    );
    (store, add2, add2_dyn)
}

#[test]
fn dynamic_add2_works() {
    let (mut store, add2, add2_dyn) = setup_add2();
    for a in 0..10 {
        for b in 0..10 {
            let params = [Val::I32(a), Val::I32(b)];
            let expected = a + b;
            let mut result = Val::I32(0);
            // Call to Func with statically typed closure.
            add2.call(&mut store, &params, slice::from_mut(&mut result))
                .unwrap();
            // Reset result before execution.
            result = Val::I32(0);
            // Call to Func with dynamically typed closure.
            add2_dyn
                .call(&mut store, &params, slice::from_mut(&mut result))
                .unwrap();
            assert_eq!(result.i32(), Some(expected));
        }
    }
}

#[test]
fn static_add2_works() {
    let (mut store, add2, add2_dyn) = setup_add2();
    let add2 = add2.typed::<(i32, i32), i32>(&mut store).unwrap();
    let add2_dyn = add2_dyn.typed::<(i32, i32), i32>(&mut store).unwrap();
    for a in 0..10 {
        for b in 0..10 {
            let expected = a + b;
            assert_eq!(add2.call(&mut store, (a, b)).unwrap(), expected);
            assert_eq!(add2_dyn.call(&mut store, (a, b)).unwrap(), expected);
        }
    }
}

// Returns a Wasm store and two three-way addition [`Func`] instances.
fn setup_add3() -> (Store<()>, Func, Func) {
    let mut store = test_setup();
    let add3 = Func::wrap(&mut store, |v0: i32, v1: i32, v2: i32| v0 + v1 + v2);
    let add3_dyn = Func::new(
        &mut store,
        FuncType::new([ValType::I32, ValType::I32, ValType::I32], [ValType::I32]),
        |_caller, inputs: &[Val], results: &mut [Val]| {
            assert_eq!(inputs.len(), 3);
            assert_eq!(results.len(), 1);
            let a = &inputs[0].i32().unwrap();
            let b = &inputs[1].i32().unwrap();
            let c = &inputs[2].i32().unwrap();
            results[0] = (a + b + c).into();
            Ok(())
        },
    );
    (store, add3, add3_dyn)
}

#[test]
fn dynamic_add3_works() {
    let (mut store, add3, add3_dyn) = setup_add3();
    for a in 0..5 {
        for b in 0..5 {
            for c in 0..5 {
                let params = [Val::I32(a), Val::I32(b), Val::I32(c)];
                let expected = a + b + c;
                let mut result = Val::I32(0);
                // Call to Func with statically typed closure.
                add3.call(&mut store, &params, slice::from_mut(&mut result))
                    .unwrap();
                assert_eq!(result.i32(), Some(expected));
                // Reset result before execution.
                result = Val::I32(0);
                // Call to Func with dynamically typed closure.
                add3_dyn
                    .call(&mut store, &params, slice::from_mut(&mut result))
                    .unwrap();
                assert_eq!(result.i32(), Some(expected));
            }
        }
    }
}

#[test]
fn static_add3_works() {
    let (mut store, add3, add3_dyn) = setup_add3();
    let add3 = add3.typed::<(i32, i32, i32), i32>(&mut store).unwrap();
    let add3_dyn = add3_dyn.typed::<(i32, i32, i32), i32>(&mut store).unwrap();
    for a in 0..5 {
        for b in 0..5 {
            for c in 0..5 {
                let expected = a + b + c;
                assert_eq!(add3.call(&mut store, (a, b, c)).unwrap(), expected);
                assert_eq!(add3_dyn.call(&mut store, (a, b, c)).unwrap(), expected);
            }
        }
    }
}

// Returns a `Store` and two Wasm host functions that duplicate their inputs.
fn setup_duplicate() -> (Store<()>, Func, Func) {
    let mut store = test_setup();
    let duplicate = Func::wrap(&mut store, |value: i32| (value, value));
    let duplicate_dyn = Func::new(
        &mut store,
        FuncType::new([ValType::I32], [ValType::I32, ValType::I32]),
        |_caller, inputs: &[Val], results: &mut [Val]| {
            assert_eq!(inputs.len(), 1);
            assert_eq!(results.len(), 2);
            let input = inputs[0].i32().unwrap();
            results[0] = input.into();
            results[1] = input.into();
            Ok(())
        },
    );
    (store, duplicate, duplicate_dyn)
}

#[test]
fn dynamic_duplicate_works() {
    let (mut store, duplicate, duplicate_dyn) = setup_duplicate();
    for input in 0..10 {
        let params = [Val::I32(input)];
        let expected = [Val::I32(input), Val::I32(input)];
        let mut results = [Val::I32(0), Val::I32(0)];
        // Call to Func with statically typed closure.
        duplicate.call(&mut store, &params, &mut results).unwrap();
        assert_eq!(results[0].i32(), expected[0].i32());
        assert_eq!(results[1].i32(), expected[1].i32());
        // Reset result before execution.
        results = [Val::I32(0), Val::I32(0)];
        // Call to Func with dynamically typed closure.
        duplicate_dyn
            .call(&mut store, &params, &mut results)
            .unwrap();
        assert_eq!(results[0].i32(), expected[0].i32());
        assert_eq!(results[1].i32(), expected[1].i32());
    }
}

#[test]
fn static_duplicate_works() {
    let (mut store, duplicate, duplicate_dyn) = setup_duplicate();
    let duplicate = duplicate.typed::<i32, (i32, i32)>(&mut store).unwrap();
    let duplicate_dyn = duplicate_dyn.typed::<i32, (i32, i32)>(&mut store).unwrap();
    for input in 0..10 {
        assert_eq!(duplicate.call(&mut store, input).unwrap(), (input, input));
        assert_eq!(
            duplicate_dyn.call(&mut store, input).unwrap(),
            (input, input)
        );
    }
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
            Val::I32(0),
            Val::I32(1),
            Val::I32(2),
            Val::I32(3),
            Val::I32(4),
            Val::I32(5),
            Val::I32(6),
            Val::I32(7),
            Val::I32(8),
            Val::I32(9),
            Val::I32(10),
            Val::I32(11),
            Val::I32(12),
            Val::I32(13),
            Val::I32(14),
            Val::I32(15),
        ],
        &mut [],
    )
    .unwrap();
}

#[test]
fn static_many_params_works() {
    let (mut store, func) = setup_many_params();
    let typed_func = func.typed::<I32x16, ()>(&mut store).unwrap();
    let inputs = ascending_tuple();
    let result = typed_func.call(&mut store, inputs);
    assert_matches!(result, Ok(()));
}

fn setup_many_results() -> (Store<()>, Func) {
    let mut store = test_setup();
    // Function taking 16 arguments (maximum) and doing nothing.
    let func = Func::wrap(&mut store, ascending_tuple);
    (store, func)
}

#[test]
fn dynamic_many_results_works() {
    let (mut store, func) = setup_many_results();
    let mut results = [0; 16].map(Val::I32);
    func.call(&mut store, &[], &mut results).unwrap();
    let mut i = 0;
    let expected = [0; 16].map(|_| {
        let value = Val::I32(i as _);
        i += 1;
        value
    });
    assert_eq!(
        results.map(|result| result.i32().unwrap()),
        expected.map(|expected| expected.i32().unwrap())
    )
}

#[test]
fn static_many_results_works() {
    let (mut store, func) = setup_many_results();
    let typed_func = func.typed::<(), I32x16>(&mut store).unwrap();
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
    let mut results = [0; 16].map(Val::I32);
    let inputs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].map(Val::I32);
    func.call(&mut store, &inputs, &mut results).unwrap();
    assert_eq!(
        results.map(|result| result.i32().unwrap()),
        inputs.map(|input| input.i32().unwrap()),
    )
}

#[test]
fn static_many_params_many_results_works() {
    let (mut store, func) = setup_many_params_many_results();
    let typed_func = func.typed::<I32x16, I32x16>(&mut store).unwrap();
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
    let mut results = [0; 6].map(Val::I32);
    let inputs = [
        Val::I32(0),
        Val::I32(1),
        Val::I64(2),
        Val::I64(3),
        Val::F32(4.0.into()),
        Val::F64(5.0.into()),
    ];
    func.call(&mut store, &inputs, &mut results).unwrap();
    assert_eq!(results[0].i32(), Some(0));
    assert_eq!(results[1].i32(), Some(1));
    assert_eq!(results[2].i64(), Some(2));
    assert_eq!(results[3].i64(), Some(3));
    assert_eq!(results[4].f32(), Some(4.0.into()));
    assert_eq!(results[5].f64(), Some(5.0.into()));
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
        .typed::<(i32, u32, i64, u64, F32, F64), (i32, u32, i64, u64, F32, F64)>(&mut store)
        .unwrap();
    let inputs = (0, 1, 2, 3, 4.0.into(), 5.0.into());
    let result = typed_func.call(&mut store, inputs).unwrap();
    assert_eq!(result, inputs);
}

#[test]
fn dynamic_type_check_works() {
    let mut store = test_setup();
    let identity = Func::wrap(&mut store, |value: i32| value);
    let mut result = Val::I32(0);
    // Case: Too few inputs given to function.
    assert_matches!(
        identity
            .call(&mut store, &[], core::slice::from_mut(&mut result))
            .unwrap_err()
            .kind(),
        ErrorKind::Func(FuncError::MismatchingParameterLen)
    );
    // Case: Too many inputs given to function.
    assert_matches!(
        identity
            .call(
                &mut store,
                &[Val::I32(0), Val::I32(1)],
                core::slice::from_mut(&mut result)
            )
            .unwrap_err()
            .kind(),
        ErrorKind::Func(FuncError::MismatchingParameterLen)
    );
    // Case: Too few outputs given to function.
    assert_matches!(
        identity
            .call(&mut store, &[Val::I32(0)], &mut [],)
            .unwrap_err()
            .kind(),
        ErrorKind::Func(FuncError::MismatchingResultLen)
    );
    // Case: Too many outputs given to function.
    assert_matches!(
        identity
            .call(&mut store, &[Val::I32(0)], &mut [Val::I32(0), Val::I32(1)],)
            .unwrap_err()
            .kind(),
        ErrorKind::Func(FuncError::MismatchingResultLen)
    );
    // Case: Mismatching type given as input to function.
    for input in &[Val::I64(0), Val::F32(0.0.into()), Val::F64(0.0.into())] {
        assert_matches!(
            identity
                .call(
                    &mut store,
                    core::slice::from_ref(input),
                    core::slice::from_mut(&mut result)
                )
                .unwrap_err()
                .kind(),
            ErrorKind::Func(FuncError::MismatchingParameterType)
        );
    }
    // Case: Allow for incorrect result type.
    //
    // The result type will be overwritten anyways.
    assert_matches!(
        identity.call(&mut store, &[Val::I32(0)], &mut [Val::I64(0)]),
        Ok(_)
    );
}

#[test]
fn static_type_check_works() {
    let mut store = test_setup();
    let identity = Func::wrap(&mut store, |value: i32| value);
    // Case: Too few inputs given to function.
    assert_matches!(
        identity.typed::<(), i32>(&mut store).unwrap_err().kind(),
        ErrorKind::Func(FuncError::MismatchingParameterLen)
    );
    // Case: Too many inputs given to function.
    assert_matches!(
        identity
            .typed::<(i32, i32), i32>(&mut store)
            .unwrap_err()
            .kind(),
        ErrorKind::Func(FuncError::MismatchingParameterLen)
    );
    // Case: Too few results given to function.
    assert_matches!(
        identity.typed::<i32, ()>(&mut store).unwrap_err().kind(),
        ErrorKind::Func(FuncError::MismatchingResultLen)
    );
    // Case: Too many results given to function.
    assert_matches!(
        identity
            .typed::<i32, (i32, i32)>(&mut store)
            .unwrap_err()
            .kind(),
        ErrorKind::Func(FuncError::MismatchingResultLen)
    );
    // Case: Mismatching type given as input to function.
    assert_matches!(
        identity.typed::<i64, i32>(&mut store).unwrap_err().kind(),
        ErrorKind::Func(FuncError::MismatchingParameterType)
    );
    // Case: Mismatching type given as output of function.
    assert_matches!(
        identity.typed::<i32, i64>(&mut store).unwrap_err().kind(),
        ErrorKind::Func(FuncError::MismatchingResultType)
    );
}
