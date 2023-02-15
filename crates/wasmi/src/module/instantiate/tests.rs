//! Regression tests for GitHub issue:
//! https://github.com/paritytech/wasmi/issues/587
//!
//! The problem was that Wasm memories (and tables) were defined twice for a
//! `wasmi` instance for every imported Wasm memory (or table). Since `wasmi`
//! does not support the `multi-memory` Wasm proposal this resulted Wasm
//! instances with more than 1 memory (or table) if the Wasm module imported
//! those entities.

use wasmi_core::ValueType;

use crate::{
    instance::InstanceEntity,
    Engine,
    Error,
    Instance,
    Linker,
    Memory,
    MemoryType,
    Module,
    Store,
    Table,
    TableType,
    Value,
};

fn try_instantiate_from_wat(wat: &str) -> Result<(Store<()>, Instance), Error> {
    let wasm = wat::parse_str(wat).unwrap();
    let engine = Engine::default();
    let module = Module::new(&engine, &mut &wasm[..])?;
    let mut store = Store::new(&engine, ());
    let mut linker = <Linker<()>>::new(&engine);
    // Define one memory that can be used by the tests as import.
    let memory_type = MemoryType::new(4, None)?;
    let memory = Memory::new(&mut store, memory_type)?;
    linker.define("env", "memory", memory)?;
    // Define one table that can be used by the tests as import.
    let table_type = TableType::new(ValueType::FuncRef, 4, None);
    let init = Value::default(table_type.element());
    let table = Table::new(&mut store, table_type, init)?;
    linker.define("env", "table", table)?;
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)?;
    Ok((store, instance))
}

fn instantiate_from_wat(wat: &str) -> (Store<()>, Instance) {
    try_instantiate_from_wat(wat).unwrap()
}

fn resolve_instance<'a>(store: &'a Store<()>, instance: &Instance) -> &'a InstanceEntity {
    store.inner.resolve_instance(instance)
}

fn assert_no_duplicates(store: &Store<()>, instance: Instance) {
    assert!(resolve_instance(store, &instance).get_memory(1).is_none());
    assert!(resolve_instance(store, &instance).get_table(1).is_none());
}

#[test]
fn test_import_memory_and_table() {
    let wat = r#"
        (module
            (import "env" "memory" (memory 4))
            (import "env" "table" (table 4 funcref))
        )"#;
    let (store, instance) = instantiate_from_wat(wat);
    assert!(resolve_instance(&store, &instance).get_memory(0).is_some());
    assert!(resolve_instance(&store, &instance).get_table(0).is_some());
    assert_no_duplicates(&store, instance);
}

#[test]
fn test_import_memory() {
    let wat = r#"
        (module
            (import "env" "memory" (memory 4))
        )"#;
    let (store, instance) = instantiate_from_wat(wat);
    assert!(resolve_instance(&store, &instance).get_memory(0).is_some());
    assert!(resolve_instance(&store, &instance).get_table(0).is_none());
    assert_no_duplicates(&store, instance);
}

#[test]
fn test_import_table() {
    let wat = r#"
        (module
            (import "env" "table" (table 4 funcref))
        )"#;
    let (store, instance) = instantiate_from_wat(wat);
    assert!(resolve_instance(&store, &instance).get_memory(0).is_none());
    assert!(resolve_instance(&store, &instance).get_table(0).is_some());
    assert_no_duplicates(&store, instance);
}

#[test]
fn test_no_memory_no_table() {
    let wat = "(module)";
    let (store, instance) = instantiate_from_wat(wat);
    assert!(resolve_instance(&store, &instance).get_memory(0).is_none());
    assert!(resolve_instance(&store, &instance).get_table(0).is_none());
    assert_no_duplicates(&store, instance);
}

#[test]
fn test_internal_memory() {
    let wat = "(module (memory 1 10) )";
    let (store, instance) = instantiate_from_wat(wat);
    assert!(resolve_instance(&store, &instance).get_memory(0).is_some());
    assert!(resolve_instance(&store, &instance).get_table(0).is_none());
    assert_no_duplicates(&store, instance);
}

#[test]
fn test_internal_table() {
    let wat = "(module (table 4 funcref) )";
    let (store, instance) = instantiate_from_wat(wat);
    assert!(resolve_instance(&store, &instance).get_memory(0).is_none());
    assert!(resolve_instance(&store, &instance).get_table(0).is_some());
    assert_no_duplicates(&store, instance);
}
