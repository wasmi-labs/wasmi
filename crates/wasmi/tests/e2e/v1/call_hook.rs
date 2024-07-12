//! Tests to check if `Store::call_hook` works as intended.

use wasmi::{core::TrapCode, CallHook, Caller, Error, Extern, Func, Linker, Module, Store};

#[derive(Default)]
/// Number of times different callback events have fired.
struct TimesCallbacksFired {
    calling_wasm: u32,
    returning_from_wasm: u32,
    calling_host: u32,
    returning_from_host: u32,
}

fn test_setup() -> (Store<TimesCallbacksFired>, Linker<TimesCallbacksFired>) {
    let store = Store::default();
    let linker = <Linker<TimesCallbacksFired>>::new(store.engine());
    (store, linker)
}

/// Prepares the test WAT and executes it. The wat defines two functions,
/// `wasm_fn_a` and `wasm_fn_b` and two imports, `host_fn_a` and `host_fn_b`.
/// `wasm_fn_a` calls `host_fn_a`, and `wasm_fn_b` calls `host_fn_b`.
/// None of the functions accept any arguments or return any value.
fn execute_wasm_fn_a(
    mut store: &mut Store<TimesCallbacksFired>,
    linker: &mut Linker<TimesCallbacksFired>,
) -> Result<(), Error> {
    const TEST_WAT: &str = r#"
    (module
        (import "env" "host_fn_a" (func $host_fn_a))
        (import "env" "host_fn_b" (func $host_fn_b))
        (func (export "wasm_fn_a")
            (call $host_fn_a)
        )
        (func (export "wasm_fn_b")
            (call $host_fn_b)
        )
    )
    "#;

    let wasm = wat::parse_str(TEST_WAT).unwrap();
    let module = Module::new(store.engine(), &wasm).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    let wasm_fn = instance
        .get_export(&store, "wasm_fn_a")
        .and_then(Extern::into_func)
        .unwrap()
        .typed::<(), ()>(&store)
        .unwrap();

    wasm_fn.call(&mut store, ())
}

#[test]
fn call_hooks_get_called() {
    let (mut store, mut linker) = test_setup();

    store.call_hook(
        |data: &mut TimesCallbacksFired, hook_type: CallHook| -> Result<(), TrapCode> {
            match hook_type {
                CallHook::CallingWasm => data.calling_wasm += 1,
                CallHook::ReturningFromWasm => data.returning_from_wasm += 1,
                CallHook::CallingHost => data.calling_host += 1,
                CallHook::ReturningFromHost => data.returning_from_host += 1,
            };

            Ok(())
        },
    );

    let host_fn_a = Func::wrap(&mut store, |mut caller: Caller<TimesCallbacksFired>| {
        // Call wasm_fn_a
        // Call host_fn_a
        assert!(caller.data().calling_wasm == 1);
        assert!(caller.data().returning_from_wasm == 0);
        assert!(caller.data().calling_host == 1);
        assert!(caller.data().returning_from_host == 0);

        caller
            .get_export("wasm_fn_b")
            .and_then(Extern::into_func)
            .unwrap()
            .typed::<(), ()>(&caller)
            .unwrap()
            .call(&mut caller, ())
            .unwrap();

        // Call wasm_fn_a
        // Call host_fn_a
        // Call wasm_fn_b
        // Call host_fn_b
        // Return host_fn_b
        // Return wasm_fn_b
        assert!(caller.data().calling_wasm == 2);
        assert!(caller.data().returning_from_wasm == 1);
        assert!(caller.data().calling_host == 2);
        assert!(caller.data().returning_from_host == 1);
    });
    linker.define("env", "host_fn_a", host_fn_a).unwrap();

    let host_fn_b = Func::wrap(&mut store, |caller: Caller<TimesCallbacksFired>| {
        // Call wasm_fn_a
        // Call host_fn_a
        // Call wasm_fn_b
        // Call host_fn_b
        assert!(caller.data().calling_wasm == 2);
        assert!(caller.data().returning_from_wasm == 0);
        assert!(caller.data().calling_host == 2);
        assert!(caller.data().returning_from_host == 0);
    });
    linker.define("env", "host_fn_b", host_fn_b).unwrap();

    execute_wasm_fn_a(&mut store, &mut linker).unwrap();

    assert!(store.data().calling_wasm == 2);
    assert!(store.data().returning_from_wasm == 2);
    assert!(store.data().calling_host == 2);
    assert!(store.data().returning_from_host == 2);
}

