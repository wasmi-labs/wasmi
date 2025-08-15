use super::*;
use crate::{
    preparsed::{serialized_module::types::SerializedValType, SerializedExternType},
    Engine, Module,
};
use wat::parse_str as parse_wat;

#[test]
fn functions_extraction() {
    // WAT with three functions: (func) (func (param i32)) (func (param i32) (result i64))
    let wat = r#"
        (module
            (func)
            (func (param i32))
            (func (param i32) (result i64)
                i64.const 0
            )
        )
    "#;
    let wasm_bytes = parse_wat(wat).expect("Failed to parse WAT");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    let result = SerializedModule::from_module(&module, &engine);
    assert!(
        result.is_ok(),
        "from_module should succeed for multi-func module"
    );
    let serialized = result.unwrap();
    // There should be 3 internal functions
    assert_eq!(serialized.internal_functions.len(), 3);
    // There should be 3 function types (no deduplication in this case)
    assert_eq!(serialized.func_types.len(), 3);
    // Check the signatures
    let sig0 = &serialized.func_types[0];
    assert_eq!(sig0.params.len(), 0);
    assert_eq!(sig0.results.len(), 0);
    let sig1 = &serialized.func_types[1];
    assert_eq!(sig1.params.len(), 1);
    assert_eq!(sig1.params[0], SerializedValType::I32);
    assert_eq!(sig1.results.len(), 0);
    let sig2 = &serialized.func_types[2];
    assert_eq!(sig2.params.len(), 1);
    assert_eq!(sig2.params[0], SerializedValType::I32);
    assert_eq!(sig2.results.len(), 1);
    assert_eq!(sig2.results[0], SerializedValType::I64);
    // The internal_functions should reference the correct type indices
    assert_eq!(serialized.internal_functions.len(), 3);
    assert_eq!(serialized.internal_functions[0].type_idx, 0);
    assert_eq!(serialized.internal_functions[1].type_idx, 1);
    assert_eq!(serialized.internal_functions[2].type_idx, 2);
}

#[test]
fn imported_tables_extraction() {
    // WAT with three table imports with different parameters
    let wat = r#"
        (module
            (import "env" "my_table1" (table 2 10 funcref))
            (import "env" "my_table2" (table 1 1 externref))
            (import "mod" "tab3" (table 5 20 funcref))
        )
    "#;
    let wasm_bytes = parse_wat(wat).expect("Failed to parse WAT");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    let func_types = extract_func_types(&module);
    let imports = super::extract_imports(&module, &func_types);
    // There should be three imports
    assert_eq!(imports.len(), 3);
    // Check first table
    let import = &imports[0];
    assert_eq!(import.module, "env");
    assert_eq!(import.name, "my_table1");
    match &import.ty {
        SerializedExternType::Table(table_ty) => {
            assert_eq!(table_ty.element, SerializedValType::FuncRef);
            assert_eq!(table_ty.min, 2);
            assert_eq!(table_ty.max, Some(10));
        }
        other => panic!("Expected table import, got {:?}", other),
    }
    // Check second table
    let import = &imports[1];
    assert_eq!(import.module, "env");
    assert_eq!(import.name, "my_table2");
    match &import.ty {
        SerializedExternType::Table(table_ty) => {
            assert_eq!(table_ty.element, SerializedValType::ExternRef);
            assert_eq!(table_ty.min, 1);
            assert_eq!(table_ty.max, Some(1));
        }
        other => panic!("Expected table import, got {:?}", other),
    }
    // Check third table
    let import = &imports[2];
    assert_eq!(import.module, "mod");
    assert_eq!(import.name, "tab3");
    match &import.ty {
        SerializedExternType::Table(table_ty) => {
            assert_eq!(table_ty.element, SerializedValType::FuncRef);
            assert_eq!(table_ty.min, 5);
            assert_eq!(table_ty.max, Some(20));
        }
        other => panic!("Expected table import, got {:?}", other),
    }
}

