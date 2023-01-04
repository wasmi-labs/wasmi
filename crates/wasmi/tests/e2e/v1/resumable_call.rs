//! Test to assert that resumable call feature works as intended.

use wasmi::{Engine, Error, Extern, Func, Linker, Module, ResumableCall, Store};
use wasmi_core::{Trap, TrapCode, Value, ValueType};

fn test_setup() -> Store<()> {
    let engine = Engine::default();
    Store::new(&engine, ())
}

#[test]
fn resumable_call() {
    let mut store = test_setup();
    let mut linker = <Linker<()>>::new();
    let host_fn = Func::wrap(&mut store, |input: i32| -> Result<i32, Trap> {
        match input {
            1 => Err(Trap::i32_exit(10)),
            2 => Err(Trap::i32_exit(20)),
            n => Ok(n + 1),
        }
    });
    linker.define("env", "host_fn", host_fn).unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (import "env" "host_fn" (func $host_fn (param i32) (result i32)))
            (func (export "wasm_fn") (param $wasm_trap i32) (result i32)
                (local $i i32)
                (local.set $i (i32.const 0))
                (local.set $i (call $host_fn (local.get $i))) ;; Ok
                (local.set $i (call $host_fn (local.get $i))) ;; Trap::i32_exit(1)
                (local.set $i (call $host_fn (local.get $i))) ;; Trap::i32_exit(2)
                (local.set $i (call $host_fn (local.get $i))) ;; Ok
                (if (i32.eq (local.get $wasm_trap) (i32.const 1))
                    (then unreachable)                        ;; trap in Wasm if $wasm_trap == 1
                )
                (local.get $i)                                ;; return i == 4
            )
        )
        "#,
    )
    .unwrap();

    let module = Module::new(store.engine(), &mut &wasm[..]).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    let wasm_fn = instance
        .get_export(&store, "wasm_fn")
        .and_then(Extern::into_func)
        .unwrap();

    run_test(wasm_fn, &mut store, false);
    run_test(wasm_fn, &mut store, true);
}

fn run_test(wasm_fn: Func, mut store: &mut Store<()>, wasm_trap: bool) {
    let mut results = [Value::I32(0)];
    let invocation = match wasm_fn
        .call_resumable(
            &mut store,
            &[Value::I32(wasm_trap as i32)],
            &mut results[..],
        )
        .unwrap()
    {
        ResumableCall::Resumable(invocation) => {
            assert_eq!(invocation.host_error().i32_exit_status(), Some(10));
            assert_eq!(
                invocation.host_func().func_type(&store).results(),
                &[ValueType::I32]
            );
            invocation
        }
        ResumableCall::Finished(_) => panic!("expected host function trap with exit code 10"),
    };
    let invocation = match invocation
        .resume(&mut store, &[Value::I32(2)], &mut results[..])
        .unwrap()
    {
        ResumableCall::Resumable(invocation) => {
            assert_eq!(invocation.host_error().i32_exit_status(), Some(20));
            assert_eq!(
                invocation.host_func().func_type(&store).results(),
                &[ValueType::I32]
            );
            invocation
        }
        ResumableCall::Finished(_) => panic!("expected host function trap with exit code 20"),
    };
    let result = invocation.resume(&mut store, &[Value::I32(3)], &mut results[..]);
    if wasm_trap {
        match result {
            Ok(_) => panic!("expected resumed function to trap in Wasm"),
            Err(trap) => match trap {
                Error::Trap(trap) => {
                    assert!(matches!(
                        trap.trap_code(),
                        Some(TrapCode::UnreachableCodeReached)
                    ));
                }
                _ => panic!("expected Wasm trap"),
            },
        }
    } else {
        match result {
            Ok(ResumableCall::Resumable(_)) | Err(_) => {
                panic!("expected resumed function to finish")
            }
            Ok(ResumableCall::Finished(())) => {
                assert_eq!(results, [Value::I32(4)]);
            }
        }
    }
}
