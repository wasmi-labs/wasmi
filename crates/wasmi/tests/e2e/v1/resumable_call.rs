//! Test to assert that resumable call feature works as intended.

use wasmi::{Engine, Extern, Func, Linker, Module, ResumableCall, Store};
use wasmi_core::{Trap, Value, ValueType};

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

    let mut results = [Value::I32(0)];
    println!("execute_resumable");
    let invocation = match wasm_fn
        .call_resumable(&mut store, &[Value::I32(0)], &mut results[..])
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
    println!("resume");
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
    println!("resume");
    match invocation
        .resume(&mut store, &[Value::I32(3)], &mut results[..])
        .unwrap()
    {
        ResumableCall::Resumable(_) => panic!("expected resumed function to finish"),
        ResumableCall::Finished(()) => {
            assert_eq!(results, [Value::I32(4)]);
        }
    }
    println!("finish");
}