#[test]
fn imported_memories_extraction() {
    // WAT with three memory imports with different parameters
    let wat = r#"
        (module
            (import "env" "mem1" (memory 1 2))
            (import "env" "mem2" (memory 3 5))
            (import "mod" "mem3" (memory 10 20))
        )
    "#;
    let wasm_bytes = parse_wat(wat).expect("Failed to parse WAT");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    let func_types = extract_func_types(&module);
    let imports = super::extract_imports(&module, &func_types);
    // There should be three imports
    assert_eq!(imports.len(), 3);
    // Check first memory
    let import = &imports[0];
    assert_eq!(import.module, "env");
    assert_eq!(import.name, "mem1");
    match &import.ty {
        SerializedExternType::Memory(mem_ty) => {
            assert_eq!(mem_ty.min, 1);
            assert_eq!(mem_ty.max, Some(2));
        }
        other => panic!("Expected memory import, got {:?}", other),
    }
    // Check second memory
    let import = &imports[1];
    assert_eq!(import.module, "env");
    assert_eq!(import.name, "mem2");
    match &import.ty {
        SerializedExternType::Memory(mem_ty) => {
            assert_eq!(mem_ty.min, 3);
            assert_eq!(mem_ty.max, Some(5));
        }
        other => panic!("Expected memory import, got {:?}", other),
    }
    // Check third memory
    let import = &imports[2];
    assert_eq!(import.module, "mod");
    assert_eq!(import.name, "mem3");
    match &import.ty {
        SerializedExternType::Memory(mem_ty) => {
            assert_eq!(mem_ty.min, 10);
            assert_eq!(mem_ty.max, Some(20));
        }
        other => panic!("Expected memory import, got {:?}", other),
    }
}

#[test]
fn imported_globals_extraction() {
    // WAT with three global imports with different types and mutability
    let wat = r#"
        (module
            (import "env" "g1" (global i32))
            (import "env" "g2" (global (mut f64)))
            (import "mod" "g3" (global i64))
        )
    "#;
    let wasm_bytes = parse_wat(wat).expect("Failed to parse WAT");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    let func_types = extract_func_types(&module);
    let imports = super::extract_imports(&module, &func_types);
    // There should be three imports
    assert_eq!(imports.len(), 3);
    // Check first global
    let import = &imports[0];
    assert_eq!(import.module, "env");
    assert_eq!(import.name, "g1");
    match &import.ty {
        SerializedExternType::Global(global_ty) => {
            assert_eq!(global_ty.val_type, SerializedValType::I32);
            assert!(!global_ty.mutable);
        }
        other => panic!("Expected global import, got {:?}", other),
    }
    // Check second global
    let import = &imports[1];
    assert_eq!(import.module, "env");
    assert_eq!(import.name, "g2");
    match &import.ty {
        SerializedExternType::Global(global_ty) => {
            assert_eq!(global_ty.val_type, SerializedValType::F64);
            assert!(global_ty.mutable);
        }
        other => panic!("Expected global import, got {:?}", other),
    }
    // Check third global
    let import = &imports[2];
    assert_eq!(import.module, "mod");
    assert_eq!(import.name, "g3");
    match &import.ty {
        SerializedExternType::Global(global_ty) => {
            assert_eq!(global_ty.val_type, SerializedValType::I64);
            assert!(!global_ty.mutable);
        }
        other => panic!("Expected global import, got {:?}", other),
    }
}

