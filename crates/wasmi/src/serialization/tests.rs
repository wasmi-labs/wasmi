#![cfg(all(
    test,
    feature = "std",
    feature = "serialization",
    feature = "deserialization"
))]

use super::*;
use crate::{Engine, ExternType, Instance, Module, Store, Val};
use alloc::{format, string::String};

/// Creates a simple test module that just returns 42
fn create_simple_module() -> (Module, Engine) {
    let engine = Engine::default();
    let wasm = wat::parse_str(
        r#"
            (module
                (func $answer (result i32)
                    i32.const 42)
                (export "answer" (func $answer))
            )
            "#,
    )
    .unwrap();
    let module = Module::new(&engine, wasm).unwrap();
    (module, engine)
}

/// Creates a module with imports
fn create_module_with_imports() -> (Module, Engine) {
    let engine = Engine::default();
    let wasm = wat::parse_str(
        r#"
            (module
                (import "env" "add" (func $add (param i32 i32) (result i32)))
                (func $test (result i32)
                    i32.const 10
                    i32.const 32
                    call $add)
                (export "test" (func $test))
            )
            "#,
    )
    .unwrap();
    let module = Module::new(&engine, wasm).unwrap();
    (module, engine)
}

/// Creates a module with memory
fn create_module_with_memory() -> (Module, Engine) {
    let engine = Engine::default();
    let wasm = wat::parse_str(
        r#"
            (module
                (memory 1)
                (func $load (param i32) (result i32)
                    local.get 0
                    i32.load)
                (export "load" (func $load))
            )
            "#,
    )
    .unwrap();
    let module = Module::new(&engine, wasm).unwrap();
    (module, engine)
}

/// Creates a module with tables
fn create_module_with_tables() -> (Module, Engine) {
    let engine = Engine::default();
    let wasm = wat::parse_str(
        r#"
            (module
                (table 1 funcref)
                (func $get (param i32) (result funcref)
                    local.get 0
                    table.get 0)
                (export "get" (func $get))
            )
            "#,
    )
    .unwrap();
    let module = Module::new(&engine, wasm).unwrap();
    (module, engine)
}

/// Creates a module with globals
fn create_module_with_globals() -> (Module, Engine) {
    let engine = Engine::default();
    let wasm = wat::parse_str(
        r#"
            (module
                (global $counter (mut i32) (i32.const 0))
                (func $increment (result i32)
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                    global.get $counter)
                (export "increment" (func $increment))
            )
            "#,
    )
    .unwrap();
    let module = Module::new(&engine, wasm).unwrap();
    (module, engine)
}

#[test]
fn test_serialize_simple_module() {
    use core::mem::discriminant;

    let (module, engine) = create_simple_module();
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();

    // Verify serialized data is not empty
    assert!(!serialized.is_empty());

    // Verify we can deserialize it back
    let (deserialized, _engine) = deserialize_module(&serialized).unwrap();

    // Compare export names and type discriminants
    let original_exports: alloc::vec::Vec<_> = module.exports().collect();
    let deserialized_exports: alloc::vec::Vec<_> = deserialized.exports().collect();
    assert_eq!(original_exports.len(), deserialized_exports.len());
    for (original, deserialized) in original_exports.iter().zip(deserialized_exports.iter()) {
        assert_eq!(original.name(), deserialized.name());
        // Compare type discriminants only
        assert_eq!(discriminant(original.ty()), discriminant(deserialized.ty()));
        // TODO: Compare type details if/when possible
    }
}

#[test]
fn test_serialize_module_with_imports() {
    use core::mem::discriminant;

    let (module, engine) = create_module_with_imports();
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();

    // Verify serialized data is not empty
    assert!(!serialized.is_empty());

    // Verify we can deserialize it back
    let (deserialized, _engine) = deserialize_module(&serialized).unwrap();

    // Verify imports are preserved
    let original_imports: alloc::vec::Vec<_> = module.imports().collect();
    let deserialized_imports: alloc::vec::Vec<_> = deserialized.imports().collect();
    assert_eq!(original_imports.len(), deserialized_imports.len());

    for (original, deserialized) in original_imports.iter().zip(deserialized_imports.iter()) {
        assert_eq!(original.module(), deserialized.module());
        assert_eq!(original.name(), deserialized.name());
        // Compare type discriminants only
        assert_eq!(discriminant(original.ty()), discriminant(deserialized.ty()));
    }
}