/// Utility function to generate a callback that fails after is has been called
/// `n` times.
fn generate_trap_after_n_calls(
    limit: u32,
    trap_code: TrapCode,
) -> Box<
    dyn FnMut(&mut TimesCallbacksFired, CallHook) -> Result<(), TrapCode> + Send + Sync + 'static,
> {
    Box::new(move |data, hook_type| -> Result<(), TrapCode> {
        if (data.calling_wasm
            + data.returning_from_wasm
            + data.calling_host
            + data.returning_from_host)
            >= limit
        {
            return Err(trap_code);
        }

        match hook_type {
            CallHook::CallingWasm => data.calling_wasm += 1,
            CallHook::ReturningFromWasm => data.returning_from_wasm += 1,
            CallHook::CallingHost => data.calling_host += 1,
            CallHook::ReturningFromHost => data.returning_from_host += 1,
        };

        Ok(())
    })
}

#[test]
fn call_hook_prevents_wasm_execution() {
    let (mut store, mut linker) = test_setup();

    store.call_hook(generate_trap_after_n_calls(
        0,
        TrapCode::BadConversionToInteger,
    ));

    let should_not_run = Func::wrap(&mut store, |_: Caller<TimesCallbacksFired>| {
        panic!("Host function that should not run due to trap from call hook executed");
    });

    linker.define("env", "host_fn_a", should_not_run).unwrap();
    linker.define("env", "host_fn_b", should_not_run).unwrap();

    let result = execute_wasm_fn_a(&mut store, &mut linker).map_err(|err| {
        err.as_trap_code()
            .expect("The returned error is not a trap code")
    });

    assert_eq!(result, Err(TrapCode::BadConversionToInteger));
}

#[test]
fn call_hook_prevents_host_execution() {
    let (mut store, mut linker) = test_setup();

    store.call_hook(generate_trap_after_n_calls(1, TrapCode::BadSignature));

    let should_not_run = Func::wrap(&mut store, |_: Caller<TimesCallbacksFired>| {
        panic!("Host function that should not run due to trap from call hook executed");
    });

    linker.define("env", "host_fn_a", should_not_run).unwrap();
    linker.define("env", "host_fn_b", should_not_run).unwrap();

    let result = execute_wasm_fn_a(&mut store, &mut linker).map_err(|err| {
        err.as_trap_code()
            .expect("The returned error is not a trap code")
    });

    assert_eq!(result, Err(TrapCode::BadSignature));
}

#[test]
fn call_hook_prevents_nested_wasm_execution() {
    let (mut store, mut linker) = test_setup();

    store.call_hook(generate_trap_after_n_calls(
        2,
        TrapCode::GrowthOperationLimited,
    ));

    let host_fn_a = Func::wrap(&mut store, |mut caller: Caller<TimesCallbacksFired>| {
        let result = caller
            .get_export("wasm_fn_b")
            .and_then(Extern::into_func)
            .unwrap()
            .typed::<(), ()>(&caller)
            .unwrap()
            .call(&mut caller, ())
            .map_err(|err| {
                err.as_trap_code()
                    .expect("The returned error is not a trap code")
            });

        assert_eq!(result, Err(TrapCode::GrowthOperationLimited));
    });

    let should_not_run = Func::wrap(&mut store, |_: Caller<TimesCallbacksFired>| {
        panic!("Host function that should not run due to trap from call hook executed");
    });

    linker.define("env", "host_fn_a", host_fn_a).unwrap();
    linker.define("env", "host_fn_b", should_not_run).unwrap();

    // wasm_fn_a should also return a `TrapCode` from `CallHook::ReturningFromHost` hook.
    let result = execute_wasm_fn_a(&mut store, &mut linker).map_err(|err| {
        err.as_trap_code()
            .expect("The returned error is not a trap code")
    });

    assert_eq!(result, Err(TrapCode::GrowthOperationLimited));
}
