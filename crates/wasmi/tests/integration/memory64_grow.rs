use wasmi::{Engine, Instance, Module, Store};

/// A `memory64` whose declared maximum is the largest the spec allows
/// (`2^48` pages at the default 64 KiB page size) must still grow its
/// first page without aborting the interpreter.
#[test]
fn grow_memory64_with_maximum_size() -> Result<(), wasmi::Error> {
    const WASM: &str = r#"
        (module
            (memory i64 0 0x1000000000000)
            (func (export "grow") (param $delta i64) (result i64)
                (memory.grow (local.get $delta))
            )
        )
    "#;
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::new(&engine, WASM)?;
    let instance = Instance::new(&mut store, &module, &[])?;
    let grow = instance.get_typed_func::<i64, i64>(&store, "grow")?;
    assert_eq!(grow.call(&mut store, 1)?, 0);
    assert_eq!(grow.call(&mut store, 1)?, 1);
    Ok(())
}