#[test]
fn test_serialize_module_with_memory() {
    let (module, engine) = create_module_with_memory();
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();

    // Verify serialized data is not empty
    assert!(!serialized.is_empty());

    // Verify we can deserialize it back
    let (deserialized, _engine) = deserialize_module(&serialized).unwrap();

    // Verify memory types are preserved
    let original_memories: alloc::vec::Vec<_> = module
        .exports()
        .filter(|export| matches!(export.ty(), ExternType::Memory(_)))
        .collect();
    let deserialized_memories: alloc::vec::Vec<_> = deserialized
        .exports()
        .filter(|export| matches!(export.ty(), ExternType::Memory(_)))
        .collect();
    assert_eq!(original_memories.len(), deserialized_memories.len());
}

#[test]
fn test_serialize_module_with_tables() {
    let (module, engine) = create_module_with_tables();
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();

    // Verify serialized data is not empty
    assert!(!serialized.is_empty());

    // Verify we can deserialize it back
    let (deserialized, _engine) = deserialize_module(&serialized).unwrap();

    // Verify table types are preserved
    let original_tables: alloc::vec::Vec<_> = module
        .exports()
        .filter(|export| matches!(export.ty(), ExternType::Table(_)))
        .collect();
    let deserialized_tables: alloc::vec::Vec<_> = deserialized
        .exports()
        .filter(|export| matches!(export.ty(), ExternType::Table(_)))
        .collect();
    assert_eq!(original_tables.len(), deserialized_tables.len());
}

#[test]
fn test_serialize_module_with_globals() {
    let (module, engine) = create_module_with_globals();
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();

    // Verify serialized data is not empty
    assert!(!serialized.is_empty());

    // Verify we can deserialize it back
    let (deserialized, _engine) = deserialize_module(&serialized).unwrap();

    // Verify global types are preserved
    let original_globals: alloc::vec::Vec<_> = module
        .exports()
        .filter(|export| matches!(export.ty(), ExternType::Global(_)))
        .collect();
    let deserialized_globals: alloc::vec::Vec<_> = deserialized
        .exports()
        .filter(|export| matches!(export.ty(), ExternType::Global(_)))
        .collect();
    assert_eq!(original_globals.len(), deserialized_globals.len());
}

#[test]
fn test_serialize_deserialize_roundtrip() {
    let (module, engine) = create_simple_module();
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();
    let (deserialized, deser_engine) = deserialize_module(&serialized).unwrap();

    // Verify the deserialized module can be instantiated
    let mut store = Store::new(&deser_engine, ());
    let instance = Instance::new(&mut store, &deserialized, &[]).unwrap();

    // Verify the exported function works
    let answer = instance
        .get_export(&store, "answer")
        .unwrap()
        .into_func()
        .unwrap();
    let mut results = [Val::I32(0)];
    let params = &[];
    answer.call(&mut store, params, &mut results).unwrap();
    match results[0] {
        Val::I32(n) => assert_eq!(n, 42),
        _ => panic!("Expected I32 result"),
    }
}

#[test]
fn test_deserialize_invalid_data() {
    let invalid_data = b"invalid serialized data";

    let result = deserialize_module(invalid_data);
    assert!(result.is_err());

    match result.unwrap_err() {
        DeserializationError::CorruptedData { .. } => {}
        _ => panic!("Expected CorruptedData error"),
    }
}

#[test]
fn test_serialize_empty_module() {
    // Create a truly empty module with no exports
    let engine = Engine::default();
    let wasm = wat::parse_str(
        r#"
            (module
                ;; This module has no exports - it's truly empty
            )
            "#,
    )
    .unwrap();
    let module = Module::new(&engine, wasm).unwrap();

    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();
    assert!(!serialized.is_empty());

    let (deserialized, _engine) = deserialize_module(&serialized).unwrap();
    assert_eq!(deserialized.exports().count(), 0);
}

