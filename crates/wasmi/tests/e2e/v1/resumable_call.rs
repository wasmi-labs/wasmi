//! Test to assert that resumable call feature works as intended.

use wasmi::{
    Engine,
    Error,
    Extern,
    Func,
    Linker,
    Module,
    ResumableCall,
    ResumableInvocation,
    Store,
    TypedResumableCall,
    TypedResumableInvocation,
};
use wasmi_core::{Trap, TrapCode, Value, ValueType};

fn test_setup() -> Store<()> {
    let engine = Engine::default();
    Store::new(&engine, ())
}

#[test]
fn resumable_call_host() {
    let mut store = test_setup();
    let host_fn = Func::wrap(&mut store, || -> Result<(), Trap> {
        Err(Trap::i32_exit(100))
    });
    // Even though the called host function traps we expect a normal error
    // since the host function is the root function of the call and therefore
    // it would not make sense to resume it.
    let error = host_fn
        .call_resumable(&mut store, &[], &mut [])
        .unwrap_err();
    match error {
        Error::Trap(trap) => {
            assert_eq!(trap.i32_exit_status(), Some(100));
        }
        _ => panic!("expected Wasm trap"),
    }
    // The same test for `TypedFunc`:
    let trap = host_fn
        .typed::<(), ()>(&store)
        .unwrap()
        .call_resumable(&mut store, ())
        .unwrap_err();
    assert_eq!(trap.i32_exit_status(), Some(100));
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
    run_test_typed(wasm_fn, &mut store, false);
    run_test_typed(wasm_fn, &mut store, true);
}

trait AssertResumable {
    type Results;
    type Invocation;

    fn assert_resumable(
        self,
        store: &Store<()>,
        exit_status: i32,
        host_results: &[ValueType],
    ) -> Self::Invocation;
    fn assert_finish(self) -> Self::Results;
}

impl AssertResumable for ResumableCall {
    type Results = ();
    type Invocation = ResumableInvocation;

    fn assert_resumable(
        self,
        store: &Store<()>,
        exit_status: i32,
        host_results: &[ValueType],
    ) -> Self::Invocation {
        match self {
            Self::Resumable(invocation) => {
                assert_eq!(invocation.host_error().i32_exit_status(), Some(exit_status));
                assert_eq!(invocation.host_func().ty(store).results(), host_results,);
                invocation
            }
            Self::Finished => panic!("expected host function trap with exit code 10"),
        }
    }

    fn assert_finish(self) -> Self::Results {
        match self {
            Self::Finished => (),
            Self::Resumable(_) => panic!("expected the resumable call to finish"),
        }
    }
}

fn run_test(wasm_fn: Func, mut store: &mut Store<()>, wasm_trap: bool) {
    let mut results = [Value::I32(0)];
    let invocation = wasm_fn
        .call_resumable(
            &mut store,
            &[Value::I32(wasm_trap as i32)],
            &mut results[..],
        )
        .unwrap()
        .assert_resumable(store, 10, &[ValueType::I32]);
    let invocation = invocation
        .resume(&mut store, &[Value::I32(2)], &mut results[..])
        .unwrap()
        .assert_resumable(store, 20, &[ValueType::I32]);
    let call = invocation.resume(&mut store, &[Value::I32(3)], &mut results[..]);
    if wasm_trap {
        match call.unwrap_err() {
            Error::Trap(trap) => {
                assert!(matches!(
                    trap.trap_code(),
                    Some(TrapCode::UnreachableCodeReached)
                ));
            }
            _ => panic!("expected Wasm trap"),
        }
    } else {
        call.unwrap().assert_finish();
        assert_eq!(results, [Value::I32(4)]);
    }
}

impl<Results> AssertResumable for TypedResumableCall<Results> {
    type Results = Results;
    type Invocation = TypedResumableInvocation<Results>;

    fn assert_resumable(
        self,
        store: &Store<()>,
        exit_status: i32,
        host_results: &[ValueType],
    ) -> Self::Invocation {
        match self {
            Self::Resumable(invocation) => {
                assert_eq!(invocation.host_error().i32_exit_status(), Some(exit_status));
                assert_eq!(invocation.host_func().ty(store).results(), host_results,);
                invocation
            }
            Self::Finished(_) => panic!("expected host function trap with exit code 10"),
        }
    }

    fn assert_finish(self) -> Self::Results {
        match self {
            Self::Finished(results) => results,
            Self::Resumable(_) => panic!("expected the resumable call to finish"),
        }
    }
}

fn run_test_typed(wasm_fn: Func, mut store: &mut Store<()>, wasm_trap: bool) {
    let invocation = wasm_fn
        .typed::<i32, i32>(&store)
        .unwrap()
        .call_resumable(&mut store, wasm_trap as i32)
        .unwrap()
        .assert_resumable(store, 10, &[ValueType::I32]);
    let invocation = invocation
        .resume(&mut store, &[Value::I32(2)])
        .unwrap()
        .assert_resumable(store, 20, &[ValueType::I32]);
    let call = invocation.resume(&mut store, &[Value::I32(3)]);
    if wasm_trap {
        match call.unwrap_err() {
            Error::Trap(trap) => {
                assert!(matches!(
                    trap.trap_code(),
                    Some(TrapCode::UnreachableCodeReached)
                ));
            }
            _ => panic!("expected Wasm trap"),
        }
    } else {
        assert_eq!(call.unwrap().assert_finish(), 4);
    }
}
