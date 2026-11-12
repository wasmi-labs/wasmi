//! Test to assert that host functions that call back into
//! Wasm works correctly.

use wasmi::{Caller, Engine, Error, Extern, Func, Instance, Memory, MemoryType, Module, Store};

fn test_setup() -> Store<()> {
    let engine = Engine::default();
    Store::new(&engine, ())
}

#[test]
fn host_calls_func_that_grows_memory() -> Result<(), Error> {
    let mut store = test_setup();
    let host_fn = Func::wrap(&mut store, call_func_that_grows_memory);
    let wasm = r#"
        (module
            (import "env" "host_memory_grow" (func $host_memory_grow))
            (memory $m 1 2)
            (func (export "memory.grow(1)")
                (drop (memory.grow $m (i32.const 1)))
            )
            (func (export "test")
                (call $host_memory_grow)
                ;; access first byte of new page
                ;; should work if growing was successful, otherwise will trap
                (drop (i32.load8_u $m (i32.const 65536)))
            )
        )
        "#;
    let module = Module::new(store.engine(), wasm)?;
    let instance = Instance::new(&mut store, &module, &[Extern::Func(host_fn)])?;
    instance
        .get_func(&store, "test")
        .unwrap()
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;
    Ok(())
}

fn call_func_that_grows_memory(mut caller: Caller<()>) -> Result<(), Error> {
    let wasm_fn = caller
        .get_export("memory.grow(1)")
        .and_then(Extern::into_func)
        .unwrap()
        .typed::<(), ()>(&caller)?;
    wasm_fn.call(&mut caller, ())?;
    Ok(())
}

#[test]
fn host_grows_memory() -> Result<(), Error> {
    let mut store = test_setup();
    let host_fn = Func::wrap(&mut store, grow_memory);
    let mem = Memory::new(&mut store, MemoryType::new(1, Some(2)))?;
    let wasm = r#"
        (module
            (import "env" "host_memory_grow" (func $host_memory_grow))
            (import "env" "mem" (memory $m 1 2))
            (export "mem" (memory $m))
            (func (export "test")
                (call $host_memory_grow)
                ;; access first byte of new page
                ;; should work if growing was successful, otherwise will trap
                (drop (i32.load8_u $m (i32.const 65536)))
            )
        )
        "#;
    let module = Module::new(store.engine(), wasm)?;
    let instance = Instance::new(
        &mut store,
        &module,
        &[Extern::Func(host_fn), Extern::Memory(mem)],
    )?;
    instance
        .get_func(&store, "test")
        .unwrap()
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;
    Ok(())
}

fn grow_memory(mut caller: Caller<()>) -> Result<(), Error> {
    let memory = caller
        .get_export("mem")
        .and_then(Extern::into_memory)
        .unwrap();
    memory.grow(&mut caller, 1)?;
    Ok(())
}