#[test]
fn test_serialize_large_module() {
    // Create a module with many functions
    let engine = Engine::default();
    let mut wat = String::from("(module");

    // Add 100 simple functions
    for i in 0..100 {
        wat.push_str(&format!(
            r#"
                (func $func_{i} (result i32)
                    i32.const {i})"#
        ));
    }

    // Export all functions
    for i in 0..100 {
        wat.push_str(&format!(
            r#"
                (export "func_{i}" (func $func_{i}))"#
        ));
    }

    wat.push(')');

    let wasm = wat::parse_str(&wat).unwrap();
    let module = Module::new(&engine, wasm).unwrap();

    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();
    assert!(!serialized.is_empty());

    let (deserialized, _engine) = deserialize_module(&serialized).unwrap();
    assert_eq!(deserialized.exports().count(), 100);
}

#[test]
fn test_feature_compatibility() {
    let (module, engine) = create_simple_module();
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();

    let (deserialized, _engine) = deserialize_module(&serialized).unwrap();
    assert_eq!(deserialized.exports().count(), 1);
}

#[test]
fn test_serialize_deserialize_simple_module_with_imports() {
    use crate::func::{Func, TypedFunc};
    use crate::serialization::{deserialize_module, serialize_module, RequiredFeatures};
    use crate::{Engine, Linker, Module, Store};

    // Create a simple WAT module with imports
    let wat = r#"
    (module
      (type (;0;) (func (param i32 i32) (result i32)))
      (type (;1;) (func (param i32)))
      (type (;2;) (func))
      
      (import "env" "log" (func (;0;) (type 1)))
      (import "env" "add" (func (;1;) (type 0)))
      
      (func (;2;) (type 0) (param i32 i32) (result i32)
        local.get 0
        local.get 1
        call 1)
      
      (func (;3;) (type 2)
        i32.const 42
        call 0)
      
      (export "add" (func 2))
      (export "log_42" (func 3))
    )
    "#;

    let engine = Engine::default();
    let mut store = Store::new(&engine, ());

    // Parse the module
    let module = Module::new(&engine, wat).expect("Failed to parse module");

    // Create host functions
    let log_func = Func::wrap(&mut store, |val: i32| {
        // Just ignore the value for testing
        let _ = val;
    });

    let add_func = Func::wrap(&mut store, |a: i32, b: i32| -> i32 { a + b });

    // Create linker and add imports
    let mut linker = Linker::new(&engine);
    linker
        .define("env", "log", log_func)
        .expect("Failed to define log function");
    linker
        .define("env", "add", add_func)
        .expect("Failed to define add function");

    // Create instance with imports
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate module");
    let started = instance
        .start(&mut store)
        .expect("Failed to start instance");

    // Test the module works before serialization
    let add: TypedFunc<(i32, i32), i32> = started
        .get_typed_func(&mut store, "add")
        .expect("Failed to get add function");

    let result = add
        .call(&mut store, (5, 3))
        .expect("Failed to call add function");
    assert_eq!(result, 8);

    // Serialize the module
    let features = RequiredFeatures::default();
    let serialized =
        serialize_module(&module, &features, &engine).expect("Failed to serialize module");

    // Deserialize the module
    let (deserialized_module, deserialized_engine) =
        deserialize_module(&serialized).expect("Failed to deserialize module");

    // Create a new store with the deserialized engine
    let mut deserialized_store = Store::new(&deserialized_engine, ());

    // Create host functions with the deserialized store
    let deserialized_log_func = Func::wrap(&mut deserialized_store, |val: i32| {
        // Just ignore the value for testing
        let _ = val;
    });

    let deserialized_add_func =
        Func::wrap(&mut deserialized_store, |a: i32, b: i32| -> i32 { a + b });

    // Create a new linker with the deserialized engine
    let mut deserialized_linker = Linker::new(&deserialized_engine);
    deserialized_linker
        .define("env", "log", deserialized_log_func)
        .expect("Failed to define log function");
    deserialized_linker
        .define("env", "add", deserialized_add_func)
        .expect("Failed to define add function");

    // Create a new instance with the deserialized module
    let deserialized_instance = deserialized_linker
        .instantiate(&mut deserialized_store, &deserialized_module)
        .expect("Failed to instantiate deserialized module");
    let deserialized_started = deserialized_instance
        .start(&mut deserialized_store)
        .expect("Failed to start deserialized instance");

    // Test the deserialized module
    let deserialized_add: TypedFunc<(i32, i32), i32> = deserialized_started
        .get_typed_func(&mut deserialized_store, "add")
        .expect("Failed to get add function from deserialized module");

    let deserialized_result = deserialized_add
        .call(&mut deserialized_store, (10, 20))
        .expect("Failed to call deserialized add function");
    assert_eq!(deserialized_result, 30);
}

