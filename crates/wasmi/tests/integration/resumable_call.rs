//! Test to assert that resumable call feature works as intended.

use core::slice;
use wasmi::{
    core::{TrapCode, ValType},
    errors::ErrorKind,
    AsContext,
    AsContextMut,
    Caller,
    Config,
    Engine,
    Error,
    Extern,
    Func,
    Linker,
    Module,
    ResumableCall,
    ResumableCallHostTrap,
    Store,
    TypedFunc,
    TypedResumableCall,
    TypedResumableCallHostTrap,
    Val,
};

fn test_setup(remaining: u32) -> (Store<TestData>, Linker<TestData>) {
    let mut config = Config::default();
    config.wasm_tail_call(true);
    let engine = Engine::new(&config);
    let store = Store::new(
        &engine,
        TestData {
            _remaining: remaining,
        },
    );
    let linker = <Linker<TestData>>::new(&engine);
    (store, linker)
}

#[derive(Debug)]
pub struct TestData {
    /// How many host calls must be made before returning a non error value.
    _remaining: u32,
}

fn resumable_call_smoldot_common(wasm: &str) -> (Store<TestData>, TypedFunc<(), i32>) {
    let (mut store, mut linker) = test_setup(0);
    // The important part about this test is that this
    // host function has more results than parameters.
    linker
        .func_wrap(
            "env",
            "host_fn",
            |mut _caller: Caller<'_, TestData>| -> Result<i32, Error> { Err(Error::i32_exit(100)) },
        )
        .unwrap();
    // The Wasm defines a single function that calls the
    // host function, returns 10 if the output is 0 and
    // returns 20 otherwise.
    let module = Module::new(store.engine(), wasm).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    let wasm_fn = instance.get_typed_func::<(), i32>(&store, "test").unwrap();
    (store, wasm_fn)
}

pub trait UnwrapResumable {
    type Results;

    fn unwrap_resumable(self) -> TypedResumableCallHostTrap<Self::Results>;
}

impl<Results> UnwrapResumable for Result<TypedResumableCall<Results>, Error> {
    type Results = Results;

    fn unwrap_resumable(self) -> TypedResumableCallHostTrap<Self::Results> {
        match self.unwrap() {
            TypedResumableCall::HostTrap(invocation) => invocation,
            TypedResumableCall::Finished(_) => panic!("expected TypedResumableCall::Resumable"),
        }
    }
}

#[test]
fn resumable_call_smoldot_01() {
    let (mut store, wasm_fn) = resumable_call_smoldot_common(
        r#"
        (module
            (import "env" "host_fn" (func $host_fn (result i32)))
            (func (export "test") (result i32)
                (call $host_fn)
            )
        )
        "#,
    );
    let invocation = wasm_fn.call_resumable(&mut store, ()).unwrap_resumable();
    match invocation.resume(&mut store, &[Val::I32(42)]).unwrap() {
        TypedResumableCall::Finished(result) => assert_eq!(result, 42),
        TypedResumableCall::HostTrap(_) => panic!("expected TypeResumableCall::Finished"),
    }
}

#[test]
fn resumable_call_smoldot_tail_01() {
    let (mut store, wasm_fn) = resumable_call_smoldot_common(
        r#"
        (module
            (import "env" "host_fn" (func $host_fn (result i32)))
            (func (export "test") (result i32)
                (return_call $host_fn)
            )
        )
        "#,
    );
    assert_eq!(
        wasm_fn
            .call_resumable(&mut store, ())
            .unwrap_err()
            .i32_exit_status(),
        Some(100),
    );
}

#[test]
fn resumable_call_smoldot_tail_02() {
    let (mut store, wasm_fn) = resumable_call_smoldot_common(
        r#"
        (module
            (import "env" "host_fn" (func $host (result i32)))
            (func $wasm (result i32)
                (return_call $host)
            )
            (func (export "test") (result i32)
                (call $wasm)
            )
        )
        "#,
    );
    let invocation = wasm_fn.call_resumable(&mut store, ()).unwrap_resumable();
    match invocation.resume(&mut store, &[Val::I32(42)]).unwrap() {
        TypedResumableCall::Finished(result) => assert_eq!(result, 42),
        TypedResumableCall::HostTrap(_) => panic!("expected TypeResumableCall::Finished"),
    }
}

