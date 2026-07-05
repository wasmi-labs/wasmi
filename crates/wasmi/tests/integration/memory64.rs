use wasmi::{Engine, Instance, Module, Store};

/// A 64-bit memory whose maximum equals the largest allowed value (`2^48` pages)
/// must still translate in-bounds constant-address accesses to real load/store ops.
///
/// Regression test: the translator sized the memory's maximum via `max << page_size_log2`
/// in `u64`, which overflows to `0` for `max == 2^48`, folding every non-zero constant
/// address into an unconditional `MemoryOutOfBounds` trap.
#[test]
fn mem64_max_size_constant_access_does_not_trap() {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let wasm = r#"
        (module
            (memory i64 1 0x1000000000000)
            (func (export "test") (result i64)
                (i64.store (i64.const 8) (i64.const 42))
                (i64.load (i64.const 8))
            )
        )
    "#;
    let module = Module::new(&engine, wasm).unwrap();
    let instance = Instance::new(&mut store, &module, &[]).unwrap();
    let test = instance
        .get_func(&store, "test")
        .unwrap()
        .typed::<(), i64>(&store)
        .unwrap();
    assert_eq!(test.call(&mut store, ()).unwrap(), 42);
}