#[test]
fn imported_functions_extraction() {
    // WAT with three function imports and one internal function with a unique signature
    let wat = r#"
        (module
            (import "env" "f1" (func (param i32)))
            (import "env" "f2" (func (param i64 f32) (result f64)))
            (import "mod" "f3" (func (result i32)))
            (func (param f32 f32) (result f32)
                local.get 0
                local.get 1
                f32.add)
        )
    "#;
    let wasm_bytes = parse_wat(wat).expect("Failed to parse WAT");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    let func_types = super::extract_func_types(&module);
    let imports = super::extract_imports(&module, &func_types);
    // There should be three imports
    assert_eq!(imports.len(), 3);
    // Check first function import
    let import = &imports[0];
    assert_eq!(import.module, "env");
    assert_eq!(import.name, "f1");
    match &import.ty {
        SerializedExternType::Func(idx) => {
            let idx = *idx as usize;
            let ft = &func_types[idx];
            assert_eq!(ft.params.len(), 1);
            assert_eq!(ft.params[0], SerializedValType::I32);
            assert_eq!(ft.results.len(), 0);
        }
        other => panic!("Expected function import, got {:?}", other),
    }
    // Check second function import
    let import = &imports[1];
    assert_eq!(import.module, "env");
    assert_eq!(import.name, "f2");
    match &import.ty {
        SerializedExternType::Func(idx) => {
            let idx = *idx as usize;
            let ft = &func_types[idx];
            assert_eq!(ft.params.len(), 2);
            assert_eq!(ft.params[0], SerializedValType::I64);
            assert_eq!(ft.params[1], SerializedValType::F32);
            assert_eq!(ft.results.len(), 1);
            assert_eq!(ft.results[0], SerializedValType::F64);
        }
        other => panic!("Expected function import, got {:?}", other),
    }
    // Check third function import
    let import = &imports[2];
    assert_eq!(import.module, "mod");
    assert_eq!(import.name, "f3");
    match &import.ty {
        SerializedExternType::Func(idx) => {
            let idx = *idx as usize;
            let ft = &func_types[idx];
            assert_eq!(ft.params.len(), 0);
            assert_eq!(ft.results.len(), 1);
            assert_eq!(ft.results[0], SerializedValType::I32);
        }
        other => panic!("Expected function import, got {:?}", other),
    }
    // Check the internal function's type is present and correct
    let internal_func_types: Vec<_> = module.internal_funcs().map(|(ty, _)| ty).collect();
    // There should be one internal function
    assert_eq!(internal_func_types.len(), 1);
    // Find its index in func_types
    let idx = func_types.iter().position(|ft| {
        ft.params.len() == 2
            && ft.params[0] == SerializedValType::F32
            && ft.params[1] == SerializedValType::F32
            && ft.results.len() == 1
            && ft.results[0] == SerializedValType::F32
    });
    assert!(
        idx.is_some(),
        "Internal function type should be present in func_types"
    );
}

