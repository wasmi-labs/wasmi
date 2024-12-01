use crate::{Engine, Func, Linker, Module, Store, Val};

/// Common routine to setup the tests.
fn setup_test(wasm: &str) -> (Store<()>, Func) {
    let engine = Engine::default();
    let mut store = <Store<()>>::new(&engine, ());
    let linker = <Linker<()>>::new(&engine);
    let module = Module::new(&engine, wasm).unwrap();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .ensure_no_start(&mut store)
        .unwrap();
    let func = instance.get_func(&store, "test").unwrap();
    (store, func)
}

#[test]
fn many_params() {
    let wat = include_str!("wat/many_params.wat");
    let (mut store, func) = setup_test(wat);
    func.call(&mut store, &[0; 150].map(Val::I32), &mut [])
        .unwrap();
}

#[test]
fn many_results() {
    let wat = include_str!("wat/many_results.wat");
    let (mut store, func) = setup_test(wat);
    let mut results = [0; 150].map(Val::I32);
    func.call(&mut store, &[], &mut results).unwrap();
    for (i, result) in results.iter().enumerate() {
        let &Val::I32(result) = result else {
            panic!("unexpected result type at index {i}: {result:?}");
        };
        assert!(result as usize == i % 10);
    }
}