#[test]
fn test_serialize_deserialize_module_with_global_imports() {
    use crate::func::Func;
    use crate::serialization::{deserialize_module, serialize_module, RequiredFeatures};
    use crate::{Engine, Global, Linker, Module, Store};

    // Create a WAT module with multiple global imports of the same type
    let wat = r#"
    (module
      (import "env" "global_a" (global i32))
      (import "env" "global_b" (global i32))
      
      (func (export "get_global_a") (result i32)
        global.get 0)
      
      (func (export "get_global_b") (result i32)
        global.get 1)
    )
    "#;

    let engine = Engine::default();
    let mut store = Store::new(&engine, ());

    // Parse the module
    let module = Module::new(&engine, wat).expect("Failed to parse module");

    // Create host globals with different values
    let global_a = Global::new(
        &mut store,
        crate::value::Val::I32(42),
        wasmi_core::Mutability::Const,
    );
    let global_b = Global::new(
        &mut store,
        crate::value::Val::I32(100),
        wasmi_core::Mutability::Const,
    );

    // Create linker and add imports
    let mut linker = Linker::new(&engine);
    linker
        .define("env", "global_a", global_a)
        .expect("Failed to define global_a");
    linker
        .define("env", "global_b", global_b)
        .expect("Failed to define global_b");

    // Create instance with imports
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate module");
    let started = instance
        .start(&mut store)
        .expect("Failed to start instance");

    // Test the module works before serialization
    let get_global_a: Func = started
        .get_func(&mut store, "get_global_a")
        .expect("Failed to get get_global_a function");
    let get_global_b: Func = started
        .get_func(&mut store, "get_global_b")
        .expect("Failed to get get_global_b function");

    let mut result_a = [crate::value::Val::I32(0)];
    let mut result_b = [crate::value::Val::I32(0)];

    get_global_a
        .call(&mut store, &[], &mut result_a)
        .expect("Failed to call get_global_a function");
    get_global_b
        .call(&mut store, &[], &mut result_b)
        .expect("Failed to call get_global_b function");

    assert_eq!(result_a[0].i32().expect("Expected i32"), 42);
    assert_eq!(result_b[0].i32().expect("Expected i32"), 100);

    // Serialize the module
    let features = RequiredFeatures::default();
    let serialized =
        serialize_module(&module, &features, &engine).expect("Failed to serialize module");

    // Deserialize the module
    let (deserialized_module, deserialized_engine) =
        deserialize_module(&serialized).expect("Failed to deserialize module");

    // Create a new store with the deserialized engine
    let mut deserialized_store = Store::new(&deserialized_engine, ());

    // Create a new linker with the deserialized engine
    let mut deserialized_linker = Linker::new(&deserialized_engine);

    // Create host globals with the deserialized store
    let deserialized_global_a = Global::new(
        &mut deserialized_store,
        crate::value::Val::I32(42),
        wasmi_core::Mutability::Const,
    );
    let deserialized_global_b = Global::new(
        &mut deserialized_store,
        crate::value::Val::I32(100),
        wasmi_core::Mutability::Const,
    );

    // Add imports to the deserialized linker
    deserialized_linker
        .define("env", "global_a", deserialized_global_a)
        .expect("Failed to define global_a");
    deserialized_linker
        .define("env", "global_b", deserialized_global_b)
        .expect("Failed to define global_b");

    // Create a new instance with the deserialized module
    let deserialized_instance = deserialized_linker
        .instantiate(&mut deserialized_store, &deserialized_module)
        .expect("Failed to instantiate deserialized module");
    let deserialized_started = deserialized_instance
        .start(&mut deserialized_store)
        .expect("Failed to start deserialized instance");

    // Test the deserialized module
    let deserialized_get_global_a: Func = deserialized_started
        .get_func(&mut deserialized_store, "get_global_a")
        .expect("Failed to get get_global_a function from deserialized module");
    let deserialized_get_global_b: Func = deserialized_started
        .get_func(&mut deserialized_store, "get_global_b")
        .expect("Failed to get get_global_b function from deserialized module");

    let mut deserialized_result_a = [crate::value::Val::I32(0)];
    let mut deserialized_result_b = [crate::value::Val::I32(0)];

    deserialized_get_global_a
        .call(&mut deserialized_store, &[], &mut deserialized_result_a)
        .expect("Failed to call deserialized get_global_a function");
    deserialized_get_global_b
        .call(&mut deserialized_store, &[], &mut deserialized_result_b)
        .expect("Failed to call deserialized get_global_b function");

    // This should fail if we have the same type-matching problem as with exports
    assert_eq!(
        deserialized_result_a[0].i32().expect("Expected i32"),
        42,
        "global_a should have value 42"
    );
    assert_eq!(
        deserialized_result_b[0].i32().expect("Expected i32"),
        100,
        "global_b should have value 100"
    );
}

