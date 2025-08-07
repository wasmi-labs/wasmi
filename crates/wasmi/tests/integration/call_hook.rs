//! Tests to check if `Store::call_hook` works as intended.

use wasmi::{
    AsContext,
    AsContextMut,
    CallHook,
    Caller,
    Error,
    Extern,
    Func,
    Linker,
    Module,
    Store,
    TrapCode,
};

/// Number of times different callback events have fired.
#[derive(Default)]
struct CallHookTestState {
    calling_wasm: u32,
    returning_from_wasm: u32,
    calling_host: u32,
    returning_from_host: u32,
    erroneous_callback_invocation: bool,
}

fn test_setup() -> (Store<CallHookTestState>, Linker<CallHookTestState>) {
    let store = Store::default();
    let linker = <Linker<CallHookTestState>>::new(store.engine());
    (store, linker)
}

/// Prepares the test WAT and executes it. The wat defines two functions,
/// `wasm_fn_a` and `wasm_fn_b` and two imports, `host_fn_a` and `host_fn_b`.
/// `wasm_fn_a` calls `host_fn_a`, and `wasm_fn_b` calls `host_fn_b`.
/// None of the functions accept any arguments or return any value.
fn execute_wasm_fn_a(
    mut store: &mut Store<CallHookTestState>,
    linker: &mut Linker<CallHookTestState>,
) -> Result<(), Error> {
    let wasm = r#"
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
    let module = Module::new(store.engine(), wasm).unwrap();
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    let wasm_fn = instance
        .get_export(store.as_context(), "wasm_fn_a")
        .and_then(Extern::into_func)
        .unwrap()
        .typed::<(), ()>(&store)
        .unwrap();

    wasm_fn.call(store.as_context_mut(), ())
}

#[test]
fn call_hooks_get_called() {
    let (mut store, mut linker) = test_setup();

    store.call_hook(
        |data: &mut CallHookTestState, hook_type: CallHook| -> Result<(), Error> {
            match hook_type {
                CallHook::CallingWasm => data.calling_wasm += 1,
                CallHook::ReturningFromWasm => data.returning_from_wasm += 1,
                CallHook::CallingHost => data.calling_host += 1,
                CallHook::ReturningFromHost => data.returning_from_host += 1,
            };

            Ok(())
        },
    );

    let host_fn_a = Func::wrap(&mut store, |mut caller: Caller<CallHookTestState>| {
        // Call wasm_fn_a
        // Call host_fn_a
        assert_eq!(caller.data().calling_wasm, 1);
        assert_eq!(caller.data().returning_from_wasm, 0);
        assert_eq!(caller.data().calling_host, 1);
        assert_eq!(caller.data().returning_from_host, 0);

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
        assert_eq!(caller.data().calling_wasm, 2);
        assert_eq!(caller.data().returning_from_wasm, 1);
        assert_eq!(caller.data().calling_host, 2);
        assert_eq!(caller.data().returning_from_host, 1);
    });
    linker.define("env", "host_fn_a", host_fn_a).unwrap();

    let host_fn_b = Func::wrap(&mut store, |caller: Caller<CallHookTestState>| {
        // Call wasm_fn_a
        // Call host_fn_a
        // Call wasm_fn_b
        // Call host_fn_b
        assert_eq!(caller.data().calling_wasm, 2);
        assert_eq!(caller.data().returning_from_wasm, 0);
        assert_eq!(caller.data().calling_host, 2);
        assert_eq!(caller.data().returning_from_host, 0);
    });
    linker.define("env", "host_fn_b", host_fn_b).unwrap();

    execute_wasm_fn_a(&mut store, &mut linker).unwrap();

    assert_eq!(store.data().calling_wasm, 2);
    assert_eq!(store.data().returning_from_wasm, 2);
    assert_eq!(store.data().calling_host, 2);
    assert_eq!(store.data().returning_from_host, 2);
}

/// Utility function to generate a callback that fails after is has been called
/// `n` times.
#[allow(clippy::type_complexity)]
fn generate_error_after_n_calls<E: Into<Error> + Clone + Send + Sync + 'static>(
    limit: u32,
    error: E,
) -> Box<dyn FnMut(&mut CallHookTestState, CallHook) -> Result<(), Error> + Send + Sync> {
    Box::new(move |data, hook_type| -> Result<(), Error> {
        if (data.calling_wasm
            + data.returning_from_wasm
            + data.calling_host
            + data.returning_from_host)
            >= limit
        {
            return Err(error.clone().into());
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

    store.call_hook(generate_error_after_n_calls(
        0,
        wasmi_core::TrapCode::BadConversionToInteger,
    ));

    let should_not_run = Func::wrap(&mut store, |mut caller: Caller<CallHookTestState>| {
        caller.data_mut().erroneous_callback_invocation = true;
    });

    linker.define("env", "host_fn_a", should_not_run).unwrap();
    linker.define("env", "host_fn_b", should_not_run).unwrap();

    let result = execute_wasm_fn_a(&mut store, &mut linker).map_err(|err| {
        err.as_trap_code()
            .expect("The returned error is not a trap code")
    });

    assert!(
        !store.data().erroneous_callback_invocation,
        "A callback that should have been prevented was executed."
    );
    assert_eq!(result, Err(TrapCode::BadConversionToInteger));
}

#[test]
fn call_hook_prevents_host_execution() {
    let (mut store, mut linker) = test_setup();

    store.call_hook(generate_error_after_n_calls(1, TrapCode::BadSignature));

    let should_not_run = Func::wrap(&mut store, |mut caller: Caller<CallHookTestState>| {
        caller.data_mut().erroneous_callback_invocation = true;
    });

    linker.define("env", "host_fn_a", should_not_run).unwrap();
    linker.define("env", "host_fn_b", should_not_run).unwrap();

    let result = execute_wasm_fn_a(&mut store, &mut linker).map_err(|err| {
        err.as_trap_code()
            .expect("The returned error is not a trap code")
    });

    assert!(
        !store.data().erroneous_callback_invocation,
        "A callback that should have been prevented was executed."
    );
    assert_eq!(result, Err(TrapCode::BadSignature));
}

#[test]
fn call_hook_prevents_nested_wasm_execution() {
    let (mut store, mut linker) = test_setup();

    store.call_hook(generate_error_after_n_calls(
        2,
        TrapCode::GrowthOperationLimited,
    ));

    let host_fn_a = Func::wrap(&mut store, |mut caller: Caller<CallHookTestState>| {
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

    let should_not_run = Func::wrap(&mut store, |mut caller: Caller<CallHookTestState>| {
        caller.data_mut().erroneous_callback_invocation = true;
    });

    linker.define("env", "host_fn_a", host_fn_a).unwrap();
    linker.define("env", "host_fn_b", should_not_run).unwrap();

    // wasm_fn_a should also return a `TrapCode` from `CallHook::ReturningFromHost` hook.
    let result = execute_wasm_fn_a(&mut store, &mut linker).map_err(|err| {
        err.as_trap_code()
            .expect("The returned error is not a trap code")
    });

    assert!(
        !store.data().erroneous_callback_invocation,
        "A callback that should have been prevented was executed."
    );
    assert_eq!(result, Err(TrapCode::GrowthOperationLimited));
}
