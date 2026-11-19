use wasmi::{Engine, Instance, Module, Store};

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
#[cfg_attr(
    not(feature = "memory64"),
    ignore = "requires the memory64 crate feature"
)]
fn instantiate_out_of_memory() {
    let wasm = r#"
        (module
            (memory (;0;) i64 1 1)
            (func (export ""))
            (data (i64.const -1095216660480) "\ff")
        )
    "#;
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).unwrap();
    let mut store = Store::new(&engine, ());
    Instance::new(&mut store, &module, &[]).unwrap_err();
}

#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
#[should_panic = "cannot instantiate a module compiled for a different engine than the store's"]
fn instantiate_module_of_foreign_engine() {
    let wasm = r#"
        (module
            (func (export "f") (param i32) (result i32)
                (i32.add (local.get 0) (i32.const 40))
            )
        )
    "#;
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let foreign = Engine::default();
    let module = Module::new(&foreign, wasm).unwrap();
    _ = Instance::new(&mut store, &module, &[]);
}