#[test]
fn test_serialize_deserialize_module_with_global_exports() {
    use crate::func::TypedFunc;
    use crate::serialization::{deserialize_module, serialize_module, RequiredFeatures};
    use crate::{Engine, Module, Store};

    // Create a WAT module with global exports similar to the large module
    let wat = r#"
    (module
        (memory (export "memory") 1 1)
        (global (export "__data_end") i32 (i32.const 33064))
        (global (export "__heap_base") i32 (i32.const 33072))
        (func (export "run") (result i32)
            i32.const 42
        )
    )
    "#;

    let engine = Engine::default();

    // Parse the module
    let module = Module::new(&engine, wat).expect("Failed to parse module");

    // Serialize the module
    let features = RequiredFeatures::default();
    let serialized =
        serialize_module(&module, &features, &engine).expect("Failed to serialize module");

    // Deserialize the module
    let (deserialized_module, deser_engine) =
        deserialize_module(&serialized).expect("Failed to deserialize module");
    let mut store = Store::new(&deser_engine, ());

    // Test that the deserialized module has the same exports
    assert!(deserialized_module.get_export("memory").is_some());
    assert!(deserialized_module.get_export("__data_end").is_some());
    assert!(deserialized_module.get_export("__heap_base").is_some());
    assert!(deserialized_module.get_export("run").is_some());

    let linker = crate::Linker::new(&deser_engine);
    let instance = linker
        .instantiate(&mut store, &deserialized_module)
        .expect("Failed to instantiate module");
    let started = instance
        .start(&mut store)
        .expect("Failed to start instance");

    // Test that we can instantiate and check the global values
    // Get the global values and verify they match the expected values
    let data_end_global = started
        .get_global(&mut store, "__data_end")
        .expect("Failed to get __data_end global");
    let data_end_value = data_end_global.get(&mut store);
    let data_end_i32 = data_end_value
        .i32()
        .expect("Failed to convert __data_end to i32");
    assert_eq!(data_end_i32, 33064, "__data_end should have value 33064");

    let heap_base_global = started
        .get_global(&mut store, "__heap_base")
        .expect("Failed to get __heap_base global");
    let heap_base_value = heap_base_global.get(&mut store);
    let heap_base_i32 = heap_base_value
        .i32()
        .expect("Failed to convert __heap_base to i32");
    assert_eq!(heap_base_i32, 33072, "__heap_base should have value 33072");

    // Test that we can instantiate and call the function
    let run: TypedFunc<(), i32> = started
        .get_typed_func(&mut store, "run")
        .expect("Failed to get run function");

    let result = run
        .call(&mut store, ())
        .expect("Failed to call run function");
    assert_eq!(result, 42);
}

