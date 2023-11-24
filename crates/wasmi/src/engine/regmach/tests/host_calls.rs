//! This submodule tests the unusual use case of calling host functions through the engine from the host side.

use crate::{core::Trap, Caller, Config, Engine, EngineBackend, Func, Store};

/// Setup a new `Store` for testing with initial value of 5.
fn setup_store() -> Store<i32> {
    let mut config = Config::default();
    config.set_engine_backend(EngineBackend::RegisterMachine);
    let engine = Engine::new(&config);
    Store::new(&engine, 5_i32)
}

#[test]
fn host_call_from_host_params_0_results_0() {
    let mut store = setup_store();
    let err_if_zero = Func::wrap(&mut store, |caller: Caller<i32>| {
        if *caller.data() == 0 {
            return Err(Trap::new("test trap"));
        }
        Ok(())
    });
    let err_if_zero = err_if_zero.typed::<(), ()>(&mut store).unwrap();
    assert!(err_if_zero.call(&mut store, ()).is_ok());
    *store.data_mut() = 0;
    assert!(err_if_zero.call(&mut store, ()).is_err());
}

#[test]
fn host_call_from_host_params_0_results_1() {
    let mut store = setup_store();
    let data_plus_42 = Func::wrap(&mut store, |caller: Caller<i32>| caller.data() + 42_i32);
    let data_plus_42 = data_plus_42.typed::<(), i32>(&mut store).unwrap();
    assert_eq!(data_plus_42.call(&mut store, ()).unwrap(), 47_i32);
    *store.data_mut() = 10;
    assert_eq!(data_plus_42.call(&mut store, ()).unwrap(), 52_i32);
}

#[test]
fn host_call_from_host_params_2_results_1() {
    let mut store = setup_store();
    let sum_with_data = Func::wrap(&mut store, |caller: Caller<i32>, a: i32, b: i32| {
        caller.data() + a + b
    });
    let sum_with_data = sum_with_data.typed::<(i32, i32), i32>(&mut store).unwrap();
    assert_eq!(sum_with_data.call(&mut store, (1, 2)).unwrap(), 8_i32);
    *store.data_mut() = 10;
    assert_eq!(sum_with_data.call(&mut store, (10, 15)).unwrap(), 35_i32);
}

#[test]
fn host_call_from_host_params_0_results_2() {
    let mut store = setup_store();
    let get_data =
        Func::wrap(&mut store, |caller: Caller<i32>| {
            let data = *caller.data();
            (data + data, data * data)
        });
    let get_data = get_data.typed::<(), (i32, i32)>(&mut store).unwrap();
    assert_eq!(get_data.call(&mut store, ()).unwrap(), (10, 25));
    *store.data_mut() = 10;
    assert_eq!(get_data.call(&mut store, ()).unwrap(), (20, 100));
}

#[test]
fn host_call_from_host_params_4_results_4() {
    let mut store = setup_store();
    // Function f(D, a,b,c,d) that outputs (d+D, c+D, b+D, a+D)
    let reverse_and_add = Func::wrap(
        &mut store,
        |caller: Caller<i32>, a: i32, b: i32, c: i32, d: i32| {
            let offset = caller.data();
            (d + offset, c + offset, b + offset, a + offset)
        },
    );
    let reverse_and_add = reverse_and_add
        .typed::<(i32, i32, i32, i32), (i32, i32, i32, i32)>(&mut store)
        .unwrap();
    assert_eq!(
        reverse_and_add.call(&mut store, (1, 2, 3, 4)).unwrap(),
        (9, 8, 7, 6)
    );
    *store.data_mut() = 10;
    assert_eq!(
        reverse_and_add.call(&mut store, (40, 30, 20, 10)).unwrap(),
        (20, 30, 40, 50)
    );
}