#[test]
fn internal_func_indices_are_correct() {
    // WAT with two imported functions and three internal functions
    let wat = r#"
        (module
            (import "env" "f1" (func (param i32)))
            (import "env" "f2" (func (param f32)))
            (func (param i32) (result i32)
                local.get 0
            )
            (func (param f32 f32) (result f32)
                local.get 0
                local.get 1
                f32.add)
            (func (result i32)
                i32.const 42)
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = crate::Engine::default();
    let module = crate::Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    let serialized = super::SerializedModule::from_module(&module, &engine)
        .expect("Serialization should succeed");
    // There are 2 imported and 3 internal functions, so internal_functions should have length 3
    assert_eq!(serialized.internal_functions.len(), 3);
    // The indices in internal_functions should not overlap with the imported function indices
    // Imported functions are always at the start, so their indices are 0 and 1
    // Internal functions should have indices 2, 3, 4 (in the function index space)
    assert_eq!(serialized.internal_functions[0].type_idx, 2);
    assert_eq!(serialized.internal_functions[1].type_idx, 3);
    assert_eq!(serialized.internal_functions[2].type_idx, 4);
}

#[test]
fn active_data_segment_bytes_are_preserved() {
    // WAT with a memory and an active data segment
    let wat = r#"
        (module
            (memory 1)
            (data (i32.const 4) "hello world")
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = crate::Engine::default();
    let module = crate::Module::new(&engine, &wasm_bytes).expect("Failed to create Module");

    let serialized = super::SerializedModule::from_module(&module, &engine)
        .expect("Serialization should succeed");
    // There should be one data segment
    assert_eq!(serialized.data_segments.len(), 1);
    let data_segment = &serialized.data_segments[0];
    match data_segment {
        SerializedDataSegment::Active(active) => {
            assert_eq!(active.memory_index, 0);
            assert_eq!(active.offset, 4);
            assert_eq!(active.bytes, b"hello world");
        }
        other => panic!("Expected Active data segment, got {:?}", other),
    }
}

#[test]
fn passive_data_segment_bytes_are_preserved() {
    // WAT with a memory and a passive data segment
    let wat = r#"
        (module
            (memory 1)
            (data "passive data!")
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = crate::Engine::default();
    let module = crate::Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    let serialized = super::SerializedModule::from_module(&module, &engine)
        .expect("Serialization should succeed");
    // There should be one data segment
    assert_eq!(serialized.data_segments.len(), 1);
    let data_segment = &serialized.data_segments[0];
    match data_segment {
        SerializedDataSegment::Passive(passive) => {
            assert_eq!(passive.bytes, b"passive data!");
        }
        other => panic!("Expected Passive data segment, got {:?}", other),
    }
}

#[test]
fn mixed_active_and_passive_data_segments_are_preserved() {
    // WAT with a memory, two active and two passive data segments
    let wat = r#"
        (module
            (memory 1)
            (data (i32.const 0) "active1")
            (data (i32.const 7) "active2")
            (data "passive1")
            (data "passive2")
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = crate::Engine::default();
    let module = crate::Module::new(&engine, &wasm_bytes).expect("Failed to create Module");
    let serialized = super::SerializedModule::from_module(&module, &engine)
        .expect("Serialization should succeed");
    // There should be four data segments
    assert_eq!(serialized.data_segments.len(), 4);
    // Check the first two are active, the last two are passive
    match &serialized.data_segments[0] {
        SerializedDataSegment::Active(active) => {
            assert_eq!(active.offset, 0);
            assert_eq!(active.bytes, b"active1");
        }
        other => panic!("Expected Active data segment, got {:?}", other),
    }
    match &serialized.data_segments[1] {
        SerializedDataSegment::Active(active) => {
            assert_eq!(active.offset, 7);
            assert_eq!(active.bytes, b"active2");
        }
        other => panic!("Expected Active data segment, got {:?}", other),
    }
    match &serialized.data_segments[2] {
        SerializedDataSegment::Passive(passive) => {
            assert_eq!(passive.bytes, b"passive1");
        }
        other => panic!("Expected Passive data segment, got {:?}", other),
    }
    match &serialized.data_segments[3] {
        SerializedDataSegment::Passive(passive) => {
            assert_eq!(passive.bytes, b"passive2");
        }
        other => panic!("Expected Passive data segment, got {:?}", other),
    }
}

#[test]
fn compiled_function_bytes_roundtrip() {
    // WAT with a single internal function
    let wat = r#"
        (module
            (func (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add)
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("Failed to parse WAT");
    let engine = crate::Engine::default();
    let module = crate::Module::new(&engine, &wasm_bytes).expect("Failed to create Module");

    let serialized = super::SerializedModule::from_module(&module, &engine)
        .expect("Serialization should succeed");
    // For now, compiled_funcs is empty, so let's just check the engine's code for the internal function
    // Get the function index for the internal function (should be 0, as there are no imports)
    let func_idx = 0u32;
    // Get the EngineFunc for this function
    let engine_func = module
        .header()
        .get_engine_func(crate::module::export::FuncIdx::from(func_idx))
        .expect("EngineFunc should exist");

    // Check that the instructions in the serialized module match the engine's instructions
    let expected_instrs = engine
        .get_instructions(engine_func)
        .expect("Failed to get instructions")
        .to_vec();
    assert_eq!(
        serialized.internal_functions.len(),
        1,
        "There should be one internal function"
    );
    assert_eq!(
        serialized.internal_functions[0].instructions, expected_instrs,
        "Serialized instructions should match engine instructions"
    );
}