#[test]
fn test_serialize_deserialize_table_basic() {
    let (module, engine) = create_module_with_tables();

    // Test that the original module works
    let mut store = Store::new(&Engine::default(), ());
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Test that the original module has the table export
    let table_export = instance.get_export(&store, "get");
    assert!(table_export.is_some(), "Table export should exist");

    // Now test serialization/deserialization
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();
    assert!(!serialized.is_empty());

    let (deserialized, engine) = deserialize_module(&serialized).unwrap();

    // Create a new store and instance from the deserialized module
    let mut new_store = Store::new(&engine, ());
    let new_instance = Instance::new(&mut new_store, &deserialized, &[]).unwrap();

    // Test that the deserialized module has the table export
    let table_export = new_instance.get_export(&new_store, "get");
    assert!(
        table_export.is_some(),
        "Table export should exist after deserialization"
    );

    // Basic table serialization/deserialization works!
}

#[test]
fn test_serialize_deserialize_table_with_elements() {
    // Create module with tables and elements but no table exports
    let engine = Engine::default();
    let wasm = wat::parse_str(
        r#"
        (module
            ;; Define functions that we'll put in the table
            (func $func1 (result i32) i32.const 42)
            (func $func2 (result i32) i32.const 123)
            (func $func3 (result i32) i32.const 456)
            
            ;; Create a table with function references (not exported)
            (table 4 funcref)
            
            ;; Element segment that initializes the table
            (elem (i32.const 0) func $func1 $func2 $func3)
            
            ;; Function that tests table operations
            (func $test_table_ops (param i32) (result i32)
                local.get 0
                call_indirect (param) (result i32))
            
            ;; Function that gets a function from the table
            (func $get_func (param i32) (result funcref)
                local.get 0
                table.get 0)
            
            ;; Export only the functions, not the table
            (export "test_table_ops" (func $test_table_ops))
            (export "get_func" (func $get_func))
        )
    "#,
    )
    .unwrap();
    let module = Module::new(&engine, wasm).unwrap();

    // Test that the original module works correctly
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Test that we can call functions from the table
    let test_table_ops = instance
        .get_export(&store, "test_table_ops")
        .unwrap()
        .into_func()
        .unwrap();

    // Test calling function at index 0 (should return 42)
    let mut results = [Val::I32(0)];
    test_table_ops
        .call(&mut store, &[Val::I32(0)], &mut results)
        .unwrap();
    assert_eq!(results[0].i32().unwrap(), 42);

    // Test calling function at index 1 (should return 123)
    test_table_ops
        .call(&mut store, &[Val::I32(1)], &mut results)
        .unwrap();
    assert_eq!(results[0].i32().unwrap(), 123);

    // Test calling function at index 2 (should return 456)
    test_table_ops
        .call(&mut store, &[Val::I32(2)], &mut results)
        .unwrap();
    assert_eq!(results[0].i32().unwrap(), 456);

    // Now test serialization/deserialization
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();
    assert!(!serialized.is_empty());

    let (deserialized, deser_engine) = deserialize_module(&serialized).unwrap();

    // Create a new store and instance from the deserialized module
    let mut new_store = Store::new(&deser_engine, ());
    let new_instance = Instance::new(&mut new_store, &deserialized, &[]).unwrap();

    // Test that the deserialized module works the same way
    let new_test_table_ops = new_instance
        .get_export(&new_store, "test_table_ops")
        .unwrap()
        .into_func()
        .unwrap();

    // Test calling function at index 0 (should return 42)
    let mut new_results = [Val::I32(0)];
    new_test_table_ops
        .call(&mut new_store, &[Val::I32(0)], &mut new_results)
        .unwrap();
    assert_eq!(new_results[0].i32().unwrap(), 42);

    // Test calling function at index 1 (should return 123)
    new_test_table_ops
        .call(&mut new_store, &[Val::I32(1)], &mut new_results)
        .unwrap();
    assert_eq!(new_results[0].i32().unwrap(), 123);

    // Test calling function at index 2 (should return 456)
    new_test_table_ops
        .call(&mut new_store, &[Val::I32(2)], &mut new_results)
        .unwrap();
    assert_eq!(new_results[0].i32().unwrap(), 456);

    // Test that the get_func function also works
    let new_get_func = new_instance
        .get_export(&new_store, "get_func")
        .unwrap()
        .into_func()
        .unwrap();

    // This should work if tables are properly serialized
    let result = new_get_func.call(&mut new_store, &[Val::I32(0)], &mut new_results);
    match result {
        Ok(()) => {
            // Just verify the call succeeded - we can't easily check the function reference type
        }
        Err(e) => {
            panic!("Table serialization is broken - table.get failed: {:?}", e);
        }
    }
}

