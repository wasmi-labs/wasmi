use wasmi::{Engine, Instance, Linker, Memory, MemoryType, Module, Store};

/// A Wasm module that imports 2 Wasm memories and exports a function
/// that copies a single byte from `mem0` to `mem1`.
const WASM: &str = r#"
    (module
        (import "host" "mem0" (memory $mem0 1))
        (import "host" "mem1" (memory $mem1 1))
        (func (export "copy_from_mem0_to_mem1") (param $ptr i32)
            (i32.store8 $mem1
                (local.get $ptr)
                (i32.load8_s $mem0 (local.get $ptr))
            )
        )
    )
"#;

fn common_setup() -> Result<(Store<()>, Module, Memory, Memory), wasmi::Error> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let mem_ty = MemoryType::new(1, None);
    let mem0 = Memory::new(&mut store, mem_ty)?;
    let mem1 = Memory::new(&mut store, mem_ty)?;
    let module = Module::new(&engine, WASM)?;
    Ok((store, module, mem0, mem1))
}

#[test]
fn multi_memory_using_instance_new() -> Result<(), wasmi::Error> {
    let (mut store, module, mem0, mem1) = common_setup()?;
    let instance = Instance::new(&mut store, &module, &[mem0.into(), mem1.into()])?;
    test_copying_works(&mut store, &instance, mem0, mem1, 42)
}

#[test]
fn multi_memory_using_linker() -> Result<(), wasmi::Error> {
    let (mut store, module, mem0, mem1) = common_setup()?;
    let mut linker = <Linker<()>>::new(store.engine());
    linker.define("host", "mem0", mem0)?;
    linker.define("host", "mem1", mem1)?;
    let instance = linker.instantiate_and_start(&mut store, &module)?;
    test_copying_works(&mut store, &instance, mem0, mem1, 42)
}

fn test_copying_works(
    store: &mut Store<()>,
    instance: &Instance,
    mem0: Memory,
    mem1: Memory,
    ptr: usize,
) -> Result<(), wasmi::Error> {
    let copy_from_m0_to_m1 =
        instance.get_typed_func::<u32, ()>(&mut *store, "copy_from_mem0_to_mem1")?;
    assert_eq!(mem1.data(&store)[ptr], 0);
    mem0.data_mut(&mut *store)[ptr] = 100;
    copy_from_m0_to_m1.call(&mut *store, ptr as u32)?;
    assert_eq!(mem1.data(&store)[ptr], 100);
    Ok(())
}
