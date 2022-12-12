use wasmi::{AsContext, AsContextMut, Engine, Error, Linker, Memory, MemoryType, Module, Store};

#[test]
fn test_import_one_memory_but_there_are_two_memories() -> Result<(), Error> {
    let engine = Engine::default();
    let wat = r#"
            (module
                (import "env" "memory" (memory (;0;) 4))
            )
        "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = Module::new(&engine, &mut &wasm[..])?;

    let mut store = Store::new(&engine, ());

    let mut linker = <Linker<()>>::new();
    let mem_type = MemoryType::new(4, None)?;
    let memory = Memory::new(store.as_context_mut(), mem_type)?;
    linker.define("env", "memory", memory)?;
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)?;

    // #0 memory
    let mem0 = instance.get_memory(store.as_context(), 0).unwrap();
    // Bug here:
    // we only import one memory and not define memory inside module.
    // this should not be defined internally.
    // #1 memory
    let mem1 = instance.get_memory(store.as_context(), 1).unwrap();

    assert_eq!(
        mem0.memory_type(store.as_context()),
        mem1.memory_type(store.as_context())
    );

    Ok(())
}

#[test]
fn test_import_and_define_memory() -> Result<(), Error> {
    let wat = r#"
            (module
                (import "env" "memory" (memory (;0;) 4))
                (memory 1 1 10)
            )
        "#;
    wat::parse_str(wat).unwrap_err();

    Ok(())
}

#[test]
fn test_no_memory() -> Result<(), Error> {
    let engine = Engine::default();
    let wat = r#"
            (module)
        "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = Module::new(&engine, &mut &wasm[..])?;

    let mut store = Store::new(&engine, ());

    let linker = <Linker<()>>::new();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)?;

    if let Some(_mem) = instance.get_memory(store.as_context(), 0) {
        panic!("Not import or define memory");
    }

    Ok(())
}

#[test]
fn test_define_memory_but_not_import() -> Result<(), Error> {
    let engine = Engine::default();
    let wat = r#"
            (module
                (memory $mem 1 10)
            )
        "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = Module::new(&engine, &mut &wasm[..])?;

    let mut store = Store::new(&engine, ());

    let linker = <Linker<()>>::new();
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)?;

    instance
        .get_memory(store.as_context(), 0)
        .expect("Must exist");
    if let Some(_mem) = instance.get_memory(store.as_context(), 1) {
        panic!("Not import another memory");
    }

    Ok(())
}