#[test]
fn resumable_call_smoldot_02() {
    let (mut store, wasm_fn) = resumable_call_smoldot_common(
        r#"
        (module
            (import "env" "host_fn" (func $host_fn (result i32)))
            (func (export "test") (result i32)
                (if (result i32) (i32.ne (call $host_fn) (i32.const 0))
                    (then
                        (i32.const 11) ;; EXPECTED
                    )
                    (else
                        (i32.const 22) ;; FAILURE
                    )
                )
            )
        )
        "#,
    );
    let invocation = wasm_fn.call_resumable(&mut store, ()).unwrap_resumable();
    match invocation.resume(&mut store, &[Val::I32(42)]).unwrap() {
        TypedResumableCall::Finished(result) => assert_eq!(result, 11),
        TypedResumableCall::HostTrap(_) => panic!("expected TypeResumableCall::Finished"),
    }
}

#[test]
fn resumable_call_host() {
    let (mut store, _linker) = test_setup(0);
    let host_fn = Func::wrap(&mut store, || -> Result<(), Error> {
        Err(Error::i32_exit(100))
    });
    // Even though the called host function traps we expect a normal error
    // since the host function is the root function of the call and therefore
    // it would not make sense to resume it.
    let error = host_fn
        .call_resumable(&mut store, &[], &mut [])
        .unwrap_err();
    match error.i32_exit_status() {
        Some(100) => {}
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
    let (mut store, mut linker) = test_setup(0);
    let host_fn = Func::wrap(&mut store, |input: i32| -> Result<i32, Error> {
        match input {
            1 => Err(Error::i32_exit(10)),
            2 => Err(Error::i32_exit(20)),
            n => Ok(n + 1),
        }
    });
    linker.define("env", "host_fn", host_fn).unwrap();
    let wasm = r#"
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
            "#;
    let module = Module::new(store.engine(), wasm).unwrap();
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
        store: &Store<TestData>,
        exit_status: i32,
        host_results: &[ValType],
    ) -> Self::Invocation;
    fn assert_finish(self) -> Self::Results;
}

impl AssertResumable for ResumableCall {
    type Results = ();
    type Invocation = ResumableCallHostTrap;

    fn assert_resumable(
        self,
        store: &Store<TestData>,
        exit_status: i32,
        host_results: &[ValType],
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

fn run_test(wasm_fn: Func, store: &mut Store<TestData>, wasm_trap: bool) {
    let mut results = Val::I32(0);
    let invocation = wasm_fn
        .call_resumable(
            store.as_context_mut(),
            &[Val::I32(wasm_trap as i32)],
            slice::from_mut(&mut results),
        )
        .unwrap()
        .assert_resumable(store, 10, &[ValType::I32]);
    let invocation = invocation
        .resume(
            store.as_context_mut(),
            &[Val::I32(2)],
            slice::from_mut(&mut results),
        )
        .unwrap()
        .assert_resumable(store, 20, &[ValType::I32]);
    let call = invocation.resume(store, &[Val::I32(3)], slice::from_mut(&mut results));
    if wasm_trap {
        match call.unwrap_err().kind() {
            ErrorKind::TrapCode(trap) => {
                assert!(matches!(trap, TrapCode::UnreachableCodeReached,));
            }
            _ => panic!("expected Wasm trap"),
        }
    } else {
        call.unwrap().assert_finish();
        assert_eq!(results.i32(), Some(4));
    }
}

impl<Results> AssertResumable for TypedResumableCall<Results> {
    type Results = Results;
    type Invocation = TypedResumableCallHostTrap<Results>;

    fn assert_resumable(
        self,
        store: &Store<TestData>,
        exit_status: i32,
        host_results: &[ValType],
    ) -> Self::Invocation {
        match self {
            Self::HostTrap(invocation) => {
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
            Self::HostTrap(_) => panic!("expected the resumable call to finish"),
        }
    }
}

fn run_test_typed(wasm_fn: Func, store: &mut Store<TestData>, wasm_trap: bool) {
    let invocation = wasm_fn
        .typed::<i32, i32>(store.as_context())
        .unwrap()
        .call_resumable(store.as_context_mut(), wasm_trap as i32)
        .unwrap()
        .assert_resumable(store, 10, &[ValType::I32]);
    let invocation = invocation
        .resume(store.as_context_mut(), &[Val::I32(2)])
        .unwrap()
        .assert_resumable(store, 20, &[ValType::I32]);
    let call = invocation.resume(store, &[Val::I32(3)]);
    if wasm_trap {
        match call.unwrap_err().kind() {
            ErrorKind::TrapCode(trap) => {
                assert!(matches!(trap, TrapCode::UnreachableCodeReached,));
            }
            _ => panic!("expected Wasm trap"),
        }
    } else {
        assert_eq!(call.unwrap().assert_finish(), 4);
    }
}