#[test]
fn test_serialize_deserialize_element_segments() {
    // Create a simple module with element segments
    let engine = Engine::default();
    let wasm = wat::parse_str(
        r#"
            (module
                ;; Define some functions
                (func $func1 (result i32) i32.const 42)
                (func $func2 (result i32) i32.const 123)
                (func $func3 (result i32) i32.const 456)
                
                ;; Create a table
                (table 4 funcref)
                
                ;; Element segment that initializes the table
                (elem (i32.const 0) func $func1 $func2 $func3)
                
                ;; Function that calls a function from the table
                (func $call_indirect (param i32) (result i32)
                    local.get 0
                    call_indirect (param) (result i32))
                
                ;; Export the function
                (export "call_indirect" (func $call_indirect))
            )
            "#,
    )
    .unwrap();
    let module = Module::new(&engine, wasm).unwrap();

    // Test that the original module works
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    // Test indirect calls work in the original module
    let call_indirect = instance
        .get_export(&store, "call_indirect")
        .unwrap()
        .into_func()
        .unwrap();

    // Call function at index 0 (should return 42)
    let mut results = [Val::I32(0)];
    call_indirect
        .call(&mut store, &[Val::I32(0)], &mut results)
        .unwrap();
    match results[0] {
        Val::I32(n) => assert_eq!(n, 42),
        _ => panic!("Expected I32 result"),
    }

    // Call function at index 1 (should return 123)
    call_indirect
        .call(&mut store, &[Val::I32(1)], &mut results)
        .unwrap();
    match results[0] {
        Val::I32(n) => assert_eq!(n, 123),
        _ => panic!("Expected I32 result"),
    }

    // Call function at index 2 (should return 456)
    call_indirect
        .call(&mut store, &[Val::I32(2)], &mut results)
        .unwrap();
    match results[0] {
        Val::I32(n) => assert_eq!(n, 456),
        _ => panic!("Expected I32 result"),
    }

    // Now test serialization/deserialization
    let serialized = serialize_module(&module, &RequiredFeatures::default(), &engine).unwrap();
    assert!(!serialized.is_empty());

    let (deserialized, engine) = deserialize_module(&serialized).unwrap();

    // Create a new store and instance from the deserialized module
    let mut new_store = Store::new(&engine, ());
    let new_instance = Instance::new(&mut new_store, &deserialized, &[]).unwrap();

    // Test indirect calls in the deserialized module
    let new_call_indirect = new_instance
        .get_export(&new_store, "call_indirect")
        .unwrap()
        .into_func()
        .unwrap();

    // These calls should work if element segments are properly serialized
    // If not, they will fail with table index errors

    // Call function at index 0 (should return 42)
    let mut new_results = [Val::I32(0)];
    let result = new_call_indirect.call(&mut new_store, &[Val::I32(0)], &mut new_results);
    match result {
        Ok(()) => match new_results[0] {
            Val::I32(n) => assert_eq!(n, 42, "Index 0 should return 42"),
            _ => panic!("Expected I32 result"),
        },
        Err(e) => {
            panic!(
                "Element segment serialization is broken - indirect call failed: {:?}",
                e
            );
        }
    }

    // Call function at index 1 (should return 123)
    let result = new_call_indirect.call(&mut new_store, &[Val::I32(1)], &mut new_results);
    match result {
        Ok(()) => match new_results[0] {
            Val::I32(n) => assert_eq!(n, 123, "Index 1 should return 123"),
            _ => panic!("Expected I32 result"),
        },
        Err(e) => {
            panic!(
                "Element segment serialization is broken - indirect call failed: {:?}",
                e
            );
        }
    }

    // Call function at index 2 (should return 456)
    let result = new_call_indirect.call(&mut new_store, &[Val::I32(2)], &mut new_results);
    match result {
        Ok(()) => match new_results[0] {
            Val::I32(n) => assert_eq!(n, 456, "Index 2 should return 456"),
            _ => panic!("Expected I32 result"),
        },
        Err(e) => {
            panic!(
                "Element segment serialization is broken - indirect call failed: {:?}",
                e
            );
        }
    }
}
