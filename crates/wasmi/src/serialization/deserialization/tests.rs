use crate::{
    serialization::{deserialize_module, serialize_module},
    Engine, Linker, Module, Store,
};
use alloc::vec::Vec;

#[test]
fn imports_roundtrip() {
    // WAT with a function, table, memory, and global import
    let wat = r#"
        (module
            (import "env" "f" (func (param i32) (result i32)))
            (import "env" "tab" (table 2 10 funcref))
            (import "env" "mem" (memory 1 2))
            (import "mod" "g" (global i64))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    // Serialize
    let ser =
        crate::serialization::SerializedModule::from_module(&module, &engine).expect("serialize");
    let bytes = postcard::to_allocvec(&ser).expect("postcard serialize");
    // Deserialize
    let (deser_mod, _other_engine) = deserialize_module(&bytes).expect("failed to deserialize");
    // Compare imports
    let orig_imports: Vec<_> = module.imports().collect();
    let deser_imports: Vec<_> = deser_mod.imports().collect();
    assert_eq!(orig_imports.len(), deser_imports.len(), "import count");
    for (orig, deser) in orig_imports.iter().zip(deser_imports.iter()) {
        assert_eq!(orig.module(), deser.module(), "import module");
        assert_eq!(orig.name(), deser.name(), "import name");
        match (orig.ty(), deser.ty()) {
            (crate::ExternType::Func(a), crate::ExternType::Func(b)) => {
                assert_eq!(a, b, "import func type")
            }
            (crate::ExternType::Table(a), crate::ExternType::Table(b)) => {
                assert_eq!(a, b, "import table type")
            }
            (crate::ExternType::Memory(a), crate::ExternType::Memory(b)) => {
                assert_eq!(a, b, "import memory type")
            }
            (crate::ExternType::Global(a), crate::ExternType::Global(b)) => {
                assert_eq!(a, b, "import global type")
            }
            (a, b) => panic!("import type mismatch: {:?} vs {:?}", a, b),
        }
    }
}

#[test]
fn addition_module_serialize_deserialize_and_run() {
    // WAT for a simple addition function: (func (param i32 i32) (result i32) local.get 0 local.get 1 i32.add)
    let wat = r#"
        (module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    // Serialize
    let ser =
        crate::serialization::SerializedModule::from_module(&module, &engine).expect("serialize");
    let bytes = postcard::to_allocvec(&ser).expect("postcard serialize");
    // Deserialize with a new engine
    let (deser_mod, other_engine) =
        crate::serialization::deserialize_module(&bytes).expect("failed to deserialize");
    // Run the exported add function

    let mut store = Store::new(&other_engine, ());
    let linker = <Linker<()>>::new(&other_engine);
    let instantiated = linker
        .instantiate(&mut store, &deser_mod)
        .expect("failed to instantiate");
    let started = instantiated.start(&mut store).expect("failed to start");
    let add = started
        .get_typed_func::<(i32, i32), i32>(&mut store, "add")
        .expect("failed to get function");

    let result = add.call(&mut store, (31, 11)).expect("call");
    assert_eq!(result, 42, "deserialized module should add correctly");
}

#[test]
fn memory_export_roundtrip() {
    // WAT with a memory export
    let wat = r#"
        (module
            (memory (export "memory") 1 2)
            (data (i32.const 0) "Hello, World!")
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create Module");

    // Serialize
    let bytes = serialize_module(&module, &engine).expect("failed to serialize module");

    // Deserialize with a new engine
    let (deser_mod, other_engine) =
        crate::serialization::deserialize_module(&bytes).expect("failed to deserialize");

    // Instantiate and check that memory is exported
    let mut store = Store::new(&other_engine, ());
    let linker = <Linker<()>>::new(&other_engine);
    let instantiated = linker
        .instantiate(&mut store, &deser_mod)
        .expect("failed to instantiate");
    let started = instantiated.start(&mut store).expect("failed to start");

    // Check that memory is exported
    let memory = started
        .get_export(&mut store, "memory")
        .expect("memory export should exist")
        .into_memory()
        .expect("export should be a memory");

    // Verify the memory has the expected size (1 page = 64KB)
    assert_eq!(memory.size(&store), 1, "memory should have 1 page");

    // Verify the data was initialized correctly
    let mut data = [0u8; 13];
    memory
        .read(&store, 0, &mut data)
        .expect("should read memory");
    assert_eq!(
        &data, b"Hello, World!",
        "memory should contain the expected data"
    );
}

#[test]
fn module_with_host_import_roundtrip() {
    // WAT module that imports a host function and exports a function that uses it
    let wat = r#"
        (module
            (import "host" "reduce_by_one" (func $reduce_by_one (param i32) (result i32)))
            (func $add_reduced (param i32 i32) (result i32)
                local.get 0
                call $reduce_by_one
                local.get 1
                i32.add
            )
            (export "add_reduced" (func $add_reduced))
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = crate::Engine::default();
    let module = crate::Module::new(&engine, &wasm_bytes).expect("Failed to create Module");

    // Serialize the module
    let bytes = serialize_module(&module, &engine).expect("failed to serialize module");

    // Deserialize with a new engine
    let (deserialized_module, new_engine) =
        deserialize_module(&bytes).expect("failed to deserialize module");

    // Create a store and linker
    let mut store = crate::Store::new(&new_engine, ());
    let mut linker = crate::Linker::new(&new_engine);

    // Define the host function
    linker
        .define(
            "host",
            "reduce_by_one",
            crate::Func::wrap(&mut store, |x: i32| x - 1),
        )
        .expect("Failed to define host function");

    // Instantiate the module
    let instance = linker
        .instantiate(&mut store, &deserialized_module)
        .expect("Failed to instantiate module");

    let started = instance.start(&mut store).expect("failed to start");

    // Get the exported function
    let add_reduced = started
        .get_typed_func::<(i32, i32), i32>(&store, "add_reduced")
        .expect("Failed to get exported function");

    // Test the function: (5, 3) should return (5-1) + 3 = 7
    let result = add_reduced
        .call(&mut store, (5, 3))
        .expect("Failed to call function");

    assert_eq!(result, 7, "Expected (5-1) + 3 = 7, got {result}");

    // Test another case: (10, 5) should return (10-1) + 5 = 14
    let result2 = add_reduced
        .call(&mut store, (10, 5))
        .expect("Failed to call function");

    assert_eq!(result2, 14, "Expected (10-1) + 5 = 14, got {result2}");
}
